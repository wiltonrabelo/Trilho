//! Provedor Codex CLI — usa login ChatGPT/Codex já autenticado no PC
//! (`codex login`), sem API key no Trilho.
//!
//! Invoca `codex exec` em modo não-interativo com sandbox read-only e cwd
//! neutro (não opera no working tree do usuário). Tool-calling do Trilho via
//! protocolo textual (`<<<TRILHO_TOOL_CALLS>>>`).

use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

use crate::application::{
    GitError, LlmChatRequest, LlmChatResponse, LlmProvider,
};

use super::cli_protocol::{flatten_messages, split_text_and_tool_calls};

const CODEX_TIMEOUT_SECS: u64 = 300;

pub struct CodexCliProvider {
    pub model: String,
}

impl LlmProvider for CodexCliProvider {
    fn chat(&self, req: &LlmChatRequest) -> Result<LlmChatResponse, GitError> {
        let (system, prompt) = flatten_messages(req);
        let model = if req.model.trim().is_empty() {
            self.model.as_str()
        } else {
            req.model.trim()
        };
        let raw = run_codex_exec(&prompt, system.as_deref(), model)?;
        let text = parse_codex_exec_output(&raw)?;
        if req.tools.is_empty() {
            return Ok(LlmChatResponse {
                content: Some(text),
                tool_calls: vec![],
            });
        }
        Ok(split_text_and_tool_calls(&text, &req.tools))
    }
}

pub fn parse_codex_exec_output(raw: &str) -> Result<String, GitError> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Err(GitError::Io("Codex CLI retornou resposta vazia.".into()));
    }
    // Com --json: JSONL — pegar último item com texto útil.
    if trimmed.lines().any(|l| l.trim_start().starts_with('{')) {
        let mut last_text: Option<String> = None;
        for line in trimmed.lines() {
            let line = line.trim();
            if !line.starts_with('{') {
                continue;
            }
            if let Ok(v) = serde_json::from_str::<serde_json::Value>(line) {
                if let Some(msg) = extract_jsonl_text(&v) {
                    if !msg.is_empty() {
                        last_text = Some(msg);
                    }
                }
                if v.get("type").and_then(|t| t.as_str()) == Some("error")
                    || v.get("type").and_then(|t| t.as_str()) == Some("turn.failed")
                {
                    let msg = v
                        .get("message")
                        .or_else(|| v.get("error"))
                        .and_then(|m| m.as_str())
                        .unwrap_or("falha no Codex");
                    return Err(GitError::Io(format_codex_cli_err(msg, line)));
                }
            }
        }
        if let Some(t) = last_text {
            return Ok(t);
        }
    }
    Ok(trimmed.to_string())
}

fn extract_jsonl_text(v: &serde_json::Value) -> Option<String> {
    if let Some(s) = v
        .pointer("/item/text")
        .and_then(|t| t.as_str())
        .or_else(|| v.get("text").and_then(|t| t.as_str()))
        .or_else(|| v.pointer("/item/content").and_then(|t| t.as_str()))
        .or_else(|| v.get("last_agent_message").and_then(|t| t.as_str()))
        .or_else(|| v.get("message").and_then(|t| t.as_str()))
    {
        return Some(s.to_string());
    }
    None
}

fn format_codex_cli_err(msg: &str, raw: &str) -> String {
    let lower = msg.to_lowercase();
    let raw_l = raw.to_lowercase();
    let looks_auth = lower.contains("not logged")
        || lower.contains("not authenticated")
        || lower.contains("please login")
        || lower.contains("please log in")
        || lower.contains("codex login")
        || lower.contains("authentication")
        || lower.contains("unauthorized")
        || raw_l.contains("codex login")
        || raw_l.contains("not logged");
    if looks_auth && !lower.contains("author") {
        return "Codex CLI não está autenticado com ChatGPT. No terminal, rode \
`codex login` (conta ChatGPT com acesso Codex) e tente de novo.".into();
    }
    let looks_quota = lower.contains("rate limit")
        || lower.contains("usage limit")
        || lower.contains("quota")
        || lower.contains("too many requests")
        || (lower.contains("limit") && lower.contains("exceed"));
    if looks_quota {
        return "Limite de uso do plano ChatGPT/Codex atingido. Aguarde a renovação \
ou confira o uso em chatgpt.com.".into();
    }
    let snippet: String = raw.chars().take(220).collect();
    format!("Codex CLI: {msg} ({snippet})")
}

