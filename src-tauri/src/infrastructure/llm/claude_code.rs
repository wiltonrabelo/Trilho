//! Provedor Claude Code CLI — usa o login/plano já autenticado no PC
//! (`claude auth login`), sem API key no Trilho.
//!
//! Não usa `--bare` (esse modo ignora OAuth e exige `ANTHROPIC_API_KEY`).
//! Tool-calling do Trilho: o CLI não devolve `tool_use` da API; o adaptador
//! usa protocolo textual (`<<<TRILHO_TOOL_CALLS>>>`) que o runtime interpreta
//! e executa na allowlist (mesmo loop dos outros provedores).

use std::collections::HashSet;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

use serde_json::Value;

use crate::application::{
    GitError, LlmChatRequest, LlmChatResponse, LlmMessage, LlmProvider, LlmToolCall, LlmToolDef,
};

const CLAUDE_TIMEOUT_SECS: u64 = 300;

pub const TOOL_BLOCK_START: &str = "<<<TRILHO_TOOL_CALLS>>>";
pub const TOOL_BLOCK_END: &str = "<<<END_TRILHO_TOOL_CALLS>>>";

pub struct ClaudeCodeProvider {
    pub model: String,
}

impl LlmProvider for ClaudeCodeProvider {
    fn chat(&self, req: &LlmChatRequest) -> Result<LlmChatResponse, GitError> {
        let (system, prompt) = flatten_messages(req);
        let model = if req.model.trim().is_empty() {
            self.model.as_str()
        } else {
            req.model.trim()
        };
        let raw = run_claude_print(&prompt, system.as_deref(), model)?;
        let text = parse_claude_print_output(&raw)?;
        if req.tools.is_empty() {
            return Ok(LlmChatResponse {
                content: Some(text),
                tool_calls: vec![],
            });
        }
        Ok(split_text_and_tool_calls(&text, &req.tools))
    }
}

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
    let catalog_json =
        serde_json::to_string_pretty(&catalog).unwrap_or_else(|_| "[]".into());
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
            id: format!("cc_{i}_{name}"),
            name: name.to_string(),
            arguments,
        });
    }
    out
}

pub fn parse_claude_print_output(raw: &str) -> Result<String, GitError> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Err(GitError::Io("Claude Code retornou resposta vazia.".into()));
    }
    // Preferir JSON estruturado (`--output-format json`).
    // Às vezes o CLI emite mais de uma linha — usa o último objeto parseável.
    let json_candidate = trimmed
        .lines()
        .rev()
        .find(|l| l.trim_start().starts_with('{'))
        .unwrap_or(trimmed);
    if let Ok(v) = serde_json::from_str::<Value>(json_candidate) {
        let is_error = v.get("is_error").and_then(|b| b.as_bool()).unwrap_or(false);
        let subtype = v.get("subtype").and_then(|s| s.as_str()).unwrap_or("");
        let subtype_err = matches!(
            subtype,
            "error" | "failure" | "api_error" | "authentication_failed" | "billing_error"
        );
        if is_error || subtype_err {
            let msg = v
                .get("result")
                .and_then(|r| r.as_str())
                .or_else(|| v.pointer("/error/message").and_then(|m| m.as_str()))
                .or_else(|| v.get("error").and_then(|e| e.as_str()))
                .or_else(|| v.get("message").and_then(|m| m.as_str()))
                .unwrap_or("erro reportado pelo Claude Code");
            return Err(GitError::Io(format_claude_cli_err(msg, json_candidate)));
        }
        if let Some(err) = v.get("error").and_then(|e| e.as_str()) {
            return Err(GitError::Io(format_claude_cli_err(err, json_candidate)));
        }
        if let Some(msg) = v.pointer("/error/message").and_then(|m| m.as_str()) {
            return Err(GitError::Io(format_claude_cli_err(msg, json_candidate)));
        }
        if let Some(result) = v.get("result").and_then(|r| r.as_str()) {
            if !result.is_empty() {
                return Ok(result.to_string());
            }
        }
        // Alguns builds usam `content` / `text`.
        if let Some(text) = v
            .get("content")
            .and_then(|c| c.as_str())
            .or_else(|| v.get("text").and_then(|c| c.as_str()))
        {
            if !text.is_empty() {
                return Ok(text.to_string());
            }
        }
    }
    // Fallback: texto puro.
    Ok(trimmed.to_string())
}

