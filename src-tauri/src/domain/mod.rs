//! Camada de Domínio — entidades puras, sem dependência de infraestrutura.

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
}

/// Metadados do repositório aberto.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RepoInfo {
    pub path: String,
    pub branch: Option<String>,
    pub upstream: Option<String>,
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
