//! Casos de uso de escrita M3 — preview (RF-08) e execução com gates.

use crate::application::operations::{
    CreateCommit, GitOperation, PullFfOnly, PushUpstream, RevertCommit, Stage, StageAll,
    StageMany, UncommitSoft, Unstage, UnstageAll, UnstageMany,
};
use crate::application::write_gates::head_is_local_only;
use crate::application::{GitError, RepoContext};
use crate::domain::{OperationPreview, WriteRequest};
use crate::infrastructure::{validate_git_object_id, validate_repo_relative_path};

/// Extrai o path Git de um rótulo de rename (`old → new`).
fn git_path_from_display(display: &str) -> &str {
    display
        .split_once(" → ")
        .map(|(_, new)| new)
        .unwrap_or(display)
}

fn validate_paths(paths: &[String]) -> Result<Vec<String>, GitError> {
    if paths.is_empty() {
        return Err(GitError::Git("Nenhum arquivo selecionado.".into()));
    }
    paths
        .iter()
        .map(|p| {
            validate_repo_relative_path(git_path_from_display(p))
                .map_err(|e| GitError::Git(e.to_string()))
        })
        .collect()
}

pub fn preview_write(
    ctx: &RepoContext,
    repo_path: &str,
    req: &WriteRequest,
) -> Result<OperationPreview, GitError> {
    let (commands, description, blocked) = match req {
        WriteRequest::Stage { path } => {
            let path = validate_repo_relative_path(git_path_from_display(path))
                .map_err(|e| GitError::Git(e.to_string()))?;
            let op = Stage { path };
            (
                ctx.preview_op(&op),
                op.description().to_string(),
                None,
            )
        }
        WriteRequest::StageMany { paths } => {
            let paths = match validate_paths(paths) {
                Ok(p) => p,
                Err(GitError::Git(msg)) if msg.contains("Nenhum") => {
                    return Ok(blocked_preview(repo_path, &msg));
                }
                Err(e) => return Err(e),
            };
            let count = paths.len();
            let op = StageMany { paths };
            (
                ctx.preview_op(&op),
                format!("{} ({count} arquivo(s)).", op.description()),
                None,
            )
        }
        WriteRequest::StageAll => {
            let op = StageAll;
            (
                ctx.preview_op(&op),
                op.description().to_string(),
                None,
            )
        }
        WriteRequest::Unstage { path } => {
            let path = validate_repo_relative_path(git_path_from_display(path))
                .map_err(|e| GitError::Git(e.to_string()))?;
            let op = Unstage { path };
            (
                ctx.preview_op(&op),
                op.description().to_string(),
                None,
            )
        }
        WriteRequest::UnstageMany { paths } => {
            let paths = match validate_paths(paths) {
                Ok(p) => p,
                Err(GitError::Git(msg)) if msg.contains("Nenhum") => {
                    return Ok(blocked_preview(repo_path, &msg));
                }
                Err(e) => return Err(e),
            };
            let count = paths.len();
            let op = UnstageMany { paths };
            (
                ctx.preview_op(&op),
                format!("{} ({count} arquivo(s)).", op.description()),
                None,
            )
        }
        WriteRequest::UnstageAll => {
            let op = UnstageAll;
            (
                ctx.preview_op(&op),
                op.description().to_string(),
                None,
            )
        }
        WriteRequest::Commit {
            summary,
            body,
            amend,
        } => {
            if summary.trim().is_empty() {
                return Ok(blocked_preview(
                    repo_path,
                    "A mensagem do commit (resumo) não pode ficar vazia.",
                ));
            }
            let op = CreateCommit {
                summary: summary.clone(),
                body: body.clone(),
                amend: *amend,
            };
            let blocked = if *amend {
                gate_amend(ctx)?
            } else {
                None
            };
            (
                ctx.preview_op(&op),
                op.description().to_string(),
                blocked,
            )
        }
        WriteRequest::Uncommit => {
            let op = UncommitSoft;
            (
                ctx.preview_op(&op),
                op.description().to_string(),
                gate_uncommit(ctx)?,
            )
        }
        WriteRequest::Revert { commit_id } => {
            let sha = validate_git_object_id(commit_id)
                .map_err(|e| GitError::Git(e.to_string()))?;
            let op = RevertCommit { sha };
            (
                ctx.preview_op(&op),
                op.description().to_string(),
                None,
            )
        }
        WriteRequest::Push => {
            let op = PushUpstream;
            (
                ctx.preview_op(&op),
                op.description().to_string(),
                gate_push(ctx)?,
            )
        }
        WriteRequest::PullFfOnly => {
            let op = PullFfOnly;
            let blocked = gate_pull(ctx)?;
            (
                ctx.preview_op(&op),
                op.description().to_string(),
                blocked,
            )
        }
    };

    Ok(OperationPreview {
        commands,
        description,
        repo_path: repo_path.to_string(),
        blocked,
    })
}

