use super::*;

pub(super) fn ref_resolves(ctx: &RepoContext, git_ref: &str) -> bool {
    if validate_compare_ref(git_ref).is_err() {
        return false;
    }
    // `writer()` é o runner CLI do repo (SafeGitCli), não um lock de escrita.
    // rev-parse é read-only; a API de leitura (Git2Reader) não expõe rev-parse genérico.
    ctx.writer()
        .run(&GitCommand {
            args: vec![
                "rev-parse".into(),
                "--verify".into(),
                format!("{git_ref}^{{commit}}"),
            ],
        })
        .is_ok()
}

/// Base da revisão: origem inferida da branch, senão main/master/develop.
pub(super) fn resolve_review_base(ctx: &RepoContext) -> Option<String> {
    if let Ok(origin) = ctx.reader().get_branch_origin() {
        if let Some(candidate) = origin.candidate {
            if ref_resolves(ctx, &candidate) {
                return Some(candidate);
            }
        }
    }
    for name in [
        "main",
        "master",
        "origin/main",
        "origin/master",
        "develop",
        "origin/develop",
    ] {
        if ref_resolves(ctx, name) {
            return Some(name.to_string());
        }
    }
    None
}

/// Corta `text` respeitando fronteira UTF-8; `budget` é em bytes.
pub(super) fn append_truncated(out: &mut String, budget: &mut usize, text: &str) {
    if *budget == 0 {
        return;
    }
    let mut end = (*budget).min(text.len()).min(MAX_DIFF_BYTES);
    while end > 0 && !text.is_char_boundary(end) {
        end -= 1;
    }
    out.push_str(&text[..end]);
    *budget = budget.saturating_sub(end);
    if end < text.len() {
        out.push_str("\n…[truncado]");
    }
}

pub(super) fn push_commit_files_packet(ctx: &RepoContext, commit_id: &str, out: &mut String, budget: &mut usize) {
    out.push_str(&format!("\n### Arquivos do commit `{commit_id}`\n"));
    let files = match ctx.reader().list_commit_files(commit_id) {
        Ok(f) => f,
        Err(e) => {
            out.push_str(&format!("erro ao listar arquivos: {e}\n"));
            return;
        }
    };
    if files.is_empty() {
        out.push_str("(nenhum arquivo)\n");
        return;
    }
    for f in files.iter().take(MAX_REVIEW_FILES) {
        out.push_str(&format!("- {}\n", f.path));
    }
    if files.len() > MAX_REVIEW_FILES {
        out.push_str(&format!(
            "… +{} arquivo(s) omitido(s)\n",
            files.len() - MAX_REVIEW_FILES
        ));
    }
    out.push_str("\n### Diffs\n");
    for f in files.iter().take(MAX_REVIEW_FILES) {
        if *budget < 400 {
            out.push_str("\n…[orçamento de diffs esgotado]\n");
            break;
        }
        let op = CommitFileDiff {
            sha: commit_id.to_string(),
            path: f.path.clone(),
        };
        match ctx.execute(&op) {
            Ok(diff) => {
                out.push_str(&format!("\n#### {}\n```diff\n", f.path));
                append_truncated(out, budget, &diff);
                out.push_str("\n```\n");
            }
            Err(e) => out.push_str(&format!("\n#### {}: erro ({e})\n", f.path)),
        }
    }
}

