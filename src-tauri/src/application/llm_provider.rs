//! RF-21 — porta LlmProvider (tool calling).

use serde_json::Value;

use crate::application::GitError;

#[derive(Debug, Clone)]
pub struct LlmToolDef {
    pub name: String,
    pub description: String,
    pub parameters: Value,
}

#[derive(Debug, Clone)]
pub struct LlmToolCall {
    pub id: String,
    pub name: String,
    pub arguments: String,
}

#[derive(Debug, Clone)]
pub enum LlmMessage {
    System(String),
    User(String),
    Assistant {
        content: Option<String>,
        tool_calls: Vec<LlmToolCall>,
    },
    Tool {
        tool_call_id: String,
        name: String,
        content: String,
    },
}

#[derive(Debug, Clone)]
pub struct LlmChatRequest {
    pub model: String,
    pub messages: Vec<LlmMessage>,
    pub tools: Vec<LlmToolDef>,
}

#[derive(Debug, Clone)]
pub struct LlmChatResponse {
    pub content: Option<String>,
    pub tool_calls: Vec<LlmToolCall>,
}

/// Porta plugável (Ollama / OpenAI / Anthropic).
pub trait LlmProvider: Send + Sync {
    fn chat(&self, req: &LlmChatRequest) -> Result<LlmChatResponse, GitError>;
}
