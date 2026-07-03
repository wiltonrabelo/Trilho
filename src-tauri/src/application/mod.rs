//! Camada de Aplicação — portas (traits) e estado compartilhado.

mod app_state;

pub use app_state::AppState;

use crate::domain::{Commit, RepoStatus, SyncInfo};
use std::fmt;

/// Erros de leitura/escrita do Git na fronteira da aplicação.
#[derive(Debug)]
pub enum GitError {
    Io(String),
    NotARepository,
    NoRepositoryOpen,
    Auth(String),
    Git(String),
}

impl fmt::Display for GitError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            GitError::Io(msg) => write!(f, "Erro de I/O: {msg}"),
            GitError::NotARepository => write!(f, "O caminho não é um repositório Git."),
            GitError::NoRepositoryOpen => write!(f, "Nenhum repositório aberto."),
            GitError::Auth(msg) => write!(f, "{msg}"),
            GitError::Git(msg) => write!(f, "{msg}"),
        }
    }
}

impl std::error::Error for GitError {}

impl GitError {
    pub fn from_git_stderr(stderr: &str) -> Self {
        let lower = stderr.to_lowercase();
        if is_auth_failure(&lower) {
            return GitError::Auth(auth_action_message(&lower));
        }
        GitError::Git(map_git_stderr(stderr))
    }

    pub fn is_auth(&self) -> bool {
        matches!(self, GitError::Auth(_))
    }
}

fn is_auth_failure(lower: &str) -> bool {
    lower.contains("authentication failed")
        || lower.contains("could not read username")
        || lower.contains("terminal prompts disabled")
        || lower.contains("invalid credentials")
        || lower.contains("could not read password")
        || lower.contains("access denied")
        || lower.contains("permission denied (publickey)")
        || lower.contains("support for password authentication was removed")
}

/// Mensagem acionável para RF-10 parcial (MVP §4).
fn auth_action_message(lower: &str) -> String {
    if lower.contains("terminal prompts disabled") {
        return "Credencial ausente ou expirada. Clique em «Conectar / Reautenticar» — \
                o Git Credential Manager (GCM) abrirá a janela de login. \
                Se não abrir, rode `git fetch` uma vez no terminal para registrar a credencial."
            .into();
    }
    "Falha de autenticação no remoto. Clique em «Conectar / Reautenticar» para abrir \
     o Git Credential Manager (GCM) ou configure o credential helper do Windows."
        .into()
}

fn map_git_stderr(stderr: &str) -> String {
    let trimmed = stderr.trim();
    if trimmed.is_empty() {
        "Comando Git falhou.".into()
    } else {
        trimmed.lines().next().unwrap_or(trimmed).to_string()
    }
}

/// Representa um comando Git a ser pré-visualizado (RF-08) e/ou executado.
#[derive(Debug, Clone)]
pub struct GitCommand {
    pub args: Vec<String>,
}

/// Porta de LEITURA do repositório (grafo, status, blame, ...).
pub trait GitReader: Send + Sync {
    fn list_commits(&self, limit: usize, skip: usize) -> Result<Vec<Commit>, GitError>;
    fn get_status(&self) -> Result<RepoStatus, GitError>;
    fn get_sync_info(&self) -> Result<SyncInfo, GitError>;
}

/// Porta de ESCRITA do repositório (commit, restore, reset, revert, push, ...).
pub trait GitWriter: Send + Sync {
    fn preview(&self, command: &GitCommand) -> Vec<String>;
    fn run(&self, command: &GitCommand) -> Result<String, GitError>;
}