pub(super) fn push_branch_files_packet(
    ctx: &RepoContext,
    left: &str,
    right: &str,
    out: &mut String,
    budget: &mut usize,
) {
    out.push_str(&format!(
        "\n### Arquivos alterados (`{left}`...`{right}`, merge-base)\n"
    ));
    let mut summary = match list_branch_diff(ctx.writer(), left, right, BranchDiffMode::MergeBase)
    {
        Ok(s) => s,
        Err(e) => {
            out.push_str(&format!("erro ao listar diff da branch: {e}\n"));
            return;
        }
    };
    if summary.files.is_empty() {
        out.push_str("(nenhuma diferença nesta faixa)\n");
        return;
    }
    summary
        .files
        .sort_by_key(|f| std::cmp::Reverse(f.additions + f.deletions));
    let total = summary.files.len();
    for f in summary.files.iter().take(MAX_REVIEW_FILES) {
        out.push_str(&format!(
            "- {} (+{} −{})\n",
            f.path, f.additions, f.deletions
        ));
    }
    if total > MAX_REVIEW_FILES {
        out.push_str(&format!(
            "… +{} arquivo(s) omitido(s) da lista detalhada\n",
            total - MAX_REVIEW_FILES
        ));
    }
    out.push_str("\n### Diffs\n");
    for f in summary.files.iter().take(MAX_REVIEW_FILES) {
        if *budget < 400 {
            out.push_str("\n…[orçamento de diffs esgotado]\n");
            break;
        }
        match get_branch_file_diff(
            ctx.writer(),
            left,
            right,
            BranchDiffMode::MergeBase,
            &f.path,
        ) {
            Ok(diff) => {
                out.push_str(&format!("\n#### {}\n```diff\n", f.path));
                append_truncated(out, budget, &diff);
                out.push_str("\n```\n");
            }
            Err(e) => out.push_str(&format!("\n#### {}: erro ({e})\n", f.path)),
        }
    }
}

pub(super) fn push_working_tree_packet(ctx: &RepoContext, out: &mut String, budget: &mut usize) {
    out.push_str("\n### Alterações locais (working tree / stage)\n");
    let st = match ctx.reader().get_status() {
        Ok(s) => s,
        Err(e) => {
            out.push_str(&format!("erro no status: {e}\n"));
            return;
        }
    };
    let mut paths: Vec<(String, bool)> = Vec::new();
    for f in &st.staged {
        paths.push((f.path.clone(), true));
    }
    for f in st.unstaged.iter().chain(st.untracked.iter()) {
        if !paths.iter().any(|(p, _)| p == &f.path) {
            paths.push((f.path.clone(), false));
        }
    }
    if paths.is_empty() {
        out.push_str("(sem alterações locais)\n");
        return;
    }
    for (path, staged) in paths.iter().take(MAX_REVIEW_FILES) {
        out.push_str(&format!(
            "- {} ({})\n",
            path,
            if *staged { "staged" } else { "unstaged" }
        ));
    }
    out.push_str("\n### Diffs\n");
    for (path, staged) in paths.iter().take(MAX_REVIEW_FILES) {
        if *budget < 400 {
            out.push_str("\n…[orçamento de diffs esgotado]\n");
            break;
        }
        let op = FileDiff {
            path: path.clone(),
            staged: *staged,
        };
        match ctx.execute(&op) {
            Ok(diff) => {
                out.push_str(&format!("\n#### {} ({})\n```diff\n", path, if *staged { "staged" } else { "unstaged" }));
                append_truncated(out, budget, &diff);
                out.push_str("\n```\n");
            }
            Err(e) => out.push_str(&format!("\n#### {path}: erro ({e})\n")),
        }
    }
}

