//! RF-11 — tipos do log de auditoria.

use serde::{Deserialize, Serialize};

/// Ações registradas no log (PLANO §RF-11 / §6).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum AuditAction {
    Add,
    Commit,
    Push,
    PushForce,
    Reset,
    Revert,
    CherryPick,
    Reword,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum AuditResult {
    Success,
    Error,
}

/// Uma linha do log JSONL.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct AuditEntry {
    /// ISO 8601 com offset.
    pub timestamp: String,
    pub action: AuditAction,
    /// Comando(s) Git exibidos no preview (já sanitizados).
    pub command: String,
    pub repo: String,
    pub branch: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub commits: Vec<String>,
    pub result: AuditResult,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    /// RF-21 — origem pelo assistente LLM (reservado).
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub from_assistant: bool,
}
