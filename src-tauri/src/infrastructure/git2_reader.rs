//! Adaptador de leitura via libgit2 (RF-01, RF-04 parcial).

use crate::application::{
    apply_reflog_hint, infer_branch_origin, GitOperation, RevListAheadBehind, StatusPorcelain,
};
use crate::application::{GitCommand, GitError, GitReader};
use crate::domain::{
    BlameLine, BlameSource, BranchOrigin, Commit, FileChange, FileChangeKind, RepoStatus, SyncInfo,
    TrailEntry, TrailKind,
};
use crate::infrastructure::blame::blame_file;
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
    fn list_commits(
        &self,
        limit: usize,
        skip: usize,
        first_parent: bool,
    ) -> Result<Vec<Commit>, GitError> {
        let repo = self.open()?;
        let upstream_oid = resolve_head_upstream(&repo).and_then(|u| u.upstream_oid);
        let refs_by_oid = collect_ref_map(&repo);

        let mut revwalk = repo.revwalk().map_err(|e| GitError::Io(e.to_string()))?;
        revwalk
            .push_head()
            .map_err(|e| GitError::Io(e.to_string()))?;
        if first_parent {
            // Trilha da branch atual: segue só o primeiro pai (RF-01).
            revwalk
                .simplify_first_parent()
                .map_err(|e| GitError::Io(e.to_string()))?;
        }
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
            commits.push(build_commit(&repo, oid, upstream_oid, &refs_by_oid)?);
        }
        Ok(commits)
    }

    fn get_dual_trail(&self, base: &str, limit: usize) -> Result<Vec<TrailEntry>, GitError> {
        let repo = self.open()?;
        let upstream_oid = resolve_head_upstream(&repo).and_then(|u| u.upstream_oid);
        let refs_by_oid = collect_ref_map(&repo);

        let head_oid = repo
            .head()
            .ok()
            .and_then(|h| h.target())
            .ok_or(GitError::NotARepository)?;
        let base_tip = crate::application::branch_tip(&repo, base)
            .ok_or_else(|| GitError::Git(format!("Branch base «{base}» não encontrada.")))?;
        let merge_base = repo
            .merge_base(head_oid, base_tip)
            .map_err(|e| GitError::Io(e.to_string()))?;

        // Cada linha (branch atual e base) recebe metade do orçamento; o trilho
        // comum preenche o restante.
        let half = (limit / 2).max(1);
        let current = first_parent_until(&repo, head_oid, merge_base, half);
        let base_chain = first_parent_until(&repo, base_tip, merge_base, half);

        let mut entries: Vec<(TrailEntry, i64)> = Vec::new();
        for oid in current {
            let commit = build_commit(&repo, oid, upstream_oid, &refs_by_oid)?;
            let time = commit_time(&repo, oid);
            entries.push((
                TrailEntry {
                    commit,
                    trail: TrailKind::Current,
                },
                time,
            ));
        }
        for oid in base_chain {
            let commit = build_commit(&repo, oid, upstream_oid, &refs_by_oid)?;
            let time = commit_time(&repo, oid);
            entries.push((
                TrailEntry {
                    commit,
                    trail: TrailKind::Base,
                },
                time,
            ));
        }
        // Intercala as duas linhas por data (desc); a ordem interna de cada
        // linha (ancestralidade first-parent) é preservada pelo sort estável.
        entries.sort_by_key(|b| std::cmp::Reverse(b.1));

        let mut result: Vec<TrailEntry> = entries.into_iter().map(|(e, _)| e).collect();

        // Trilho comum a partir do merge-base (inclusive).
        let remaining = limit.saturating_sub(result.len()).max(1);
        let mut shared_oid = Some(merge_base);
        let mut taken = 0usize;
        while let Some(oid) = shared_oid {
            if taken >= remaining {
                break;
            }
            result.push(TrailEntry {
                commit: build_commit(&repo, oid, upstream_oid, &refs_by_oid)?,
                trail: TrailKind::Shared,
            });
            taken += 1;
            shared_oid = repo.find_commit(oid).ok().and_then(|c| c.parent_id(0).ok());
        }

        Ok(result)
    }

    fn get_status(&self) -> Result<RepoStatus, GitError> {
        let op = StatusPorcelain;
        let output = self.cli.run(&op.command())?;
        status_parser::parse_porcelain_v2(&output)
    }

    fn list_commit_files(&self, sha: &str) -> Result<Vec<FileChange>, GitError> {
        let repo = self.open()?;
        let oid = git2::Oid::from_str(sha).map_err(|e| GitError::Git(e.to_string()))?;
        let commit = repo
            .find_commit(oid)
            .map_err(|e| GitError::Git(e.to_string()))?;
        let tree = commit.tree().map_err(|e| GitError::Io(e.to_string()))?;
        // Diff contra o 1º pai (árvore vazia se for o commit raiz).
        let parent_tree = if commit.parent_count() > 0 {
            let parent = commit.parent(0).map_err(|e| GitError::Io(e.to_string()))?;
            Some(parent.tree().map_err(|e| GitError::Io(e.to_string()))?)
        } else {
            None
        };

        let mut opts = git2::DiffOptions::new();
        let mut diff = repo
            .diff_tree_to_tree(parent_tree.as_ref(), Some(&tree), Some(&mut opts))
            .map_err(|e| GitError::Io(e.to_string()))?;
        // Detecta renomeações para exibir "R" em vez de add+delete.
        let mut find_opts = git2::DiffFindOptions::new();
        find_opts.renames(true).copies(true);
        diff.find_similar(Some(&mut find_opts)).ok();

        let mut files: Vec<FileChange> = diff
            .deltas()
            .filter_map(|delta| {
                let kind = match delta.status() {
                    git2::Delta::Added => FileChangeKind::Added,
                    git2::Delta::Deleted => FileChangeKind::Deleted,
                    git2::Delta::Renamed | git2::Delta::Copied => FileChangeKind::Renamed,
                    _ => FileChangeKind::Modified,
                };
                let path = delta
                    .new_file()
                    .path()
                    .or_else(|| delta.old_file().path())
                    .map(|p| p.to_string_lossy().into_owned())?;
                if path.is_empty() {
                    return None;
                }
                Some(FileChange {
                    path,
                    kind,
                    staged: false,
                })
            })
            .collect();
        files.sort_by(|a, b| a.path.cmp(&b.path));
        Ok(files)
    }

    fn get_sync_info(&self) -> Result<SyncInfo, GitError> {
        let repo = self.open()?;
        let upstream = resolve_head_upstream(&repo);

        let upstream_name = upstream.as_ref().and_then(|u| u.upstream_name.clone());

        let (ahead, behind) = if let Some(name) = upstream_name.as_deref() {
            let op = RevListAheadBehind {
                upstream: name.to_string(),
            };
            let out = self
                .cli
                .run(&op.command())
                .unwrap_or_else(|_| "0\t0".into());
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

    fn get_branch_origin(&self) -> Result<BranchOrigin, GitError> {
        let repo = self.open()?;
        let mut origin = infer_branch_origin(&repo);
        if let Some(branch) = origin.current_branch.clone() {
            let reflog = self
                .cli
                .run(&GitCommand {
                    args: vec![
                        "reflog".into(),
                        "show".into(),
                        branch,
                        "-n".into(),
                        "20".into(),
                    ],
                })
                .ok();
            origin = apply_reflog_hint(origin, reflog.as_deref());
        }
        Ok(origin)
    }

    fn get_file_blame(
        &self,
        path: &str,
        source: BlameSource,
        commit_id: Option<&str>,
        start_line: u32,
        end_line: u32,
    ) -> Result<Vec<BlameLine>, GitError> {
        blame_file(&self.cli, path, source, commit_id, start_line, end_line)
    }
}
pub fn repo_info(repo_path: &str) -> Result<crate::domain::RepoInfo, GitError> {
    let repo = Repository::discover(repo_path).map_err(|_| GitError::NotARepository)?;
    let head = repo.head().ok();
    let is_detached = head.as_ref().map(|h| !h.is_branch()).unwrap_or(false);

    let upstream_ref = resolve_head_upstream(&repo);
    let branch = if is_detached {
        None
    } else {
        upstream_ref.as_ref().map(|u| u.branch.clone())
    };
    let upstream = upstream_ref.and_then(|u| u.upstream_name);

    let has_commits =
        head.as_ref().and_then(|h| h.target()).is_some() && repo.is_empty().ok() == Some(false);
    let has_remote = repo.remotes().map(|names| !names.is_empty()).unwrap_or(false);

    Ok(crate::domain::RepoInfo {
        path: repo_path.to_string(),
        branch,
        upstream,
        has_remote,
        is_detached,
        has_commits,
    })
}

/// Refs (branches locais/remotas e tags) por commit apontado — calculado uma
/// vez por chamada e consultado por commit (chips de ref na UI).
fn collect_ref_map(repo: &Repository) -> std::collections::HashMap<git2::Oid, Vec<String>> {
    let mut map: std::collections::HashMap<git2::Oid, Vec<String>> =
        std::collections::HashMap::new();
    if let Ok(refs) = repo.references() {
        for reference in refs.flatten() {
            let Some(name) = reference.shorthand().map(|s| s.to_string()) else {
                continue;
            };
            if name == "HEAD" || name.ends_with("/HEAD") {
                continue;
            }
            // Tags anotadas apontam para objeto tag; resolve para o commit.
            let target = reference
                .peel_to_commit()
                .ok()
                .map(|c| c.id())
                .or_else(|| reference.target());
            if let Some(oid) = target {
                map.entry(oid).or_default().push(name);
            }
        }
    }
    for names in map.values_mut() {
        names.sort();
    }
    map
}

/// Monta o DTO de commit (comum a list_commits e à trilha dupla).
fn build_commit(
    repo: &Repository,
    oid: git2::Oid,
    upstream_oid: Option<git2::Oid>,
    refs_by_oid: &std::collections::HashMap<git2::Oid, Vec<String>>,
) -> Result<Commit, GitError> {
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

    let summary = commit.summary().unwrap_or("(sem mensagem)").to_string();
    let author_name = commit.author().name().unwrap_or("Desconhecido").to_string();
    let authored_at = oid_time_to_iso(&commit);

    Ok(Commit {
        id: oid.to_string(),
        short_id: format!("{:.7}", oid),
        summary,
        author_name,
        authored_at,
        is_local_only,
        parent_ids,
        refs: refs_by_oid.get(&oid).cloned().unwrap_or_default(),
    })
}

/// Cadeia first-parent de `start` até (exclusivo) `stop`, com teto.
fn first_parent_until(
    repo: &Repository,
    start: git2::Oid,
    stop: git2::Oid,
    cap: usize,
) -> Vec<git2::Oid> {
    let mut chain = Vec::new();
    let mut cursor = Some(start);
    while let Some(oid) = cursor {
        if oid == stop || chain.len() >= cap {
            break;
        }
        chain.push(oid);
        cursor = repo.find_commit(oid).ok().and_then(|c| c.parent_id(0).ok());
    }
    chain
}

fn commit_time(repo: &Repository, oid: git2::Oid) -> i64 {
    repo.find_commit(oid)
        .map(|c| c.time().seconds())
        .unwrap_or(0)
}

fn oid_time_to_iso(commit: &git2::Commit) -> String {
    let time = commit.time();
    let secs = time.seconds();
    let offset_min = time.offset_minutes();
    let offset =
        FixedOffset::east_opt(offset_min * 60).unwrap_or(FixedOffset::east_opt(0).unwrap());
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
        (parts[0].parse().unwrap_or(0), parts[1].parse().unwrap_or(0))
    } else {
        (0, 0)
    }
}

