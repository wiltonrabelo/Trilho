//! Gates e helpers de histórico — commit, uncommit, revert, cherry-pick,
//! reword e reset.

use super::gates::{
    gate_clean_worktree, gate_force_push_upstream, gate_not_head_commit, gate_sequencer_idle,
    is_ancestor_of_head, is_head_commit,
};
use crate::application::operations::ResetMode;
use crate::application::write_gates::{head_is_local_only, is_commit_on_remote};
use crate::application::{GitCommand, GitError, GitWriter, RepoContext};
use crate::domain::ResetModeDto;
use crate::infrastructure::{
    commit_summary, is_merge_commit, repo_info, validate_git_object_id,
};

pub(super) fn gate_amend(ctx: &RepoContext) -> Result<Option<String>, GitError> {
    if head_is_local_only(ctx.reader(), ctx.writer())? {
        Ok(None)
    } else {
        Ok(Some(
            "O último commit já foi enviado — amend não está disponível no MVP.".into(),
        ))
    }
}

/// Revert de commit de MERGE exige `-m <pai>` (fora do MVP): sem gate, o
/// `git revert` falharia com erro críptico depois da confirmação.
pub(super) fn gate_revert_merge(ctx: &RepoContext, sha: &str) -> Result<Option<String>, GitError> {
    if is_merge_commit(ctx.repo_path(), sha)? {
        return Ok(Some(
            "Este é um commit de MERGE — revertê-lo exige escolher qual lado manter \
             (git revert -m), operação avançada fora do MVP. Reverta os commits \
             individuais da branch mesclada, se necessário."
                .into(),
        ));
    }
    Ok(None)
}

pub(super) fn gate_revert(ctx: &RepoContext, repo_path: &str) -> Result<Option<String>, GitError> {
    if let Some(msg) = gate_sequencer_idle(repo_path, "reverter")? {
        return Ok(Some(msg));
    }
    let status = ctx.reader().get_status()?;
    if !status.staged.is_empty() || !status.unstaged.is_empty() || !status.untracked.is_empty() {
        return Ok(Some(
            "Working tree com alterações — faça commit, stash ou descarte antes de reverter um commit."
                .into(),
        ));
    }
    Ok(None)
}

pub(super) fn resolve_cherry_pick_shas(
    commit_id: &Option<String>,
    commit_ids: &[String],
) -> Result<Vec<String>, GitError> {
    let raw: Vec<String> = if !commit_ids.is_empty() {
        commit_ids.to_vec()
    } else if let Some(id) = commit_id {
        vec![id.clone()]
    } else {
        return Err(GitError::Git(
            "Cherry-pick exige pelo menos um commit.".into(),
        ));
    };
    let mut shas = Vec::with_capacity(raw.len());
    for id in raw {
        shas.push(validate_git_object_id(&id).map_err(|e| GitError::Git(e.to_string()))?);
    }
    Ok(shas)
}

pub(super) fn gate_cherry_pick_shas(
    ctx: &RepoContext,
    repo_path: &str,
    shas: &[String],
) -> Result<Option<String>, GitError> {
    let anchor = shas.first().map(String::as_str).unwrap_or("");
    if let Some(msg) = gate_cherry_pick(ctx, repo_path, anchor)? {
        return Ok(Some(msg));
    }
    for sha in shas {
        if let Some(msg) = gate_cherry_pick_merge(ctx, sha)? {
            return Ok(Some(msg));
        }
        if let Some(msg) = gate_not_head_commit(repo_path, sha, "cherry-pick")? {
            return Ok(Some(msg));
        }
        if let Some(msg) = gate_cherry_pick_foreign(ctx, sha)? {
            return Ok(Some(msg));
        }
    }
    Ok(None)
}

pub(super) fn cherry_pick_description(shas: &[String], record_origin: bool) -> String {
    let mut desc = if shas.len() > 1 {
        format!(
            "Aplica {} commits no topo da branch atual (do mais antigo ao mais recente).",
            shas.len()
        )
    } else {
        "Aplica as alterações do commit selecionado no topo da branch atual.".into()
    };
    if record_origin {
        desc.push_str(" Registra a origem de cada commit na mensagem (-x).");
    }
    desc
}

fn gate_cherry_pick(
    ctx: &RepoContext,
    repo_path: &str,
    _sha: &str,
) -> Result<Option<String>, GitError> {
    if let Some(msg) = gate_sequencer_idle(repo_path, "cherry-pick")? {
        return Ok(Some(msg));
    }
    let status = ctx.reader().get_status()?;
    if !status.staged.is_empty() || !status.unstaged.is_empty() || !status.untracked.is_empty() {
        return Ok(Some(
            "Working tree com alterações — faça commit, stash ou descarte antes do cherry-pick."
                .into(),
        ));
    }
    Ok(None)
}

fn gate_cherry_pick_merge(ctx: &RepoContext, sha: &str) -> Result<Option<String>, GitError> {
    if is_merge_commit(ctx.repo_path(), sha)? {
        let summary =
            commit_summary(ctx.repo_path(), sha)?.unwrap_or_else(|| "merge".to_string());
        return Ok(Some(format!(
            "«{summary}» ({:.7}) é commit de merge — cherry-pick exige escolher qual lado manter \
             (git cherry-pick -m), operação avançada fora do MVP. Desmarque merges e escolha só \
             commits normais.",
            sha
        )));
    }
    Ok(None)
}

