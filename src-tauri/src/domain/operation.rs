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

/// Pedido de clone remoto (RF-22) — não exige repositório aberto.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CloneRequest {
    pub url: String,
    pub parent_dir: String,
    pub folder_name: String,
    /// Branch inicial (`git clone --branch`). `None` = padrão do remoto.
    #[serde(default)]
    pub branch: Option<String>,
    /// Profundidade shallow (`git clone --depth`). `None` = clone completo.
    #[serde(default)]
    pub depth: Option<u32>,
}

/// Pedido de operação de escrita — espelha o frontend.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase", tag = "kind")]
pub enum WriteRequest {
    Stage {
        path: String,
    },
    StageMany {
        paths: Vec<String>,
    },
    StageAll,
    Unstage {
        path: String,
    },
    UnstageMany {
        paths: Vec<String>,
    },
    UnstageAll,
    Commit {
        summary: String,
        #[serde(default)]
        body: Option<String>,
        #[serde(default)]
        amend: bool,
    },
    Uncommit,
    Revert {
        #[serde(rename = "commitId")]
        commit_id: String,
    },
    Push,
    PullFfOnly,
    /// Completa clone raso (`git fetch --unshallow`).
    UnshallowHistory,
    Publish {
        // Um único nome de campo: aliases + payload com os dois nomes causavam
        // `duplicate field 'url'` na deserialização (serde trata alias como o
        // MESMO campo). O contrato com o front é só `url`.
        #[serde(default)]
        url: Option<String>,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn revert_deserializa_commit_id_camel_case() {
        let req: WriteRequest = serde_json::from_str(
            r#"{"kind":"revert","commitId":"abcdef0123456789abcdef0123456789abcdef01"}"#,
        )
        .unwrap();
        match req {
            WriteRequest::Revert { commit_id } => {
                assert_eq!(commit_id.len(), 40);
            }
            _ => panic!("variant errada"),
        }
    }

    #[test]
    fn publish_deserializa_url() {
        let req: WriteRequest =
            serde_json::from_str(r#"{"kind":"publish","url":"https://github.com/u/r.git"}"#)
                .unwrap();
        match req {
            WriteRequest::Publish { url } => {
                assert_eq!(url.as_deref(), Some("https://github.com/u/r.git"));
            }
            _ => panic!("variant errada"),
        }
    }

    #[test]
    fn publish_sem_url_e_valido() {
        let req: WriteRequest = serde_json::from_str(r#"{"kind":"publish"}"#).unwrap();
        assert!(matches!(req, WriteRequest::Publish { url: None }));
    }

    /// Regressão do bug de campo duplicado: payload que o front antigo enviava
    /// (`url` + `remoteUrl`) não pode mais explodir — `remoteUrl` é ignorado.
    #[test]
    fn publish_ignora_campo_extra_remote_url() {
        let req: WriteRequest = serde_json::from_str(
            r#"{"kind":"publish","url":"git@github.com:u/r.git","remoteUrl":"git@github.com:u/r.git"}"#,
        )
        .unwrap();
        match req {
            WriteRequest::Publish { url } => {
                assert_eq!(url.as_deref(), Some("git@github.com:u/r.git"));
            }
            _ => panic!("variant errada"),
        }
    }
}