fn run_codex_exec(
    prompt: &str,
    system: Option<&str>,
    model: &str,
) -> Result<String, GitError> {
    let mut stdin_body = String::new();
    if let Some(sys) = system {
        if !sys.is_empty() {
            stdin_body.push_str(sys);
            stdin_body.push_str("\n\n---\n\n");
        }
    }
    stdin_body.push_str(prompt);
    stdin_body.push_str(
        "\n\nResponda ao pedido completo. Devolva só a resposta final \
(ou o bloco <<<TRILHO_TOOL_CALLS>>> se precisar de tools do Trilho).",
    );

    // Flags base; `--ask-for-approval never` é tentado e removido se a build não tiver.
    let mut arg_sets: Vec<Vec<String>> = Vec::new();
    for with_approval in [true, false] {
        let mut args = vec![
            "exec".into(),
            "--ephemeral".into(),
            "--sandbox".into(),
            "read-only".into(),
            "--skip-git-repo-check".into(),
        ];
        if with_approval {
            args.push("--ask-for-approval".into());
            args.push("never".into());
        }
        if !model.is_empty() {
            args.push("--model".into());
            args.push(model.to_string());
        }
        args.push("-".into());
        arg_sets.push(args);
    }

    let candidates = resolve_codex_bins();
    if candidates.is_empty() {
        return Err(GitError::Io(
            "Codex CLI não encontrado. Instale a extensão Codex/ChatGPT no VS Code \
(ou `npm i -g @openai/codex`), rode `codex login` e tente de novo.".into(),
        ));
    }
    let mut last_skip: Option<GitError> = None;
    let mut last_real: Option<GitError> = None;
    for bin in &candidates {
        for args in &arg_sets {
            match spawn_codex(bin, args, Some(stdin_body.as_bytes())) {
                Ok(out) => return Ok(out),
                Err(e) => {
                    if is_skippable_codex_launch_err(&e) {
                        last_skip = Some(e);
                        break; // próximo binário
                    }
                    let msg = e.to_string().to_lowercase();
                    // Flag inexistente nesta build → tenta o próximo conjunto de args.
                    if msg.contains("ask-for-approval")
                        && (msg.contains("unknown")
                            || msg.contains("unexpected")
                            || msg.contains("unrecognized")
                            || msg.contains("usage:"))
                    {
                        last_real = Some(e);
                        continue;
                    }
                    if bin.is_absolute() {
                        return Err(e);
                    }
                    last_real = Some(e);
                    break;
                }
            }
        }
    }
    Err(last_real.or(last_skip).unwrap_or_else(|| {
        GitError::Io(
            "Codex CLI não encontrado. Instale a extensão Codex/ChatGPT no VS Code \
(ou `npm i -g @openai/codex`), rode `codex login` e tente de novo.".into(),
        )
    }))
}

/// Só tenta o próximo candidato se o binário nem chegou a subir (ausente / PE errado).
fn is_skippable_codex_launch_err(err: &GitError) -> bool {
    let msg = err.to_string();
    msg.contains("não encontrado (")
        || (msg.contains("não encontrado") && !msg.contains("falha ao iniciar"))
        || msg.contains("os error 193")
        || msg.contains("%1 não é um aplicativo Win32")
        || msg.contains("not a valid Win32 application")
}

