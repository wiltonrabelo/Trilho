//! Adaptador de leitura via libgit2 (RF-01, RF-04 parcial).

use crate::application::{GitOperation, RevListAheadBehind, StatusPorcelain};
use crate::application::{GitError, GitReader};
use crate::domain::{Commit, RepoStatus, SyncInfo};
use crate::infrastructure::git_cli::SafeGitCli;
use crate::infrastructure::status_parser;
use crate::infrastructure::upstream::resolve_head_upstream;
use chrono::{DateTime, FixedOffset, TimeZone, Utc};
use git2::{Repository, Sort};
use std::path::Path;

pub struct Git2Reader {
    repo_path: String,
    cli: SafeGitCli,
}

impl Git2Reader {
    pub fn new(repo_path: &str) -> Result<Self, GitError> {
        Repository::discover(repo_path).map_err(|_| GitError::NotARepository)?;
        Ok(Self {
            repo_path: repo_path.to_string(),
            cli: SafeGitCli::new(repo_path),
        })
    }

    fn open(&self) -> Result<Repository, GitError> {
        Repository::discover(&self.repo_path).map_err(|e| GitError::Io(e.to_string()))
    }
}

impl GitReader for Git2Reader {
    fn list_commits(&self, limit: usize, skip: usize) -> Result<Vec<Commit>, GitError> {
        let repo = self.open()?;
        let upstream_oid = resolve_head_upstream(&repo).and_then(|u| u.upstream_oid);

        let mut revwalk = repo
            .revwalk()
            .map_err(|e| GitError::Io(e.to_string()))?;
        revwalk
            .push_head()
            .map_err(|e| GitError::Io(e.to_string()))?;
        revwalk.set_sorting(Sort::TOPOLOGICAL | Sort::TIME).ok();

        let mut commits = Vec::with_capacity(limit.min(256));
        for (idx, oid) in revwalk.enumerate() {
            if idx < skip {
                continue;
            }
            if commits.len() >= limit {
                break;
            }
            let oid = oid.map_err(|e| GitError::Io(e.to_string()))?;
            let commit = repo
                .find_commit(oid)
                .map_err(|e| GitError::Io(e.to_string()))?;
            let is_local_only = match upstream_oid {
                Some(upstream_oid) => {
                    if oid == upstream_oid {
                        false
                    } else {
                        repo.graph_descendant_of(upstream_oid, oid)
                            .map(|on_remote| !on_remote)
                            .unwrap_or(true)
                    }
                }
                None => false,
            };

            let parent_ids: Vec<String> = (0..commit.parent_count())
                .filter_map(|i| commit.parent_id(i).ok().map(|p| p.to_string()))
                .collect();

            commits.push(Commit {
                id: oid.to_string(),
                short_id: format!("{:.7}", oid),
                summary: commit.summary().unwrap_or("(sem mensagem)").to_string(),
                author_name: commit.author().name().unwrap_or("Desconhecido").to_string(),
                authored_at: oid_time_to_iso(&commit),
                is_local_only,
                parent_ids,
            });
        }
        Ok(commits)
    }

    fn get_status(&self) -> Result<RepoStatus, GitError> {
        let op = StatusPorcelain;
        let output = self.cli.run(&op.command())?;
        status_parser::parse_porcelain_v2(&output)
    }

    fn get_sync_info(&self) -> Result<SyncInfo, GitError> {
        let repo = self.open()?;
        let upstream = resolve_head_upstream(&repo);

        let upstream_name = upstream.as_ref().and_then(|u| u.upstream_name.clone());

        let (ahead, behind) = if let Some(name) = upstream_name.as_deref() {
            let op = RevListAheadBehind {
                upstream: name.to_string(),
            };
            let out = self.cli.run(&op.command()).unwrap_or_else(|_| "0\t0".into());
            parse_ahead_behind(&out)
        } else {
            (0, 0)
        };

        Ok(SyncInfo {
            last_fetch_at: None,
            upstream: upstream_name,
            ahead,
            behind,
        })
    }
}

pub fn repo_info(repo_path: &str) -> Result<crate::domain::RepoInfo, GitError> {
    let repo = Repository::discover(repo_path).map_err(|_| GitError::NotARepository)?;
    let head = repo.head().ok();
    let is_detached = head
        .as_ref()
        .map(|h| !h.is_branch())
        .unwrap_or(false);

    let upstream_ref = resolve_head_upstream(&repo);
    let branch = if is_detached {
        None
    } else {
        upstream_ref.as_ref().map(|u| u.branch.clone())
    };
    let upstream = upstream_ref.and_then(|u| u.upstream_name);

    let has_commits = head
        .as_ref()
        .and_then(|h| h.target())
        .is_some()
        && repo.is_empty().ok() == Some(false);

    Ok(crate::domain::RepoInfo {
        path: repo_path.to_string(),
        branch,
        upstream,
        is_detached,
        has_commits,
    })
}

fn oid_time_to_iso(commit: &git2::Commit) -> String {
    let time = commit.time();
    let secs = time.seconds();
    let offset_min = time.offset_minutes();
    let offset = FixedOffset::east_opt(offset_min * 60).unwrap_or(FixedOffset::east_opt(0).unwrap());
    if let Some(dt) = offset.timestamp_opt(secs, 0).single() {
        dt.to_rfc3339()
    } else {
        Utc.timestamp_opt(secs, 0)
            .single()
            .map(|d: DateTime<Utc>| d.to_rfc3339())
            .unwrap_or_else(|| Utc::now().to_rfc3339())
    }
}

fn parse_ahead_behind(output: &str) -> (u32, u32) {
    let parts: Vec<&str> = output.trim().split('\t').collect();
    if parts.len() == 2 {
        (
            parts[0].parse().unwrap_or(0),
            parts[1].parse().unwrap_or(0),
        )
    } else {
        (0, 0)
    }
}

pub fn is_git_repo(path: &str) -> bool {
    Path::new(path).join(".git").exists()
        || git2::Repository::discover(path).is_ok()
}

#[cfg(test)]
mod integration_tests {
    use super::*;
    use std::fs;
    use std::process::Command;

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
    fn lista_commits_em_repo_temp() {
        let dir = std::env::temp_dir().join(format!("trilho-reader-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        init_repo_with_commit(&dir);
        let path = dir.to_string_lossy();
        let reader = Git2Reader::new(&path).expect("reader");
        let commits = reader.list_commits(10, 0).expect("commits");
        assert_eq!(commits.len(), 1);
        assert_eq!(commits[0].summary, "init");
        let info = repo_info(&path).expect("info");
        assert!(info.has_commits);
        let _ = fs::remove_dir_all(&dir);
    }
}
