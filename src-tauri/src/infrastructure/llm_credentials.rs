//! RF-21 — API keys no Windows Credential Manager via `git credential`.

use std::io::Write;
use std::process::{Command, Stdio};

fn host_for(provider: &str) -> String {
    format!("trilho.llm.{provider}")
}

fn run_credential(args: &[&str], stdin: &[u8]) -> Result<String, String> {
    let mut child = Command::new("git")
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("Não foi possível executar git: {e}"))?;
    if let Some(mut s) = child.stdin.take() {
        s.write_all(stdin)
            .map_err(|e| format!("Falha ao enviar credencial: {e}"))?;
    }
    let output = child
        .wait_with_output()
        .map_err(|e| format!("Falha ao aguardar git: {e}"))?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Credencial LLM: {stderr}"));
    }
    Ok(String::from_utf8_lossy(&output.stdout).into_owned())
}

fn parse_password(fill: &str) -> Option<String> {
    for line in fill.lines() {
        if let Some(rest) = line.strip_prefix("password=") {
            let p = rest.trim();
            if !p.is_empty() {
                return Some(p.to_string());
            }
        }
    }
    None
}

pub fn store_llm_api_key(provider: &str, key: &str) -> Result<(), String> {
    let key = key.trim();
    if key.is_empty() {
        return Err("Informe a chave da API.".into());
    }
    if key.contains('\n') || key.contains('\r') {
        return Err("Chave inválida.".into());
    }
    let host = host_for(provider);
    let input = format!("protocol=https\nhost={host}\nusername=trilho\npassword={key}\n\n");
    run_credential(&["credential", "approve"], input.as_bytes())?;
    if get_llm_api_key(provider).is_none() {
        return Err("Chave salva, mas o helper não confirmou — tente de novo.".into());
    }
    Ok(())
}

pub fn clear_llm_api_key(provider: &str) -> Result<(), String> {
    let host = host_for(provider);
    let input = format!("protocol=https\nhost={host}\nusername=trilho\n\n");
    let _ = run_credential(&["credential", "reject"], input.as_bytes());
    Ok(())
}

pub fn get_llm_api_key(provider: &str) -> Option<String> {
    let host = host_for(provider);
    let input = format!("protocol=https\nhost={host}\n\n");
    let fill = run_credential(&["credential", "fill"], input.as_bytes()).ok()?;
    parse_password(&fill)
}

pub fn has_llm_api_key(provider: &str) -> bool {
    get_llm_api_key(provider).is_some_and(|k| !k.is_empty())
}
