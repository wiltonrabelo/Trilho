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

/// Resultado de clone remoto — repo aberto + aviso opcional do checklist pós-clone.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CloneResult {
    pub repo: super::RepoInfo,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub warning: Option<String>,
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
    /// RF-13 — aplica um ou mais commits no topo da branch atual (`git cherry-pick`).
    #[serde(rename = "cherryPick")]
    CherryPick {
        /// Compatível com recorte 1 (um commit).
        #[serde(rename = "commitId", default)]
        commit_id: Option<String>,
        /// Vários commits — aplicados do mais antigo ao mais recente.
        #[serde(rename = "commitIds", default)]
        commit_ids: Vec<String>,
        /// Anexa «(cherry picked from commit …)» na mensagem (`-x`).
        #[serde(rename = "recordOrigin", default)]
        record_origin: bool,
    },
    Push,
    PullFfOnly,
    /// Completa clone raso (`git fetch --unshallow`).
    UnshallowHistory,
    /// Troca para branch local (`git switch`).
    SwitchBranch {
        branch: String,
        /// Se preenchido, cria/rastreia `remote/branch` (`git switch --track`).
        #[serde(default, rename = "trackRemote")]
        track_remote: Option<String>,
    },
    /// Guarda alterações em stash (`git stash push`).
    StashPush {
        #[serde(default)]
        message: Option<String>,
        #[serde(default, rename = "includeUntracked")]
        include_untracked: bool,
    },
    /// RF-23 — reaplica stash (`git stash apply`).
    StashApply {
        index: usize,
    },
    /// RF-23 — reaplica e remove (`git stash pop`).
    StashPop {
        index: usize,
    },
    /// RF-23 — descarta stash (`git stash drop`).
    StashDrop {
        index: usize,
    },
    /// RF-24 — cria tag em commit (`git tag`).
    CreateTag {
        name: String,
        #[serde(rename = "commitId")]
        commit_id: String,
        #[serde(default = "default_true")]
        annotated: bool,
        #[serde(default)]
        message: Option<String>,
        #[serde(default, rename = "pushToRemote")]
        push_to_remote: bool,
    },
    /// RF-24 — remove tag local (`git tag -d`).
    DeleteTag {
        name: String,
    },
    /// RF-18 — descarta alterações não staged (`git restore --worktree`).
    DiscardWorktree {
        path: String,
    },
    DiscardWorktreeMany {
        paths: Vec<String>,
    },
    DiscardWorktreeAll,
    /// RF-18 — remove não rastreado (`git clean`).
    RemoveUntracked {
        path: String,
    },
    RemoveUntrackedMany {
        paths: Vec<String>,
    },
    /// RF-18 — descarta hunk (`git apply --reverse`).
    DiscardHunk {
        path: String,
        patch: String,
    },
    /// RF-20 — aceita um lado inteiro (`git checkout --ours|--theirs` + `git add`).
    ResolveConflictSide {
        path: String,
        /// `ours` | `theirs`
        side: String,
    },
    /// RF-20 — grava conteúdo resolvido no working tree + `git add`.
    ResolveConflictContent {
        path: String,
        content: String,
    },
    /// Cancela revert em andamento (`git revert --abort`).
    AbortRevert,
    /// Finaliza revert após resolver conflitos (`git revert --continue`).
    ContinueRevert,
    /// Cancela merge em andamento (`git merge --abort`).
    AbortMerge,
    /// Finaliza merge após resolver conflitos (`git merge --continue`).
    ContinueMerge,
    /// Cancela cherry-pick em andamento (`git cherry-pick --abort`).
    AbortCherryPick,
    /// Finaliza cherry-pick após resolver conflitos (`git cherry-pick --continue`).
    ContinueCherryPick,
    /// RF-16 — reescreve mensagem de commit local via rebase (`git rebase -i`).
    Reword {
        #[serde(rename = "commitId")]
        commit_id: String,
        summary: String,
        #[serde(default)]
        body: Option<String>,
        /// RF-16 recorte 2 — obrigatório quando o commit já está no remoto.
        #[serde(default, rename = "forcePush")]
        force_push: bool,
    },
    /// RF-07 — move HEAD para commit ancestral (`git reset`).
    Reset {
        #[serde(rename = "commitId")]
        commit_id: String,
        #[serde(default = "default_reset_mode")]
        mode: ResetModeDto,
        #[serde(default, rename = "forcePush")]
        force_push: bool,
    },
    /// RF-09 — push forçado standalone (`git push --force-with-lease`).
    #[serde(rename = "pushForce")]
    PushForce,
    Publish {
        // Um único nome de campo: aliases + payload com os dois nomes causavam
        // `duplicate field 'url'` na deserialização (serde trata alias como o
        // MESMO campo). O contrato com o front é só `url`.
        #[serde(default)]
        url: Option<String>,
    },
}

fn default_true() -> bool {
    true
}

