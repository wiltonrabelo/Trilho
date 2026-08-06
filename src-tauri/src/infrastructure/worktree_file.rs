//! Leitura/gravação de arquivos no working tree (editor interno).

use git2::Repository;
use std::path::Path;

use crate::application::GitError;
use crate::infrastructure::validation::validate_repo_relative_path;

/// Grava conteúdo no working tree sem alterar o stage.
pub fn save_worktree_file(repo_path: &str, path: &str, content: &str) -> Result<(), GitError> {
    let path = validate_repo_relative_path(path)?;
    let repo = Repository::discover(repo_path)
        .map_err(|e| GitError::Io(format!("Não foi possível abrir o repositório: {e}")))?;
    let workdir = repo
        .workdir()
        .ok_or_else(|| GitError::Git("Repositório bare — sem working tree.".into()))?;
    let full = workdir.join(&path);
    if let Some(parent) = full.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| GitError::Io(format!("Não foi possível criar pasta: {e}")))?;
    }
    std::fs::write(&full, content)
        .map_err(|e| GitError::Io(format!("Não foi possível gravar {path}: {e}")))?;
    Ok(())
}

/// Verifica se o path existe no disco dentro do working tree.
pub fn worktree_file_exists(repo_path: &str, path: &str) -> Result<bool, GitError> {
    let path = validate_repo_relative_path(path)?;
    let repo = Repository::discover(repo_path)
        .map_err(|e| GitError::Io(format!("Não foi possível abrir o repositório: {e}")))?;
    let workdir = repo
        .workdir()
        .ok_or_else(|| GitError::Git("Repositório bare — sem working tree.".into()))?;
    let full = workdir.join(Path::new(&path));
    Ok(full.is_file())
}

fn display_to_git_path(display_path: &str) -> String {
    display_path
        .rsplit(" → ")
        .next()
        .unwrap_or(display_path)
        .trim()
        .to_string()
}

fn resolve_workdir_path(repo_path: &str, path: &str) -> Result<std::path::PathBuf, GitError> {
    let path = validate_repo_relative_path(&display_to_git_path(path))?;
    let repo = Repository::discover(repo_path)
        .map_err(|e| GitError::Io(format!("Não foi possível abrir o repositório: {e}")))?;
    let workdir = repo
        .workdir()
        .ok_or_else(|| GitError::Git("Repositório bare — sem working tree.".into()))?;
    // Git usa `/`; no Windows, path misturado (`C:\repo\src/a.ts`) quebra o Explorer.
    #[cfg(windows)]
    let full = workdir.join(path.replace('/', "\\"));
    #[cfg(not(windows))]
    let full = workdir.join(Path::new(&path));
    Ok(full)
}

/// Caminho absoluto nativo do arquivo no working tree (para clipboard / Explorer).
pub fn absolute_worktree_path(repo_path: &str, path: &str) -> Result<String, GitError> {
    let full = resolve_workdir_path(repo_path, path)?;
    Ok(native_path_string(&full))
}

/// Formato que o Explorer/`start` aceitam no Windows (sem `\\?\`, só `\`).
fn native_path_string(path: &Path) -> String {
    let mut s = path
        .canonicalize()
        .unwrap_or_else(|_| path.to_path_buf())
        .to_string_lossy()
        .into_owned();
    if let Some(stripped) = s.strip_prefix(r"\\?\") {
        s = stripped.to_string();
    }
    #[cfg(windows)]
    {
        s = s.replace('/', "\\");
    }
    s
}

/// Abre o arquivo (ou pasta) com o aplicativo padrão do SO.
pub fn open_worktree_path(repo_path: &str, path: &str) -> Result<(), GitError> {
    let full = resolve_workdir_path(repo_path, path)?;
    if !full.exists() {
        return Err(GitError::Io(
            "Arquivo não existe no working tree (removido ou nunca gravado).".into(),
        ));
    }
    open_path_os(&full)
}

/// Revela o arquivo no Explorer (Windows) / pasta pai.
pub fn reveal_worktree_path(repo_path: &str, path: &str) -> Result<(), GitError> {
    let full = resolve_workdir_path(repo_path, path)?;
    if full.exists() {
        reveal_path_os(&full)
    } else if let Some(parent) = full.parent().filter(|p| p.exists()) {
        open_folder_os(parent)
    } else {
        Err(GitError::Io(
            "Caminho não existe no working tree para revelar.".into(),
        ))
    }
}

fn open_path_os(path: &Path) -> Result<(), GitError> {
    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x0800_0000;
        let native = native_path_string(path);
        let status = std::process::Command::new("cmd")
            .args(["/C", "start", "", &native])
            .creation_flags(CREATE_NO_WINDOW)
            .status()
            .map_err(|e| GitError::Io(format!("Falha ao abrir: {e}")))?;
        if !status.success() {
            return Err(GitError::Io("Não foi possível abrir o arquivo.".into()));
        }
        return Ok(());
    }
    #[cfg(not(target_os = "windows"))]
    {
        let _ = path;
        Err(GitError::Io("Abrir arquivo só é suportado no Windows.".into()))
    }
}

fn open_folder_os(path: &Path) -> Result<(), GitError> {
    #[cfg(target_os = "windows")]
    {
        let native = native_path_string(path);
        // Sem CREATE_NO_WINDOW — explorer GUI falha e abre a Área de Trabalho.
        let _ = std::process::Command::new("explorer")
            .arg(&native)
            .spawn()
            .map_err(|e| GitError::Io(format!("Falha ao abrir pasta: {e}")))?;
        return Ok(());
    }
    #[cfg(not(target_os = "windows"))]
    {
        let _ = path;
        Err(GitError::Io("Abrir pasta só é suportado no Windows.".into()))
    }
}

