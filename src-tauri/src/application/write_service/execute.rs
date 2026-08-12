//! Dispatcher de execução — aplica cada `WriteRequest` já validado pelo
//! preview.

use super::branch::resolve_switch_track;
use super::commit_history::{
    format_reword_message, reset_needs_force_push, resolve_cherry_pick_shas,
    reword_target_on_remote, RESET_HARD_STASH_MSG,
};
use super::conflict_sequencer::normalize_conflict_side;
use super::gates::{
    branch_name_for_backup, cherry_pick_in_progress, has_tracked_worktree_changes,
    revert_in_progress,
};
use super::sync_remote::{
    execute_force_push_with_lease, execute_publish, push_upstream_op, sync_local_upstream_ref,
};
use super::validate_paths_obrigatorios;
use crate::domain::{caminho_git_do_rotulo, ResetMode};
use crate::application::backup_ref::create_backup_ref;
use crate::application::operations::{
    AbortCherryPick, AbortMerge, AbortRevert, ApplyReversePatch, CherryPickCommit, CreateCommit,
    CreateTag, DeleteLocalBranch, DeleteRemoteBranch, DeleteTag, DiscardWorktree,
    DiscardWorktreeAll, DiscardWorktreeMany, PullFfOnly, PushTag, RemoveUntracked,
    RemoveUntrackedMany, ResetCommit, RevertCommit, SkipCherryPick, SkipRevert, Stage,
    StageAll, StageMany, StashApply, StashDrop, StashPop, StashPush, SwitchBranch, UncommitSoft,
    UnshallowRemote, Unstage, UnstageAll, UnstageMany,
};
use crate::application::{GitError, GitWriter, RepoContext};
use crate::domain::WriteRequest;
use crate::infrastructure::{
    execute_reword, stash_reference, validate_clone_branch, validate_git_object_id,
    validate_remote_name, validate_repo_relative_path, validate_tag_name,
};

