//! DTOs de operações de escrita (M3 — RF-08).

use serde::{Deserialize, Serialize};

/// Pré-visualização RF-08: comando exato + efeito em linguagem natural.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OperationPreview {
    pub commands: Vec<String>,
    pub description: String,
    pub repo_path: String,
    /// Se preenchido, a operação não pode ser executada (gate de segurança).
    pub blocked: Option<String>,
}

/// Pedido de operação de escrita — espelha o frontend.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase", tag = "kind")]
pub enum WriteRequest {
    Stage { path: String },
    StageMany { paths: Vec<String> },
    StageAll,
    Unstage { path: String },
    UnstageMany { paths: Vec<String> },
    UnstageAll,
    Commit {
        summary: String,
        #[serde(default)]
        body: Option<String>,
        #[serde(default)]
        amend: bool,
    },
    Uncommit,
    Revert { commit_id: String },
    Push,
    PullFfOnly,
}
