//! Camada de Domínio — entidades puras, sem dependência de infraestrutura.

mod blame;
mod branch_origin;
mod operation;

pub use blame::{BlameLine, BlameSource};
pub use branch_origin::{BranchOrigin, OriginConfidence};
pub use operation::{OperationPreview, WriteRequest};

use serde::Serialize;
/// Um commit da trilha (RF-01). Serializa em camelCase para o frontend.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Commit {
    pub id: String,
    pub short_id: String,
    pub summary: String,
    pub author_name: String,
    /// Data/hora de autoria em ISO 8601.
    pub authored_at: String,
    /// Commit ainda não enviado ao remoto (badge "local").
    pub is_local_only: bool,
    /// SHAs dos commits pais (para layout de lanes no M1-b).
    pub parent_ids: Vec<String>,
    /// Refs que apontam para este commit (branches locais/remotas e tags).
    pub refs: Vec<String>,
}

/// A qual linha da trilha dupla o commit pertence (RF-01 + RF-02).
#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum TrailKind {
    /// Commit exclusivo da branch atual (acima do merge-base).
    Current,
    /// Commit exclusivo da branch base após a divergência (ex.: development).
    Base,
    /// Trilho comum antes da divergência (o primeiro é o merge-base).
    Shared,
}

/// Item da trilha dupla: commit + a linha a que pertence.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TrailEntry {
    pub commit: Commit,
    pub trail: TrailKind,
}

/// Metadados do repositório aberto.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RepoInfo {
    pub path: String,
    pub branch: Option<String>,
    pub upstream: Option<String>,
    pub has_remote: bool,
    /// URL do remoto principal (origin ou o primeiro) — pré-preenche o Publicar.
    pub remote_url: Option<String>,
    pub is_detached: bool,
    pub has_commits: bool,
}

/// Classificação de alteração de arquivo (RF-04).
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum FileChangeKind {
    Modified,
    Added,
    Deleted,
    Renamed,
    Untracked,
}

/// Um arquivo alterado no working tree ou staging.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FileChange {
    pub path: String,
    pub kind: FileChangeKind,
    pub staged: bool,
}

/// Status agregado do repositório (RF-04).
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RepoStatus {
    pub staged: Vec<FileChange>,
    pub unstaged: Vec<FileChange>,
    pub untracked: Vec<FileChange>,
}

/// Indicador de sincronização com remoto (RF-10 parcial / fetch).
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SyncInfo {
    pub last_fetch_at: Option<String>,
    pub upstream: Option<String>,
    pub ahead: u32,
    pub behind: u32,
}
