//! RF-21 — adaptadores HTTP (Ollama / OpenAI / Anthropic) via ureq.

use serde_json::{json, Value};

use crate::application::{
    LlmChatRequest, LlmChatResponse, LlmMessage, LlmProvider, LlmToolCall, GitError,
};

const USER_AGENT: &str = "Trilho/0.1";
const TIMEOUT_SECS: u64 = 120;

fn http_err(e: ureq::Error) -> GitError {
    match e {
        ureq::Error::Status(code, resp) => {
            let body = resp.into_string().unwrap_or_default();
            let snippet: String = body.chars().take(200).collect();
            GitError::Io(format!("LLM HTTP {code}: {snippet}"))
        }
        ureq::Error::Transport(t) => GitError::Io(format!("LLM rede: {t}")),
    }
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
        let body = json!({
            "model": req.model,
            "messages": messages_openai(req),
            "tools": tools_openai(req),
            "tool_choice": "auto",
            "stream": false,
        });
        let resp = ureq::post(&url)
            .set("User-Agent", USER_AGENT)
            .set("Content-Type", "application/json")
            .timeout(std::time::Duration::from_secs(TIMEOUT_SECS))
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
