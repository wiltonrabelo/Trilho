//! Resolução de upstream da branch HEAD (DRY para git2_reader e repo_info).

use git2::{BranchType, Oid, Repository};

#[derive(Debug, Clone)]
pub struct UpstreamRef {
    pub branch: String,
    pub upstream_name: Option<String>,
    pub upstream_oid: Option<Oid>,
}

/// Resolve branch local em checkout e seu upstream, se existir.
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
    Some(UpstreamRef {
        branch,
        upstream_name,
        upstream_oid,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::process::Command;

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
}