/// Locais conhecidos: extensão (plataforma atual) → npm → PATH.
pub fn resolve_codex_bins() -> Vec<PathBuf> {
    let mut out: Vec<PathBuf> = Vec::new();
    if let Some(home) = std::env::var_os("USERPROFILE")
        .or_else(|| std::env::var_os("HOME"))
        .map(PathBuf::from)
    {
        // Preferir binário absoluto da extensão (evita pegar ELF linux no PATH/lista).
        for base in [
            home.join(".vscode").join("extensions"),
            home.join(".cursor").join("extensions"),
        ] {
            push_extension_codex_bins(&base, &mut out);
        }
        if let Some(appdata) = std::env::var_os("APPDATA").map(PathBuf::from) {
            push_if_runnable(appdata.join("npm").join("codex.cmd"), &mut out);
            push_if_runnable(appdata.join("npm").join("codex.exe"), &mut out);
        }
        #[cfg(windows)]
        {
            push_if_runnable(home.join(".local").join("bin").join("codex.exe"), &mut out);
        }
        #[cfg(not(windows))]
        {
            push_if_runnable(home.join(".local").join("bin").join("codex"), &mut out);
        }
    }
    // PATH por último (nome curto).
    #[cfg(windows)]
    {
        out.push(PathBuf::from("codex.exe"));
        out.push(PathBuf::from("codex.cmd"));
        out.push(PathBuf::from("codex"));
    }
    #[cfg(not(windows))]
    {
        out.push(PathBuf::from("codex"));
    }
    out
}

fn push_if_runnable(p: PathBuf, bins: &mut Vec<PathBuf>) {
    if p.is_file() && looks_runnable_on_host(&p) {
        bins.push(p);
    }
}

/// No Windows, só .exe/.cmd com cabeçalho PE (MZ) — ignora ELF linux da extensão.
fn looks_runnable_on_host(path: &Path) -> bool {
    #[cfg(windows)]
    {
        let name = path
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("")
            .to_ascii_lowercase();
        if name.ends_with(".cmd") || name.ends_with(".bat") {
            return true;
        }
        if !name.ends_with(".exe") {
            return false;
        }
        // Evitar bin/linux-* mesmo se renomeado.
        let s = path.to_string_lossy().replace('\\', "/").to_ascii_lowercase();
        if s.contains("/linux-") || s.contains("/darwin-") {
            return false;
        }
        matches_pe_mz(path)
    }
    #[cfg(not(windows))]
    {
        let _ = path;
        true
    }
}

fn matches_pe_mz(path: &Path) -> bool {
    use std::io::Read as _;
    let Ok(mut f) = std::fs::File::open(path) else {
        return false;
    };
    let mut magic = [0u8; 2];
    matches!(f.read_exact(&mut magic), Ok(())) && magic == *b"MZ"
}

fn push_extension_codex_bins(extensions_dir: &Path, bins: &mut Vec<PathBuf>) {
    let Ok(rd) = std::fs::read_dir(extensions_dir) else {
        return;
    };
    let mut found: Vec<(String, PathBuf)> = Vec::new();
    for ent in rd.flatten() {
        let name = ent.file_name().to_string_lossy().to_string();
        // Extensão oficial: openai.chatgpt-* (embute Codex em bin/<platform>/)
        if !name.starts_with("openai.chatgpt-") && !name.starts_with("openai.codex-") {
            continue;
        }
        let root = ent.path();
        // Só a pasta da plataforma atual — a extensão traz linux+windows juntos.
        let platform_bins: Vec<PathBuf> = {
            #[cfg(windows)]
            {
                vec![
                    root.join("bin").join("windows-x86_64").join("codex.exe"),
                    root.join("bin").join("windows-arm64").join("codex.exe"),
                    root.join("bin").join("codex.exe"),
                ]
            }
            #[cfg(all(unix, target_os = "linux"))]
            {
                vec![
                    root.join("bin").join("linux-x86_64").join("codex"),
                    root.join("bin").join("linux-aarch64").join("codex"),
                    root.join("bin").join("codex"),
                ]
            }
            #[cfg(all(unix, target_os = "macos"))]
            {
                vec![
                    root.join("bin").join("darwin-aarch64").join("codex"),
                    root.join("bin").join("darwin-x86_64").join("codex"),
                    root.join("bin").join("codex"),
                ]
            }
            #[cfg(not(any(
                windows,
                all(unix, target_os = "linux"),
                all(unix, target_os = "macos")
            )))]
            {
                vec![root.join("bin").join("codex")]
            }
        };
        for p in platform_bins {
            if p.is_file() && looks_runnable_on_host(&p) {
                found.push((name.clone(), p));
            }
        }
        // Heurística legada: node_modules/@openai/codex-*
        let vendor_roots = [
            root.join("node_modules").join("@openai"),
            root.join("resources").join("node_modules").join("@openai"),
        ];
        for vr in &vendor_roots {
            let Ok(sub) = std::fs::read_dir(vr) else {
                continue;
            };
            for pkg in sub.flatten() {
                let pname = pkg.file_name().to_string_lossy().to_string();
                if !pname.starts_with("codex") {
                    continue;
                }
                if let Some(bin) = find_codex_exe_under(&pkg.path()) {
                    if looks_runnable_on_host(&bin) {
                        found.push((name.clone(), bin));
                    }
                }
            }
        }
    }
    found.sort_by(|a, b| b.0.cmp(&a.0));
    for (_, p) in found {
        bins.push(p);
    }
}

