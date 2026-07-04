//! Composição DIP — portas concretas por repositório aberto.

use crate::application::operations::GitOperation;
use crate::application::{GitError, GitReader, GitWriter};
use crate::infrastructure::{Git2Reader, SafeGitCli};
use std::sync::Arc;

/// Contexto de um repositório: leitor (git2) + escritor (CLI) com path fixo.
pub struct RepoContext {
    repo_path: String,
    reader: Arc<dyn GitReader>,
    writer: SafeGitCli,
}

impl RepoContext {
    pub fn open(repo_path: &str) -> Result<Self, GitError> {
        Ok(Self {
            repo_path: repo_path.to_string(),
            reader: Arc::new(Git2Reader::new(repo_path)?),
            writer: SafeGitCli::new(repo_path),
        })
    }

    pub fn repo_path(&self) -> &str {
        &self.repo_path
    }

    pub fn reader(&self) -> &dyn GitReader {
        self.reader.as_ref()
    }

    pub fn writer(&self) -> &SafeGitCli {
        &self.writer
    }

    pub fn execute(&self, op: &dyn GitOperation) -> Result<String, GitError> {
        self.execute_op(op)
    }

    pub fn execute_op<O: GitOperation + ?Sized>(&self, op: &O) -> Result<String, GitError> {
        let cmd = op.command();
        match op.stdin_payload() {
            Some(data) => self.writer.run_with_stdin(&cmd, &data),
            None => GitWriter::run(self.writer(), &cmd),
        }
    }

    pub fn preview_op<O: GitOperation + ?Sized>(&self, op: &O) -> Vec<String> {
        GitWriter::preview(self.writer(), &op.command())
    }
}