fn format_claude_cli_err(msg: &str, raw: &str) -> String {
    let lower = msg.to_lowercase();
    // Evitar substrings amplas ("auth", "usage") — casam "author", "Usage: flag...".
    let looks_auth = lower.contains("authentication_failed")
        || lower.contains("authentication failed")
        || lower.contains("not authenticated")
        || lower.contains("not logged")
        || lower.contains("unauthorized")
        || lower.contains("please login")
        || lower.contains("please log in")
        || lower.contains("claude auth login")
        || lower.contains("oauth");
    if looks_auth {
        return "Claude Code não está autenticado. No terminal, rode \
`claude auth login` com a conta do plano Pro/Max e tente de novo.".into();
    }
    let looks_quota = lower.contains("rate_limit")
        || lower.contains("rate limit")
        || lower.contains("usage limit")
        || lower.contains("quota")
        || lower.contains("too many requests")
        || (lower.contains("limit") && lower.contains("exceed"));
    if looks_quota {
        return "Limite de uso do plano Claude esgotado ou em cooldown. \
Aguarde ou confira o uso em claude.ai e tente de novo.".into();
    }
    if lower.contains("billing_error") || lower.contains("billing") {
        return "Problema de faturamento/plano no Claude. Confira a assinatura \
em claude.ai e o status do Claude Code (`claude auth status`).".into();
    }
    let snippet: String = raw.chars().take(220).collect();
    format!("Claude Code: {msg} ({snippet})")
}

fn run_claude_print(
    prompt: &str,
    system: Option<&str>,
    model: &str,
) -> Result<String, GitError> {
    // Corpo grande via stdin — evita limite de linha de comando do Windows (~32k).
    let mut stdin_body = String::new();
    if let Some(sys) = system {
        if !sys.is_empty() {
            stdin_body.push_str(sys);
            stdin_body.push_str("\n\n---\n\n");
        }
    }
    stdin_body.push_str(prompt);

    let mut args = vec![
        "-p".to_string(),
        "Responda ao pedido completo enviado via stdin. Devolva só a resposta final."
            .into(),
        "--output-format".into(),
        "json".into(),
        // Documentado em code.claude.com/docs/en/permission-modes (dontAsk =
        // auto-nega tools não pré-aprovadas; adequado a -p não-interativo).
        "--permission-mode".into(),
        "dontAsk".into(),
        // Sem tools nativas do Claude — só o protocolo textual do Trilho.
        "--allowedTools".into(),
        String::new(),
    ];
    if !model.is_empty() {
        args.push("--model".into());
        args.push(model.to_string());
    }

    // PATH + instalação nativa + binário embutido na extensão VS Code/Cursor.
    let candidates = resolve_claude_bins();
    let mut last_err = None;
    for bin in &candidates {
        match spawn_claude(bin, &args, Some(stdin_body.as_bytes())) {
            Ok(out) => return Ok(out),
            Err(e) => {
                let msg = e.to_string();
                if msg.contains("não encontrado")
                    || msg.contains("not found")
                    || msg.contains("Claude Code CLI")
                {
                    last_err = Some(e);
                    continue;
                }
                return Err(e);
            }
        }
    }
    Err(last_err.unwrap_or_else(missing_cli_err))
}

/// Locais conhecidos do CLI: PATH, `~/.local/bin`, extensão anthropic.claude-code.
pub fn resolve_claude_bins() -> Vec<PathBuf> {
    let mut out: Vec<PathBuf> = Vec::new();
    // Preferir PATH (instalação oficial / shim).
    for name in ["claude", "claude.cmd"] {
        out.push(PathBuf::from(name));
    }
    if let Some(home) = std::env::var_os("USERPROFILE")
        .or_else(|| std::env::var_os("HOME"))
        .map(PathBuf::from)
    {
        out.push(home.join(".local").join("bin").join("claude.exe"));
        out.push(home.join(".local").join("bin").join("claude"));
        // Extensão VS Code / Cursor: anthropic.claude-code-*/resources/native-binary/claude.exe
        for ext_root in [
            home.join(".vscode").join("extensions"),
            home.join(".cursor").join("extensions"),
        ] {
            push_extension_claude_bins(&ext_root, &mut out);
        }
    }
    out
}

