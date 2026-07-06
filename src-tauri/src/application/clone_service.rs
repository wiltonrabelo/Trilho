//! RF-22 — clonar repositório remoto (fora de RepoContext).

use crate::application::GitError;
use crate::application::AppState;
use crate::domain::{CloneRequest, OperationPreview};
use crate::infrastructure::{
    defensive_config_args, validate_clone_branch, validate_clone_depth, validate_clone_destination,
    validate_folder_name, validate_remote_url, repo_name_from_url,
};
use serde::Serialize;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use tauri::{AppHandle, Emitter};

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct CloneProgressEvent {
    line: String,
}

fn resolve_dest(parent_dir: &str, folder_name: &str) -> PathBuf {
    Path::new(parent_dir).join(folder_name)
}

fn clone_options(req: &CloneRequest) -> Result<(Option<String>, Option<u32>), GitError> {
    Ok((
        validate_clone_branch(req.branch.as_deref())?,
        validate_clone_depth(req.depth)?,
    ))
}

fn append_clone_flags(parts: &mut Vec<String>, branch: &Option<String>, depth: &Option<u32>) {
    if let Some(d) = depth {
        parts.push("--depth".into());
        parts.push(d.to_string());
    }
    if let Some(b) = branch {
        parts.push("--branch".into());
        parts.push(b.clone());
    }
}

fn append_clone_flags_cmd(cmd: &mut Command, branch: &Option<String>, depth: &Option<u32>) {
    if let Some(d) = depth {
        cmd.arg("--depth").arg(d.to_string());
    }
    if let Some(b) = branch {
        cmd.arg("--branch").arg(b);
    }
}

fn format_clone_command(url: &str, dest: &Path, branch: &Option<String>, depth: &Option<u32>) -> String {
    let mut parts = vec!["git".to_string()];
    parts.extend(defensive_config_args());
    parts.push("clone".into());
    parts.push("--progress".into());
    append_clone_flags(&mut parts, branch, depth);
    parts.push(url.to_string());
    parts.push(dest.display().to_string());
    parts.join(" ")
}

fn clone_description(label: &str, dest: &Path, branch: &Option<String>, depth: &Option<u32>) -> String {
    let mut desc = format!("Clonar «{label}» em {}", dest.display());
    if let Some(b) = branch {
        desc.push_str(&format!(" (branch «{b}»)"));
    }
    if let Some(d) = depth {
        desc.push_str(&format!(" — profundidade {d}"));
    }
    desc
}

pub fn preview_clone(req: &CloneRequest) -> Result<OperationPreview, GitError> {
    let url = validate_remote_url(&req.url)?;
    let folder = validate_folder_name(&req.folder_name)?;
    let (branch, depth) = clone_options(req)?;
    let parent = req.parent_dir.trim();
    if parent.is_empty() {
        return Ok(blocked_preview(
            "",
            "Escolha a pasta de destino do clone.",
        ));
    }
    let dest = resolve_dest(parent, &folder);
    let blocked = validate_clone_destination(&dest)
        .err()
        .map(|e| e.to_string());
    let label = repo_name_from_url(&url).unwrap_or(folder);
    Ok(OperationPreview {
        commands: vec![format_clone_command(&url, &dest, &branch, &depth)],
        description: clone_description(&label, &dest, &branch, &depth),
        repo_path: parent.to_string(),
        blocked,
    })
}

pub fn list_clone_remote_branches(url: &str) -> Result<Vec<String>, GitError> {
    let url = validate_remote_url(url)?;
    let mut cmd = Command::new("git");
    cmd.args(defensive_config_args())
        .args(["ls-remote", "--heads", &url])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .env("GIT_TERMINAL_PROMPT", "0")
        .env("GCM_INTERACTIVE", "always");

    let output = cmd
        .output()
        .map_err(|e| GitError::Io(format!("Não foi possível listar branches remotas: {e}")))?;

    if !output.status.success() {
        return Err(GitError::from_git_stderr(&String::from_utf8_lossy(
            &output.stderr,
        )));
    }

    parse_ls_remote_heads(&String::from_utf8_lossy(&output.stdout))
}

fn parse_ls_remote_heads(stdout: &str) -> Result<Vec<String>, GitError> {
    let mut branches: Vec<String> = stdout
        .lines()
        .filter_map(|line| {
            let (_, ref_name) = line.split_once('\t')?;
            ref_name.strip_prefix("refs/heads/").map(str::to_string)
        })
        .collect();

    if branches.is_empty() && !stdout.trim().is_empty() {
        return Err(GitError::Git(
            "Não foi possível interpretar a lista de branches do remoto.".into(),
        ));
    }

    branches.sort_by(|a, b| branch_sort_key(a).cmp(&branch_sort_key(b)));
    Ok(branches)
}

