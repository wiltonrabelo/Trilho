//! Watcher seletivo do repositório (RF-19).
//!
//! Observa apenas:
//! - metadados Git: HEAD, index, packed-refs, refs/**
//! - worktree com filtros (ignora .git/objects, node_modules, target, …)

use git2::Repository;
use notify::{Config, Event, RecommendedWatcher, RecursiveMode, Watcher};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::{mpsc, Arc};
use std::thread;
use std::time::Duration;
use tauri::{AppHandle, Emitter};

const DEBOUNCE_MS: u64 = 400;

pub struct RepoWatcher {
    stop: mpsc::Sender<()>,
    handle: thread::JoinHandle<()>,
}

impl RepoWatcher {
    pub fn start(
        repo_path: String,
        app: AppHandle,
        suppress: Arc<AtomicU32>,
        watch_pending: Arc<AtomicBool>,
    ) -> Result<Self, String> {
        let (git_dir, worktree) = resolve_git_paths(&repo_path)?;
        let (stop_tx, stop_rx) = mpsc::channel();

        let handle = thread::spawn(move || {
            let (event_tx, event_rx) = mpsc::channel();
            let mut watcher = match RecommendedWatcher::new(
                move |res| {
                    if let Ok(event) = res {
                        let _ = event_tx.send(event);
                    }
                },
                Config::default(),
            ) {
                Ok(w) => w,
                Err(e) => {
                    eprintln!("watcher init failed: {e}");
                    return;
                }
            };

            register_selective_watches(&mut watcher, &git_dir, &worktree);

            let mut pending = false;
            loop {
                if stop_rx.try_recv().is_ok() {
                    break;
                }

                match event_rx.recv_timeout(Duration::from_millis(100)) {
                    Ok(event) if is_relevant_event(&event, &worktree, &git_dir) => {
                        pending = true;
                    }
                    Ok(_) => {}
                    Err(mpsc::RecvTimeoutError::Timeout) => {}
                    Err(mpsc::RecvTimeoutError::Disconnected) => break,
                }

                if pending {
                    thread::sleep(Duration::from_millis(DEBOUNCE_MS));
                    pending = false;
                    while event_rx.try_recv().is_ok() {}

                    if suppress.load(Ordering::SeqCst) > 0 {
                        watch_pending.store(true, Ordering::SeqCst);
                        continue;
                    }

                    let _ = app.emit("repo-changed", ());
                }
            }
        });

        Ok(Self {
            stop: stop_tx,
            handle,
        })
    }

    pub fn stop(self) {
        let _ = self.stop.send(());
        let _ = self.handle.join();
    }
}

fn resolve_git_paths(repo_path: &str) -> Result<(PathBuf, PathBuf), String> {
    let repo = Repository::discover(repo_path)
        .map_err(|_| "Não foi possível localizar o diretório .git.".to_string())?;
    let git_dir = repo.path().to_path_buf();
    let worktree = repo
        .workdir()
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| PathBuf::from(repo_path));
    Ok((git_dir, worktree))
}

fn register_selective_watches(
    watcher: &mut RecommendedWatcher,
    git_dir: &Path,
    worktree: &Path,
) {
    for file in ["HEAD", "index", "packed-refs"] {
        let p = git_dir.join(file);
        if p.exists() {
            let _ = watcher.watch(&p, RecursiveMode::NonRecursive);
        }
    }
    let refs = git_dir.join("refs");
    if refs.exists() {
        let _ = watcher.watch(&refs, RecursiveMode::Recursive);
    }
    if worktree.exists() {
        let _ = watcher.watch(worktree, RecursiveMode::Recursive);
    }
}

/// Filtra eventos do worktree; metadados em `.git/refs` etc. passam direto.
pub fn is_relevant_event(event: &Event, worktree: &Path, git_dir: &Path) -> bool {
    event.paths.iter().any(|p| is_relevant_path(p, worktree, git_dir))
}

fn is_relevant_path(path: &Path, worktree: &Path, git_dir: &Path) -> bool {
    let normalized = path.to_string_lossy().replace('\\', "/");
    let lower = normalized.to_lowercase();

    // Metadados Git permitidos
    if lower.ends_with("/head") || lower.ends_with("/index") || lower.ends_with("/packed-refs") {
        return true;
    }
    if lower.contains("/.git/refs/") {
        return true;
    }

    // Ignora resto de .git (objects, logs, …)
    if lower.contains("/.git/") {
        return false;
    }

    // Ignora diretórios pesados comuns
    for skip in [
        "/node_modules/",
        "/target/",
        "/dist/",
        "/.cargo/",
        "/__history/",
    ] {
        if lower.contains(skip) {
            return false;
        }
    }

    // Worktree: deve estar dentro da raiz
    path.starts_with(worktree) || path.starts_with(git_dir)
}

#[cfg(test)]
mod tests {
    use super::*;
    use notify::EventKind;

    #[test]
    fn ignora_objects_e_node_modules() {
        let wt = PathBuf::from("C:/repo");
        let git = PathBuf::from("C:/repo/.git");
        assert!(!is_relevant_path(
            &PathBuf::from("C:/repo/.git/objects/ab/cd"),
            &wt,
            &git
        ));
        assert!(!is_relevant_path(
            &PathBuf::from("C:/repo/node_modules/foo"),
            &wt,
            &git
        ));
        assert!(is_relevant_path(
            &PathBuf::from("C:/repo/.git/HEAD"),
            &wt,
            &git
        ));
        assert!(is_relevant_path(
            &PathBuf::from("C:/repo/src/main.rs"),
            &wt,
            &git
        ));
    }

    #[test]
    fn evento_filtra_paths_irrelevantes() {
        let wt = PathBuf::from("C:/repo");
        let git = PathBuf::from("C:/repo/.git");
        let event = Event {
            kind: EventKind::Modify(notify::event::ModifyKind::Data(
                notify::event::DataChange::Any,
            )),
            paths: vec![
                PathBuf::from("C:/repo/node_modules/x"),
                PathBuf::from("C:/repo/src/app.rs"),
            ],
            attrs: notify::event::EventAttributes::new(),
        };
        assert!(is_relevant_event(&event, &wt, &git));
    }
}
