//! Gates de segurança para operações de escrita (RF-06, RF-07, RF-15).

use crate::application::{GitCommand, GitError, GitReader};
use crate::infrastructure::SafeGitCli;
use git2::Repository;

/// `true` se `sha` já está no remote-tracking ref (commit enviado).
pub fn is_commit_on_remote(
    cli: &SafeGitCli,
    upstream_ref: &str,
    sha: &str,
) -> Result<bool, GitError> {
    let op = GitCommand {
        args: vec![
            "merge-base".into(),
            "--is-ancestor".into(),
            sha.into(),
            upstream_ref.into(),
        ],
    };
    // run_bool distingue "não é ancestral" (exit 1) de ERRO real (exit ≥128):
    // erro propaga e o gate permanece FECHADO — nunca falha-aberto.
    cli.run_bool(&op)
}

/// HEAD ainda não enviado ao upstream (pode amend/uncommit).
pub fn head_is_local_only(reader: &dyn GitReader, cli: &SafeGitCli) -> Result<bool, GitError> {
    let sync = reader.get_sync_info()?;
    let Some(upstream) = sync.upstream else {
        // Sem upstream: trata como local (não há remoto para comparar).
        return Ok(true);
    };
    let repo_path = cli.repo_path();
    let repo = Repository::discover(repo_path).map_err(|e| GitError::Io(e.to_string()))?;
    let head = repo
        .head()
        .ok()
        .and_then(|h| h.target())
        .ok_or(GitError::Git("Repositório sem HEAD.".into()))?;
    let upstream_oid = repo
        .revparse_single(&upstream)
        .ok()
        .and_then(|o| o.peel_to_commit().ok())
        .map(|c| c.id());
    if upstream_oid == Some(head) {
        return Ok(false);
    }
    is_commit_on_remote(cli, &upstream, &head.to_string()).map(|on| !on)
}
