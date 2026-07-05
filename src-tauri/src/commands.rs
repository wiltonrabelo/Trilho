//! Comandos IPC expostos ao frontend — fachada fina sobre casos de uso.

use crate::application::{
    execute_write, preview_write, AppState, CommitFileDiff, FetchRemote, FileDiff, GitError,
    GitReader, RepoContext, ShowCommit,
};
use crate::domain::{Commit, OperationPreview, RepoInfo, RepoStatus, SyncInfo, WriteRequest};
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

fn repo_context(state: &State<'_, AppState>) -> Result<RepoContext, String> {
    let path = state
        .repo_path()
        .map_err(|_| GitError::NoRepositoryOpen.to_string())?;
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
        .list_commits(50, 0, false)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn validate_repo_path(path: String) -> Result<(), String> {
    AppState::validate_path(&path)
}

// Comandos potencialmente demorados são `async`: no Tauri 2, comando síncrono
// roda na MAIN thread e congela a UI — em repositório grande (SysPDV: 40k
// commits, 300+ branches) isso travava o app inteiro ao abrir.
#[tauri::command]
pub async fn open_repo(
    path: String,
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<RepoInfo, String> {
    state.set_repo(path.clone(), &app)?;
    repo_info(&path).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn close_repo(state: State<'_, AppState>) -> Result<(), String> {
    state.clear_repo();
    Ok(())
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
pub async fn list_commits(
    limit: usize,
    skip: usize,
    first_parent: bool,
    state: State<'_, AppState>,
) -> Result<Vec<Commit>, String> {
    let ctx = repo_context(&state)?;
    ctx.reader()
        .list_commits(limit.min(500), skip, first_parent)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_repo_status(state: State<'_, AppState>) -> Result<RepoStatus, String> {
    repo_context(&state)?
        .reader()
        .get_status()
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_file_diff(
    path: String,
    staged: bool,
    state: State<'_, AppState>,
) -> Result<String, String> {
    let path = validate_repo_relative_path(&path).map_err(|e| e.to_string())?;
    let ctx = repo_context(&state)?;
    let op = FileDiff { path, staged };
    ctx.execute(&op).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_commit_diff(
    commit_id: String,
    state: State<'_, AppState>,
) -> Result<String, String> {
    let sha = validate_git_object_id(&commit_id).map_err(|e| e.to_string())?;
    let ctx = repo_context(&state)?;
    let op = ShowCommit { sha };
    ctx.execute(&op).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn list_commit_files(
    commit_id: String,
    state: State<'_, AppState>,
) -> Result<Vec<crate::domain::FileChange>, String> {
    let sha = validate_git_object_id(&commit_id).map_err(|e| e.to_string())?;
    repo_context(&state)?
        .reader()
        .list_commit_files(&sha)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_commit_file_diff(
    commit_id: String,
    path: String,
    state: State<'_, AppState>,
) -> Result<String, String> {
    let sha = validate_git_object_id(&commit_id).map_err(|e| e.to_string())?;
    let path = validate_repo_relative_path(&path).map_err(|e| e.to_string())?;
    let ctx = repo_context(&state)?;
    let op = CommitFileDiff { sha, path };
    ctx.execute(&op).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_sync_info(state: State<'_, AppState>) -> Result<SyncInfo, String> {
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
pub async fn fetch_remote(app: AppHandle, state: State<'_, AppState>) -> Result<SyncInfo, String> {
    let ctx = repo_context(&state)?;
    state
        .with_watch_suppressed(&app, || ctx.execute(&FetchRemote))
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

#[tauri::command]
pub async fn get_branch_origin(
    state: State<'_, AppState>,
) -> Result<crate::domain::BranchOrigin, String> {
    repo_context(&state)?
        .reader()
        .get_branch_origin()
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_dual_trail(
    base: String,
    limit: usize,
    state: State<'_, AppState>,
) -> Result<Vec<crate::domain::TrailEntry>, String> {
    // Mesmas regras de saneamento de path servem para nome de ref (sem '-'
    // inicial, sem NUL, sem '..').
    let base = validate_repo_relative_path(&base).map_err(|e| e.to_string())?;
    repo_context(&state)?
        .reader()
        .get_dual_trail(&base, limit.min(600))
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_file_blame(
    path: String,
    source: String,
    commit_id: Option<String>,
    start_line: u32,
    end_line: u32,
    state: State<'_, AppState>,
) -> Result<Vec<crate::domain::BlameLine>, String> {
    let path = validate_repo_relative_path(&path).map_err(|e| e.to_string())?;
    let source = parse_blame_source(&source).map_err(|e| e.to_string())?;
    let commit_ref = commit_id
        .as_deref()
        .map(validate_git_object_id)
        .transpose()
        .map_err(|e| e.to_string())?;
    repo_context(&state)?
        .reader()
        .get_file_blame(&path, source, commit_ref.as_deref(), start_line, end_line)
        .map_err(|e| e.to_string())
}

fn parse_blame_source(raw: &str) -> Result<crate::domain::BlameSource, GitError> {
    match raw {
        "commit" => Ok(crate::domain::BlameSource::Commit),
        "workingTree" => Ok(crate::domain::BlameSource::WorkingTree),
        "staging" => Ok(crate::domain::BlameSource::Staging),
        _ => Err(GitError::Git(format!("Fonte de blame inválida: {raw}"))),
    }
}

#[tauri::command]
pub fn preview_publish_operation(
    remote_url: Option<String>,
    state: State<'_, AppState>,
) -> Result<OperationPreview, String> {
    let path = state.repo_path()?;
    let ctx = repo_context(&state)?;
    preview_write(
        &ctx,
        &path,
        &WriteRequest::Publish {
            url: remote_url,
        },
    )
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn execute_publish_operation(
    remote_url: Option<String>,
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let ctx = repo_context(&state)?;
    state
        .with_watch_suppressed(&app, || {
            execute_write(
                &ctx,
                WriteRequest::Publish {
                    url: remote_url,
                },
            )
        })
        .map_err(|e: GitError| e.to_string())?;
    let _ = app.emit("repo-changed", ());
    Ok(())
}

#[tauri::command]
pub fn preview_write_operation(
    request: WriteRequest,
    state: State<'_, AppState>,
) -> Result<OperationPreview, String> {
    let path = state.repo_path()?;
    let ctx = repo_context(&state)?;
    preview_write(&ctx, &path, &request).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn execute_write_operation(
    request: WriteRequest,
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let ctx = repo_context(&state)?;
    state
        .with_watch_suppressed(&app, || execute_write(&ctx, request))
        .map_err(|e: GitError| e.to_string())?;
    let _ = app.emit("repo-changed", ());
    Ok(())
}
