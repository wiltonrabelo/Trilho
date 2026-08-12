//! RF-21 — orquestração do chat: tool-calling allowlisted + writes pendentes.

use serde_json::{json, Value};

use crate::application::{
    CommitFileDiff, FileDiff, GitCommand, GitError, LlmChatRequest, LlmMessage, LlmProvider,
    LlmToolCall, LlmToolDef, RepoContext,
};
use crate::domain::{
    AssistantSettings, AssistantUiContext, BlameSource, BranchDiffMode, ChatAssistantResponse,
    ChatMessage, LlmProviderKind, WriteRequest,
};
use crate::infrastructure::llm::{
    ping_ollama, AnthropicProvider, ClaudeCodeProvider, CodexCliProvider, OllamaProvider,
    OpenAiProvider,
};
use crate::infrastructure::{
    get_branch_file_diff, get_branch_pr_status, get_conflict_file, get_llm_api_key, list_branch_diff,
    list_local_branches as fetch_local_branches, list_remote_branches, list_stashes, list_tags,
    validate_clone_branch, validate_compare_ref, validate_git_object_id, validate_remote_url,
    validate_repo_relative_path, validate_tag_name,
};

const MAX_TOOL_ROUNDS: usize = 6;
const MAX_COMMITS: usize = 30;
/// Limite por trecho de diff/tool (bytes UTF-8, com corte em fronteira de char).
const MAX_DIFF_BYTES: usize = 12_000;
const MAX_BLAME_LINES: u32 = 40;
const MAX_BRANCH_DIFF_FILES: usize = 80;
/// Revisão determinística: quantos arquivos e orçamento de **bytes UTF-8** no pacote.
const MAX_REVIEW_FILES: usize = 8;
const MAX_REVIEW_PACKET_BYTES: usize = 28_000;

mod prompts;
mod review;
mod tools;

#[cfg(test)]
mod tests;

use prompts::*;
use review::*;
use tools::*;

pub fn build_provider(settings: &AssistantSettings) -> Result<Box<dyn LlmProvider>, GitError> {
    match settings.provider {
        LlmProviderKind::Ollama => Ok(Box::new(OllamaProvider {
            base_url: settings.ollama_base_url.clone(),
        })),
        LlmProviderKind::OpenAi => {
            let key = get_llm_api_key("openai").ok_or_else(|| {
                GitError::Io(
                    "Chave OpenAI ausente. Salve a API key nas configurações do assistente.".into(),
                )
            })?;
            Ok(Box::new(OpenAiProvider {
                api_key: key,
                base_url: "https://api.openai.com/v1".into(),
            }))
        }
        LlmProviderKind::Anthropic => {
            let key = get_llm_api_key("anthropic").ok_or_else(|| {
                GitError::Io(
                    "Chave Anthropic ausente. Salve a API key nas configurações do assistente."
                        .into(),
                )
            })?;
            Ok(Box::new(AnthropicProvider { api_key: key }))
        }
        LlmProviderKind::ClaudeCode => Ok(Box::new(ClaudeCodeProvider {
            model: if settings.model.trim().is_empty() {
                "sonnet".into()
            } else {
                settings.model.clone()
            },
        })),
        LlmProviderKind::CodexCli => Ok(Box::new(CodexCliProvider {
            model: if settings.model.trim().is_empty() {
                "gpt-5.4-mini".into()
            } else {
                settings.model.clone()
            },
        })),
    }
}

