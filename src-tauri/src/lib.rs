//! Trilho — biblioteca principal (Tauri 2: lib.rs + main.rs).

mod application;
mod commands;
mod domain;
mod infrastructure;
#[cfg(test)]
mod security_contract;

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

/// Encaixa a janela na área de trabalho (tela menos a barra de tarefas) quando
/// ela não cabe. O Windows não encolhe uma janela criada maior que o monitor:
/// o tamanho padrão (1200x800) numa tela de 1024x768 nasce com a borda direita
/// fora da tela e o rodapé atrás da barra. Janela maximizada não precisa de
/// nada — nela quem calcula o retângulo é o próprio Windows.
///
/// `pode_maximizar` só vale na inicialização. Depois disso a correção é
/// encolher: maximizar prenderia o usuário, porque ao restaurar a janela
/// voltaria ao tamanho que não cabe e seria maximizada de novo.
fn encaixar_na_area_de_trabalho(win: &tauri::WebviewWindow, pode_maximizar: bool) {
    if win.is_maximized().unwrap_or(false) {
        return;
    }
    let Ok(Some(monitor)) = win.current_monitor() else {
        return;
    };
    let Ok(tamanho) = win.outer_size() else {
        return;
    };
    let area = monitor.work_area();
    if tamanho.width <= area.size.width && tamanho.height <= area.size.height {
        return;
    }
    if pode_maximizar {
        let _ = win.maximize();
        return;
    }
    // `set_size` dimensiona a área de conteúdo, e `outer_size` mede a janela
    // com a moldura: pedir o tamanho da área de trabalho direto deixaria a
    // barra de título e as bordas sobrando para fora dela.
    let Ok(interno) = win.inner_size() else {
        return;
    };
    let moldura_largura = tamanho.width.saturating_sub(interno.width);
    let moldura_altura = tamanho.height.saturating_sub(interno.height);
    let _ = win.set_size(tauri::PhysicalSize::new(
        area.size.width.saturating_sub(moldura_largura),
        area.size.height.saturating_sub(moldura_altura),
    ));
    let _ = win.set_position(tauri::PhysicalPosition::new(area.position.x, area.position.y));
}

/// Encaixe na abertura e a cada volta de foco. Mudança de resolução ou de
/// escala acontece com o app em segundo plano (o usuário vai às configurações
/// do Windows e volta), então o retorno do foco é o momento natural de
/// reencaixar. Não usamos `Resized` de propósito: ele dispara a cada quadro do
/// arrasto e brigaria com quem está redimensionando a janela na mão.
fn instalar_encaixe_de_janela(app: &tauri::AppHandle) {
    let Some(win) = app.get_webview_window("main") else {
        return;
    };
    encaixar_na_area_de_trabalho(&win, true);
    let win2 = win.clone();
    win.on_window_event(move |event| {
        if matches!(
            event,
            tauri::WindowEvent::Focused(true) | tauri::WindowEvent::ScaleFactorChanged { .. }
        ) {
            encaixar_na_area_de_trabalho(&win2, false);
        }
    });
}

/// Ao focar a janela, restaura o ícone (mitiga perda após idle no Windows).
fn install_window_icon_keepalive(app: &tauri::AppHandle) {
    let Some(win) = app.get_webview_window("main") else {
        return;
    };
    reapply_main_window_icon(app);
    let app2 = app.clone();
    win.on_window_event(move |event| {
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
    win.on_window_event(move |event| {
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
            instalar_encaixe_de_janela(app.handle());
            install_window_icon_keepalive(app.handle());
            #[cfg(debug_assertions)]
            install_dev_idle_recovery(app.handle());
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_app_info,
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
            commands::open_repo_folder,
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
