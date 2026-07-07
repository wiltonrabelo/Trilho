//! Camada de Infraestrutura — adaptadores concretos.

mod blame;
mod blame_parser;
mod branches;
mod credential;
mod git2_reader;
mod git_cli;
mod reword;
mod repo_watcher;
mod ssh_keys;
mod stashes;
mod status_parser;
mod tags;
mod upstream;
mod validation;

pub use ssh_keys::{read_ssh_public_key, test_github_ssh, SshKeyInfo, SshTestResult};

pub use branches::{list_local_branches, list_remote_branches, RemoteBranchRef};
pub use stashes::{list_stashes, stash_reference, StashEntry};
pub use tags::{list_tags, TagEntry};
pub use upstream::{fetch_all_remote_branch_refs, sync_upstream_remote_ref};

pub use credential::{
    detect_credential_status, ensure_gcm_configured, store_github_pat, trigger_github_login,
    CredentialStatus,
};
pub use git2_reader::{repo_info, Git2Reader};
pub use git_cli::{defensive_config_args, SafeGitCli};
pub use reword::execute_reword;
pub use repo_watcher::RepoWatcher;
pub use validation::{
    repo_name_from_url, validate_clone_branch, validate_clone_depth, validate_clone_destination,
    validate_folder_name, validate_git_object_id, validate_remote_name,
    validate_remote_url, validate_repo_relative_path, validate_tag_name,
};

use crate::application::{BlameProvider, GitError, GitReader, TrailReader};
use crate::domain::{Commit, TrailEntry, TrailKind};

/// Adaptador mock (M0) — mantido para fallback web e testes.
pub struct MockGitReader;

impl MockGitReader {
    pub fn new() -> Self {
        Self
    }
}

impl Default for MockGitReader {
    fn default() -> Self {
        Self::new()
    }
}

impl TrailReader for MockGitReader {
    fn list_commits(
        &self,
        limit: usize,
        after: Option<&str>,
        _first_parent: bool,
    ) -> Result<Vec<Commit>, GitError> {
        let sample = vec![
            Commit {
                id: "merge01abcdef0123456789abcdef0123456789ab".into(),
                short_id: "merge01".into(),
                summary: "merge(M1-b): grafo com lanes".into(),
                body: None,
                author_name: "Você".into(),
                authored_at: "2026-07-03T14:00:00-03:00".into(),
                is_local_only: true,
                parent_ids: vec![
                    "9f3a1c2e5b7d0a4f6e8c1b2d3a4f5e6c7d8b9a0f".into(),
                    "feat01abcdef0123456789abcdef0123456789ab".into(),
                ],
                refs: vec![],
            },
            Commit {
                id: "feat01abcdef0123456789abcdef0123456789ab".into(),
                short_id: "feat01".into(),
                summary: "feat: spike lanes no grafo".into(),
                body: None,
                author_name: "Você".into(),
                authored_at: "2026-07-03T12:00:00-03:00".into(),
                is_local_only: true,
                parent_ids: vec!["1b2c3d4e5f60718293a4b5c6d7e8f90a1b2c3d4e".into()],
                refs: vec![],
            },
            Commit {
                id: "9f3a1c2e5b7d0a4f6e8c1b2d3a4f5e6c7d8b9a0f".into(),
                short_id: "9f3a1c2".into(),
                summary: "feat: estrutura inicial do Trilho (M0)".into(),
                body: None,
                author_name: "Você".into(),
                authored_at: "2026-07-02T14:10:00-03:00".into(),
                is_local_only: true,
                parent_ids: vec!["1b2c3d4e5f60718293a4b5c6d7e8f90a1b2c3d4e".into()],
                refs: vec![],
            },
            Commit {
                id: "1b2c3d4e5f60718293a4b5c6d7e8f90a1b2c3d4e".into(),
                short_id: "1b2c3d4".into(),
                summary: "chore: configuração de tema claro/escuro".into(),
                body: None,
                author_name: "Você".into(),
                authored_at: "2026-07-02T11:05:00-03:00".into(),
                is_local_only: false,
                parent_ids: vec![],
                refs: vec![],
            },
        ];
        let start = match after {
            None => 0,
            Some(id) => sample
                .iter()
                .position(|c| c.id == id)
                .map(|i| i + 1)
                .unwrap_or(sample.len()),
        };
        Ok(sample.into_iter().skip(start).take(limit).collect())
    }

    fn list_commit_files(&self, _sha: &str) -> Result<Vec<crate::domain::FileChange>, GitError> {
        use crate::domain::{FileChange, FileChangeKind};
        Ok(vec![
            FileChange {
                path: "src/App.tsx".into(),
                kind: FileChangeKind::Modified,
                staged: false,
            },
            FileChange {
                path: "src/lib/graph/layout-lanes.ts".into(),
                kind: FileChangeKind::Added,
                staged: false,
            },
        ])
    }

    fn get_dual_trail(&self, _base: &str, limit: usize) -> Result<Vec<TrailEntry>, GitError> {
        Ok(self
            .list_commits(limit, None, true)?
            .into_iter()
            .map(|commit| TrailEntry {
                commit,
                trail: TrailKind::Current,
            })
            .collect())
    }
}

impl GitReader for MockGitReader {
    fn get_status(&self) -> Result<crate::domain::RepoStatus, GitError> {
        Ok(crate::domain::RepoStatus {
            staged: vec![],
            unstaged: vec![],
            untracked: vec![],
            operation_in_progress: None,
        })
    }

    fn get_sync_info(&self) -> Result<crate::domain::SyncInfo, GitError> {
        Ok(crate::domain::SyncInfo {
            last_fetch_at: None,
            upstream: None,
            ahead: 0,
            behind: 0,
        })
    }

    fn get_branch_origin(&self) -> Result<crate::domain::BranchOrigin, GitError> {
        Ok(crate::domain::BranchOrigin {
            current_branch: Some("master".into()),
            candidate: Some("main".into()),
            confidence: crate::domain::OriginConfidence::Medium,
            explanation: "Mock — origem inferida de main.".into(),
            signals: vec!["mock".into()],
            merge_base_id: None,
        })
    }
}

impl BlameProvider for MockGitReader {
    fn get_file_blame(
        &self,
        path: &str,
        _source: crate::domain::BlameSource,
        _commit_id: Option<&str>,
        start_line: u32,
        end_line: u32,
    ) -> Result<Vec<crate::domain::BlameLine>, GitError> {
        Ok((start_line..=end_line)
            .map(|line| crate::domain::BlameLine {
                line,
                commit_id: "1b2c3d4e5f60718293a4b5c6d7e8f90a1b2c3d4e".into(),
                short_id: "1b2c3d4".into(),
                author: "Mock".into(),
                authored_at: "2026-07-02T11:05:00-03:00".into(),
                summary: format!("mock blame — {path}"),
                content: format!("linha {line}"),
            })
            .collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::TrailReader;
    use git_cli::defensive_base_args;

    #[test]
    fn mock_reader_respeita_limite() {
        let reader = MockGitReader::new();
        let commits = reader.list_commits(2, None, false).expect("deve listar");
        assert_eq!(commits.len(), 2);
    }

    #[test]
    fn defensive_base_via_git_cli() {
        let args = defensive_base_args("C:/repo");
        assert!(args.contains(&"gc.auto=0".to_string()));
    }
}