fn branch_sort_key(name: &str) -> (u8, &str) {
    match name {
        "main" => (0, name),
        "master" => (1, name),
        "develop" | "development" => (2, name),
        _ => (3, name),
    }
}

pub fn execute_clone(req: &CloneRequest, app: &AppHandle) -> Result<String, GitError> {
    let url = validate_remote_url(&req.url)?;
    let folder = validate_folder_name(&req.folder_name)?;
    let (branch, depth) = clone_options(req)?;
    let parent = req.parent_dir.trim();
    if parent.is_empty() {
        return Err(GitError::Git("Escolha a pasta de destino do clone.".into()));
    }
    let dest = resolve_dest(parent, &folder);
    validate_clone_destination(&dest)?;

    let mut cmd = Command::new("git");
    cmd.args(defensive_config_args())
        .arg("clone")
        .arg("--progress");
    append_clone_flags_cmd(&mut cmd, &branch, &depth);
    cmd.arg(&url)
        .arg(&folder)
        .current_dir(parent)
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .env("GIT_TERMINAL_PROMPT", "0")
        .env("GCM_INTERACTIVE", "always");

    let mut child = cmd
        .spawn()
        .map_err(|e| GitError::Io(format!("Não foi possível executar git clone: {e}")))?;

    if let Some(stderr) = child.stderr.take() {
        let app = app.clone();
        std::thread::spawn(move || {
            let reader = BufReader::new(stderr);
            for line in reader.lines().map_while(Result::ok) {
                let trimmed = line.trim();
                if !trimmed.is_empty() {
                    let _ = app.emit(
                        "clone-progress",
                        CloneProgressEvent {
                            line: trimmed.to_string(),
                        },
                    );
                }
            }
        });
    }

    let status = child
        .wait()
        .map_err(|e| GitError::Io(format!("Falha ao aguardar git clone: {e}")))?;

    if !status.success() {
        return Err(GitError::Git(
            "Clone falhou. Verifique a URL, permissões e credenciais.".into(),
        ));
    }

    let dest_str = dest.to_string_lossy().to_string();
    AppState::validate_path(&dest_str).map_err(GitError::Git)?;
    Ok(dest_str)
}

fn blocked_preview(repo_path: &str, msg: &str) -> OperationPreview {
    OperationPreview {
        commands: vec![],
        description: msg.to_string(),
        repo_path: repo_path.to_string(),
        blocked: Some(msg.to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn preview_bloqueia_destino_existente() {
        let dir = std::env::temp_dir().join(format!("trilho-clone-prev-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("occupied"), "x").unwrap();
        let preview = preview_clone(&CloneRequest {
            url: "https://github.com/user/repo.git".into(),
            parent_dir: dir.to_string_lossy().into(),
            folder_name: "occupied".into(),
            branch: None,
            depth: None,
        })
        .expect("preview");
        assert!(preview.blocked.is_some());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn clone_deserializa_campos() {
        let req: CloneRequest = serde_json::from_str(
            r#"{"url":"https://github.com/u/r.git","parentDir":"C:\\repos","folderName":"r","branch":"main","depth":1}"#,
        )
        .unwrap();
        assert_eq!(req.url, "https://github.com/u/r.git");
        assert_eq!(req.parent_dir, "C:\\repos");
        assert_eq!(req.folder_name, "r");
        assert_eq!(req.branch.as_deref(), Some("main"));
        assert_eq!(req.depth, Some(1));
    }

    #[test]
    fn parse_ls_remote_heads_ordenacao() {
        let out = "abc\trefs/heads/feature\n\
                   def\trefs/heads/main\n\
                   ghi\trefs/heads/develop\n";
        let branches = parse_ls_remote_heads(out).unwrap();
        assert_eq!(branches, vec!["main", "develop", "feature"]);
    }

    #[test]
    fn preview_inclui_branch_e_depth() {
        let preview = preview_clone(&CloneRequest {
            url: "https://github.com/user/repo.git".into(),
            parent_dir: "C:\\repos".into(),
            folder_name: "repo".into(),
            branch: Some("dev".into()),
            depth: Some(5),
        })
        .expect("preview");
        assert!(preview.commands[0].contains("--branch dev"));
        assert!(preview.commands[0].contains("--depth 5"));
        assert!(preview.description.contains("dev"));
        assert!(preview.description.contains("5"));
    }
}