/// Cherry-pick só faz sentido para commits fora do histórico da branch atual.
fn gate_cherry_pick_foreign(ctx: &RepoContext, sha: &str) -> Result<Option<String>, GitError> {
    if is_ancestor_of_head(ctx.writer(), sha)? {
        return Ok(Some(
            "Este commit já faz parte do histórico da branch atual — use cherry-pick para trazer \
             commits de outras branches que ainda não estão aqui."
                .into(),
        ));
    }
    Ok(None)
}

pub(super) fn format_reword_message(summary: &str, body: Option<&str>) -> String {
    let summary = summary.trim();
    match body.map(str::trim).filter(|b| !b.is_empty()) {
        Some(b) => format!("{summary}\n\n{b}"),
        None => summary.to_string(),
    }
}

pub(super) fn gate_reword(
    ctx: &RepoContext,
    repo_path: &str,
    sha: &str,
    summary: &str,
    force_push: bool,
) -> Result<Option<String>, GitError> {
    if summary.trim().is_empty() {
        return Ok(Some("A mensagem do commit não pode ficar vazia.".into()));
    }
    if let Some(msg) = gate_clean_worktree(ctx)? {
        return Ok(Some(msg));
    }
    if is_merge_commit(repo_path, sha)? {
        return Ok(Some(
            "Commit de merge — reword exige operação avançada fora do MVP.".into(),
        ));
    }
    // Reword reaplica com cherry-pick linear; merges no caminho trazem
    // commits laterais e costumam gerar conflito (ex.: «could not apply …»).
    // No HEAD o intervalo é vazio — esta checagem não bloqueia.
    if range_has_merge_commits(ctx.writer(), sha)? {
        return Ok(Some(
            "Há merges no histórico após este commit — o Trilho ainda não reaplica \
             merges no reword. Escolha um commit mais recente (após o último merge) \
             ou reescreva a mensagem só em histórico linear."
                .into(),
        ));
    }
    let on_remote = reword_target_on_remote(ctx, sha)?;
    if on_remote {
        if !force_push {
            return Ok(Some(
                "Este commit já foi enviado ao remoto — confirme o push forçado para concluir o reword."
                    .into(),
            ));
        }
        if let Some(msg) = gate_force_push_upstream(ctx)? {
            return Ok(Some(msg));
        }
    }
    Ok(None)
}

pub(super) fn reword_target_on_remote(ctx: &RepoContext, sha: &str) -> Result<bool, GitError> {
    let sync = ctx.reader().get_sync_info()?;
    let Some(upstream) = sync.upstream.as_deref() else {
        return Ok(false);
    };
    is_commit_on_remote(ctx.writer(), upstream, sha)
}

/// `true` se existir algum merge em `sha..HEAD` (histórico não linear).
fn range_has_merge_commits(cli: &dyn GitWriter, sha: &str) -> Result<bool, GitError> {
    let out = cli.run(&GitCommand {
        args: vec![
            "rev-list".into(),
            "--merges".into(),
            "--count".into(),
            format!("{sha}..HEAD"),
        ],
    })?;
    let count: u64 = out.trim().parse().unwrap_or(0);
    Ok(count > 0)
}

pub(super) fn reset_mode_from_dto(mode: ResetModeDto) -> ResetMode {
    match mode {
        ResetModeDto::Soft => ResetMode::Soft,
        ResetModeDto::Mixed => ResetMode::Mixed,
        ResetModeDto::Hard => ResetMode::Hard,
    }
}

pub(super) const RESET_HARD_STASH_MSG: &str = "trilho: backup antes de reset --hard";

/// `true` se o upstream aponta para um commit estritamente posterior ao alvo do reset.
pub(super) fn reset_needs_force_push(ctx: &RepoContext, sha: &str) -> Result<bool, GitError> {
    let sync = ctx.reader().get_sync_info()?;
    let Some(upstream) = sync.upstream else {
        return Ok(false);
    };
    let upstream_sha = ctx
        .writer()
        .run(&GitCommand {
            args: vec!["rev-parse".into(), upstream.clone()],
        })?
        .trim()
        .to_string();
    if upstream_sha.eq_ignore_ascii_case(sha) {
        return Ok(false);
    }
    is_commit_on_remote(ctx.writer(), &upstream, sha)
}

pub(super) fn gate_reset(
    ctx: &RepoContext,
    repo_path: &str,
    sha: &str,
    _mode: ResetMode,
    force_push: bool,
    needs_force: bool,
) -> Result<Option<String>, GitError> {
    let info = repo_info(repo_path)?;
    if info.is_detached {
        return Ok(Some(
            "Repositório em detached HEAD — troque para uma branch antes de resetar.".into(),
        ));
    }
    if let Some(msg) = gate_sequencer_idle(repo_path, "resetar")? {
        return Ok(Some(msg));
    }
    if is_head_commit(repo_path, sha)? {
        return Ok(Some(
            "Este já é o último commit (HEAD) — escolha um commit anterior para resetar.".into(),
        ));
    }
    if !is_ancestor_of_head(ctx.writer(), sha)? {
        return Ok(Some(
            "Este commit não faz parte do histórico atual da branch — só é possível resetar \
             para commits ancestrais do HEAD."
                .into(),
        ));
    }
    if needs_force && force_push {
        if let Some(msg) = gate_force_push_upstream(ctx)? {
            return Ok(Some(msg));
        }
    }
    Ok(None)
}

pub(super) fn gate_uncommit(ctx: &RepoContext) -> Result<Option<String>, GitError> {
    if head_is_local_only(ctx.reader(), ctx.writer())? {
        Ok(None)
    } else {
        Ok(Some(
            "O último commit já foi enviado — uncommit só vale para commits locais.".into(),
        ))
    }
}
