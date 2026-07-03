//! Comandos IPC expostos ao frontend.

use crate::application::{AppState, GitCommand, GitError, GitReader};
use crate::domain::{Commit, RepoInfo, RepoStatus, SyncInfo};
use crate::infrastructure::{validate_git_object_id, validate_repo_relative_path, repo_info, Git2Reader, MockGitReader, SafeGitCli};
use chrono::Utc;
use serde::Serialize;
use tauri::{AppHandle, Emitter, State};

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AppInfo {
    pub name: String,
    pub version: String,
}

fn reader_for(state: &State<AppState>) -> Result<Git2Reader, String> {
    let path = state.repo_path()?;
    Git2Reader::new(&path).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_app_info() -> AppInfo {
    AppInfo {
        name: "Trilho".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
    }
}

#[tauri::command]
pub fn list_commits_mock() -> Result<Vec<Commit>, String> {
    MockGitReader::new()
        .list_commits(50, 0)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn validate_repo_path(path: String) -> Result<(), String> {
    AppState::validate_path(&path)
}

#[tauri::command]
pub fn open_repo(path: String, app: AppHandle, state: State<AppState>) -> Result<RepoInfo, String> {
    state.set_repo(path.clone(), &app)?;
    repo_info(&path).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn close_repo(state: State<AppState>) {
    state.clear_repo();
}

#[tauri::command]
pub fn get_repo_info(state: State<AppState>) -> Result<RepoInfo, String> {
    let path = state.repo_path()?;
    repo_info(&path).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_recent_repos(state: State<AppState>) -> Vec<String> {
    state.recent_repos()
}

#[tauri::command]
pub fn list_commits(
    limit: usize,
    skip: usize,
    state: State<AppState>,
) -> Result<Vec<Commit>, String> {
    let reader = reader_for(&state)?;
    reader
        .list_commits(limit.min(500), skip)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_repo_status(state: State<AppState>) -> Result<RepoStatus, String> {
    reader_for(&state)?
        .get_status()
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_file_diff(
    path: String,
    staged: bool,
    state: State<AppState>,
) -> Result<String, String> {
    let repo = state.repo_path()?;
    let path = validate_repo_relative_path(&path).map_err(|e| e.to_string())?;
    let mut args = vec!["diff".into(), "--no-color".into()];
    if staged {
        args.push("--cached".into());
    }
    args.push("--".into());
    args.push(path);
    SafeGitCli::run(
        &repo,
        &GitCommand { args },
    )
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_commit_diff(
    commit_id: String,
    state: State<AppState>,
) -> Result<String, String> {
    let repo = state.repo_path()?;
    let commit_id = validate_git_object_id(&commit_id).map_err(|e| e.to_string())?;
    SafeGitCli::run(
        &repo,
        &GitCommand {
            args: vec![
                "show".into(),
                "--no-color".into(),
                "--format=".into(),
                commit_id,
            ],
        },
    )
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_sync_info(state: State<AppState>) -> Result<SyncInfo, String> {
    let mut info = reader_for(&state)?
        .get_sync_info()
        .map_err(|e| e.to_string())?;
    info.last_fetch_at = state.last_fetch_at();
    Ok(info)
}

#[tauri::command]
pub fn fetch_remote(app: AppHandle, state: State<AppState>) -> Result<SyncInfo, String> {
    let repo = state.repo_path()?;
    state.with_watch_suppressed(&app, || {
        SafeGitCli::run(
            &repo,
            &GitCommand {
                args: vec!["fetch".into(), "--prune".into()],
            },
        )
    })
    .map_err(|e: GitError| e.to_string())?;

    let now = Utc::now().to_rfc3339();
    state.set_last_fetch_at(now.clone());

    // Reconciliação pós-fetch (RF-19).
    let _ = app.emit("repo-changed", ());

    let mut info = reader_for(&state)?
        .get_sync_info()
        .map_err(|e| e.to_string())?;
    info.last_fetch_at = Some(now);
    Ok(info)
}
