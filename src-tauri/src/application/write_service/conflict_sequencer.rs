//! Gates do sequencer — resolver conflitos, abortar, continuar e pular
//! revert/merge/cherry-pick.

use crate::application::{GitError, RepoContext};
use crate::domain::FileChangeKind;

pub(super) fn gate_abort_revert(repo_path: &str) -> Result<Option<String>, GitError> {
    if std::path::Path::new(repo_path)
        .join(".git/REVERT_HEAD")
        .exists()
    {
        Ok(None)
    } else {
        Ok(Some("Não há revert em andamento.".into()))
    }
}

pub(super) fn gate_skip_revert(repo_path: &str) -> Result<Option<String>, GitError> {
    if std::path::Path::new(repo_path)
        .join(".git/REVERT_HEAD")
        .exists()
    {
        Ok(None)
    } else {
        Ok(Some("Não há revert em andamento para pular.".into()))
    }
}

pub(super) fn gate_skip_cherry_pick(repo_path: &str) -> Result<Option<String>, GitError> {
    if std::path::Path::new(repo_path)
        .join(".git/CHERRY_PICK_HEAD")
        .exists()
    {
        Ok(None)
    } else {
        Ok(Some("Não há cherry-pick em andamento para pular.".into()))
    }
}

pub(super) fn gate_continue_revert(
    repo_path: &str,
    ctx: &RepoContext,
) -> Result<Option<String>, GitError> {
    if !std::path::Path::new(repo_path)
        .join(".git/REVERT_HEAD")
        .exists()
    {
        return Ok(Some("Não há revert em andamento.".into()));
    }
    let status = ctx.reader().get_status()?;
    if status
        .staged
        .iter()
        .chain(status.unstaged.iter())
        .any(|f| f.kind == FileChangeKind::Conflicted)
    {
        return Ok(Some(
            "Ainda há conflitos não resolvidos — resolva os arquivos antes de continuar o revert."
                .into(),
        ));
    }
    Ok(None)
}

pub(super) fn gate_abort_merge(repo_path: &str) -> Result<Option<String>, GitError> {
    if std::path::Path::new(repo_path)
        .join(".git/MERGE_HEAD")
        .exists()
    {
        Ok(None)
    } else {
        Ok(Some("Não há merge em andamento.".into()))
    }
}

pub(super) fn gate_continue_merge(
    repo_path: &str,
    ctx: &RepoContext,
) -> Result<Option<String>, GitError> {
    if !std::path::Path::new(repo_path)
        .join(".git/MERGE_HEAD")
        .exists()
    {
        return Ok(Some("Não há merge em andamento.".into()));
    }
    let status = ctx.reader().get_status()?;
    if status
        .staged
        .iter()
        .chain(status.unstaged.iter())
        .any(|f| f.kind == FileChangeKind::Conflicted)
    {
        return Ok(Some(
            "Ainda há conflitos não resolvidos — resolva os arquivos antes de continuar o merge."
                .into(),
        ));
    }
    Ok(None)
}

pub(super) fn gate_abort_cherry_pick(repo_path: &str) -> Result<Option<String>, GitError> {
    if std::path::Path::new(repo_path)
        .join(".git/CHERRY_PICK_HEAD")
        .exists()
    {
        Ok(None)
    } else {
        Ok(Some("Não há cherry-pick em andamento.".into()))
    }
}

pub(super) fn gate_continue_cherry_pick(
    repo_path: &str,
    ctx: &RepoContext,
) -> Result<Option<String>, GitError> {
    if !std::path::Path::new(repo_path)
        .join(".git/CHERRY_PICK_HEAD")
        .exists()
    {
        return Ok(Some("Não há cherry-pick em andamento.".into()));
    }
    let status = ctx.reader().get_status()?;
    if status
        .staged
        .iter()
        .chain(status.unstaged.iter())
        .any(|f| f.kind == FileChangeKind::Conflicted)
    {
        return Ok(Some(
            "Ainda há conflitos não resolvidos — resolva os arquivos antes de continuar o cherry-pick."
                .into(),
        ));
    }
    Ok(None)
}

pub(super) fn normalize_conflict_side(side: &str) -> Result<&'static str, GitError> {
    match side.trim().to_ascii_lowercase().as_str() {
        "ours" => Ok("ours"),
        "theirs" => Ok("theirs"),
        _ => Err(GitError::Git(
            "Lado inválido — use «ours» (atual) ou «theirs» (entrando).".into(),
        )),
    }
}

pub(super) fn gate_resolve_conflict(
    ctx: &RepoContext,
    path: &str,
) -> Result<Option<String>, GitError> {
    let status = ctx.reader().get_status()?;
    let conflicted = status
        .staged
        .iter()
        .chain(status.unstaged.iter())
        .any(|f| f.path == path && f.kind == FileChangeKind::Conflicted);
    if !conflicted {
        return Ok(Some(format!(
            "«{path}» não está marcado como conflito no status."
        )));
    }
    Ok(None)
}
