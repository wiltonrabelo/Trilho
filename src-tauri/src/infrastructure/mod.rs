//! Camada de Infraestrutura — adaptadores concretos.

mod git_cli;
mod git2_reader;
mod repo_watcher;
mod status_parser;
mod validation;

pub use git_cli::SafeGitCli;
pub use git2_reader::{repo_info, Git2Reader};
pub use repo_watcher::RepoWatcher;
pub use validation::{validate_git_object_id, validate_repo_relative_path};

use crate::application::{GitError, GitReader};
use crate::domain::Commit;

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

impl GitReader for MockGitReader {
    fn list_commits(&self, limit: usize, skip: usize) -> Result<Vec<Commit>, GitError> {
        let sample = vec![
            Commit {
                id: "9f3a1c2e5b7d0a4f6e8c1b2d3a4f5e6c7d8b9a0f".into(),
                short_id: "9f3a1c2".into(),
                summary: "feat: estrutura inicial do Trilho (M0)".into(),
                author_name: "Você".into(),
                authored_at: "2026-07-02T14:10:00-03:00".into(),
                is_local_only: true,
            },
            Commit {
                id: "1b2c3d4e5f60718293a4b5c6d7e8f90a1b2c3d4e".into(),
                short_id: "1b2c3d4".into(),
                summary: "chore: configuração de tema claro/escuro".into(),
                author_name: "Você".into(),
                authored_at: "2026-07-02T11:05:00-03:00".into(),
                is_local_only: false,
            },
            Commit {
                id: "abcdef0123456789abcdef0123456789abcdef01".into(),
                short_id: "abcdef0".into(),
                summary: "docs: plano e MVP aprovados".into(),
                author_name: "Você".into(),
                authored_at: "2026-07-01T18:42:00-03:00".into(),
                is_local_only: false,
            },
        ];
        Ok(sample.into_iter().skip(skip).take(limit).collect())
    }

    fn get_status(&self) -> Result<crate::domain::RepoStatus, GitError> {
        Ok(crate::domain::RepoStatus {
            staged: vec![],
            unstaged: vec![],
            untracked: vec![],
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::GitReader;
    use git_cli::defensive_base_args;

    #[test]
    fn mock_reader_respeita_limite() {
        let reader = MockGitReader::new();
        let commits = reader.list_commits(2, 0).expect("deve listar");
        assert_eq!(commits.len(), 2);
    }

    #[test]
    fn defensive_base_via_git_cli() {
        let args = defensive_base_args("C:/repo");
        assert!(args.contains(&"gc.auto=0".to_string()));
    }
}
