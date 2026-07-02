//! Trilho — biblioteca principal (padrão Tauri 2: lib.rs + main.rs).
//!
//! Arquitetura em camadas (Clean Architecture / Ports & Adapters):
//!   domain          → entidades puras
//!   application     → portas (traits) e regras de caso de uso
//!   infrastructure  → adaptadores concretos (git2/CLI; mocks no M0)
//!   commands        → fachada IPC exposta ao frontend

mod application;
mod commands;
mod domain;
mod infrastructure;

pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            commands::get_app_info,
            commands::list_commits_mock
        ])
        .run(tauri::generate_context!())
        .expect("erro ao iniciar o Trilho");
}