fn reveal_path_os(path: &Path) -> Result<(), GitError> {
    #[cfg(target_os = "windows")]
    {
        let native = native_path_string(path);
        // Dois args: `/select,` + path. Um único `/select,C:\...` com `/` misturado
        // ou CREATE_NO_WINDOW faz o Explorer cair na Área de Trabalho.
        let _ = std::process::Command::new("explorer")
            .args(["/select,", &native])
            .spawn()
            .map_err(|e| GitError::Io(format!("Falha ao revelar: {e}")))?;
        return Ok(());
    }
    #[cfg(not(target_os = "windows"))]
    {
        let _ = path;
        Err(GitError::Io(
            "Revelar no Explorer só é suportado no Windows.".into(),
        ))
    }
}

/// Abre o Git Bash com cwd no repositório aberto.
pub fn open_git_bash(repo_path: &str) -> Result<(), GitError> {
    let root = Path::new(repo_path);
    if !root.is_dir() {
        return Err(GitError::Io(
            "Pasta do repositório inválida ou inexistente.".into(),
        ));
    }

    #[cfg(target_os = "windows")]
    {
        let bash = find_git_bash()?;
        let native = native_path_string(root);
        // `git-bash.exe --cd=<dir>` inicia já na pasta do repo.
        // Sem CREATE_NO_WINDOW — precisa de janela visível.
        let mut cmd = std::process::Command::new(&bash);
        if bash
            .file_name()
            .and_then(|n| n.to_str())
            .is_some_and(|n| n.eq_ignore_ascii_case("git-bash.exe"))
        {
            cmd.arg(format!("--cd={native}"));
        } else {
            cmd.current_dir(root).args(["--login", "-i"]);
        }
        cmd.spawn()
            .map_err(|e| GitError::Io(format!("Falha ao abrir o Git Bash: {e}")))?;
        return Ok(());
    }

    #[cfg(not(target_os = "windows"))]
    {
        let _ = root;
        Err(GitError::Io(
            "Abrir Git Bash só é suportado no Windows.".into(),
        ))
    }
}

#[cfg(target_os = "windows")]
fn find_git_bash() -> Result<std::path::PathBuf, GitError> {
    use std::path::PathBuf;

    let mut candidates: Vec<PathBuf> = Vec::new();

    for env_key in ["PROGRAMFILES", "ProgramFiles(x86)", "LOCALAPPDATA"] {
        if let Ok(base) = std::env::var(env_key) {
            let root = PathBuf::from(base);
            if env_key == "LOCALAPPDATA" {
                candidates.push(root.join(r"Programs\Git\git-bash.exe"));
                candidates.push(root.join(r"Programs\Git\bin\bash.exe"));
            } else {
                candidates.push(root.join(r"Git\git-bash.exe"));
                candidates.push(root.join(r"Git\bin\bash.exe"));
            }
        }
    }

    if let Ok(output) = std::process::Command::new("where").arg("git").output() {
        if output.status.success() {
            if let Some(line) = String::from_utf8_lossy(&output.stdout)
                .lines()
                .map(str::trim)
                .find(|l| !l.is_empty())
            {
                let git_exe = PathBuf::from(line);
                // ...\Git\cmd\git.exe → ...\Git\git-bash.exe
                if let Some(git_root) = git_exe.parent().and_then(|p| p.parent()) {
                    candidates.push(git_root.join("git-bash.exe"));
                    candidates.push(git_root.join(r"bin\bash.exe"));
                }
            }
        }
    }

    for path in candidates {
        if path.is_file() {
            return Ok(path);
        }
    }

    Err(GitError::Io(
        "Git Bash não encontrado. Instale o Git for Windows (git-bash.exe) ou verifique se está no PATH."
            .into(),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::infrastructure::git_cli::SafeGitCli;
    use std::fs;
    use std::process::Command;

    fn init_repo(dir: &std::path::Path) {
        Command::new("git")
            .args(["init", dir.to_str().unwrap()])
            .output()
            .unwrap();
        Command::new("git")
            .args(["config", "user.email", "t@t.com"])
            .current_dir(dir)
            .output()
            .unwrap();
        Command::new("git")
            .args(["config", "user.name", "t"])
            .current_dir(dir)
            .output()
            .unwrap();
    }

    #[test]
    fn native_path_normaliza_barras_no_windows() {
        let p = Path::new(r"C:\repo").join("src/App.tsx");
        let native = native_path_string(&p);
        #[cfg(windows)]
        assert!(!native.contains('/'), "got {native}");
        assert!(native.to_lowercase().contains("app.tsx"));
    }

    #[test]
    fn salva_arquivo_no_working_tree() {
        let dir = std::env::temp_dir().join(format!("trilho-wt-save-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        init_repo(&dir);
        fs::write(dir.join("foo.txt"), "antes\n").unwrap();
        let cli = SafeGitCli::new(dir.to_str().unwrap());
        cli.run(&crate::application::GitCommand {
            args: vec!["add".into(), "foo.txt".into()],
        })
        .unwrap();
        cli.run(&crate::application::GitCommand {
            args: vec!["commit".into(), "-m".into(), "init".into()],
        })
        .unwrap();

        save_worktree_file(dir.to_str().unwrap(), "foo.txt", "depois\n").unwrap();
        let disk = fs::read_to_string(dir.join("foo.txt")).unwrap();
        assert_eq!(disk, "depois\n");
    }
}