fn push_extension_claude_bins(ext_root: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(ext_root) else {
        return;
    };
    let mut dirs: Vec<PathBuf> = entries
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| {
            p.file_name()
                .and_then(|n| n.to_str())
                .map(|n| n.starts_with("anthropic.claude-code-"))
                .unwrap_or(false)
        })
        .collect();
    // Semver numérico (não lexicográfico: "1.10" > "1.9").
    dirs.sort_by(|a, b| {
        extension_semver_key(b.file_name().and_then(|n| n.to_str()).unwrap_or(""))
            .cmp(&extension_semver_key(
                a.file_name().and_then(|n| n.to_str()).unwrap_or(""),
            ))
    });
    for dir in dirs {
        let exe = dir
            .join("resources")
            .join("native-binary")
            .join("claude.exe");
        if exe.is_file() {
            out.push(exe);
        }
        let unix = dir.join("resources").join("native-binary").join("claude");
        if unix.is_file() {
            out.push(unix);
        }
    }
}

/// Extrai `(major, minor, patch)` de `anthropic.claude-code-2.1.223-win32-x64`.
fn extension_semver_key(folder_name: &str) -> (u32, u32, u32) {
    const PREFIX: &str = "anthropic.claude-code-";
    let rest = folder_name.strip_prefix(PREFIX).unwrap_or(folder_name);
    let mut nums = rest
        .split(|c: char| !c.is_ascii_digit())
        .filter(|s| !s.is_empty())
        .filter_map(|s| s.parse::<u32>().ok());
    (
        nums.next().unwrap_or(0),
        nums.next().unwrap_or(0),
        nums.next().unwrap_or(0),
    )
}

fn missing_cli_err() -> GitError {
    GitError::Io(
        "Claude Code CLI não encontrado. \
O app Claude Desktop (chat) sozinho não basta. \
Opções: (1) extensão Claude Code no VS Code/Cursor instalada, ou \
(2) no PowerShell: irm https://claude.ai/install.ps1 | iex \
Depois autentique (`claude` ou login na extensão), reinicie o Trilho e teste de novo.".into(),
    )
}