#[allow(dead_code)] // utilitário de validação — usar em validate_repo_path no M3
pub fn is_git_repo(path: &str) -> bool {
    Path::new(path).join(".git").exists() || git2::Repository::discover(path).is_ok()
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
        let commits = reader.list_commits(10, 0, false).expect("commits");
        assert_eq!(commits.len(), 1);
        assert_eq!(commits[0].summary, "init");
        let info = repo_info(&path).expect("info");
        assert!(info.has_commits);
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn trilha_dupla_separa_branch_base_e_trilho_comum() {
        let dir = std::env::temp_dir().join(format!("trilho-dual-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        init_repo_with_commit(&dir); // trilho comum: "init"
        let default = Command::new("git")
            .args(["rev-parse", "--abbrev-ref", "HEAD"])
            .current_dir(&dir)
            .output()
            .unwrap();
        let default = String::from_utf8_lossy(&default.stdout).trim().to_string();

        // Branch feature com 1 commit próprio.
        Command::new("git")
            .args(["checkout", "-b", "feature"])
            .current_dir(&dir)
            .output()
            .unwrap();
        fs::write(dir.join("feat.txt"), "y").unwrap();
        Command::new("git")
            .args(["add", "feat.txt"])
            .current_dir(&dir)
            .output()
            .unwrap();
        Command::new("git")
            .args(["commit", "-m", "feat"])
            .current_dir(&dir)
            .output()
            .unwrap();

        // Base avança em paralelo (commit que "volta pra development").
        Command::new("git")
            .args(["checkout", &default])
            .current_dir(&dir)
            .output()
            .unwrap();
        fs::write(dir.join("base.txt"), "z").unwrap();
        Command::new("git")
            .args(["add", "base.txt"])
            .current_dir(&dir)
            .output()
            .unwrap();
        Command::new("git")
            .args(["commit", "-m", "base work"])
            .current_dir(&dir)
            .output()
            .unwrap();
        Command::new("git")
            .args(["checkout", "feature"])
            .current_dir(&dir)
            .output()
            .unwrap();

        let path = dir.to_string_lossy();
        let reader = Git2Reader::new(&path).expect("reader");
        let trail = reader.get_dual_trail(&default, 50).expect("dual trail");

        let kind_of = |summary: &str| {
            trail
                .iter()
                .find(|e| e.commit.summary == summary)
                .map(|e| e.trail)
        };
        assert_eq!(kind_of("feat"), Some(TrailKind::Current));
        assert_eq!(kind_of("base work"), Some(TrailKind::Base));
        assert_eq!(kind_of("init"), Some(TrailKind::Shared));
        // Trilho comum vem por último (abaixo da divergência).
        assert_eq!(trail.last().unwrap().commit.summary, "init");
        let _ = fs::remove_dir_all(&dir);
    }
}
