//! Casos de uso de escrita M3 — preview (RF-08) e execução com gates.

mod branch;
mod commit_history;
mod conflict_sequencer;
mod execute;
mod gates;
mod preview;
mod staging;
mod stash_tag;
mod sync_remote;
#[cfg(test)]
mod tests;

pub use execute::execute_write_prevalidated;
pub use preview::preview_write;

use crate::application::GitError;
use crate::infrastructure::validate_repo_relative_path;
#[cfg(test)]
use crate::application::RepoContext;
#[cfg(test)]
use crate::domain::WriteRequest;

/// Extrai o path Git de um rótulo de rename (`old → new`).
fn git_path_from_display(display: &str) -> &str {
    display
        .split_once(" → ")
        .map(|(_, new)| new)
        .unwrap_or(display)
}

fn validate_paths(paths: &[String]) -> Result<Vec<String>, GitError> {
    if paths.is_empty() {
        return Err(GitError::Git("Nenhum arquivo selecionado.".into()));
    }
    paths
        .iter()
        .map(|p| {
            validate_repo_relative_path(git_path_from_display(p))
                .map_err(|e| GitError::Git(e.to_string()))
        })
        .collect()
}

/// Preview (gates) + execução em um passo — usado nos testes deste módulo.
/// A produção passa por `execute_write_operation` (IPC), que revalida o
/// preview UMA vez e chama `execute_write_prevalidated` direto.
#[cfg(test)]
pub fn execute_write(ctx: &RepoContext, req: WriteRequest) -> Result<(), GitError> {
    let preview = preview_write(ctx, ctx.repo_path(), &req)?;
    if let Some(msg) = preview.blocked {
        return Err(GitError::Git(msg));
    }
    execute_write_prevalidated(ctx, req)
}
