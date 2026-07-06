//! Resolução de upstream da branch HEAD (DRY para git2_reader e repo_info).

use crate::application::GitError;
use git2::{BranchType, Oid, Repository};

#[derive(Debug, Clone)]
pub struct UpstreamRef {
    pub branch: String,
    pub upstream_name: Option<String>,
    pub upstream_oid: Option<Oid>,
}

/// Resolve branch local em checkout e seu upstream, se existir.
///
/// Usa o upstream explícito (`branch.*.merge`) quando configurado; caso contrário,
/// infere `origin/<branch>` se essa ref remota existir localmente (ex.: push feito
/// no SourceTree sem `-u`).
pub fn resolve_head_upstream(repo: &Repository) -> Option<UpstreamRef> {
    let head = repo.head().ok()?;
    if !head.is_branch() {
        return None;
    }
    let branch = head.shorthand()?.to_string();
    let mut upstream_name = None;
    let mut upstream_oid = None;
    if let Ok(local) = repo.find_branch(&branch, BranchType::Local) {
        if let Ok(upstream) = local.upstream() {
            upstream_name = upstream.name().ok().flatten().map(|s| s.to_string());
            upstream_oid = upstream.get().target();
        }
    }
    if upstream_name.is_none() {
        if let Some((name, oid)) = read_config_upstream(repo, &branch) {
            upstream_name = Some(name);
            upstream_oid = oid;
        }
    }
    if upstream_name.is_none() {
        if let Some(remote) = primary_remote_name(repo) {
            let remote_ref = format!("{remote}/{branch}");
            if let Ok(rb) = repo.find_branch(&remote_ref, BranchType::Remote) {
                upstream_name = rb.name().ok().flatten().map(|s| s.to_string());
                upstream_oid = rb.get().target();
            }
        }
    }
    Some(UpstreamRef {
        branch,
        upstream_name,
        upstream_oid,
    })
}

fn primary_remote_name(repo: &Repository) -> Option<String> {
    if repo.find_remote("origin").is_ok() {
        return Some("origin".into());
    }
    repo.remotes()
        .ok()
        .and_then(|names| names.get(0).map(|s| s.to_string()))
}

/// Lê `branch.<name>.remote` + `branch.<name>.merge` (ex.: após `push -u`).
fn read_config_upstream(repo: &Repository, branch: &str) -> Option<(String, Option<Oid>)> {
    let cfg = repo.config().ok()?;
    let remote = cfg
        .get_string(&format!("branch.{branch}.remote"))
        .ok()?;
    let merge = cfg.get_string(&format!("branch.{branch}.merge")).ok()?;
    let short = merge
        .strip_prefix("refs/heads/")
        .unwrap_or(merge.as_str());
    let upstream_name = format!("{remote}/{short}");
    let upstream_oid = repo
        .find_branch(&upstream_name, BranchType::Remote)
        .ok()
        .and_then(|b| b.get().target());
    Some((upstream_name, upstream_oid))
}