/// Modo de `git reset` exposto ao frontend (RF-07).
#[derive(Debug, Clone, Copy, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum ResetModeDto {
    Soft,
    Mixed,
    Hard,
}

fn default_reset_mode() -> ResetModeDto {
    ResetModeDto::Mixed
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
    fn cherry_pick_deserializa_commit_id() {
        let req: WriteRequest = serde_json::from_str(
            r#"{"kind":"cherryPick","commitId":"abcdef0123456789abcdef0123456789abcdef01"}"#,
        )
        .unwrap();
        match req {
            WriteRequest::CherryPick { commit_id, .. } => {
                assert_eq!(commit_id.as_deref().unwrap().len(), 40);
            }
            _ => panic!("variant errada"),
        }
    }

    #[test]
    fn cherry_pick_deserializa_multiplos_com_record_origin() {
        let req: WriteRequest = serde_json::from_str(
            r#"{"kind":"cherryPick","commitIds":["aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa","bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb"],"recordOrigin":true}"#,
        )
        .unwrap();
        match req {
            WriteRequest::CherryPick {
                commit_ids,
                record_origin,
                ..
            } => {
                assert_eq!(commit_ids.len(), 2);
                assert!(record_origin);
            }
            _ => panic!("variant errada"),
        }
    }

    #[test]
    fn reset_deserializa_modo_e_force_push() {
        let req: WriteRequest = serde_json::from_str(
            r#"{"kind":"reset","commitId":"abcdef0123456789abcdef0123456789abcdef01","mode":"hard","forcePush":true}"#,
        )
        .unwrap();
        match req {
            WriteRequest::Reset {
                commit_id,
                mode,
                force_push,
            } => {
                assert_eq!(commit_id.len(), 40);
                assert_eq!(mode, ResetModeDto::Hard);
                assert!(force_push);
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

    #[test]
    fn stash_push_deserializa_include_untracked() {
        let req: WriteRequest = serde_json::from_str(
            r#"{"kind":"stashPush","message":"wip","includeUntracked":true}"#,
        )
        .unwrap();
        match req {
            WriteRequest::StashPush {
                message,
                include_untracked,
            } => {
                assert_eq!(message.as_deref(), Some("wip"));
                assert!(include_untracked);
            }
            _ => panic!("variant errada"),
        }
    }

    #[test]
    fn reword_deserializa_force_push() {
        let req: WriteRequest = serde_json::from_str(
            r#"{"kind":"reword","commitId":"abcdef0123456789abcdef0123456789abcdef01","summary":"novo","forcePush":true}"#,
        )
        .unwrap();
        match req {
            WriteRequest::Reword {
                commit_id,
                summary,
                force_push,
                ..
            } => {
                assert_eq!(commit_id.len(), 40);
                assert_eq!(summary, "novo");
                assert!(force_push);
            }
            _ => panic!("variant errada"),
        }
    }

    #[test]
    fn stash_apply_deserializa_index() {
        let req: WriteRequest =
            serde_json::from_str(r#"{"kind":"stashApply","index":0}"#).unwrap();
        match req {
            WriteRequest::StashApply { index } => assert_eq!(index, 0),
            _ => panic!("variant errada"),
        }
    }

    #[test]
    fn delete_tag_deserializa_name() {
        let req: WriteRequest =
            serde_json::from_str(r#"{"kind":"deleteTag","name":"v1.0"}"#).unwrap();
        match req {
            WriteRequest::DeleteTag { name } => assert_eq!(name, "v1.0"),
            _ => panic!("variant errada"),
        }
    }

    #[test]
    fn create_tag_deserializa_campos() {
        let req: WriteRequest = serde_json::from_str(
            r#"{"kind":"createTag","name":"v1","commitId":"abcdef0123456789abcdef0123456789abcdef01","annotated":true,"message":"release","pushToRemote":true}"#,
        )
        .unwrap();
        match req {
            WriteRequest::CreateTag {
                name,
                commit_id,
                annotated,
                message,
                push_to_remote,
            } => {
                assert_eq!(name, "v1");
                assert_eq!(commit_id.len(), 40);
                assert!(annotated);
                assert_eq!(message.as_deref(), Some("release"));
                assert!(push_to_remote);
            }
            _ => panic!("variant errada"),
        }
    }

    #[test]
    fn switch_branch_deserializa_track_remote() {
        let req: WriteRequest = serde_json::from_str(
            r#"{"kind":"switchBranch","branch":"feature","trackRemote":"origin"}"#,
        )
        .unwrap();
        match req {
            WriteRequest::SwitchBranch {
                branch,
                track_remote,
            } => {
                assert_eq!(branch, "feature");
                assert_eq!(track_remote.as_deref(), Some("origin"));
            }
            _ => panic!("variant errada"),
        }
    }

    #[test]
    fn push_force_deserializa() {
        let req: WriteRequest = serde_json::from_str(r#"{"kind":"pushForce"}"#).unwrap();
        assert!(matches!(req, WriteRequest::PushForce));
    }
}
