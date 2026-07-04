//! Composição DIP — portas concretas por repositório aberto.

use crate::application::operations::GitOperation;
use crate::application::{GitError, GitReader, GitWriter};
use crate::infrastructure::{Git2Reader, SafeGitCli};
use std::sync::Arc;

/// Contexto de um repositório: leitor (git2) + escritor (CLI) com path fixo.
pub struct RepoContext {
    reader: Arc<dyn GitReader>,
    writer: SafeGitCli,
}

impl RepoContext {
    pub fn open(repo_path: &str) -> Result<Self, GitError> {
        Ok(Self {
            reader: Arc::new(Git2Reader::new(repo_path)?),
            writer: SafeGitCli::new(repo_path),
        })
    }

    pub fn reader(&self) -> &dyn GitReader {
        self.reader.as_ref()
    }

    #[allow(dead_code)] // M3: exposto para preview() na UI de operações
    pub fn writer(&self) -> &SafeGitCli {
        &self.writer
    }

    pub fn execute(&self, op: &dyn GitOperation) -> Result<String, GitError> {
        self.writer.run(&op.command())
    }

    #[allow(dead_code)] // M3: RF-08 preview na UI
    pub fn preview(&self, op: &dyn GitOperation) -> Vec<String> {
        self.writer.preview(&op.command())
    }
}
