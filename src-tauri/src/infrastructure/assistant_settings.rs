//! RF-21 — persistência de `assistant_settings.json` (sem API keys).

use std::fs;
use std::path::Path;

use crate::application::GitError;
use crate::domain::AssistantSettings;

const FILE_NAME: &str = "assistant_settings.json";

fn settings_path(data_dir: &Path) -> std::path::PathBuf {
    data_dir.join(FILE_NAME)
}

pub fn load_settings(data_dir: &Path) -> AssistantSettings {
    let path = settings_path(data_dir);
    fs::read_to_string(path)
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default()
}

pub fn save_settings(data_dir: &Path, settings: &AssistantSettings) -> Result<(), GitError> {
    let path = settings_path(data_dir);
    let json = serde_json::to_string_pretty(settings)
        .map_err(|e| GitError::Io(format!("Falha ao serializar settings: {e}")))?;
    fs::write(path, json).map_err(|e| GitError::Io(format!("Falha ao gravar settings: {e}")))?;
    Ok(())
}
