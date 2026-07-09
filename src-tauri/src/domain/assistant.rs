//! RF-21 — tipos do assistente LLM (settings + chat).

use serde::{Deserialize, Serialize};

use crate::domain::WriteRequest;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum LlmProviderKind {
    Ollama,
    OpenAi,
    Anthropic,
}

impl Default for LlmProviderKind {
    fn default() -> Self {
        Self::Ollama
    }
}

/// Preferências persistidas (sem API key).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct AssistantSettings {
    /// Opt-in — desligado por padrão.
    #[serde(default)]
    pub enabled: bool,
    #[serde(default)]
    pub provider: LlmProviderKind,
    /// Modelo (ex.: `llama3.2`, `gpt-4o-mini`, `claude-3-5-haiku-latest`).
    #[serde(default = "default_model")]
    pub model: String,
    /// URL base do Ollama.
    #[serde(default = "default_ollama_url")]
    pub ollama_base_url: String,
    /// Enviar metadados (branch, status resumido) ao provedor.
    #[serde(default = "default_true")]
    pub send_metadata: bool,
    /// Enviar diffs ao provedor (off por padrão).
    #[serde(default)]
    pub send_diffs: bool,
}

fn default_model() -> String {
    "llama3.2".into()
}

fn default_ollama_url() -> String {
    "http://127.0.0.1:11434".into()
}

fn default_true() -> bool {
    true
}

impl Default for AssistantSettings {
    fn default() -> Self {
        Self {
            enabled: false,
            provider: LlmProviderKind::Ollama,
            model: default_model(),
            ollama_base_url: default_ollama_url(),
            send_metadata: true,
            send_diffs: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ChatMessageDto {
    pub role: String,
    pub content: String,
}

/// Contexto da UI (RF-21 recorte 3) — seleção atual no grafo/diff/blame.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AssistantUiContext {
    #[serde(default)]
    pub selected_commit_id: Option<String>,
    #[serde(default)]
    pub selected_commit_summary: Option<String>,
    #[serde(default)]
    pub selected_file_path: Option<String>,
    #[serde(default)]
    pub blame_focus_line: Option<u32>,
    #[serde(default)]
    pub working_copy_selected: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChatAssistantRequest {
    pub messages: Vec<ChatMessageDto>,
    #[serde(default)]
    pub ui_context: Option<AssistantUiContext>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChatAssistantResponse {
    pub reply: String,
    #[serde(default)]
    pub pending_writes: Vec<WriteRequest>,
    /// Aviso (ex.: ferramenta rejeitada).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub notice: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AssistantSettingsView {
    #[serde(flatten)]
    pub settings: AssistantSettings,
    pub has_openai_key: bool,
    pub has_anthropic_key: bool,
}
