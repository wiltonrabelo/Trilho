//! Consultas pontuais ao repositório via libgit2 — mantêm o `git2` fora da
//! camada de aplicação, que só enxerga tipos do domínio (`String`, `bool`).

use crate::application::GitError;
use git2::{Oid, Repository};

fn open(repo_path: &str) -> Result<Repository, GitError> {
    Repository::discover(repo_path).map_err(|e| GitError::Io(e.to_string()))
}

fn parse_oid(sha: &str) -> Result<Oid, GitError> {
    Oid::from_str(sha).map_err(|e| GitError::Git(e.to_string()))
}

/// Resumo (primeira linha) do commit — `None` quando a mensagem é vazia.
pub fn commit_summary(repo_path: &str, sha: &str) -> Result<Option<String>, GitError> {
    let repo = open(repo_path)?;
    let commit = repo
        .find_commit(parse_oid(sha)?)
        .map_err(|_| GitError::Git("Commit não encontrado no repositório.".into()))?;
    Ok(commit.summary().map(str::to_string))
}

/// `true` se o commit tem mais de um pai (merge) — gate de revert/cherry-pick/reword.
pub fn is_merge_commit(repo_path: &str, sha: &str) -> Result<bool, GitError> {
    let repo = open(repo_path)?;
    let commit = repo
        .find_commit(parse_oid(sha)?)
        .map_err(|_| GitError::Git("Commit não encontrado no repositório.".into()))?;
    Ok(commit.parent_count() > 1)
}

/// Remoto principal: «origin» quando existe, senão o primeiro configurado.
pub fn primary_remote(repo_path: &str) -> Result<String, GitError> {
    let repo = open(repo_path)?;
    let remotes = repo.remotes().map_err(|e| GitError::Io(e.to_string()))?;
    for i in 0..remotes.len() {
        if remotes.get(i) == Some("origin") {
            return Ok("origin".into());
        }
    }
    remotes
        .get(0)
        .map(|s| s.to_string())
        .ok_or_else(|| GitError::Git("Nenhum remoto configurado.".into()))
}

/// Id do commit apontado por HEAD.
pub fn head_commit_id(repo_path: &str) -> Result<String, GitError> {
    let repo = open(repo_path)?;
    repo.head()
        .ok()
        .and_then(|h| h.target())
        .map(|oid| oid.to_string())
        .ok_or_else(|| GitError::Git("Repositório sem HEAD.".into()))
}

/// Id do commit para o qual uma revisão aponta (`None` se não resolve).
pub fn resolve_commit_id(repo_path: &str, rev: &str) -> Result<Option<String>, GitError> {
    let repo = open(repo_path)?;
    Ok(repo
        .revparse_single(rev)
        .ok()
        .and_then(|obj| obj.peel_to_commit().ok())
        .map(|commit| commit.id().to_string()))
}
