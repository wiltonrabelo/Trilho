//! Comandos IPC expostos ao frontend — fachada fina sobre casos de uso.

use crate::application::{
    AppState, FetchRemote, FileDiff, GitError, GitReader, RepoContext, ShowCommit,
};
use crate::domain::{Commit, RepoInfo, RepoStatus, SyncInfo};
use crate::infrastructure::{
    detect_credential_status, repo_info, validate_git_object_id, validate_repo_relative_path,
    CredentialStatus, MockGitReader,
};
use chrono::Utc;
use serde::Serialize;
use tauri::{AppHandle, Emitter, State};

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AppInfo {
    pub name: String,
    pub version: String,
}

fn repo_context(state: &State<AppState>) -> Result<RepoContext, String> {
    let path = state.repo_path()?;
    RepoContext::open(&path).map_err(|e| e.to_string())
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
    let ctx = repo_context(&state)?;
    ctx.reader()
        .list_commits(limit.min(500), skip)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_repo_status(state: State<AppState>) -> Result<RepoStatus, String> {
    repo_context(&state)?
        .reader()
        .get_status()
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_file_diff(
    path: String,
    staged: bool,
    state: State<AppState>,
) -> Result<String, String> {
    let path = validate_repo_relative_path(&path).map_err(|e| e.to_string())?;
    let ctx = repo_context(&state)?;
    let op = FileDiff { path, staged };
    ctx.execute(&op).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_commit_diff(
    commit_id: String,
    state: State<AppState>,
) -> Result<String, String> {
    let sha = validate_git_object_id(&commit_id).map_err(|e| e.to_string())?;
    let ctx = repo_context(&state)?;
    let op = ShowCommit { sha };
    ctx.execute(&op).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_sync_info(state: State<AppState>) -> Result<SyncInfo, String> {
    let mut info = repo_context(&state)?
        .reader()
        .get_sync_info()
        .map_err(|e| e.to_string())?;
    info.last_fetch_at = state.last_fetch_at();
    Ok(info)
}

#[tauri::command]
pub fn get_credential_status() -> CredentialStatus {
    detect_credential_status()
}

#[tauri::command]
pub fn fetch_remote(app: AppHandle, state: State<AppState>) -> Result<SyncInfo, String> {
    let ctx = repo_context(&state)?;
    state.with_watch_suppressed(&app, || ctx.execute(&FetchRemote))
        .map_err(|e: GitError| e.to_string())?;

    let now = Utc::now().to_rfc3339();
    state.set_last_fetch_at(now.clone());

    let _ = app.emit("repo-changed", ());

    let mut info = repo_context(&state)?
        .reader()
        .get_sync_info()
        .map_err(|e| e.to_string())?;
    info.last_fetch_at = Some(now);
    Ok(info)
}
