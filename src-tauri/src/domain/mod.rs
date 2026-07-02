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
