//! Gates de branch — switch e remoção local/remota.

use crate::application::{GitError, RepoContext};
use crate::infrastructure::{list_local_branches, list_remote_branches, validate_remote_name};

pub(super) fn gate_switch_branch(
    ctx: &RepoContext,
    repo_path: &str,
    branch: &str,
    track_remote: Option<&str>,
) -> Result<Option<String>, GitError> {
    let origin = ctx.reader().get_branch_origin()?;
    if origin.current_branch.as_deref() == Some(branch) {
        return Ok(Some("Você já está nesta branch.".into()));
    }

    let status = ctx.reader().get_status()?;
    if !status.staged.is_empty() || !status.unstaged.is_empty() || !status.untracked.is_empty() {
        return Ok(Some(
            "Working tree com alterações — faça commit, stash ou descarte antes de trocar de branch."
                .into(),
        ));
    }

    match track_remote {
        Some(remote) => {
            validate_remote_name(remote)?;
            let remotes = list_remote_branches(repo_path)?;
            if !remotes
                .iter()
                .any(|r| r.remote == remote && r.branch == branch)
            {
                return Ok(Some(
                    "Branch remota não encontrada — atualize com «Buscar» (fetch) e tente de novo."
                        .into(),
                ));
            }
            Ok(None)
        }
        None => {
            let locals = list_local_branches(repo_path)?;
            if !locals.iter().any(|b| b == branch) {
                return Ok(Some("Branch local não encontrada.".into()));
            }
            Ok(None)
        }
    }
}

pub(super) fn gate_delete_local_branch(
    ctx: &RepoContext,
    repo_path: &str,
    branch: &str,
) -> Result<Option<String>, GitError> {
    let origin = ctx.reader().get_branch_origin()?;
    if origin.current_branch.as_deref() == Some(branch) {
        return Ok(Some(
            "Não é possível remover a branch em checkout — troque de branch antes.".into(),
        ));
    }
    let locals = list_local_branches(repo_path)?;
    if !locals.iter().any(|b| b == branch) {
        return Ok(Some("Branch local não encontrada.".into()));
    }
    Ok(None)
}

pub(super) fn gate_delete_remote_branch(
    repo_path: &str,
    remote: &str,
    branch: &str,
) -> Result<Option<String>, GitError> {
    let remotes = list_remote_branches(repo_path)?;
    if !remotes
        .iter()
        .any(|r| r.remote == remote && r.branch == branch)
    {
        return Ok(Some(
            "Branch remota não encontrada — atualize com «Buscar» (fetch) e tente de novo.".into(),
        ));
    }
    Ok(None)
}

/// Se a branch local já existe, troca localmente em vez de `--track`.
pub(super) fn resolve_switch_track(
    repo_path: &str,
    branch: &str,
    track_remote: Option<&str>,
) -> Result<Option<String>, GitError> {
    let Some(remote) = track_remote else {
        return Ok(None);
    };
    let remote = validate_remote_name(remote)?;
    let locals = list_local_branches(repo_path)?;
    if locals.iter().any(|b| b == branch) {
        Ok(None)
    } else {
        Ok(Some(remote))
    }
}
