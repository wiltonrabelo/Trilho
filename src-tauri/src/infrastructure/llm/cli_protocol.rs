//! Protocolo textual de tools para CLIs que não devolvem tool-calling HTTP.
//! O Codex CLI emite `<<<TRILHO_TOOL_CALLS>>>`…`<<<END_TRILHO_TOOL_CALLS>>>`;
//! o runtime interpreta o bloco e executa só a allowlist.

use std::collections::HashSet;

use serde_json::Value;

use crate::application::{LlmChatRequest, LlmChatResponse, LlmMessage, LlmToolCall, LlmToolDef};

pub const TOOL_BLOCK_START: &str = "<<<TRILHO_TOOL_CALLS>>>";
pub const TOOL_BLOCK_END: &str = "<<<END_TRILHO_TOOL_CALLS>>>";

/// Monta prompt linear a partir do histórico do Trilho.
pub fn flatten_messages(req: &LlmChatRequest) -> (Option<String>, String) {
    let mut system = None;
    let mut parts: Vec<String> = Vec::new();
    for m in &req.messages {
        match m {
            LlmMessage::System(s) => system = Some(s.clone()),
            LlmMessage::User(u) => parts.push(format!("Usuário:\n{u}")),
            LlmMessage::Assistant {
                content,
                tool_calls,
            } => {
                if let Some(c) = content {
                    if !c.is_empty() {
                        parts.push(format!("Assistente:\n{c}"));
                    }
                }
                if !tool_calls.is_empty() {
                    let arr: Vec<Value> = tool_calls
                        .iter()
                        .map(|tc| {
                            let args: Value = serde_json::from_str(&tc.arguments)
                                .unwrap_or_else(|_| json_object_or_empty(&tc.arguments));
                            serde_json::json!({
                                "name": tc.name,
                                "arguments": args,
                            })
                        })
                        .collect();
                    parts.push(format!(
                        "Assistente solicitou tools:\n{TOOL_BLOCK_START}\n{}\n{TOOL_BLOCK_END}",
                        serde_json::to_string_pretty(&arr).unwrap_or_else(|_| "[]".into())
                    ));
                }
            }
            LlmMessage::Tool { name, content, .. } => {
                parts.push(format!("Resultado da ferramenta {name}:\n{content}"));
            }
        }
    }
    if !req.tools.is_empty() {
        parts.push(tools_protocol_instructions(&req.tools));
    }
    let prompt = if parts.is_empty() {
        "Responda ok.".into()
    } else {
        parts.join("\n\n")
    };
    (system, prompt)
}

fn json_object_or_empty(raw: &str) -> Value {
    let t = raw.trim();
    if t.is_empty() {
        return Value::Object(Default::default());
    }
    serde_json::from_str(t).unwrap_or(Value::String(raw.to_string()))
}

fn tools_protocol_instructions(tools: &[LlmToolDef]) -> String {
    let catalog: Vec<Value> = tools
        .iter()
        .map(|t| {
            serde_json::json!({
                "name": t.name,
                "description": t.description,
                "parameters": t.parameters,
            })
        })
        .collect();
    let catalog_json = serde_json::to_string_pretty(&catalog).unwrap_or_else(|_| "[]".into());
    format!(
        "### Protocolo de tools do Trilho (adaptador CLI)\n\
Neste modo o CLI não tem tool-calling HTTP da API. Para usar uma ferramenta \
da allowlist abaixo, emita EXATAMENTE um bloco (pode haver uma frase curta antes):\n\
{TOOL_BLOCK_START}\n\
[{{\"name\":\"count_commits\",\"arguments\":{{}}}}]\n\
{TOOL_BLOCK_END}\n\
\n\
Regras:\n\
- Só nomes da lista. `arguments` é um objeto JSON (use {{}} se vazio).\n\
- Pode pedir várias tools no mesmo array (máx. 4).\n\
- Se puder responder sem tools, responda só em texto — sem o bloco.\n\
- Não invente shell/git arbitrário; o runtime do Trilho executa a allowlist.\n\
- Isto NÃO é «JSON falso no chat»: o Trilho interpreta o bloco e devolve o resultado.\n\
\n\
Allowlist:\n{catalog_json}"
    )
}

/// Extrai tool calls do protocolo textual; o restante vira `content`.
pub fn split_text_and_tool_calls(text: &str, tools: &[LlmToolDef]) -> LlmChatResponse {
    let allowed: HashSet<&str> = tools.iter().map(|t| t.name.as_str()).collect();
    if let Some((calls, rest)) = extract_trilho_tool_calls(text, &allowed) {
        if !calls.is_empty() {
            let content = {
                let t = rest.trim();
                if t.is_empty() {
                    None
                } else {
                    Some(t.to_string())
                }
            };
            return LlmChatResponse {
                content,
                tool_calls: calls,
            };
        }
    }
    LlmChatResponse {
        content: Some(text.to_string()),
        tool_calls: vec![],
    }
}

fn extract_trilho_tool_calls(
    text: &str,
    allowed: &HashSet<&str>,
) -> Option<(Vec<LlmToolCall>, String)> {
    if let Some((inner, rest)) = extract_delimited_block(text) {
        let calls = parse_tool_calls_json(&inner, allowed);
        if !calls.is_empty() {
            return Some((calls, rest));
        }
    }
    let trimmed = text.trim();
    if let Ok(v) = serde_json::from_str::<Value>(trimmed) {
        if let Some(arr) = v.get("trilho_tool_calls").and_then(|a| a.as_array()) {
            let calls = parse_tool_calls_array(arr, allowed);
            if !calls.is_empty() {
                return Some((calls, String::new()));
            }
        }
    }
    None
}

