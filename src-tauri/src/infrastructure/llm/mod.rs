//! RF-21 — adaptadores HTTP (Ollama / OpenAI / Anthropic) + Codex CLI.

mod cli_protocol;
mod codex_cli;

pub use codex_cli::CodexCliProvider;

use serde_json::{json, Value};

use crate::application::{
    LlmChatRequest, LlmChatResponse, LlmMessage, LlmProvider, LlmToolCall, GitError,
};

const USER_AGENT: &str = "Trilho/0.1";
const TIMEOUT_SECS: u64 = 120;
/// Modelos locais (Ollama) costumam demorar mais com pacotes de revisão.
const OLLAMA_TIMEOUT_SECS: u64 = 300;

fn http_err(e: ureq::Error) -> GitError {
    match e {
        ureq::Error::Status(code, resp) => {
            let body = resp.into_string().unwrap_or_default();
            GitError::Io(format_http_status_err(code, &body))
        }
        ureq::Error::Transport(t) => {
            let detail = t.to_string();
            GitError::Io(format_transport_err(&detail))
        }
    }
}

/// Mensagens legíveis para erros HTTP comuns do Ollama/OpenAI-compat.
fn format_http_status_err(code: u16, body: &str) -> String {
    let lower = body.to_lowercase();
    if code == 403
        && (lower.contains("requires a subscription")
            || lower.contains("upgrade for access")
            || lower.contains("ollama.com/upgrade"))
    {
        return "Este modelo Ollama Cloud exige assinatura ativa. \
Abra https://ollama.com/upgrade, assine na mesma conta logada no app Ollama \
e teste a conexão de novo. Só estar logado não libera glm-5.2:cloud / \
minimax-m3:cloud."
            .into();
    }
    if code == 401
        || (code == 403
            && (lower.contains("unauthorized")
                || lower.contains("invalid api key")
                || lower.contains("authentication")))
    {
        return "Provedor recusou a autenticação. No Ollama Cloud, confira o login \
no app; em OpenAI/Anthropic, confira a API key salva no Assistente."
            .into();
    }
    if code == 404
        && (lower.contains("model") || lower.contains("not found") || lower.contains("does not exist"))
    {
        return "Modelo não encontrado. Confira o nome no campo Modelo \
(ex.: glm-5.2:cloud) e se ele aparece no app Ollama."
            .into();
    }
    let snippet: String = body.chars().take(180).collect();
    if snippet.is_empty() {
        format!("LLM HTTP {code}.")
    } else {
        format!("LLM HTTP {code}: {snippet}")
    }
}

fn format_transport_err(detail: &str) -> String {
    let lower = detail.to_lowercase();
    let looks_timeout = lower.contains("10060")
        || lower.contains("timed out")
        || lower.contains("timeout")
        || lower.contains("não respondeu")
        || lower.contains("nao respondeu")
        || lower.contains("connection reset")
        || lower.contains("connection refused")
        || lower.contains("recusou");
    if looks_timeout {
        format!(
            "Ollama não respondeu em {OLLAMA_TIMEOUT_SECS}s (ou a conexão caiu). \
Abra o app Ollama, confira se `http://127.0.0.1:11434` responde \
(Testar conexão no Assistente) e tente de novo. Detalhe: {detail}"
        )
    } else {
        format!("LLM rede: {detail}")
    }
}

/// Ping rápido em `/api/tags` — falha cedo se o Ollama estiver fora.
pub fn ping_ollama(base_url: &str) -> Result<(), GitError> {
    let url = format!("{}/api/tags", base_url.trim_end_matches('/'));
    ureq::get(&url)
        .set("User-Agent", USER_AGENT)
        .timeout(std::time::Duration::from_secs(5))
        .call()
        .map_err(|e| match e {
            ureq::Error::Status(code, _) => GitError::Io(format!(
                "Ollama respondeu HTTP {code} em /api/tags. Verifique se o serviço está saudável."
            )),
            ureq::Error::Transport(t) => GitError::Io(format_transport_err(&t.to_string())),
        })?;
    Ok(())
}

fn tools_openai(req: &LlmChatRequest) -> Value {
    req.tools
        .iter()
        .map(|t| {
            json!({
                "type": "function",
                "function": {
                    "name": t.name,
                    "description": t.description,
                    "parameters": t.parameters,
                }
            })
        })
        .collect()
}