/// Monta o pacote de revisão no processo (sem depender de tool-calling do LLM).
pub(super) fn build_code_review_packet(ctx: &RepoContext, ui: Option<&AssistantUiContext>) -> String {
    let mut out = String::from(
        "## Pacote de revisão (coletado pelo Trilho)\n\
Escopo limitado aos trechos abaixo; conteúdo grande foi truncado.\n",
    );
    let mut budget = MAX_REVIEW_PACKET_BYTES;

    if let Ok(commits) = ctx.reader().list_commits(12, None, false) {
        out.push_str("\n### Commits recentes no grafo\n");
        for c in commits.iter().take(12) {
            out.push_str(&format!("- {} — {}\n", c.short_id, c.summary));
        }
    }

    if let Some(ui) = ui {
        if let Some(id) = &ui.selected_commit_id {
            push_commit_files_packet(ctx, id, &mut out, &mut budget);
            return out;
        }
        if ui.working_copy_selected {
            push_working_tree_packet(ctx, &mut out, &mut budget);
            return out;
        }
    }

    if let Some(left) = resolve_review_base(ctx) {
        push_branch_files_packet(ctx, &left, "HEAD", &mut out, &mut budget);
        // Complemento: WT se houver mudanças locais.
        if let Ok(st) = ctx.reader().get_status() {
            if (!st.staged.is_empty() || !st.unstaged.is_empty() || !st.untracked.is_empty())
                && budget > 800 {
                    push_working_tree_packet(ctx, &mut out, &mut budget);
                }
        }
    } else {
        out.push_str(
            "\nNão achei base (main/master/origem da branch). \
Revisando só alterações locais.\n",
        );
        push_working_tree_packet(ctx, &mut out, &mut budget);
    }

    out
}

/// Há bloco ```diff com pelo menos uma linha de mudança (+/- ou @@).
pub(super) fn packet_has_reviewable_diffs(packet: &str) -> bool {
    if !packet.contains("```diff") {
        return false;
    }
    packet.lines().any(|l| {
        let t = l.trim_start();
        t.starts_with("@@")
            || ((t.starts_with('+') || t.starts_with('-'))
                && !t.starts_with("+++")
                && !t.starts_with("---"))
    })
}

/// Resposta “meta” (plano de revisão / aula) sem citar arquivos dos diffs.
pub(super) fn looks_like_meta_review(text: &str) -> bool {
    let t = text.to_lowercase();
    let markers = [
        "o que é necessário fazer para revisar",
        "o que e necessario fazer para revisar",
        "vou dividir em seções",
        "vou dividir em secoes",
        "revisar a arquitetura",
        "compreender a funcionalidade",
        "recomendações específicas",
        "recomendacoes especificas",
        "large language",
        "modelos de linguagem",
        "é necessário revisar",
        "e necessario revisar",
    ];
    let hits = markers.iter().filter(|m| t.contains(*m)).count();
    let cites_source_file = [".rs", ".ts", ".tsx", ".js", ".jsx", ".pas", ".py", ".go"]
        .iter()
        .any(|ext| text.contains(ext));
    hits >= 2 && !cites_source_file
}

/// Placeholders típicos de modelo fraco (não são arquivos reais do pacote).
pub(super) fn looks_like_placeholder_files(text: &str) -> bool {
    let t = text.to_lowercase();
    [
        "arquivo1",
        "arquivo2",
        "arquivo3",
        "file1.",
        "file2.",
        "file3.",
        "example.py",
        "exemplo.py",
        "foo.txt",
        "bar.txt",
    ]
    .iter()
    .any(|m| t.contains(m))
}

pub(super) fn looks_like_repo_path(path: &str) -> bool {
    let p = path.trim();
    if p.is_empty() || p.len() > 260 {
        return false;
    }
    if p.starts_with("http") || p.contains(' ') {
        return false;
    }
    p.contains('/') || p.contains('\\') || p.contains('.')
}

/// Paths listados no pacote (`- path (+n −m)` / `#### path`).
pub(super) fn extract_packet_file_paths(packet: &str) -> Vec<String> {
    let mut paths = Vec::new();
    for line in packet.lines() {
        let t = line.trim();
        if let Some(rest) = t.strip_prefix("#### ") {
            let p = rest.split_whitespace().next().unwrap_or("").trim();
            if looks_like_repo_path(p) {
                paths.push(p.to_string());
            }
            continue;
        }
        if let Some(rest) = t.strip_prefix("- ") {
            let mut p = rest;
            if let Some((head, _)) = rest.split_once(" (+") {
                p = head;
            } else if let Some((head, _)) = rest.split_once(" (") {
                p = head;
            }
            let p = p.trim();
            if looks_like_repo_path(p) {
                paths.push(p.to_string());
            }
        }
    }
    paths.sort();
    paths.dedup();
    paths
}

