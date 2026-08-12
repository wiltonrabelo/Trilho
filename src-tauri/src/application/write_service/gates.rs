//! Gates e checagens compartilhados entre famílias de operação.

use crate::application::{GitCommand, GitError, RepoContext};
use crate::infrastructure::{repo_info, SafeGitCli};

pub(super) fn gate_sequencer_idle(repo_path: &str, action: &str) -> Result<Option<String>, GitError> {
    let git_dir = std::path::Path::new(repo_path).join(".git");
    if git_dir.join("CHERRY_PICK_HEAD").exists() {
        return Ok(Some(format!(
            "Há um cherry-pick em andamento — finalize com «Continuar cherry-pick» ou cancele antes de {action}."
        )));
    }
    if git_dir.join("REVERT_HEAD").exists() {
        return Ok(Some(format!(
            "Há um revert em andamento — conclua ou aborte antes de {action}."
        )));
    }
    if git_dir.join("MERGE_HEAD").exists() {
        return Ok(Some(format!(
            "Há um merge em andamento — conclua ou aborte antes de {action}."
        )));
    }
    Ok(None)
}

pub(super) fn gate_not_head_commit(
    repo_path: &str,
    sha: &str,
    action: &str,
) -> Result<Option<String>, GitError> {
    if is_head_commit(repo_path, sha)? {
        return Ok(Some(format!(
            "Este é o último commit (HEAD) — não é possível fazer {action} dele."
        )));
    }
    Ok(None)
}

pub(super) fn is_ancestor_of_head(cli: &SafeGitCli, sha: &str) -> Result<bool, GitError> {
    let op = GitCommand {
        args: vec![
            "merge-base".into(),
            "--is-ancestor".into(),
            sha.into(),
            "HEAD".into(),
        ],
    };
    cli.run_bool(&op)
}

pub(super) fn gate_clean_worktree(ctx: &RepoContext) -> Result<Option<String>, GitError> {
    if let Some(op) = &ctx.reader().get_status()?.operation_in_progress {
        return Ok(Some(format!(
            "{} Conclua ou aborte a operação antes de continuar.",
            op.message
        )));
    }
    let status = ctx.reader().get_status()?;
    if !status.staged.is_empty() || !status.unstaged.is_empty() || !status.untracked.is_empty() {
        return Ok(Some(
            "Working tree com alterações — faça commit, stash ou descarte antes de continuar."
                .into(),
        ));
    }
    Ok(None)
}

pub(super) fn is_head_commit(repo_path: &str, sha: &str) -> Result<bool, GitError> {
    let head = SafeGitCli::new(repo_path).run(&crate::application::GitCommand {
        args: vec!["rev-parse".into(), "HEAD".into()],
    })?;
    Ok(head.trim().eq_ignore_ascii_case(sha))
}

pub(super) fn gate_force_push_upstream(ctx: &RepoContext) -> Result<Option<String>, GitError> {
    let sync = ctx.reader().get_sync_info()?;
    if sync.upstream.is_none() {
        return Ok(Some(
            "Branch sem upstream — configure o remoto antes do push forçado.".into(),
        ));
    }
    Ok(None)
}

pub(super) fn branch_name_for_backup(repo_path: &str) -> Result<String, GitError> {
    repo_info(repo_path)?
        .branch
        .ok_or_else(|| GitError::Git("Branch atual não identificada.".into()))
}

pub(super) fn has_tracked_worktree_changes(ctx: &RepoContext) -> Result<bool, GitError> {
    let status = ctx.reader().get_status()?;
    Ok(!status.staged.is_empty() || !status.unstaged.is_empty())
}

pub(super) fn revert_in_progress(repo_path: &str) -> bool {
    std::path::Path::new(repo_path)
        .join(".git/REVERT_HEAD")
        .exists()
}

pub(super) fn cherry_pick_in_progress(repo_path: &str) -> bool {
    std::path::Path::new(repo_path)
        .join(".git/CHERRY_PICK_HEAD")
        .exists()
}
