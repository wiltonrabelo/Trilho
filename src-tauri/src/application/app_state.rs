//! Estado compartilhado da aplicação (repositório aberto, sync, watcher).

use crate::infrastructure::RepoWatcher;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::{Arc, Mutex};
use tauri::{AppHandle, Emitter, Manager};

const MAX_RECENT: usize = 8;

pub struct AppState {
    repo_path: Mutex<Option<String>>,
    last_fetch_at: Mutex<Option<String>>,
    suppress_watch: Arc<AtomicU32>,
    watch_pending: Arc<AtomicBool>,
    recent_repos: Mutex<Vec<String>>,
    recents_file: PathBuf,
    watcher: Mutex<Option<RepoWatcher>>,
}

impl AppState {
    pub fn new(app: &AppHandle) -> Result<Self, String> {
        let data_dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
        std::fs::create_dir_all(&data_dir).map_err(|e| e.to_string())?;
        let recents_file = data_dir.join("recent_repos.json");
        let recent_repos = load_recents(&recents_file);

        Ok(Self {
            repo_path: Mutex::new(None),
            last_fetch_at: Mutex::new(None),
            suppress_watch: Arc::new(AtomicU32::new(0)),
            watch_pending: Arc::new(AtomicBool::new(false)),
            recent_repos: Mutex::new(recent_repos),
            recents_file,
            watcher: Mutex::new(None),
        })
    }

    pub fn repo_path(&self) -> Result<String, String> {
        self.repo_path
            .lock()
            .map_err(|_| "Estado indisponível.".into())
            .and_then(|guard| {
                guard
                    .clone()
                    .ok_or_else(|| "Nenhum repositório aberto.".into())
            })
    }

    pub fn set_repo(&self, path: String, app: &AppHandle) -> Result<(), String> {
        validate_git_repo(&path)?;
        self.push_recent(&path)?;
        self.save_recents()?;

        if let Ok(mut watcher) = self.watcher.lock() {
            if let Some(w) = watcher.take() {
                w.stop();
            }
            *watcher = Some(RepoWatcher::start(
                path.clone(),
                app.clone(),
                Arc::clone(&self.suppress_watch),
                Arc::clone(&self.watch_pending),
            )?);
        }

        if let Ok(mut repo) = self.repo_path.lock() {
            *repo = Some(path);
        }
        if let Ok(mut fetch) = self.last_fetch_at.lock() {
            *fetch = None;
        }
        Ok(())
    }

    pub fn clear_repo(&self) {
        if let Ok(mut watcher) = self.watcher.lock() {
            if let Some(w) = watcher.take() {
                w.stop();
            }
        }
        if let Ok(mut repo) = self.repo_path.lock() {
            *repo = None;
        }
    }

    pub fn recent_repos(&self) -> Vec<String> {
        self.recent_repos
            .lock()
            .map(|r| r.clone())
            .unwrap_or_default()
    }

    pub fn validate_path(path: &str) -> Result<(), String> {
        validate_git_repo(path)
    }

    pub fn last_fetch_at(&self) -> Option<String> {
        self.last_fetch_at.lock().ok().and_then(|g| g.clone())
    }

    pub fn set_last_fetch_at(&self, iso: String) {
        if let Ok(mut guard) = self.last_fetch_at.lock() {
            *guard = Some(iso);
        }
    }

    pub fn with_watch_suppressed<F, T>(&self, app: &AppHandle, f: F) -> T
    where
        F: FnOnce() -> T,
    {
        self.suppress_watch.fetch_add(1, Ordering::SeqCst);
        let result = f();
        self.suppress_watch.fetch_sub(1, Ordering::SeqCst);
        // Reconciliação RF-19: mudanças externas durante a janela suprimida.
        if self.watch_pending.swap(false, Ordering::SeqCst) {
            let _ = app.emit("repo-changed", ());
        }
        result
    }

    fn push_recent(&self, path: &str) -> Result<(), String> {
        let mut recents = self
            .recent_repos
            .lock()
            .map_err(|_| "Estado indisponível.".to_string())?;
        recents.retain(|p| p != path);
        recents.insert(0, path.to_string());
        recents.truncate(MAX_RECENT);
        Ok(())
    }

    fn save_recents(&self) -> Result<(), String> {
        let recents = self
            .recent_repos
            .lock()
            .map_err(|_| "Estado indisponível.".to_string())?;
        let json = serde_json::to_string_pretty(&*recents).map_err(|e| e.to_string())?;
        std::fs::write(&self.recents_file, json).map_err(|e| e.to_string())
    }
}

fn load_recents(path: &PathBuf) -> Vec<String> {
    std::fs::read_to_string(path)
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default()
}

fn validate_git_repo(path: &str) -> Result<(), String> {
    let p = PathBuf::from(path);
    if !p.is_dir() {
        return Err("O caminho informado não é uma pasta.".into());
    }
    if p.join(".git").is_dir() || p.join(".git").is_file() {
        return Ok(());
    }
    // tentativa via git2 discover
    git2::Repository::discover(&p).map_err(|_| {
        "Esta pasta não é um repositório Git. Escolha uma pasta que contenha .git.".to_string()
    })?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validate_path_rejeita_pasta_sem_git() {
        let dir = std::env::temp_dir().join(format!("trilho-nogit-{}", std::process::id()));
        let _ = std::fs::create_dir_all(&dir);
        let err = validate_git_repo(dir.to_str().unwrap()).expect_err("sem git");
        assert!(err.contains("repositório Git"));
        let _ = std::fs::remove_dir_all(&dir);
    }
}
