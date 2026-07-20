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
            commands::remove_recent_repo,
            commands::list_commits,
            commands::get_repo_status,
            commands::get_file_diff,
            commands::read_worktree_file,
            commands::open_worktree_path,
            commands::reveal_worktree_path,
            commands::resolve_worktree_path,
            commands::get_commit_diff,
            commands::list_commit_files,
            commands::get_commit_file_diff,
            commands::get_sync_info,
            commands::get_credential_status,
            commands::configure_gcm_helper,
            commands::trigger_github_login,
            commands::store_github_pat,
            commands::logout_github_account,
            commands::enable_github_use_http_path,
            commands::test_github_ssh,
            commands::get_ssh_public_key,
            commands::fetch_remote,
            commands::get_branch_origin,
            commands::get_dual_trail,
            commands::list_branch_exclusive_commits,
            commands::get_file_blame,
            commands::preview_write_operation,
            commands::execute_write_operation,
            commands::list_audit_log,
            commands::get_assistant_settings,
            commands::set_assistant_settings,
            commands::set_llm_api_key,
            commands::clear_llm_api_key,
            commands::test_llm_connection,
            commands::chat_assistant,
            commands::preview_clone_remote,
            commands::list_local_branches,
            commands::list_remote_branches,
            commands::list_stashes,
            commands::list_tags,
            commands::list_ordered_compare_refs,
            commands::list_branch_diff_files,
            commands::get_branch_file_diff_cmd,
            commands::list_clone_remote_branches,
            commands::execute_clone_remote,
            commands::get_branch_pr_status,
            commands::get_conflict_file,
        ])
        .run(tauri::generate_context!())
        .expect("erro ao iniciar o Trilho");
}
