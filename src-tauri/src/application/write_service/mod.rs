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
use crate::domain::{caminho_git_do_rotulo, caminhos_git_do_rotulo};
use crate::infrastructure::validate_repo_relative_path;
#[cfg(test)]
use crate::application::RepoContext;
#[cfg(test)]
use crate::domain::WriteRequest;

pub(super) const MSG_SELECAO_VAZIA: &str = "Nenhum arquivo selecionado.";

/// Resultado da validação de uma lista de paths. Seleção vazia não é falha:
/// no preview vira bloqueio; só na execução vira erro.
pub(super) enum PathsValidados {
    Validos(Vec<String>),
    SelecaoVazia,
}

fn validate_paths(paths: &[String]) -> Result<PathsValidados, GitError> {
    if paths.is_empty() {
        return Ok(PathsValidados::SelecaoVazia);
    }
    let validos = paths
        .iter()
        .map(|p| {
            validate_repo_relative_path(caminho_git_do_rotulo(p))
                .map_err(|e| GitError::Git(e.to_string()))
        })
        .collect::<Result<Vec<String>, GitError>>()?;
    Ok(PathsValidados::Validos(validos))
}

/// Igual a `validate_paths`, mas para a execução, onde seleção vazia já é erro
/// (o preview deveria ter bloqueado antes).
fn validate_paths_obrigatorios(paths: &[String]) -> Result<Vec<String>, GitError> {
    match validate_paths(paths)? {
        PathsValidados::Validos(p) => Ok(p),
        PathsValidados::SelecaoVazia => Err(GitError::Git(MSG_SELECAO_VAZIA.into())),
    }
}

/// Caminhos que saem do índice para um rótulo: dois quando é renomeação.
fn validate_paths_de_indice(rotulo: &str) -> Result<Vec<String>, GitError> {
    caminhos_git_do_rotulo(rotulo)
        .into_iter()
        .map(|p| validate_repo_relative_path(p).map_err(|e| GitError::Git(e.to_string())))
        .collect()
}

/// Como `validate_paths`, mas expandindo renomeações — usado no unstage, que
/// mexe no índice e precisa citar origem e destino.
fn validate_paths_de_indice_muitos(paths: &[String]) -> Result<PathsValidados, GitError> {
    if paths.is_empty() {
        return Ok(PathsValidados::SelecaoVazia);
    }
    let mut validos = Vec::with_capacity(paths.len());
    for rotulo in paths {
        validos.extend(validate_paths_de_indice(rotulo)?);
    }
    Ok(PathsValidados::Validos(validos))
}

/// Versão de execução: seleção vazia já é erro.
fn validate_paths_de_indice_obrigatorios(paths: &[String]) -> Result<Vec<String>, GitError> {
    match validate_paths_de_indice_muitos(paths)? {
        PathsValidados::Validos(p) => Ok(p),
        PathsValidados::SelecaoVazia => Err(GitError::Git(MSG_SELECAO_VAZIA.into())),
    }
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
