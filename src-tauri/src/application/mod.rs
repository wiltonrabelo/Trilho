//! Camada de Aplicação — define as PORTAS (traits) que a infraestrutura implementa.
//!
//! Decisão-chave (PLANO §4/§5): leitura e escrita são portas DISTINTAS, com
//! tecnologias distintas por natureza — `GitReader` (via git2, no futuro) e
//! `GitWriter` (via Git CLI). Não são adaptadores intercambiáveis da mesma
//! interface; cada uma é substituível por um mock em teste (respeita LSP).

// Andaimes das próximas fases (escrita/preview). Mantidos desde o M0 para
// fixar o contrato arquitetural; o uso efetivo entra nas fases de operação.
#![allow(dead_code)]

use crate::domain::Commit;
use std::fmt;

/// Erros de leitura/escrita do Git na fronteira da aplicação.
#[derive(Debug)]
pub enum GitError {
    /// Falha de I/O ou execução de comando.
    Io(String),
    /// O caminho informado não é um repositório Git.
    NotARepository,
}

impl fmt::Display for GitError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            GitError::Io(msg) => write!(f, "Erro de I/O do Git: {msg}"),
            GitError::NotARepository => write!(f, "O caminho não é um repositório Git."),
        }
    }
}

impl std::error::Error for GitError {}

/// Representa um comando Git a ser pré-visualizado (RF-08) e/ou executado.
#[derive(Debug, Clone)]
pub struct GitCommand {
    pub args: Vec<String>,
}

/// Porta de LEITURA do repositório (grafo, status, blame, ...).
pub trait GitReader: Send + Sync {
    fn list_commits(&self, limit: usize) -> Result<Vec<Commit>, GitError>;
}

/// Porta de ESCRITA do repositório (commit, restore, reset, revert, push, ...).
///
/// `preview` devolve exatamente a linha de comando que `run` executaria
/// (fidelidade do RF-08 para comandos únicos).
pub trait GitWriter: Send + Sync {
    fn preview(&self, command: &GitCommand) -> Vec<String>;
    fn run(&self, command: &GitCommand) -> Result<(), GitError>;
}
