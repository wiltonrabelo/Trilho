//! RF-11 — mapeia WriteRequest → entrada de auditoria e grava no log.

use crate::application::{GitError, RepoContext};
use crate::domain::{
    AuditAction, AuditEntry, AuditResult, OperationPreview, WriteRequest,
};
use crate::infrastructure::{append_audit_entry, now_timestamp};

/// Ações do PLANO RF-11 (add/commit/push/reset/revert + cherry-pick/reword).
pub fn audit_action_for(req: &WriteRequest) -> Option<AuditAction> {
    match req {
        WriteRequest::Stage { .. }
        | WriteRequest::StageMany { .. }
        | WriteRequest::StageAll => Some(AuditAction::Add),
        WriteRequest::Commit { .. } => Some(AuditAction::Commit),
        WriteRequest::Push | WriteRequest::Publish { .. } => Some(AuditAction::Push),
        WriteRequest::PushForce => Some(AuditAction::PushForce),
        WriteRequest::Reset { .. } => Some(AuditAction::Reset),
        WriteRequest::Revert { .. }
        | WriteRequest::ContinueRevert
        | WriteRequest::SkipRevert => Some(AuditAction::Revert),
        WriteRequest::CherryPick { .. }
        | WriteRequest::ContinueCherryPick
        | WriteRequest::SkipCherryPick => Some(AuditAction::CherryPick),
        WriteRequest::Reword { .. } => Some(AuditAction::Reword),
        _ => None,
    }
}

fn commits_for(req: &WriteRequest) -> Vec<String> {
    match req {
        WriteRequest::Revert { commit_id } => vec![commit_id.clone()],
        WriteRequest::CherryPick {
            commit_id,
            commit_ids,
            ..
        } => {
            if !commit_ids.is_empty() {
                commit_ids.clone()
            } else if let Some(id) = commit_id {
                vec![id.clone()]
            } else {
                vec![]
            }
        }
        WriteRequest::Reword { commit_id, .. } | WriteRequest::Reset { commit_id, .. } => {
            vec![commit_id.clone()]
        }
        _ => vec![],
    }
}

fn branch_name(ctx: &RepoContext) -> Option<String> {
    repo_info_branch(ctx.repo_path())
}

fn repo_info_branch(repo_path: &str) -> Option<String> {
    crate::infrastructure::repo_info(repo_path)
        .ok()
        .and_then(|i| i.branch)
}

pub fn record_write_outcome(
    app_data_dir: &std::path::Path,
    ctx: &RepoContext,
    req: &WriteRequest,
    preview: &OperationPreview,
    outcome: Result<(), &GitError>,
) {
    let Some(action) = audit_action_for(req) else {
        return;
    };
    let (result, error) = match outcome {
        Ok(()) => (AuditResult::Success, None),
        Err(e) => (AuditResult::Error, Some(e.to_string())),
    };
    let entry = AuditEntry {
        timestamp: now_timestamp(),
        action,
        command: preview.commands.join("\n"),
        repo: ctx.repo_path().to_string(),
        branch: branch_name(ctx),
        commits: commits_for(req),
        result,
        error,
        from_assistant: false,
    };
    let _ = append_audit_entry(app_data_dir, entry);
}