pub(super) fn review_cites_packet_paths(reply: &str, paths: &[String]) -> bool {
    if paths.is_empty() {
        return false;
    }
    let reply_n = reply.to_lowercase().replace('\\', "/");
    paths.iter().any(|p| {
        let full = p.to_lowercase().replace('\\', "/");
        if reply_n.contains(&full) {
            return true;
        }
        let file = full.rsplit('/').next().unwrap_or(&full);
        file.len() > 3 && reply_n.contains(file)
    })
}

pub(super) fn review_reply_is_unusable(body: &str, packet_paths: &[String]) -> bool {
    looks_like_fake_tool_json(body)
        || looks_like_meta_review(body)
        || looks_like_placeholder_files(body)
        || !review_cites_packet_paths(body, packet_paths)
}

pub(super) fn review_branch_hint(ctx: &RepoContext) -> String {
    match crate::infrastructure::repo_info(ctx.repo_path()) {
        Ok(info) => format!(
            "Branch atual: {}\n",
            info.branch.unwrap_or_else(|| "—".into())
        ),
        Err(_) => String::new(),
    }
}

pub(super) fn run_deterministic_code_review(
    ctx: &RepoContext,
    settings: &AssistantSettings,
    user_request: &str,
    ui: Option<&AssistantUiContext>,
) -> Result<ChatAssistantResponse, GitError> {
    // Falha cedo se o Ollama estiver fora — evita timeout longo no pacote grande.
    if matches!(settings.provider, LlmProviderKind::Ollama) {
        ping_ollama(&settings.ollama_base_url)?;
    }
    let packet = build_code_review_packet(ctx, ui);
    if !packet_has_reviewable_diffs(&packet) {
        return Ok(ChatAssistantResponse {
            reply: REPLY_EMPTY_REVIEW_PACKET.to_string(),
            pending_writes: vec![],
            notice: None,
        });
    }
    let packet_paths = extract_packet_file_paths(&packet);
    let provider = build_provider(settings)?;
    // Sem context_preamble completo: falar de Ollama/tools faz o modelo
    // “revisar o Assistente” em vez dos diffs.
    let mut system = REVIEW_SYSTEM_PROMPT.to_string();
    let hint = review_branch_hint(ctx);
    if !hint.is_empty() {
        system.push('\n');
        system.push_str(&hint);
    }
    let allowed = if packet_paths.is_empty() {
        String::new()
    } else {
        format!(
            "Arquivos do pacote (cite SOMENTE caminhos desta lista; não invente nomes):\n{}\n\n",
            packet_paths
                .iter()
                .take(40)
                .map(|p| format!("- {p}"))
                .collect::<Vec<_>>()
                .join("\n")
        )
    };
    let user = format!(
        "Pedido do usuário:\n{user_request}\n\n{allowed}{packet}\n\n\
Agora produza ## Achados e ## Resumo citando arquivos REAIS da lista acima. \
Proibido inventar arquivo1.txt / exemplos fictícios. \
Não faça plano de revisão nem explique Ollama/Trilho em abstrato."
    );
    let req = LlmChatRequest {
        model: settings.model.clone(),
        messages: vec![
            LlmMessage::System(system),
            LlmMessage::User(user),
        ],
        tools: vec![],
    };
    let resp = provider.chat(&req)?;
    let body = resp.content.unwrap_or_else(|| "Sem resposta do modelo.".into());
    let reply = if review_reply_is_unusable(&body, &packet_paths) {
        format!(
            "{REVIEW_REPLY_PREFIX}{}",
            reply_meta_review(settings.provider)
        )
    } else {
        format!("{REVIEW_REPLY_PREFIX}{body}")
    };
    Ok(ChatAssistantResponse {
        reply,
        pending_writes: vec![],
        notice: None,
    })
}
