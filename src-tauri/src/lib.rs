//! Trilho — biblioteca principal (Tauri 2: lib.rs + main.rs).

mod application;
mod commands;
mod domain;
mod infrastructure;

use application::AppState;
use tauri::Manager;

/// Reaplica o ícone da janela principal (titlebar + taskbar no Windows).
/// Após idle/sleep/reload da WebView2 o HWND às vezes perde o ícone e
/// cai no placeholder genérico do sistema.
fn reapply_main_window_icon(app: &tauri::AppHandle) {
    let Some(win) = app.get_webview_window("main") else {
        return;
    };
    if let Some(icon) = app.default_window_icon() {
        let _ = win.set_icon(icon.clone());
    }
}

/// Ao focar a janela, restaura o ícone (mitiga perda após idle no Windows).
fn install_window_icon_keepalive(app: &tauri::AppHandle) {
    let Some(win) = app.get_webview_window("main") else {
        return;
    };
    reapply_main_window_icon(app);
    let app2 = app.clone();
    let _ = win.on_window_event(move |event| {
        if matches!(event, tauri::WindowEvent::Focused(true)) {
            reapply_main_window_icon(&app2);
        }
    });
}

/// Em debug: ao focar a janela, se a WebView ficou na página de erro do Edge
/// (HTTP 400 após idle), força volta ao Vite em 127.0.0.1.
#[cfg(debug_assertions)]
fn install_dev_idle_recovery(app: &tauri::AppHandle) {
    let Some(win) = app.get_webview_window("main") else {
        return;
    };
    let win2 = win.clone();
    let app2 = app.clone();
    let _ = win.on_window_event(move |event| {
        if !matches!(event, tauri::WindowEvent::Focused(true)) {
            return;
        }
        let w = win2.clone();
        let app = app2.clone();
        std::thread::spawn(move || {
            std::thread::sleep(std::time::Duration::from_millis(500));
            let vite_ok = ureq::get("http://127.0.0.1:1420/")
                .timeout(std::time::Duration::from_secs(2))
                .call()
                .map(|r| (200..400).contains(&r.status()))
                .unwrap_or(false);
            if !vite_ok {
                reapply_main_window_icon(&app);
                return;
            }
            // Roda mesmo na página de erro do Edge (ainda permite eval).
            let _ = w.eval(
                r#"(function(){
  try {
    var root = document.getElementById('root');
    var text = (document.body && document.body.innerText) || '';
    var broken = !root || root.childElementCount === 0
      || /HTTP ERROR/i.test(text)
      || /n\u00e3o est\u00e1 funcionando/i.test(text)
      || /This page isn't working/i.test(text)
      || /chrome-error/i.test(String(location.href));
    if (broken) location.replace('http://127.0.0.1:1420/');
  } catch (e) {
    location.replace('http://127.0.0.1:1420/');
  }
})()"#,
            );
            // Reload da WebView pode limpar o ícone do HWND de novo.
            std::thread::sleep(std::time::Duration::from_millis(300));
            reapply_main_window_icon(&app);
        });
    });
}

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            let state = AppState::new(app.handle())?;
            app.manage(state);
            install_window_icon_keepalive(app.handle());
            #[cfg(debug_assertions)]
            install_dev_idle_recovery(app.handle());
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
            commands::open_git_bash,
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
