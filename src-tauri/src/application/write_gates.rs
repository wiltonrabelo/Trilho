//! Gates de segurança para operações de escrita (RF-06, RF-07, RF-15).

use crate::application::{GitCommand, GitError, GitReader};
use crate::infrastructure::{head_commit_id, resolve_commit_id, SafeGitCli};

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
    let head = head_commit_id(repo_path)?;
    if resolve_commit_id(repo_path, &upstream)? == Some(head.clone()) {
        return Ok(false);
    }
    is_commit_on_remote(cli, &upstream, &head).map(|on| !on)
}