pub fn run_chat(
    ctx: &RepoContext,
    settings: &AssistantSettings,
    messages: &[ChatMessage],
    ui: Option<&AssistantUiContext>,
) -> Result<ChatAssistantResponse, GitError> {
    if !settings.enabled {
        return Err(GitError::Io(
            "Assistente desligado. Ative nas configurações (opt-in).".into(),
        ));
    }

    let last_user = messages
        .iter()
        .rev()
        .find(|m| m.role == "user")
        .map(|m| m.content.as_str());

    // Gate duro: revisão sem «Enviar diffs» — não chama o LLM.
    if !settings.send_diffs {
        if let Some(last_user) = last_user {
            if looks_like_code_review_request(last_user) {
                return Ok(ChatAssistantResponse {
                    reply: REPLY_ENABLE_SEND_DIFFS.to_string(),
                    pending_writes: vec![],
                    notice: None,
                });
            }
        }
    }

    // Com diffs ligados: coleta pacote no runtime e chama o LLM sem tools
    // (llama3.2 e similares ignoram tool-calling e inventam respostas).
    if settings.send_diffs {
        if let Some(last_user) = last_user {
            if looks_like_code_review_request(last_user) {
                return run_deterministic_code_review(ctx, settings, last_user, ui);
            }
        }
    }

    let provider = build_provider(settings)?;
    let tools = allowlisted_tools(settings);

    let mut llm_messages: Vec<LlmMessage> = Vec::new();
    let mut system = SYSTEM_PROMPT.to_string();
    let preamble = context_preamble(ctx, settings, ui);
    if !preamble.is_empty() {
        system.push('\n');
        system.push_str(&preamble);
    }
    llm_messages.push(LlmMessage::System(system));

    for m in messages {
        match m.role.as_str() {
            "user" => llm_messages.push(LlmMessage::User(m.content.clone())),
            "assistant" => llm_messages.push(LlmMessage::Assistant {
                content: Some(m.content.clone()),
                tool_calls: vec![],
            }),
            _ => {}
        }
    }

    let mut pending_writes: Vec<WriteRequest> = Vec::new();
    let mut notices: Vec<String> = Vec::new();
    let mut final_reply = String::new();

    for _ in 0..MAX_TOOL_ROUNDS {
        let req = LlmChatRequest {
            model: settings.model.clone(),
            messages: llm_messages.clone(),
            tools: tools.clone(),
        };
        let resp = provider.chat(&req)?;

        if resp.tool_calls.is_empty() {
            final_reply = resp.content.unwrap_or_default();
            break;
        }

        llm_messages.push(LlmMessage::Assistant {
            content: resp.content.clone(),
            tool_calls: resp.tool_calls.clone(),
        });

        for call in &resp.tool_calls {
            match run_tool(ctx, settings, call, ui) {
                ToolOutcome::Read(content) => {
                    llm_messages.push(LlmMessage::Tool {
                        tool_call_id: call.id.clone(),
                        name: call.name.clone(),
                        content,
                    });
                }
                ToolOutcome::Write(wr) => {
                    pending_writes.push(wr);
                    llm_messages.push(LlmMessage::Tool {
                        tool_call_id: call.id.clone(),
                        name: call.name.clone(),
                        content: "Proposta registrada — o usuário precisa confirmar no Trilho (RF-08)."
                            .into(),
                    });
                }
                ToolOutcome::Rejected(msg) => {
                    notices.push(msg.clone());
                    llm_messages.push(LlmMessage::Tool {
                        tool_call_id: call.id.clone(),
                        name: call.name.clone(),
                        content: format!("REJEITADO: {msg}"),
                    });
                }
            }
        }

        if !pending_writes.is_empty()
            && resp.tool_calls.iter().all(|c| c.name.starts_with("propose_"))
        {
            let req2 = LlmChatRequest {
                model: settings.model.clone(),
                messages: llm_messages.clone(),
                tools: vec![],
            };
            if let Ok(r2) = provider.chat(&req2) {
                final_reply = r2.content.unwrap_or_else(|| {
                    "Propus ações de escrita — confirme no diálogo de pré-visualização.".into()
                });
            } else {
                final_reply =
                    "Propus ações de escrita — confirme no diálogo de pré-visualização.".into();
            }
            break;
        }
    }

    if final_reply.is_empty() && !pending_writes.is_empty() {
        final_reply =
            "Propus ações de escrita — confirme no diálogo de pré-visualização.".into();
    }
    if final_reply.is_empty() {
        final_reply = "Sem resposta do modelo.".into();
    }
    // Modelos fracos às vezes imprimem JSON de tool em vez de tool-calling.
    if looks_like_fake_tool_json(&final_reply) {
        if !settings.send_diffs && looks_like_code_review_request(
            messages
                .iter()
                .rev()
                .find(|m| m.role == "user")
                .map(|m| m.content.as_str())
                .unwrap_or(""),
        ) {
            final_reply = REPLY_ENABLE_SEND_DIFFS.to_string();
        } else {
            final_reply = concat!(
                "Não consigo executar ferramentas escrevendo JSON no chat. ",
                "Reformule o pedido (ex.: «revise esta branch contra master») ",
                "com «Enviar diffs ao provedor» ligado nas configurações do Assistente, ",
                "ou use um modelo com tool-calling mais confiável."
            )
            .to_string();
        }
    }

    Ok(ChatAssistantResponse {
        reply: final_reply,
        pending_writes,
        notice: if notices.is_empty() {
            None
        } else {
            Some(notices.join(" · "))
        },
    })
}

const REPLY_ENABLE_SEND_DIFFS: &str = "\
Para revisar código e procurar bugs, preciso da opção «Enviar diffs ao provedor \
(revisão de código)» ligada nas configurações do Assistente (acima deste chat). \
Marque a opção e peça de novo a revisão. \
Sem ela não envio diffs nem conteúdo de arquivos ao modelo.";

fn looks_like_code_review_request(text: &str) -> bool {
    let t = text.to_lowercase();
    let review = [
        "revis", "review", "bug", "melhor", "code review", "analis", "procure por",
        "procurar por", "encontre bug", "encontrar bug", "acha bug", "achados",
    ];
    review.iter().any(|k| t.contains(k))
}

fn looks_like_fake_tool_json(text: &str) -> bool {
    let t = text.trim();
    // Protocolo Claude Code: o adaptador deveria ter consumido o bloco; se sobrou
    // no reply final, não tratar como “JSON falso” genérico de modelo fraco.
    if t.contains("<<<TRILHO_TOOL_CALLS>>>") {
        return true;
    }
    // Heurística: bloco estilo {"name":"list_...","parameters":...} impresso na resposta.
    (t.contains("\"name\"") || t.contains("\"name\":"))
        && (t.contains("\"parameters\"") || t.contains("\"arguments\""))
        && (t.contains("list_")
            || t.contains("get_")
            || t.contains("propose_")
            || t.contains("show_file")
            || t.contains("\"status\""))
}

pub fn test_connection(settings: &AssistantSettings) -> Result<String, GitError> {
    let provider = build_provider(settings)?;
    let req = LlmChatRequest {
        model: settings.model.clone(),
        messages: vec![
            LlmMessage::System("Responda só com a palavra OK.".into()),
            LlmMessage::User("ping".into()),
        ],
        tools: vec![],
    };
    let resp = provider.chat(&req)?;
    Ok(resp
        .content
        .unwrap_or_else(|| "(sem texto)".into())
        .chars()
        .take(80)
        .collect())
}
