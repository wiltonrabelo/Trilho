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
    Publish {
        #[serde(default, alias = "remoteUrl", alias = "remote_url")]
        url: Option<String>,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn publish_deserializa_url_camel_snake() {
        for json in [
            r#"{"kind":"publish","url":"https://github.com/u/r.git"}"#,
            r#"{"kind":"publish","remoteUrl":"https://github.com/u/r.git"}"#,
            r#"{"kind":"publish","remote_url":"https://github.com/u/r.git"}"#,
        ] {
            let req: WriteRequest = serde_json::from_str(json).unwrap();
            match req {
                WriteRequest::Publish { url } => {
                    assert_eq!(url.as_deref(), Some("https://github.com/u/r.git"));
                }
                _ => panic!("variant err for {json}"),
            }
        }
    }
}
