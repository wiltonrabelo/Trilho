//! PAT do GitHub para API REST (PRs) — persistido no app data, independente do GCM/SSH.

use std::path::Path;
use std::sync::{Mutex, OnceLock};

use crate::application::GitError;

const FILE_NAME: &str = "github_api_pat";

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
    let path = pat_path(data_dir);
    std::fs::write(&path, pat.as_bytes()).map_err(|e| GitError::Io(e.to_string()))?;
    set_session_pat(Some(pat.to_string()));
    Ok(())
}

pub fn load_pat_file(data_dir: &Path) -> Option<String> {
    if let Some(cached) = session_pat_token() {
        return Some(cached);
    }
    let path = pat_path(data_dir);
    let pat = std::fs::read_to_string(&path).ok()?.trim().to_string();
    if pat.is_empty() {
        return None;
    }
    set_session_pat(Some(pat.clone()));
    Some(pat)
}

pub fn clear_pat_file(data_dir: &Path) -> Result<(), GitError> {
    set_session_pat(None);
    let path = pat_path(data_dir);
    if path.exists() {
        std::fs::remove_file(&path).map_err(|e| GitError::Io(e.to_string()))?;
    }
    Ok(())
}
