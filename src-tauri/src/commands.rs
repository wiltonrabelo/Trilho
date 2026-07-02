//! Comandos expostos ao frontend via IPC do Tauri.
//!
//! São a fachada fina entre a UI e a camada de aplicação. Não contêm regra de
//! negócio: apenas orquestram portas/adaptadores e convertem erros em `String`
//! para o frontend (mensagens acionáveis — nunca `stderr` cru).

use crate::application::GitReader;
use crate::domain::Commit;
use crate::infrastructure::MockGitReader;
use serde::Serialize;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AppInfo {
    pub name: String,
    pub version: String,
}

#[tauri::command]
pub fn get_app_info() -> AppInfo {
    AppInfo {
        name: "Trilho".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
    }
}

/// M0: lista commits de exemplo do adaptador mock (`GitReader`).
#[tauri::command]
pub fn list_commits_mock() -> Result<Vec<Commit>, String> {
    let reader = MockGitReader::new();
    reader.list_commits(50).map_err(|e| e.to_string())
}