/// Executa SEM recalcular o preview — o chamador (camada IPC) já revalidou os
/// gates e a igualdade do argv com o preview autorizado (A-02). Evita rodar o
/// preview três vezes por operação.
pub fn execute_write_prevalidated(ctx: &RepoContext, req: WriteRequest) -> Result<(), GitError> {
    match req {
        WriteRequest::Stage { path } => {
            let path = validate_repo_relative_path(caminho_git_do_rotulo(&path))
                .map_err(|e| GitError::Git(e.to_string()))?;
            ctx.execute_op(&Stage { path })?;
        }
        WriteRequest::StageMany { paths } => {
            let paths = validate_paths_obrigatorios(&paths)?;
            ctx.execute_op(&StageMany { paths })?;
        }
        WriteRequest::StageAll => {
            ctx.execute_op(&StageAll)?;
        }
        WriteRequest::Unstage { path } => {
            let path = validate_repo_relative_path(caminho_git_do_rotulo(&path))
                .map_err(|e| GitError::Git(e.to_string()))?;
            ctx.execute_op(&Unstage { path })?;
        }
        WriteRequest::UnstageMany { paths } => {
            let paths = validate_paths_obrigatorios(&paths)?;
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
            let sha =
                validate_git_object_id(&commit_id).map_err(|e| GitError::Git(e.to_string()))?;
            if let Err(e) = ctx.execute_op(&RevertCommit { sha }) {
                // `git revert` falha com conflito, mas deixa REVERT_HEAD — tratar como
                // sucesso parcial para a UI atualizar e mostrar os conflitos.
                if revert_in_progress(ctx.repo_path()) {
                    return Ok(());
                }
                return Err(e);
            }
        }
        WriteRequest::CherryPick {
            commit_id,
            commit_ids,
            record_origin,
        } => {
            let shas = resolve_cherry_pick_shas(&commit_id, &commit_ids)?;
            if let Err(e) = ctx.execute_op(&CherryPickCommit {
                shas,
                record_origin,
            }) {
                if cherry_pick_in_progress(ctx.repo_path()) {
                    return Ok(());
                }
                return Err(e);
            }
        }
        WriteRequest::Push => {
            let op = push_upstream_op(ctx)?;
            ctx.execute_op(&op)?;
            sync_local_upstream_ref(ctx, &op)?;
        }
        WriteRequest::PullFfOnly => {
            ctx.execute_op(&PullFfOnly)?;
        }
        WriteRequest::FetchRemote => {
            crate::infrastructure::fetch_all_remote_branch_refs(ctx.repo_path())?;
        }
        WriteRequest::UnshallowHistory => {
            ctx.execute_op(&UnshallowRemote)?;
        }
        WriteRequest::SwitchBranch {
            branch,
            track_remote,
        } => {
            let branch = validate_clone_branch(Some(&branch))?
                .ok_or_else(|| GitError::Git("Nome de branch inválido.".into()))?;
            let track_remote = resolve_switch_track(ctx.repo_path(), &branch, track_remote.as_deref())?;
            let op = SwitchBranch {
                branch,
                track_remote,
            };
            for cmd in op.all_commands() {
                GitWriter::run(ctx.writer(), &cmd)?;
            }
        }
        WriteRequest::DeleteLocalBranch { branch } => {
            let branch = validate_clone_branch(Some(&branch))?
                .ok_or_else(|| GitError::Git("Nome de branch inválido.".into()))?;
            ctx.execute_op(&DeleteLocalBranch { branch })?;
        }
        WriteRequest::DeleteRemoteBranch { remote, branch } => {
            let remote = validate_remote_name(&remote)?;
            let branch = validate_clone_branch(Some(&branch))?
                .ok_or_else(|| GitError::Git("Nome de branch inválido.".into()))?;
            ctx.execute_op(&DeleteRemoteBranch { remote, branch })?;
        }
        WriteRequest::StashPush {
            message,
            include_untracked,
        } => {
            ctx.execute_op(&StashPush {
                message,
                include_untracked,
            })?;
        }
        WriteRequest::StashApply { index } => {
            let reference = stash_reference(index)?;
            ctx.execute_op(&StashApply { reference })?;
        }
        WriteRequest::StashPop { index } => {
            let reference = stash_reference(index)?;
            ctx.execute_op(&StashPop { reference })?;
        }
        WriteRequest::StashDrop { index } => {
            let reference = stash_reference(index)?;
            ctx.execute_op(&StashDrop { reference })?;
        }
        WriteRequest::CreateTag {
            name,
            commit_id,
            annotated,
            message,
            push_to_remote,
        } => {
            let name = validate_tag_name(&name)?;
            let commit_id =
                validate_git_object_id(&commit_id).map_err(|e| GitError::Git(e.to_string()))?;
            ctx.execute_op(&CreateTag {
                name: name.clone(),
                commit_id,
                annotated,
                message,
            })?;
            if push_to_remote {
                ctx.execute_op(&PushTag {
                    remote: "origin".into(),
                    name,
                })?;
            }
        }
        WriteRequest::DeleteTag { name } => {
            let name = validate_tag_name(&name)?;
            ctx.execute_op(&DeleteTag { name })?;
        }
        WriteRequest::DiscardWorktree { path } => {
            let path = validate_repo_relative_path(caminho_git_do_rotulo(&path))
                .map_err(|e| GitError::Git(e.to_string()))?;
            ctx.execute_op(&DiscardWorktree { path })?;
        }
        WriteRequest::DiscardWorktreeMany { paths } => {
            let paths = validate_paths_obrigatorios(&paths)?;
            ctx.execute_op(&DiscardWorktreeMany { paths })?;
        }
        WriteRequest::DiscardWorktreeAll => {
            ctx.execute_op(&DiscardWorktreeAll)?;
        }
        WriteRequest::RemoveUntracked { path } => {
            let path = validate_repo_relative_path(caminho_git_do_rotulo(&path))
                .map_err(|e| GitError::Git(e.to_string()))?;
            ctx.execute_op(&RemoveUntracked { path })?;
        }
        WriteRequest::RemoveUntrackedMany { paths } => {
            let paths = validate_paths_obrigatorios(&paths)?;
            ctx.execute_op(&RemoveUntrackedMany { paths })?;
        }
        WriteRequest::DiscardHunk { path, patch, staged } => {
            let _path = validate_repo_relative_path(caminho_git_do_rotulo(&path))
                .map_err(|e| GitError::Git(e.to_string()))?;
            ctx.execute_op(&ApplyReversePatch {
                patch,
                cached: staged,
            })?;
        }
        WriteRequest::ResolveConflictSide { path, side } => {
            let path = validate_repo_relative_path(caminho_git_do_rotulo(&path))
                .map_err(|e| GitError::Git(e.to_string()))?;
            let side = normalize_conflict_side(&side)?;
            let choice = if side == "ours" {
                crate::domain::ConflictSideChoice::Ours
            } else {
                crate::domain::ConflictSideChoice::Theirs
            };
            crate::infrastructure::resolve_conflict_side(ctx.writer(), &path, choice)?;
        }
        WriteRequest::ResolveConflictContent { path, content } => {
            let path = validate_repo_relative_path(caminho_git_do_rotulo(&path))
                .map_err(|e| GitError::Git(e.to_string()))?;
            if content.contains("<<<<<<<") || content.contains(">>>>>>>") {
                return Err(GitError::Git(
                    "O conteúdo ainda contém marcadores de conflito.".into(),
                ));
            }
            crate::infrastructure::resolve_conflict_content(
                ctx.repo_path(),
                ctx.writer(),
                &path,
                &content,
            )?;
        }
        WriteRequest::SaveWorktreeFile { path, content } => {
            let path = validate_repo_relative_path(caminho_git_do_rotulo(&path))
                .map_err(|e| GitError::Git(e.to_string()))?;
            crate::infrastructure::save_worktree_file(ctx.repo_path(), &path, &content)?;
        }
        WriteRequest::AbortRevert => {
            ctx.execute_op(&AbortRevert)?;
        }
        WriteRequest::ContinueRevert => {
            ctx.writer().finish_revert()?;
        }
        WriteRequest::AbortMerge => {
            ctx.execute_op(&AbortMerge)?;
        }
        WriteRequest::ContinueMerge => {
            ctx.writer().finish_merge()?;
        }
        WriteRequest::AbortCherryPick => {
            ctx.execute_op(&AbortCherryPick)?;
        }
        WriteRequest::ContinueCherryPick => {
            ctx.writer().finish_cherry_pick()?;
        }
        WriteRequest::SkipRevert => {
            ctx.execute_op(&SkipRevert)?;
        }
        WriteRequest::SkipCherryPick => {
            ctx.execute_op(&SkipCherryPick)?;
        }
        WriteRequest::Reword {
            commit_id,
            summary,
            body,
            force_push,
        } => {
            let sha =
                validate_git_object_id(&commit_id).map_err(|e| GitError::Git(e.to_string()))?;
            let message = format_reword_message(&summary, body.as_deref());
            execute_reword(ctx.writer(), &sha, &message)?;
            if force_push && reword_target_on_remote(ctx, &sha)? {
                execute_force_push_with_lease(ctx)?;
            }
        }
        WriteRequest::Reset {
            commit_id,
            mode,
            force_push,
        } => {
            let sha =
                validate_git_object_id(&commit_id).map_err(|e| GitError::Git(e.to_string()))?;
            let reset_mode = mode;
            if reset_mode == ResetMode::Hard {
                let branch = branch_name_for_backup(ctx.repo_path())?;
                create_backup_ref(ctx.writer(), &branch)?;
                if has_tracked_worktree_changes(ctx)? {
                    ctx.execute_op(&StashPush {
                        message: Some(RESET_HARD_STASH_MSG.into()),
                        include_untracked: false,
                    })?;
                }
            }
            ctx.execute_op(&ResetCommit {
                sha,
                mode: reset_mode,
            })?;
            if force_push && reset_needs_force_push(ctx, &commit_id)? {
                let branch = branch_name_for_backup(ctx.repo_path())?;
                create_backup_ref(ctx.writer(), &branch)?;
                execute_force_push_with_lease(ctx)?;
            }
        }
        WriteRequest::PushForce => {
            let branch = branch_name_for_backup(ctx.repo_path())?;
            create_backup_ref(ctx.writer(), &branch)?;
            execute_force_push_with_lease(ctx)?;
        }
        WriteRequest::Publish { url } => execute_publish(ctx, url.as_deref())?,
    }
    Ok(())
}
