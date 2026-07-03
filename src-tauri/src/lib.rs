//! Trilho — biblioteca principal (Tauri 2: lib.rs + main.rs).

mod application;
mod commands;
mod domain;
mod infrastructure;

use application::AppState;
use tauri::Manager;

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            let state = AppState::new(app.handle())?;
            app.manage(state);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_app_info,
            commands::list_commits_mock,
            commands::validate_repo_path,
            commands::open_repo,
            commands::close_repo,
            commands::get_repo_info,
            commands::get_recent_repos,
            commands::list_commits,
            commands::get_repo_status,
            commands::get_file_diff,
            commands::get_commit_diff,
            commands::get_sync_info,
            commands::get_credential_status,
            commands::fetch_remote,
        ])
        .run(tauri::generate_context!())
        .expect("erro ao iniciar o Trilho");
}
