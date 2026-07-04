//! Operações Git encapsuladas (Command pattern — PLANO §9, RF-08).
//!
//! Cada operação define o `GitCommand` executado; preview e run usam o mesmo objeto.

use crate::application::GitCommand;

/// Operação Git de repositório — `preview()` e `run()` derivam de `command()`.
pub trait GitOperation: Send + Sync {
    fn command(&self) -> GitCommand;
}

pub struct FetchRemote;

impl GitOperation for FetchRemote {
    fn command(&self) -> GitCommand {
        GitCommand {
            args: vec!["fetch".into(), "--prune".into()],
        }
    }
}

pub struct StatusPorcelain;

impl GitOperation for StatusPorcelain {
    fn command(&self) -> GitCommand {
        GitCommand {
            args: vec![
                "status".into(),
                "--porcelain=v2".into(),
                "--branch".into(),
                "-z".into(),
            ],
        }
    }
}

pub struct FileDiff {
    pub path: String,
    pub staged: bool,
}

impl GitOperation for FileDiff {
    fn command(&self) -> GitCommand {
        let mut args = vec!["diff".into(), "--no-color".into()];
        if self.staged {
            args.push("--cached".into());
        }
        args.push("--".into());
        args.push(self.path.clone());
        GitCommand { args }
    }
}

pub struct ShowCommit {
    pub sha: String,
}

impl GitOperation for ShowCommit {
    fn command(&self) -> GitCommand {
        GitCommand {
            args: vec![
                "show".into(),
                "--no-color".into(),
                "--format=".into(),
                self.sha.clone(),
            ],
        }
    }
}

/// Diff de um único arquivo dentro de um commit (detalhes de commit, M1).
/// `--first-parent` garante um diff coerente em merges (contra o 1º pai),
/// alinhado com o cálculo de `list_commit_files`.
pub struct CommitFileDiff {
    pub sha: String,
    pub path: String,
}

impl GitOperation for CommitFileDiff {
    fn command(&self) -> GitCommand {
        GitCommand {
            args: vec![
                "show".into(),
                "--no-color".into(),
                "--format=".into(),
                "--first-parent".into(),
                self.sha.clone(),
                "--".into(),
                self.path.clone(),
            ],
        }
    }
}

pub struct RevListAheadBehind {
    pub upstream: String,
}

impl GitOperation for RevListAheadBehind {
    fn command(&self) -> GitCommand {
        GitCommand {
            args: vec![
                "rev-list".into(),
                "--left-right".into(),
                "--count".into(),
                format!("HEAD...{}", self.upstream),
            ],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fetch_preview_estavel() {
        let op = FetchRemote;
        assert_eq!(op.command().args, vec!["fetch", "--prune"]);
    }

    #[test]
    fn file_diff_staged_inclui_cached() {
        let op = FileDiff {
            path: "src/a.ts".into(),
            staged: true,
        };
        assert!(op.command().args.contains(&"--cached".to_string()));
    }
}
