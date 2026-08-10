//! PAT do GitHub para API REST (PRs) — Credential Manager via `git credential`
//! (mesmo padrão das chaves LLM). Migra e remove o arquivo legado em texto puro.

use std::path::Path;
use std::sync::{Mutex, OnceLock};

use crate::application::GitError;
use super::llm_credentials::{
    clear_llm_api_key, get_llm_api_key, has_llm_api_key, store_llm_api_key,
};

const FILE_NAME: &str = "github_api_pat";
/// Namespace no GCM — distinto de `trilho.llm.*`.
const CRED_PROVIDER: &str = "github.api";

fn session_pat() -> &'static Mutex<Option<String>> {
    static STORE: OnceLock<Mutex<Option<String>>> = OnceLock::new();
    STORE.get_or_init(|| Mutex::new(None))
}

fn pat_path(data_dir: &Path) -> std::path::PathBuf {
    data_dir.join(FILE_NAME)
}

pub fn set_session_pat(pat: Option<String>) {
    if let Ok(mut guard) = session_pat().lock() {
        *guard = pat.filter(|p| !p.trim().is_empty());
    }
}

pub fn session_pat_token() -> Option<String> {
    session_pat()
        .lock()
        .ok()
        .and_then(|g| g.clone())
        .filter(|p| !p.is_empty())
}

pub fn save_pat_file(data_dir: &Path, pat: &str) -> Result<(), GitError> {
    let pat = pat.trim();
    if pat.is_empty() {
        return Err(GitError::Io("PAT vazio.".into()));
    }
    store_llm_api_key(CRED_PROVIDER, pat).map_err(GitError::Io)?;
    set_session_pat(Some(pat.to_string()));
    // Apaga legado em texto puro, se existir.
    let path = pat_path(data_dir);
    if path.exists() {
        let _ = std::fs::remove_file(&path);
    }
    Ok(())
}

pub fn load_pat_file(data_dir: &Path) -> Option<String> {
    if let Some(cached) = session_pat_token() {
        return Some(cached);
    }
    if let Some(pat) = get_llm_api_key(CRED_PROVIDER).filter(|p| !p.is_empty()) {
        set_session_pat(Some(pat.clone()));
        return Some(pat);
    }
    // Migração única: arquivo legado → GCM.
    let path = pat_path(data_dir);
    let pat = std::fs::read_to_string(&path).ok()?.trim().to_string();
    if pat.is_empty() {
        return None;
    }
    if store_llm_api_key(CRED_PROVIDER, &pat).is_ok() {
        let _ = std::fs::remove_file(&path);
    }
    set_session_pat(Some(pat.clone()));
    Some(pat)
}

pub fn clear_pat_file(data_dir: &Path) -> Result<(), GitError> {
    set_session_pat(None);
    let _ = clear_llm_api_key(CRED_PROVIDER);
    let path = pat_path(data_dir);
    if path.exists() {
        std::fs::remove_file(&path).map_err(|e| GitError::Io(e.to_string()))?;
    }
    Ok(())
}

pub fn has_pat_stored(data_dir: &Path) -> bool {
    session_pat_token().is_some()
        || has_llm_api_key(CRED_PROVIDER)
        || pat_path(data_dir).is_file()
}