fn extract_delimited_block(text: &str) -> Option<(String, String)> {
    let start = text.find(TOOL_BLOCK_START)?;
    let after_start = start + TOOL_BLOCK_START.len();
    let end_rel = text[after_start..].find(TOOL_BLOCK_END)?;
    let inner = text[after_start..after_start + end_rel].trim().to_string();
    let mut rest = String::new();
    rest.push_str(text[..start].trim_end());
    let after_end = after_start + end_rel + TOOL_BLOCK_END.len();
    let tail = text[after_end..].trim_start();
    if !tail.is_empty() {
        if !rest.is_empty() {
            rest.push('\n');
        }
        rest.push_str(tail);
    }
    Some((inner, rest))
}

fn parse_tool_calls_json(raw: &str, allowed: &HashSet<&str>) -> Vec<LlmToolCall> {
    let cleaned = strip_md_fence(raw);
    if let Ok(arr) = serde_json::from_str::<Vec<Value>>(cleaned) {
        return parse_tool_calls_array(&arr, allowed);
    }
    if let Ok(v) = serde_json::from_str::<Value>(cleaned) {
        if let Some(arr) = v.get("trilho_tool_calls").and_then(|a| a.as_array()) {
            return parse_tool_calls_array(arr, allowed);
        }
        if v.get("name").and_then(|n| n.as_str()).is_some() {
            return parse_tool_calls_array(std::slice::from_ref(&v), allowed);
        }
    }
    Vec::new()
}

fn strip_md_fence(s: &str) -> &str {
    let t = s.trim();
    if let Some(rest) = t.strip_prefix("```json") {
        return rest
            .strip_suffix("```")
            .map(str::trim)
            .unwrap_or(rest.trim());
    }
    if let Some(rest) = t.strip_prefix("```") {
        return rest
            .strip_suffix("```")
            .map(str::trim)
            .unwrap_or(rest.trim());
    }
    t
}

fn parse_tool_calls_array(arr: &[Value], allowed: &HashSet<&str>) -> Vec<LlmToolCall> {
    let mut out = Vec::new();
    for (i, item) in arr.iter().take(4).enumerate() {
        let name = item
            .get("name")
            .and_then(|n| n.as_str())
            .unwrap_or("")
            .trim();
        if name.is_empty() || !allowed.contains(name) {
            continue;
        }
        let args = item
            .get("arguments")
            .or_else(|| item.get("parameters"))
            .cloned()
            .unwrap_or_else(|| Value::Object(Default::default()));
        let arguments = if args.is_string() {
            args.as_str().unwrap_or("{}").to_string()
        } else {
            args.to_string()
        };
        out.push(LlmToolCall {
            id: format!("cli_{i}_{name}"),
            name: name.to_string(),
            arguments,
        });
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::LlmToolDef;
    use serde_json::json;

    fn sample_tools() -> Vec<LlmToolDef> {
        vec![
            LlmToolDef {
                name: "list_commits".into(),
                description: "Lista commits".into(),
                parameters: json!({"type":"object","properties":{"limit":{"type":"integer"}}}),
            },
            LlmToolDef {
                name: "get_repo_status".into(),
                description: "Status".into(),
                parameters: json!({"type":"object","properties":{}}),
            },
        ]
    }

    #[test]
    fn flatten_junta_historico() {
        let req = LlmChatRequest {
            model: "gpt-5.4-mini".into(),
            messages: vec![
                LlmMessage::System("sys".into()),
                LlmMessage::User("olá".into()),
                LlmMessage::Assistant {
                    content: Some("oi".into()),
                    tool_calls: vec![],
                },
            ],
            tools: vec![],
        };
        let (sys, prompt) = flatten_messages(&req);
        assert_eq!(sys.as_deref(), Some("sys"));
        assert!(prompt.contains("Usuário:\nolá"));
        assert!(prompt.contains("Assistente:\noi"));
    }

    #[test]
    fn flatten_com_tools_inclui_protocolo() {
        let req = LlmChatRequest {
            model: "gpt-5.4-mini".into(),
            messages: vec![LlmMessage::User("quantos commits?".into())],
            tools: sample_tools(),
        };
        let (_, prompt) = flatten_messages(&req);
        assert!(prompt.contains(TOOL_BLOCK_START));
        assert!(prompt.contains("list_commits"));
        assert!(prompt.contains("Allowlist"));
    }

    #[test]
    fn parse_bloco_tool_calls() {
        let text = format!(
            "Vou consultar.\n{TOOL_BLOCK_START}\n\
[{{\"name\":\"list_commits\",\"arguments\":{{\"limit\":50}}}}]\n\
{TOOL_BLOCK_END}"
        );
        let resp = split_text_and_tool_calls(&text, &sample_tools());
        assert_eq!(resp.tool_calls.len(), 1);
        assert_eq!(resp.tool_calls[0].name, "list_commits");
        assert!(resp.tool_calls[0].arguments.contains("50"));
        assert_eq!(resp.content.as_deref(), Some("Vou consultar."));
    }

    #[test]
    fn rejeita_tool_fora_da_allowlist() {
        let text = format!(
            "{TOOL_BLOCK_START}\n\
[{{\"name\":\"shell\",\"arguments\":{{}}}}]\n\
{TOOL_BLOCK_END}"
        );
        let resp = split_text_and_tool_calls(&text, &sample_tools());
        assert!(resp.tool_calls.is_empty());
    }
}
