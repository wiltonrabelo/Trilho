//! Origem inferida da branch atual (RF-02).

use serde::Serialize;

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum OriginConfidence {
    High,
    Medium,
    Low,
    Indeterminate,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BranchOrigin {
    pub current_branch: Option<String>,
    pub candidate: Option<String>,
    pub confidence: OriginConfidence,
    pub explanation: String,
    pub signals: Vec<String>,
    /// Merge-base entre a HEAD e a candidata — o ponto de divergência (RF-02),
    /// usado pela Trilha para separar commits da branch dos commits da base.
    pub merge_base_id: Option<String>,
}

impl BranchOrigin {
    pub fn indeterminate(current_branch: Option<String>, explanation: impl Into<String>) -> Self {
        Self {
            current_branch,
            candidate: None,
            confidence: OriginConfidence::Indeterminate,
            explanation: explanation.into(),
            signals: vec![],
            merge_base_id: None,
        }
    }
}