pub fn execute_write(ctx: &RepoContext, req: WriteRequest) -> Result<(), GitError> {
    let preview = preview_write(ctx, ctx.repo_path(), &req)?;
    if let Some(msg) = preview.blocked {
        return Err(GitError::Git(msg));
    }

    match req {
        WriteRequest::Stage { path } => {
            let path = validate_repo_relative_path(git_path_from_display(&path))
                .map_err(|e| GitError::Git(e.to_string()))?;
            ctx.execute_op(&Stage { path })?;
        }
        WriteRequest::StageMany { paths } => {
            let paths = validate_paths(&paths)?;
            ctx.execute_op(&StageMany { paths })?;
        }
        WriteRequest::StageAll => {
            ctx.execute_op(&StageAll)?;
        }
        WriteRequest::Unstage { path } => {
            let path = validate_repo_relative_path(git_path_from_display(&path))
                .map_err(|e| GitError::Git(e.to_string()))?;
            ctx.execute_op(&Unstage { path })?;
        }
        WriteRequest::UnstageMany { paths } => {
            let paths = validate_paths(&paths)?;
            ctx.execute_op(&UnstageMany { paths })?;
        }
        WriteRequest::UnstageAll => {
            ctx.execute_op(&UnstageAll)?;
        }
        WriteRequest::Commit {
            summary,
            body,
            amend,
        } => {
            ctx.execute_op(&CreateCommit {
                summary,
                body,
                amend,
            })?;
        }
        WriteRequest::Uncommit => {
            ctx.execute_op(&UncommitSoft)?;
        }
        WriteRequest::Revert { commit_id } => {
            let sha = validate_git_object_id(&commit_id)
                .map_err(|e| GitError::Git(e.to_string()))?;
            ctx.execute_op(&RevertCommit { sha })?;
        }
        WriteRequest::Push => {
            ctx.execute_op(&PushUpstream)?;
        }
        WriteRequest::PullFfOnly => {
            ctx.execute_op(&PullFfOnly)?;
        }
    }
    Ok(())
}

fn blocked_preview(repo_path: &str, msg: &str) -> OperationPreview {
    OperationPreview {
        commands: vec![],
        description: String::new(),
        repo_path: repo_path.to_string(),
        blocked: Some(msg.to_string()),
    }
}

fn gate_amend(ctx: &RepoContext) -> Result<Option<String>, GitError> {
    if head_is_local_only(ctx.reader(), ctx.writer())? {
        Ok(None)
    } else {
        Ok(Some(
            "O último commit já foi enviado — amend não está disponível no MVP.".into(),
        ))
    }
}

fn gate_uncommit(ctx: &RepoContext) -> Result<Option<String>, GitError> {
    if head_is_local_only(ctx.reader(), ctx.writer())? {
        Ok(None)
    } else {
        Ok(Some(
            "O último commit já foi enviado — uncommit só vale para commits locais.".into(),
        ))
    }
}

fn gate_push(ctx: &RepoContext) -> Result<Option<String>, GitError> {
    let sync = ctx.reader().get_sync_info()?;
    if sync.upstream.is_none() {
        return Ok(Some(
            "Branch sem upstream — configure com git branch -u antes do push.".into(),
        ));
    }
    if sync.ahead == 0 {
        return Ok(Some("Não há commits locais para enviar.".into()));
    }
    if sync.behind > 0 {
        return Ok(Some(
            "O remoto está à frente. Atualize com «pull --ff-only» antes de enviar.".into(),
        ));
    }
    Ok(None)
}

fn gate_pull(ctx: &RepoContext) -> Result<Option<String>, GitError> {
    let sync = ctx.reader().get_sync_info()?;
    if sync.upstream.is_none() {
        return Ok(Some("Branch sem upstream configurado.".into()));
    }
    if sync.behind == 0 {
        return Ok(Some("Já está em dia com o remoto (nada para puxar).".into()));
    }
    Ok(None)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn git_path_from_rename_display() {
        assert_eq!(git_path_from_display("a.ts → b.ts"), "b.ts");
        assert_eq!(git_path_from_display("plain.ts"), "plain.ts");
    }
}