/// Garante `refs/remotes/<upstream>` quando o refspec do remoto é restrito (só `main`).
pub fn sync_upstream_remote_ref(repo_path: &str) -> Result<(), GitError> {
    use std::process::Command;

    let repo = Repository::discover(repo_path).map_err(|_| GitError::NotARepository)?;
    let head = repo.head().ok();
    let branch = head
        .as_ref()
        .filter(|h| h.is_branch())
        .and_then(|h| h.shorthand())
        .map(str::to_string);
    let Some(branch) = branch else {
        return Ok(());
    };
    let Some((upstream_name, _)) = read_config_upstream(&repo, &branch) else {
        return Ok(());
    };
    if repo
        .find_branch(&upstream_name, BranchType::Remote)
        .is_ok()
    {
        return Ok(());
    }
    let Some((remote, short)) = upstream_name.split_once('/') else {
        return Ok(());
    };
    let spec = format!("refs/heads/{short}:refs/remotes/{upstream_name}");
    let output = Command::new("git")
        .arg("-C")
        .arg(repo_path)
        .args(["fetch", remote, &spec])
        .env("GIT_TERMINAL_PROMPT", "0")
        .env("GCM_INTERACTIVE", "always")
        .output()
        .map_err(|e| GitError::Io(format!("git fetch: {e}")))?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(GitError::from_git_stderr(&stderr));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::process::Command;

    #[allow(dead_code)] // util de teste reservado
    fn init_bare_repo(path: &std::path::Path) {
        Command::new("git")
            .args(["init", "--bare"])
            .arg(path)
            .output()
            .expect("git init --bare");
    }

    fn init_repo_with_commit(path: &std::path::Path) {
        fs::create_dir_all(path).unwrap();
        Command::new("git")
            .args(["init"])
            .current_dir(path)
            .output()
            .expect("git init");
        Command::new("git")
            .args(["config", "user.email", "t@t.com"])
            .current_dir(path)
            .output()
            .unwrap();
        Command::new("git")
            .args(["config", "user.name", "T"])
            .current_dir(path)
            .output()
            .unwrap();
        fs::write(path.join("f.txt"), "x").unwrap();
        Command::new("git")
            .args(["add", "f.txt"])
            .current_dir(path)
            .output()
            .unwrap();
        Command::new("git")
            .args(["commit", "-m", "init"])
            .current_dir(path)
            .output()
            .expect("git commit");
    }

    #[test]
    fn resolve_upstream_em_repo_simples() {
        let dir = std::env::temp_dir().join(format!("trilho-upstream-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        init_repo_with_commit(&dir);
        let repo = Repository::discover(&dir).expect("discover");
        let up = resolve_head_upstream(&repo).expect("upstream ref");
        assert!(up.branch == "master" || up.branch == "main");
        assert!(up.upstream_name.is_none());
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn resolve_upstream_infere_ref_remota_sem_merge_config() {
        let root = std::env::temp_dir().join(format!("trilho-upstream-inf-{}", std::process::id()));
        let bare = root.join("remote.git");
        let local = root.join("local");
        let _ = fs::remove_dir_all(&root);
        init_bare_repo(&bare);
        init_repo_with_commit(&local);

        Command::new("git")
            .args(["remote", "add", "origin"])
            .arg(&bare)
            .current_dir(&local)
            .output()
            .expect("remote add");
        Command::new("git")
            .args(["push", "-u", "origin", "HEAD:main"])
            .current_dir(&local)
            .output()
            .expect("push main");
        Command::new("git")
            .args(["checkout", "-b", "feature-x"])
            .current_dir(&local)
            .output()
            .expect("branch");
        Command::new("git")
            .args(["push", "origin", "feature-x"])
            .current_dir(&local)
            .output()
            .expect("push feature");
        Command::new("git")
            .args(["branch", "--unset-upstream"])
            .current_dir(&local)
            .output()
            .expect("unset upstream");
        Command::new("git")
            .args(["fetch", "origin"])
            .current_dir(&local)
            .output()
            .expect("fetch");

        let repo = Repository::discover(&local).expect("discover");
        let up = resolve_head_upstream(&repo).expect("upstream ref");
        assert_eq!(up.branch, "feature-x");
        assert_eq!(up.upstream_name.as_deref(), Some("origin/feature-x"));
        assert!(up.upstream_oid.is_some());

        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn resolve_upstream_le_config_merge_com_refspec_restrito() {
        let root =
            std::env::temp_dir().join(format!("trilho-upstream-cfg-{}", std::process::id()));
        let bare = root.join("remote.git");
        let local = root.join("local");
        let _ = fs::remove_dir_all(&root);
        init_bare_repo(&bare);
        init_repo_with_commit(&local);

        Command::new("git")
            .args(["remote", "add", "origin"])
            .arg(&bare)
            .current_dir(&local)
            .output()
            .expect("remote add");
        Command::new("git")
            .args([
                "config",
                "remote.origin.fetch",
                "+refs/heads/main:refs/remotes/origin/main",
            ])
            .current_dir(&local)
            .output()
            .expect("refspec");
        Command::new("git")
            .args(["push", "-u", "origin", "HEAD:main"])
            .current_dir(&local)
            .output()
            .expect("push main");
        Command::new("git")
            .args(["checkout", "-b", "main_teste_3"])
            .current_dir(&local)
            .output()
            .expect("branch");
        Command::new("git")
            .args(["push", "-u", "origin", "main_teste_3"])
            .current_dir(&local)
            .output()
            .expect("push branch");
        Command::new("git")
            .args(["fetch", "origin"])
            .current_dir(&local)
            .output()
            .expect("fetch");

        let repo = Repository::discover(&local).expect("discover");
        let up = resolve_head_upstream(&repo).expect("upstream ref");
        assert_eq!(up.branch, "main_teste_3");
        assert_eq!(up.upstream_name.as_deref(), Some("origin/main_teste_3"));
        assert!(
            repo.find_branch("origin/main_teste_3", BranchType::Remote)
                .is_err(),
            "refspec restrito não cria origin/main_teste_3 no fetch padrão"
        );

        let _ = fs::remove_dir_all(&root);
    }
}