fn spawn_claude(
    bin: &Path,
    args: &[String],
    stdin_body: Option<&[u8]>,
) -> Result<String, GitError> {
    // cwd neutro: se herdar o repo do Trilho, o CLI carrega CLAUDE.md/plugins
    // do projeto e o startup fica bem mais lento.
    let scratch = std::env::temp_dir().join("trilho-claude-scratch");
    let _ = std::fs::create_dir_all(&scratch);

    let mut cmd = Command::new(bin);
    crate::infrastructure::subprocesso::sem_janela_de_console(&mut cmd)
        .args(args)
        .current_dir(&scratch)
        .stdin(if stdin_body.is_some() {
            Stdio::piped()
        } else {
            Stdio::null()
        })
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        // Preferir OAuth do plano: não herdar API key do ambiente do Trilho.
        .env_remove("ANTHROPIC_API_KEY");

    let bin_display = bin.display().to_string();
    let mut child = cmd.spawn().map_err(|e| {
        if e.kind() == std::io::ErrorKind::NotFound {
            missing_cli_err()
        } else {
            GitError::Io(format!("Falha ao iniciar Claude Code ({bin_display}): {e}"))
        }
    })?;

    // Drenar stdout/stderr ANTES de escrever stdin — senão pipe cheio +
    // processo bloqueado em write = deadlock com pacotes grandes.
    let stdout_pipe = child.stdout.take();
    let stderr_pipe = child.stderr.take();
    let stdout_handle = std::thread::spawn(move || {
        let mut buf = Vec::new();
        if let Some(mut r) = stdout_pipe {
            let _ = r.read_to_end(&mut buf);
        }
        String::from_utf8_lossy(&buf).to_string()
    });
    let stderr_handle = std::thread::spawn(move || {
        let mut buf = Vec::new();
        if let Some(mut r) = stderr_pipe {
            let _ = r.read_to_end(&mut buf);
        }
        String::from_utf8_lossy(&buf).to_string()
    });

    let stdin_handle = stdin_body.map(|data| {
        let data = data.to_vec();
        let mut stdin = child.stdin.take();
        std::thread::spawn(move || {
            use std::io::Write;
            if let Some(mut stdin) = stdin.take() {
                let _ = stdin.write_all(&data);
                // drop fecha o pipe (EOF para o claude).
            }
        })
    });

    let timeout = Duration::from_secs(CLAUDE_TIMEOUT_SECS);
    let start = Instant::now();
    let status = loop {
        match child.try_wait() {
            Ok(Some(st)) => break st,
            Ok(None) => {
                if start.elapsed() > timeout {
                    let _ = child.kill();
                    let _ = child.wait();
                    return Err(GitError::Io(format!(
                        "Claude Code excedeu {CLAUDE_TIMEOUT_SECS}s sem responder. \
Tente de novo ou use um modelo/prompt menor."
                    )));
                }
                std::thread::sleep(Duration::from_millis(100));
            }
            Err(e) => {
                return Err(GitError::Io(format!("Claude Code: erro ao aguardar: {e}")));
            }
        }
    };

    if let Some(h) = stdin_handle {
        let _ = h.join();
    }
    let stdout = stdout_handle.join().unwrap_or_default();
    let stderr = stderr_handle.join().unwrap_or_default();

    if !status.success() {
        let combined = format!("{stdout}\n{stderr}");
        return Err(GitError::Io(format_claude_cli_err(
            combined.trim(),
            &combined,
        )));
    }

    if stdout.trim().is_empty() && !stderr.trim().is_empty() {
        return Err(GitError::Io(format_claude_cli_err(stderr.trim(), &stderr)));
    }
    Ok(stdout)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::{LlmMessage, LlmToolDef};
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
            model: "sonnet".into(),
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
            model: "sonnet".into(),
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

    #[test]
    fn parse_json_result() {
        let raw = r#"{"result":"revisão ok","total_cost_usd":0.01}"#;
        assert_eq!(parse_claude_print_output(raw).unwrap(), "revisão ok");
    }

    #[test]
    fn parse_is_error_nao_vira_sucesso() {
        let raw = r#"{"type":"result","subtype":"error","is_error":true,"result":"authentication_failed"}"#;
        let err = parse_claude_print_output(raw).unwrap_err().to_string();
        assert!(err.contains("autenticado") || err.contains("authentication"));
    }

    #[test]
    fn parse_auth_error_mensagem_clara() {
        let err = format_claude_cli_err("authentication_failed", "authentication_failed");
        assert!(err.contains("claude auth login"));
    }

    #[test]
    fn nao_confunde_author_nem_usage_help_com_auth_quota() {
        let author = format_claude_cli_err("invalid author field", "invalid author field");
        assert!(!author.contains("não está autenticado"));
        let usage_help = format_claude_cli_err("Usage: claude [options]", "Usage: claude [options]");
        assert!(!usage_help.contains("Limite de uso"));
        let quota = format_claude_cli_err("usage limit exceeded", "usage limit exceeded");
        assert!(quota.contains("Limite de uso"));
    }

    #[test]
    fn resolve_inclui_nomes_path() {
        let bins = resolve_claude_bins();
        assert!(bins.iter().any(|p| p.ends_with("claude") || p.ends_with("claude.cmd")));
    }

    #[test]
    fn extension_semver_ordena_1_10_acima_de_1_9() {
        assert!(
            extension_semver_key("anthropic.claude-code-1.10.0-win32-x64")
                > extension_semver_key("anthropic.claude-code-1.9.0-win32-x64")
        );
        assert!(
            extension_semver_key("anthropic.claude-code-2.1.223-win32-x64")
                > extension_semver_key("anthropic.claude-code-2.1.99-win32-x64")
        );
    }
}