fn messages_openai(req: &LlmChatRequest) -> Value {
    let mut out = Vec::new();
    for m in &req.messages {
        match m {
            LlmMessage::System(c) => out.push(json!({"role":"system","content":c})),
            LlmMessage::User(c) => out.push(json!({"role":"user","content":c})),
            LlmMessage::Assistant { content, tool_calls } => {
                let mut obj = json!({"role":"assistant"});
                if let Some(c) = content {
                    obj["content"] = json!(c);
                }
                if !tool_calls.is_empty() {
                    obj["tool_calls"] = json!(tool_calls
                        .iter()
                        .map(|tc| json!({
                            "id": tc.id,
                            "type": "function",
                            "function": {
                                "name": tc.name,
                                "arguments": tc.arguments,
                            }
                        }))
                        .collect::<Vec<_>>());
                }
                out.push(obj);
            }
            LlmMessage::Tool {
                tool_call_id,
                name: _,
                content,
            } => out.push(json!({
                "role":"tool",
                "tool_call_id": tool_call_id,
                "content": content,
            })),
        }
    }
    Value::Array(out)
}

fn parse_openai_response(v: &Value) -> Result<LlmChatResponse, GitError> {
    let choice = v
        .pointer("/choices/0/message")
        .ok_or_else(|| GitError::Io("Resposta LLM sem message.".into()))?;
    let content = choice
        .get("content")
        .and_then(|c| c.as_str())
        .map(|s| s.to_string());
    let mut tool_calls = Vec::new();
    if let Some(arr) = choice.get("tool_calls").and_then(|t| t.as_array()) {
        for tc in arr {
            let id = tc
                .get("id")
                .and_then(|x| x.as_str())
                .unwrap_or("call")
                .to_string();
            let name = tc
                .pointer("/function/name")
                .and_then(|x| x.as_str())
                .unwrap_or("")
                .to_string();
            let arguments = tc
                .pointer("/function/arguments")
                .and_then(|x| x.as_str())
                .unwrap_or("{}")
                .to_string();
            if !name.is_empty() {
                tool_calls.push(LlmToolCall {
                    id,
                    name,
                    arguments,
                });
            }
        }
    }
    Ok(LlmChatResponse {
        content,
        tool_calls,
    })
}

pub struct OpenAiProvider {
    pub api_key: String,
    pub base_url: String,
}

impl LlmProvider for OpenAiProvider {
    fn chat(&self, req: &LlmChatRequest) -> Result<LlmChatResponse, GitError> {
        let url = format!(
            "{}/chat/completions",
            self.base_url.trim_end_matches('/')
        );
        let body = json!({
            "model": req.model,
            "messages": messages_openai(req),
            "tools": tools_openai(req),
            "tool_choice": "auto",
        });
        let resp = ureq::post(&url)
            .set("User-Agent", USER_AGENT)
            .set("Authorization", &format!("Bearer {}", self.api_key))
            .set("Content-Type", "application/json")
            .timeout(std::time::Duration::from_secs(TIMEOUT_SECS))
            .send_json(body)
            .map_err(http_err)?;
        let v: Value = resp
            .into_json()
            .map_err(|e| GitError::Io(format!("JSON OpenAI: {e}")))?;
        parse_openai_response(&v)
    }
}

/// Ollama — API compatível com OpenAI em `/v1/chat/completions`.
pub struct OllamaProvider {
    pub base_url: String,
}

impl LlmProvider for OllamaProvider {
    fn chat(&self, req: &LlmChatRequest) -> Result<LlmChatResponse, GitError> {
        let url = format!(
            "{}/v1/chat/completions",
            self.base_url.trim_end_matches('/')
        );
        // tools vazio + tool_choice confunde alguns builds do Ollama.
        let mut body = json!({
            "model": req.model,
            "messages": messages_openai(req),
            "stream": false,
        });
        if !req.tools.is_empty() {
            body["tools"] = tools_openai(req);
            body["tool_choice"] = json!("auto");
        }
        let resp = ureq::post(&url)
            .set("User-Agent", USER_AGENT)
            .set("Content-Type", "application/json")
            .timeout(std::time::Duration::from_secs(OLLAMA_TIMEOUT_SECS))
            .send_json(body)
            .map_err(http_err)?;
        let v: Value = resp
            .into_json()
            .map_err(|e| GitError::Io(format!("JSON Ollama: {e}")))?;
        parse_openai_response(&v)
    }
}

