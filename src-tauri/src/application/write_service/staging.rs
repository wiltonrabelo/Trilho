//! Gates de working tree — descartar alterações, remover não rastreados,
//! descartar hunks e salvar arquivos.

use super::git_path_from_display;
use crate::application::{GitError, RepoContext};
use crate::domain::{FileChange, FileChangeKind, InProgressKind};

fn status_path_matches(display: &str, path: &str) -> bool {
    git_path_from_display(display) == path
}

pub(super) fn gate_save_worktree_file(
    ctx: &RepoContext,
    path: &str,
) -> Result<Option<String>, GitError> {
    if let Some(msg) = gate_discard_blocked(ctx)? {
        return Ok(Some(
            msg.replace(
                "Descartar arquivos não cancela a operação",
                "Salvar não cancela a operação",
            ),
        ));
    }
    let status = ctx.reader().get_status()?;
    let conflicted = |f: &FileChange| {
        status_path_matches(&f.path, path) && f.kind == FileChangeKind::Conflicted
    };
    if status.staged.iter().any(conflicted) || status.unstaged.iter().any(conflicted) {
        return Ok(Some(format!(
            "«{path}» está em conflito — use o resolvedor de conflitos."
        )));
    }
    if !crate::infrastructure::worktree_file_exists(ctx.repo_path(), path)? {
        return Ok(Some(format!(
            "«{path}» não existe no working tree — não é possível salvar."
        )));
    }
    Ok(None)
}

fn gate_discard_blocked(ctx: &RepoContext) -> Result<Option<String>, GitError> {
    let status = ctx.reader().get_status()?;
    if let Some(op) = &status.operation_in_progress {
        let abort = match op.kind {
            InProgressKind::Revert => "«Abortar revert»",
            InProgressKind::Merge => "«Abortar merge»",
            InProgressKind::CherryPick => "«Abortar cherry-pick»",
        };
        return Ok(Some(format!(
            "{} Descartar arquivos não cancela a operação — use {abort}.",
            op.message
        )));
    }
    let has_conflicts = status
        .staged
        .iter()
        .chain(status.unstaged.iter())
        .any(|f| f.kind == FileChangeKind::Conflicted);
    if has_conflicts {
        return Ok(Some(
            "Arquivos em conflito — edite para resolver ou aborte a operação em andamento."
                .into(),
        ));
    }
    Ok(None)
}

pub(super) fn gate_discard_worktree(
    ctx: &RepoContext,
    paths: &[String],
) -> Result<Option<String>, GitError> {
    if let Some(msg) = gate_discard_blocked(ctx)? {
        return Ok(Some(msg));
    }
    let status = ctx.reader().get_status()?;
    for path in paths {
        let in_unstaged = status
            .unstaged
            .iter()
            .any(|f| status_path_matches(&f.path, path));
        if !in_unstaged {
            return Ok(Some(format!(
                "«{path}» não tem alterações fora do stage para descartar — use unstage se estiver só em staging."
            )));
        }
    }
    Ok(None)
}

pub(super) fn gate_discard_worktree_all(ctx: &RepoContext) -> Result<Option<String>, GitError> {
    if let Some(msg) = gate_discard_blocked(ctx)? {
        return Ok(Some(msg));
    }
    let status = ctx.reader().get_status()?;
    if status.unstaged.is_empty() {
        return Ok(Some(
            "Não há alterações fora do stage para descartar.".into(),
        ));
    }
    Ok(None)
}

pub(super) fn gate_remove_untracked(
    ctx: &RepoContext,
    paths: &[String],
) -> Result<Option<String>, GitError> {
    let status = ctx.reader().get_status()?;
    for path in paths {
        let in_untracked = status
            .untracked
            .iter()
            .any(|f| status_path_matches(&f.path, path));
        if !in_untracked {
            return Ok(Some(format!(
                "«{path}» não é um arquivo não rastreado — use descartar para alterações em arquivos rastreados."
            )));
        }
    }
    Ok(None)
}

pub(super) fn gate_discard_hunk(
    ctx: &RepoContext,
    path: &str,
    staged: bool,
    patch: &str,
) -> Result<Option<String>, GitError> {
    if let Some(msg) = gate_discard_blocked(ctx)? {
        return Ok(Some(msg));
    }
    let status = ctx.reader().get_status()?;
    let ok = if staged {
        status
            .staged
            .iter()
            .any(|f| status_path_matches(&f.path, path))
    } else {
        status
            .unstaged
            .iter()
            .any(|f| status_path_matches(&f.path, path))
    };
    if !ok {
        let hint = if staged {
            "não está em staging"
        } else {
            "não tem alterações fora do stage"
        };
        return Ok(Some(format!("«{path}» {hint} — não é possível descartar este trecho.")));
    }
    gate_reverse_patch(ctx, patch)
}

fn gate_reverse_patch(ctx: &RepoContext, patch: &str) -> Result<Option<String>, GitError> {
    if patch.trim().is_empty() {
        return Ok(Some("Nenhum trecho selecionado para descartar.".into()));
    }
    let cmd = crate::application::GitCommand {
        args: vec![
            "apply".into(),
            "--reverse".into(),
            "--check".into(),
            "-".into(),
        ],
    };
    match ctx.writer().run_with_stdin(&cmd, patch.as_bytes()) {
        Ok(_) => Ok(None),
        Err(e) => Ok(Some(format!(
            "O trecho não pode ser revertido automaticamente: {e}"
        ))),
    }
}