fn find_codex_exe_under(dir: &Path) -> Option<PathBuf> {
    let mut stack = vec![dir.to_path_buf()];
    let mut depth = 0usize;
    while let Some(cur) = stack.pop() {
        depth += 1;
        if depth > 40 {
            break;
        }
        let Ok(rd) = std::fs::read_dir(&cur) else {
            continue;
        };
        for ent in rd.flatten() {
            let p = ent.path();
            if p.is_dir() {
                stack.push(p);
                continue;
            }
            let fname = p.file_name().and_then(|s| s.to_str()).unwrap_or("");
            #[cfg(windows)]
            let want = fname.eq_ignore_ascii_case("codex.exe");
            #[cfg(not(windows))]
            let want = fname == "codex";
            if want && looks_runnable_on_host(&p) {
                return Some(p);
            }
        }
    }
    None
}

fn spawn_codex(
    bin: &Path,
    args: &[String],
    stdin_bytes: Option<&[u8]>,
) -> Result<String, GitError> {
    let tmp = std::env::temp_dir().join(format!("trilho-codex-{}", std::process::id()));
    let _ = std::fs::create_dir_all(&tmp);

    let mut cmd = Command::new(bin);
    crate::infrastructure::subprocesso::sem_janela_de_console(&mut cmd)
        .args(args)
        .current_dir(&tmp)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        // Preferir auth ChatGPT em ~/.codex — não forçar API key do ambiente.
        .env_remove("OPENAI_API_KEY")
        .env_remove("CODEX_API_KEY");

    let mut child = match cmd.spawn() {
        Ok(c) => c,
        Err(e) => {
            if e.kind() == std::io::ErrorKind::NotFound {
                return Err(GitError::Io(format!(
                    "Codex CLI não encontrado ({})",
                    bin.display()
                )));
            }
            return Err(GitError::Io(format!("Codex CLI: falha ao iniciar: {e}")));
        }
    };

    let mut stdin = child
        .stdin
        .take()
        .ok_or_else(|| GitError::Io("Codex CLI: stdin indisponível".into()))?;
    let mut stdout = child
        .stdout
        .take()
        .ok_or_else(|| GitError::Io("Codex CLI: stdout indisponível".into()))?;
    let mut stderr = child
        .stderr
        .take()
        .ok_or_else(|| GitError::Io("Codex CLI: stderr indisponível".into()))?;

    let input = stdin_bytes.unwrap_or(b"").to_vec();
    let stdin_handle = std::thread::spawn(move || {
        use std::io::Write;
        let _ = stdin.write_all(&input);
        let _ = stdin.flush();
        drop(stdin);
    });
    let stdout_handle = std::thread::spawn(move || {
        let mut buf = String::new();
        let _ = stdout.read_to_string(&mut buf);
        buf
    });
    let stderr_handle = std::thread::spawn(move || {
        let mut buf = String::new();
        let _ = stderr.read_to_string(&mut buf);
        buf
    });

    let timeout = Duration::from_secs(CODEX_TIMEOUT_SECS);
    let start = Instant::now();
    let status = loop {
        match child.try_wait() {
            Ok(Some(st)) => break st,
            Ok(None) => {
                if start.elapsed() > timeout {
                    let _ = child.kill();
                    let _ = child.wait();
                    return Err(GitError::Io(format!(
                        "Codex CLI excedeu {CODEX_TIMEOUT_SECS}s sem responder. \
Tente de novo ou use um modelo/prompt menor."
                    )));
                }
                std::thread::sleep(Duration::from_millis(100));
            }
            Err(e) => {
                return Err(GitError::Io(format!("Codex CLI: erro ao aguardar: {e}")));
            }
        }
    };

    let _ = stdin_handle.join();
    let stdout = stdout_handle.join().unwrap_or_default();
    let stderr = stderr_handle.join().unwrap_or_default();

    if !status.success() {
        let combined = format!("{stdout}\n{stderr}");
        return Err(GitError::Io(format_codex_cli_err(
            combined.trim(),
            &combined,
        )));
    }
    if stdout.trim().is_empty() && !stderr.trim().is_empty() {
        return Err(GitError::Io(format_codex_cli_err(stderr.trim(), &stderr)));
    }
    Ok(stdout)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_texto_puro() {
        assert_eq!(parse_codex_exec_output("olá").unwrap(), "olá");
    }

    #[test]
    fn parse_auth_mensagem_clara() {
        let err = format_codex_cli_err("not logged in — run codex login", "not logged in");
        assert!(err.contains("codex login"));
    }

    #[test]
    fn resolve_inclui_nomes_path() {
        let bins = resolve_codex_bins();
        assert!(bins.iter().any(|p| {
            let s = p.to_string_lossy();
            s.ends_with("codex") || s.ends_with("codex.cmd") || s.ends_with("codex.exe")
        }));
    }

    #[test]
    fn resolve_acha_bin_da_extensao_se_instalada() {
        let bins = resolve_codex_bins();
        let ext = bins.iter().any(|p| {
            let s = p.to_string_lossy().replace('\\', "/");
            s.contains("openai.chatgpt-")
                && s.contains("windows-x86_64")
                && s.ends_with("codex.exe")
        });
        let has_win_bin = std::env::var_os("USERPROFILE")
            .or_else(|| std::env::var_os("HOME"))
            .map(PathBuf::from)
            .map(|h| h.join(".vscode").join("extensions"))
            .and_then(|d| std::fs::read_dir(d).ok())
            .map(|rd| {
                rd.flatten().any(|e| {
                    let n = e.file_name().to_string_lossy().to_string();
                    if !n.starts_with("openai.chatgpt-") {
                        return false;
                    }
                    e.path()
                        .join("bin")
                        .join("windows-x86_64")
                        .join("codex.exe")
                        .is_file()
                })
            })
            .unwrap_or(false);
        if has_win_bin {
            assert!(
                ext,
                "extensão com bin/windows-x86_64/codex.exe mas resolve não achou"
            );
            // Não deve oferecer o ELF linux.
            assert!(bins.iter().all(|p| {
                !p.to_string_lossy()
                    .replace('\\', "/")
                    .to_ascii_lowercase()
                    .contains("/linux-")
            }));
        }
    }

    #[test]
    fn pe_mz_detecta_exe() {
        #[cfg(windows)]
        {
            let bins = resolve_codex_bins();
            if let Some(exe) = bins.iter().find(|p| {
                p.to_string_lossy()
                    .replace('\\', "/")
                    .contains("windows-x86_64/codex.exe")
            }) {
                assert!(matches_pe_mz(exe));
            }
        }
    }

    #[test]
    fn skippable_so_launch_nao_runtime() {
        assert!(is_skippable_codex_launch_err(&GitError::Io(
            "Codex CLI não encontrado (codex)".into()
        )));
        assert!(is_skippable_codex_launch_err(&GitError::Io(
            "Codex CLI: falha ao iniciar: %1 não é um aplicativo Win32 válido. (os error 193)"
                .into()
        )));
        // Não mascarar erro real do exec (antes caía nisso por conter "Codex CLI").
        assert!(!is_skippable_codex_launch_err(&GitError::Io(
            "Codex CLI: unknown flag --ask-for-approval (Usage: …)".into()
        )));
        assert!(!is_skippable_codex_launch_err(&GitError::Io(
            "Codex CLI não está autenticado com ChatGPT. No terminal, rode `codex login`…".into()
        )));
    }
}