pub struct AnthropicProvider {
    pub api_key: String,
}

fn messages_anthropic(req: &LlmChatRequest) -> (Option<String>, Value) {
    let mut system = None;
    let mut out = Vec::new();
    for m in &req.messages {
        match m {
            LlmMessage::System(c) => {
                system = Some(c.clone());
            }
            LlmMessage::User(c) => out.push(json!({"role":"user","content":c})),
            LlmMessage::Assistant { content, tool_calls } => {
                let mut blocks = Vec::new();
                if let Some(c) = content {
                    if !c.is_empty() {
                        blocks.push(json!({"type":"text","text":c}));
                    }
                }
                for tc in tool_calls {
                    let args: Value =
                        serde_json::from_str(&tc.arguments).unwrap_or_else(|_| json!({}));
                    blocks.push(json!({
                        "type":"tool_use",
                        "id": tc.id,
                        "name": tc.name,
                        "input": args,
                    }));
                }
                out.push(json!({"role":"assistant","content":blocks}));
            }
            LlmMessage::Tool {
                tool_call_id,
                name: _,
                content,
            } => {
                out.push(json!({
                    "role":"user",
                    "content":[{
                        "type":"tool_result",
                        "tool_use_id": tool_call_id,
                        "content": content,
                    }]
                }));
            }
        }
    }
    (system, Value::Array(out))
}

fn tools_anthropic(req: &LlmChatRequest) -> Value {
    req.tools
        .iter()
        .map(|t| {
            json!({
                "name": t.name,
                "description": t.description,
                "input_schema": t.parameters,
            })
        })
        .collect()
}

impl LlmProvider for AnthropicProvider {
    fn chat(&self, req: &LlmChatRequest) -> Result<LlmChatResponse, GitError> {
        let (system, messages) = messages_anthropic(req);
        let mut body = json!({
            "model": req.model,
            "max_tokens": 2048,
            "messages": messages,
            "tools": tools_anthropic(req),
        });
        if let Some(s) = system {
            body["system"] = json!(s);
        }
        let resp = ureq::post("https://api.anthropic.com/v1/messages")
            .set("User-Agent", USER_AGENT)
            .set("x-api-key", &self.api_key)
            .set("anthropic-version", "2023-06-01")
            .set("Content-Type", "application/json")
            .timeout(std::time::Duration::from_secs(TIMEOUT_SECS))
            .send_json(body)
            .map_err(http_err)?;
        let v: Value = resp
            .into_json()
            .map_err(|e| GitError::Io(format!("JSON Anthropic: {e}")))?;

        let mut content_text = String::new();
        let mut tool_calls = Vec::new();
        if let Some(arr) = v.get("content").and_then(|c| c.as_array()) {
            for block in arr {
                let ty = block.get("type").and_then(|t| t.as_str()).unwrap_or("");
                if ty == "text" {
                    if let Some(t) = block.get("text").and_then(|t| t.as_str()) {
                        if !content_text.is_empty() {
                            content_text.push('\n');
                        }
                        content_text.push_str(t);
                    }
                } else if ty == "tool_use" {
                    let id = block
                        .get("id")
                        .and_then(|x| x.as_str())
                        .unwrap_or("call")
                        .to_string();
                    let name = block
                        .get("name")
                        .and_then(|x| x.as_str())
                        .unwrap_or("")
                        .to_string();
                    let input = block.get("input").cloned().unwrap_or(json!({}));
                    tool_calls.push(LlmToolCall {
                        id,
                        name,
                        arguments: input.to_string(),
                    });
                }
            }
        }
        Ok(LlmChatResponse {
            content: if content_text.is_empty() {
                None
            } else {
                Some(content_text)
            },
            tool_calls,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn http_403_subscription_vira_mensagem_clara() {
        let body = r#"{"error":{"message":"this model requires a subscription, upgrade for access: https://ollama.com/upgrade","type":"api_error"}}"#;
        let msg = format_http_status_err(403, body);
        assert!(msg.contains("assinatura"));
        assert!(msg.contains("ollama.com/upgrade"));
        assert!(!msg.contains("api_error"));
    }

    #[test]
    fn http_404_modelo_vira_mensagem_clara() {
        let msg = format_http_status_err(404, r#"{"error":"model not found"}"#);
        assert!(msg.contains("Modelo não encontrado"));
    }
}
