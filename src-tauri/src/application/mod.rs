//! Camada de Aplicação — portas (traits) e estado compartilhado.

mod app_state;
mod branch_origin;
mod operations;
mod repo_context;
mod write_gates;
mod write_service;

pub use app_state::AppState;
pub use branch_origin::{apply_reflog_hint, branch_tip, infer_branch_origin};
pub use operations::{
    CommitFileDiff, FetchRemote, FileDiff, GitOperation, RevListAheadBehind, ShowCommit,
    StatusPorcelain,
};
pub use repo_context::RepoContext;
pub use write_service::{execute_write, preview_write};

use crate::domain::{
    BlameLine, BlameSource, BranchOrigin, Commit, FileChange, RepoStatus, SyncInfo, TrailEntry,
};
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

    #[allow(dead_code)] // usado nos testes de classificação de auth (mod tests)
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
        return "Comando Git falhou.".into();
    }
    let lower = trimmed.to_lowercase();
    // Blame/diff de arquivo sem histórico (novo ou não rastreado): o git aborta
    // com "no such path ... in HEAD". Traduz para algo acionável, sem vazar
    // "fatal:" cru (MVP §4 — nunca stderr cru).
    if lower.contains("no such path") || lower.contains("no such ref") {
        return "Arquivo novo ou não rastreado — ainda não há histórico no \
                repositório para exibir o blame. Faça o commit para acompanhá-lo."
            .into();
    }
    if lower.contains("non-fast-forward")
        || lower.contains("rejected")
        || lower.contains("fetch first")
    {
        return "O remoto está à frente — use «Atualizar (pull --ff-only)» e tente o push de novo."
            .into();
    }
    // Demais falhas: primeira linha, sem o prefixo técnico "fatal:"/"error:".
    let first = trimmed.lines().next().unwrap_or(trimmed);
    first
        .trim_start_matches("fatal: ")
        .trim_start_matches("error: ")
        .trim()
        .to_string()
}

/// Representa um comando Git a ser pré-visualizado (RF-08) e/ou executado.
#[derive(Debug, Clone)]
pub struct GitCommand {
    pub args: Vec<String>,
}

/// Porta de LEITURA do repositório (grafo, status, blame, ...).
pub trait GitReader: Send + Sync {
    /// `first_parent`: trilha da branch atual (`--first-parent`) — visão padrão
    /// legível em repositórios com muitos merges (RF-01); `false` = grafo completo.
    fn list_commits(
        &self,
        limit: usize,
        skip: usize,
        first_parent: bool,
    ) -> Result<Vec<Commit>, GitError>;
    /// Trilha dupla: first-parent da branch atual + first-parent da `base`
    /// até o merge-base, e o trilho comum abaixo dele (RF-01/RF-02).
    fn get_dual_trail(&self, base: &str, limit: usize) -> Result<Vec<TrailEntry>, GitError>;
    fn get_status(&self) -> Result<RepoStatus, GitError>;
    /// Arquivos alterados por um commit (diff contra o 1º pai; árvore vazia se
    /// for o commit raiz) — alimenta os "detalhes de commit" (M1).
    fn list_commit_files(&self, sha: &str) -> Result<Vec<FileChange>, GitError>;
    fn get_sync_info(&self) -> Result<SyncInfo, GitError>;
    fn get_branch_origin(&self) -> Result<BranchOrigin, GitError>;
    fn get_file_blame(
        &self,
        path: &str,
        source: BlameSource,
        commit_id: Option<&str>,
        start_line: u32,
        end_line: u32,
    ) -> Result<Vec<BlameLine>, GitError>;
}

/// Porta de ESCRITA do repositório (commit, restore, reset, revert, push, ...).
pub trait GitWriter: Send + Sync {
    fn preview(&self, command: &GitCommand) -> Vec<String>;
    fn run(&self, command: &GitCommand) -> Result<String, GitError>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn blame_de_arquivo_novo_vira_mensagem_amigavel() {
        let err = GitError::from_git_stderr(
            "fatal: no such path 'Docs/novo.md' in HEAD\n",
        );
        let msg = err.to_string();
        assert!(!msg.contains("fatal"), "não deve vazar 'fatal:' cru: {msg}");
        assert!(!msg.contains("HEAD"), "não deve vazar termos crus: {msg}");
        assert!(msg.to_lowercase().contains("hist"), "deve explicar: {msg}");
    }

    #[test]
    fn stderr_generico_perde_prefixo_tecnico() {
        let err = GitError::from_git_stderr("fatal: bad revision 'zzz'\n");
        assert_eq!(err.to_string(), "bad revision 'zzz'");
    }

    #[test]
    fn falha_de_auth_e_classificada() {
        let err = GitError::from_git_stderr("fatal: Authentication failed for 'https://...'");
        assert!(err.is_auth());
    }
}
