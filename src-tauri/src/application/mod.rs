//! Camada de Aplicação — portas (traits) e estado compartilhado.

mod clone_post_check;
mod clone_service;
mod app_state;
mod branch_origin;
mod operations;
mod repo_context;
mod write_gates;
mod write_service;

pub use clone_post_check::validate_post_clone;
pub use clone_service::{execute_clone, list_clone_remote_branches, preview_clone};
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
        if is_network_failure(&lower) {
            return GitError::Git(network_action_message());
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
        // GitHub via SSH: "ERROR: Permission to owner/repo.git denied to user."
        || (lower.contains("permission to") && lower.contains("denied"))
}

fn is_network_failure(lower: &str) -> bool {
    lower.contains("could not resolve host")
        || lower.contains("failed to resolve")
        || lower.contains("name or service not known")
        || lower.contains("no address associated with hostname")
        || lower.contains("temporary failure in name resolution")
        || lower.contains("connection refused")
        || lower.contains("failed to connect")
        || lower.contains("unable to connect")
        || lower.contains("network is unreachable")
        || lower.contains("connection timed out")
        || lower.contains("operation timed out")
        || lower.contains("curl: (6)")
        || lower.contains("curl: (7)")
        || lower.contains("curl: (28)")
}

fn network_action_message() -> String {
    "Sem conexão com a internet ou o servidor Git está inacessível. \
     Verifique sua rede e tente «Fetch» novamente quando estiver online."
        .into()
}

/// Mensagem acionável para RF-10 parcial (MVP §4).
fn auth_action_message(lower: &str) -> String {
    // "Permission to owner/repo denied to user": autenticou, mas com a conta
    // errada — problema de ACESSO, não de credencial ausente.
    if lower.contains("permission to") && lower.contains("denied to") {
        let account = lower
            .split("denied to ")
            .nth(1)
            .and_then(|rest| rest.split_whitespace().next())
            .map(|s| s.trim_end_matches('.').to_string());
        return match account {
            Some(user) => format!(
                "Sem permissão no repositório remoto: você está autenticado como \
                 «{user}», que não tem acesso a esse repositório. Confira se a URL \
                 aponta para a conta certa ou conceda acesso no GitHub."
            ),
            None => "Sem permissão no repositório remoto — a conta autenticada não \
                     tem acesso. Confira a URL ou conceda acesso no GitHub."
                .into(),
        };
    }
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
    if lower.contains("is unmerged") || lower.contains("unmerged") {
        return "Arquivo em conflito — resolva manualmente ou aborte o revert/merge em andamento."
            .into();
    }
    if lower.contains("has only 0 lines") {
        return "Arquivo vazio nesta versão — sem linhas para blame.".into();
    }
    if lower.contains("nothing to commit") && lower.contains("reverting") {
        return "O revert não gerou alterações para commitar — o Trilho tentará pular \
                este patch automaticamente."
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

/// Porta de leitura da trilha de commits (RF-01).
pub trait TrailReader: Send + Sync {
    fn list_commits(
        &self,
        limit: usize,
        after: Option<&str>,
        first_parent: bool,
    ) -> Result<Vec<Commit>, GitError>;
    fn get_dual_trail(&self, base: &str, limit: usize) -> Result<Vec<TrailEntry>, GitError>;
    fn list_commit_files(&self, sha: &str) -> Result<Vec<FileChange>, GitError>;
}

/// Porta de blame por linha (RF-03).
pub trait BlameProvider: Send + Sync {
    fn get_file_blame(
        &self,
        path: &str,
        source: BlameSource,
        commit_id: Option<&str>,
        start_line: u32,
        end_line: u32,
    ) -> Result<Vec<BlameLine>, GitError>;
}

/// Porta de LEITURA do repositório — status, sync, origem da branch.
pub trait GitReader: TrailReader + BlameProvider + Send + Sync {
    fn get_status(&self) -> Result<RepoStatus, GitError>;
    fn get_sync_info(&self) -> Result<SyncInfo, GitError>;
    fn get_branch_origin(&self) -> Result<BranchOrigin, GitError>;
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
        let err = GitError::from_git_stderr("fatal: no such path 'Docs/novo.md' in HEAD\n");
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
    fn falha_de_rede_vira_mensagem_acionavel() {
        let err = GitError::from_git_stderr(
            "fatal: unable to access 'https://github.com/u/r.git/': \
             Could not resolve host: github.com\n",
        );
        let msg = err.to_string();
        assert!(
            msg.contains("internet") || msg.contains("rede"),
            "deve orientar sobre rede: {msg}"
        );
        assert!(
            !msg.contains("resolve host"),
            "não deve vazar stderr cru: {msg}"
        );
    }

    #[test]
    fn falha_de_auth_e_classificada() {
        let err = GitError::from_git_stderr("fatal: Authentication failed for 'https://...'");
        assert!(err.is_auth());
    }

    /// Regressão: negação de acesso do GitHub via SSH não pode vazar crua.
    #[test]
    fn permissao_negada_no_remoto_vira_mensagem_acionavel() {
        let err = GitError::from_git_stderr(
            "ERROR: Permission to wiltonrabelo/Trilho.git denied to wiltonlopesrabelo.\n\
             fatal: Could not read from remote repository.",
        );
        assert!(err.is_auth());
        let msg = err.to_string();
        assert!(
            msg.contains("wiltonlopesrabelo"),
            "deve dizer QUAL conta autenticou: {msg}"
        );
        assert!(
            !msg.starts_with("ERROR"),
            "não deve vazar stderr cru: {msg}"
        );
    }
}
