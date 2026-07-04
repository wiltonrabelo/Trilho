//! Blame por linha (RF-03).

use serde::Serialize;

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum BlameSource {
    Commit,
    WorkingTree,
    Staging,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BlameLine {
    pub line: u32,
    pub commit_id: String,
    pub short_id: String,
    pub author: String,
    pub authored_at: String,
    pub summary: String,
    pub content: String,
}
