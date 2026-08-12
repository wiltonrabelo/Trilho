//! Gates e helpers de stash e tags.

use crate::application::operations::{CreateTag, PushTag};
use crate::application::{GitError, RepoContext};
use crate::infrastructure::{
    list_stashes, repo_info, validate_git_object_id, validate_tag_name, SafeGitCli,
};

pub(super) fn gate_stash(
    ctx: &RepoContext,
    include_untracked: bool,
) -> Result<Option<String>, GitError> {
    let status = ctx.reader().get_status()?;
    let has_tracked = !status.staged.is_empty() || !status.unstaged.is_empty();
    let has_untracked = !status.untracked.is_empty();
    if !has_tracked && (!include_untracked || !has_untracked) {
        return Ok(Some("Não há alterações para guardar no stash.".into()));
    }
    Ok(None)
}

pub(super) fn stash_effect_description(
    ctx: &RepoContext,
    include_untracked: bool,
) -> Result<String, GitError> {
    let status = ctx.reader().get_status()?;
    let mut parts = Vec::new();
    if !status.staged.is_empty() {
        parts.push(format!("{} em stage", status.staged.len()));
    }
    if !status.unstaged.is_empty() {
        parts.push(format!("{} não staged", status.unstaged.len()));
    }
    if include_untracked && !status.untracked.is_empty() {
        parts.push(format!("{} não rastreados", status.untracked.len()));
    }
    let detail = if parts.is_empty() {
        "alterações rastreadas".to_string()
    } else {
        parts.join(", ")
    };
    Ok(format!(
        "Guarda {detail} em uma pilha temporária (stash). A working tree ficará limpa."
    ))
}

pub(super) fn gate_stash_restore(ctx: &RepoContext) -> Result<Option<String>, GitError> {
    let status = ctx.reader().get_status()?;
    if !status.staged.is_empty() || !status.unstaged.is_empty() || !status.untracked.is_empty() {
        return Ok(Some(
            "Working tree com alterações — commit, stash ou descarte antes de reaplicar.".into(),
        ));
    }
    Ok(None)
}

pub(super) fn gate_stash_index(repo_path: &str, index: usize) -> Result<Option<String>, GitError> {
    let stashes = list_stashes(repo_path)?;
    if stashes.iter().any(|s| s.index == index) {
        Ok(None)
    } else {
        Ok(Some("Stash não encontrado — a pilha pode ter mudado.".into()))
    }
}

pub(super) fn stash_label(repo_path: &str, index: usize) -> String {
    list_stashes(repo_path)
        .ok()
        .and_then(|ss| {
            ss.into_iter()
                .find(|s| s.index == index)
                .map(|s| s.message)
        })
        .unwrap_or_else(|| format!("stash@{{{index}}}"))
}

fn gate_tag_exists(repo_path: &str, name: &str) -> Result<Option<String>, GitError> {
    let cli = SafeGitCli::new(repo_path);
    let op = crate::application::GitCommand {
        args: vec![
            "show-ref".into(),
            "--verify".into(),
            "--quiet".into(),
            format!("refs/tags/{name}"),
        ],
    };
    if cli.run_bool(&op)? {
        Ok(Some(format!(
            "Já existe uma tag «{name}» neste repositório."
        )))
    } else {
        Ok(None)
    }
}

fn gate_push_tag(ctx: &RepoContext) -> Result<Option<String>, GitError> {
    let info = repo_info(ctx.repo_path())?;
    if !info.has_remote {
        return Ok(Some(
            "Não há remoto configurado — desmarque «Enviar ao remoto».".into(),
        ));
    }
    Ok(None)
}

pub(super) fn gate_tag_missing(repo_path: &str, name: &str) -> Result<Option<String>, GitError> {
    match gate_tag_exists(repo_path, name)? {
        Some(_) => Ok(None),
        None => Ok(Some(format!("Tag «{name}» não encontrada."))),
    }
}

pub(super) fn preview_create_tag(
    ctx: &RepoContext,
    repo_path: &str,
    name: &str,
    commit_id: &str,
    annotated: bool,
    message: Option<&str>,
    push_to_remote: bool,
) -> Result<(Vec<String>, String, Option<String>), GitError> {
    let name = match validate_tag_name(name) {
        Ok(n) => n,
        Err(GitError::Git(msg)) => return Ok((vec![], String::new(), Some(msg))),
        Err(e) => return Err(e),
    };
    let commit_id = match validate_git_object_id(commit_id) {
        Ok(id) => id,
        Err(GitError::Git(msg)) => return Ok((vec![], String::new(), Some(msg))),
        Err(e) => return Err(e),
    };

    let mut blocked = gate_tag_exists(repo_path, &name)?;
    if annotated && message.map(str::trim).filter(|m| !m.is_empty()).is_none() {
        blocked = blocked.or(Some(
            "Tags anotadas precisam de uma mensagem.".into(),
        ));
    }

    let tag_op = CreateTag {
        name: name.clone(),
        commit_id: commit_id.clone(),
        annotated,
        message: message.map(str::to_string),
    };
    let mut commands = ctx.preview_op(&tag_op);
    let kind = if annotated { "anotada" } else { "leve" };
    let short: String = commit_id.chars().take(7).collect();
    let mut description = format!("Cria tag {kind} «{name}» no commit {short}.");

    if push_to_remote {
        blocked = blocked.or(gate_push_tag(ctx)?);
        let push_op = PushTag {
            remote: "origin".into(),
            name: name.clone(),
        };
        commands.extend(ctx.preview_op(&push_op));
        description.push_str(" Em seguida envia a tag ao remoto origin.");
    }

    Ok((commands, description, blocked))
}
