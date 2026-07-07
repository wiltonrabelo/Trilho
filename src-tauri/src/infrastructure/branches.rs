//! Listagem de branches locais e remotas.

use git2::{BranchType, Repository};
use serde::Serialize;

use crate::application::GitError;

fn branch_sort_key(name: &str) -> (u8, &str) {
    match name {
        "main" => (0, name),
        "master" => (1, name),
        _ => (2, name),
    }
}

/// Referência a branch remota (`origin/feature` → remote=`origin`, branch=`feature`).
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RemoteBranchRef {
    pub remote: String,
    pub branch: String,
}

/// Branches locais (`refs/heads/*`), ordenadas (main/master primeiro).
pub fn list_local_branches(repo_path: &str) -> Result<Vec<String>, GitError> {
    let repo = Repository::open(repo_path).map_err(|e| GitError::Io(e.to_string()))?;
    let mut branches = Vec::new();
    let iter = repo
        .branches(Some(BranchType::Local))
        .map_err(|e| GitError::Git(e.to_string()))?;
    for item in iter {
        let (branch, _) = item.map_err(|e| GitError::Git(e.to_string()))?;
        if let Some(name) = branch
            .name()
            .map_err(|e| GitError::Git(e.to_string()))?
        {
            branches.push(name.to_string());
        }
    }
    branches.sort_by(|a, b| {
        let ka = branch_sort_key(a);
        let kb = branch_sort_key(b);
        ka.cmp(&kb).then_with(|| a.cmp(b))
    });
    Ok(branches)
}

/// Branches remotas (`refs/remotes/*`), exceto `*/HEAD`.
pub fn list_remote_branches(repo_path: &str) -> Result<Vec<RemoteBranchRef>, GitError> {
    let repo = Repository::open(repo_path).map_err(|e| GitError::Io(e.to_string()))?;
    let mut refs = Vec::new();
    let iter = repo
        .branches(Some(BranchType::Remote))
        .map_err(|e| GitError::Git(e.to_string()))?;
    for item in iter {
        let (branch, _) = item.map_err(|e| GitError::Git(e.to_string()))?;
        let Some(full_name) = branch
            .name()
            .map_err(|e| GitError::Git(e.to_string()))?
        else {
            continue;
        };
        if full_name.ends_with("/HEAD") {
            continue;
        }
        let Some((remote, branch_name)) = full_name.split_once('/') else {
            continue;
        };
        refs.push(RemoteBranchRef {
            remote: remote.to_string(),
            branch: branch_name.to_string(),
        });
    }
    refs.sort_by(|a, b| {
        a.remote
            .cmp(&b.remote)
            .then_with(|| {
                let ka = branch_sort_key(&a.branch);
                let kb = branch_sort_key(&b.branch);
                ka.cmp(&kb)
            })
            .then_with(|| a.branch.cmp(&b.branch))
    });
    Ok(refs)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::process::Command;

    fn init_repo(dir: &std::path::Path) {
        Command::new("git")
            .args(["init", "-b", "main"])
            .current_dir(dir)
            .output()
            .unwrap();
        std::fs::write(dir.join("f.txt"), "a").unwrap();
        Command::new("git")
            .args(["add", "f.txt"])
            .current_dir(dir)
            .output()
            .unwrap();
        Command::new("git")
            .args(["commit", "-m", "init"])
            .current_dir(dir)
            .output()
            .unwrap();
    }

    #[test]
    fn lista_branches_locais() {
        let dir = std::env::temp_dir().join(format!("trilho-branches-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        init_repo(&dir);
        Command::new("git")
            .args(["branch", "feature-x"])
            .current_dir(&dir)
            .output()
            .unwrap();

        let branches = list_local_branches(&dir.to_string_lossy()).expect("listar");
        assert!(branches.contains(&"main".to_string()));
        assert!(branches.contains(&"feature-x".to_string()));
        let _ = std::fs::remove_dir_all(&dir);
    }

    fn setup_remote_with_branch(work: &std::path::Path, remote_branch: &str) {
        let base = work.parent().unwrap();
        let bare = base.join("origin.git");
        let _ = std::fs::remove_dir_all(&bare);
        Command::new("git")
            .args(["init", "--bare", "-b", "main"])
            .arg(&bare)
            .output()
            .unwrap();
        init_repo(work);
        Command::new("git")
            .args(["remote", "add", "origin"])
            .arg(&bare)
            .current_dir(work)
            .output()
            .unwrap();
        Command::new("git")
            .args(["push", "-u", "origin", "main"])
            .current_dir(work)
            .output()
            .unwrap();
        Command::new("git")
            .args(["branch", remote_branch])
            .current_dir(work)
            .output()
            .unwrap();
        Command::new("git")
            .args(["push", "origin", remote_branch])
            .current_dir(work)
            .output()
            .unwrap();
        Command::new("git")
            .args(["checkout", "main"])
            .current_dir(work)
            .output()
            .unwrap();
        Command::new("git")
            .args(["branch", "-D", remote_branch])
            .current_dir(work)
            .output()
            .unwrap();
        Command::new("git")
            .args(["fetch", "origin"])
            .current_dir(work)
            .output()
            .unwrap();
    }

    #[test]
    fn lista_branches_remotas() {
        let dir = std::env::temp_dir().join(format!("trilho-remote-br-{}", std::process::id()));
        let work = dir.join("work");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&work).unwrap();
        setup_remote_with_branch(&work, "feature-remote");

        let remotes = list_remote_branches(&work.to_string_lossy()).expect("listar");
        assert!(remotes.iter().any(|r| r.remote == "origin" && r.branch == "main"));
        assert!(
            remotes
                .iter()
                .any(|r| r.remote == "origin" && r.branch == "feature-remote")
        );
        let _ = std::fs::remove_dir_all(&dir);
    }
}
