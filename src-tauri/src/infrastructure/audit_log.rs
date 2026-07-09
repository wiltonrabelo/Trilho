//! RF-11 — persistência JSONL + retenção de 7 dias.

use std::fs::{self, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};

use chrono::{Duration, Local, NaiveDate};

use crate::application::GitError;
use crate::domain::{AuditEntry, AuditResult};

const RETENTION_DAYS: i64 = 7;
const FILE_PREFIX: &str = "actions-";
const FILE_SUFFIX: &str = ".jsonl";

/// Diretório `%APPDATA%/…/logs` (ou equivalente via `app_data_dir`).
pub fn logs_dir(app_data_dir: &Path) -> PathBuf {
    app_data_dir.join("logs")
}

pub fn ensure_logs_dir(app_data_dir: &Path) -> Result<PathBuf, GitError> {
    let dir = logs_dir(app_data_dir);
    fs::create_dir_all(&dir).map_err(|e| GitError::Io(format!("Não foi possível criar logs/: {e}")))?;
    Ok(dir)
}

fn today_file(dir: &Path) -> PathBuf {
    let day = Local::now().format("%Y-%m-%d");
    dir.join(format!("{FILE_PREFIX}{day}{FILE_SUFFIX}"))
}

/// Remove arquivos `actions-YYYY-MM-DD.jsonl` com mais de 7 dias.
pub fn purge_old_logs(app_data_dir: &Path) -> Result<usize, GitError> {
    let dir = ensure_logs_dir(app_data_dir)?;
    let cutoff = (Local::now().date_naive() - Duration::days(RETENTION_DAYS)).to_string();
    let mut removed = 0;
    let entries = fs::read_dir(&dir)
        .map_err(|e| GitError::Io(format!("Não foi possível listar logs/: {e}")))?;
    for entry in entries.flatten() {
        let name = entry.file_name();
        let name = name.to_string_lossy();
        let Some(date_str) = name
            .strip_prefix(FILE_PREFIX)
            .and_then(|s| s.strip_suffix(FILE_SUFFIX))
        else {
            continue;
        };
        if date_str < cutoff.as_str() {
            if fs::remove_file(entry.path()).is_ok() {
                removed += 1;
            }
        }
    }
    Ok(removed)
}

/// Sanitiza texto antes de gravar — nunca tokens/senhas.
pub fn sanitize_for_audit(text: &str) -> String {
    let mut out = text.to_string();
    // Tokens GitHub clássicos / fine-grained / ghp_
    let patterns = [
        (r"(?i)ghp_[A-Za-z0-9]{20,}", "[redacted-token]"),
        (r"(?i)github_pat_[A-Za-z0-9_]{20,}", "[redacted-token]"),
        (r"(?i)gho_[A-Za-z0-9]{20,}", "[redacted-token]"),
        (r"(?i)ghu_[A-Za-z0-9]{20,}", "[redacted-token]"),
        (r"(?i)ghs_[A-Za-z0-9]{20,}", "[redacted-token]"),
        (r"(?i)ghr_[A-Za-z0-9]{20,}", "[redacted-token]"),
        // password=… / token=… em URLs
        (r"(?i)(password|passwd|token|secret|authorization)=([^&\s]+)", "$1=[redacted]"),
        // https://user:pass@host
        (r"(https?://)([^:@/\s]+):([^@/\s]+)@", "$1$2:[redacted]@"),
    ];
    for (pat, repl) in patterns {
        if let Ok(re) = regex_lite_replace(pat, repl, &out) {
            out = re;
        }
    }
    out
}

/// Substituição simples sem dependência `regex` — padrões fixos do RF-11.
fn regex_lite_replace(pat: &str, repl: &str, input: &str) -> Result<String, ()> {
    // Implementação mínima via varredura para os padrões acima (sem crate regex).
    match pat {
        p if p.contains("ghp_") => Ok(redact_prefix_token(input, "ghp_")),
        p if p.contains("github_pat_") => Ok(redact_prefix_token(input, "github_pat_")),
        p if p.contains("gho_") => Ok(redact_prefix_token(input, "gho_")),
        p if p.contains("ghu_") => Ok(redact_prefix_token(input, "ghu_")),
        p if p.contains("ghs_") => Ok(redact_prefix_token(input, "ghs_")),
        p if p.contains("ghr_") => Ok(redact_prefix_token(input, "ghr_")),
        p if p.contains("password|passwd|token") => Ok(redact_query_secrets(input)),
        p if p.contains("https?://") => Ok(redact_url_userinfo(input)),
        _ => {
            let _ = repl;
            Err(())
        }
    }
}

fn redact_prefix_token(input: &str, prefix: &str) -> String {
    let lower = input.to_ascii_lowercase();
    let pref_lower = prefix.to_ascii_lowercase();
    let mut out = String::with_capacity(input.len());
    let bytes = input.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if lower[i..].starts_with(&pref_lower) {
            out.push_str("[redacted-token]");
            i += prefix.len();
            while i < bytes.len() {
                let c = bytes[i] as char;
                if c.is_ascii_alphanumeric() || c == '_' {
                    i += 1;
                } else {
                    break;
                }
            }
        } else {
            out.push(bytes[i] as char);
            i += 1;
        }
    }
    out
}

fn redact_query_secrets(input: &str) -> String {
    let keys = ["password=", "passwd=", "token=", "secret=", "authorization="];
    let mut out = input.to_string();
    for key in keys {
        let lower = out.to_ascii_lowercase();
        let mut search_from = 0;
        while let Some(rel) = lower[search_from..].find(key) {
            let start = search_from + rel;
            let val_start = start + key.len();
            let val_end = out[val_start..]
                .find(|c: char| c == '&' || c.is_whitespace())
                .map(|n| val_start + n)
                .unwrap_or(out.len());
            out.replace_range(val_start..val_end, "[redacted]");
            search_from = val_start + "[redacted]".len();
            if search_from >= out.len() {
                break;
            }
        }
    }
    out
}

fn redact_url_userinfo(input: &str) -> String {
    // https://user:pass@host → https://user:[redacted]@host
    let mut out = String::with_capacity(input.len());
    let mut rest = input;
    while let Some(scheme_at) = rest.find("://") {
        out.push_str(&rest[..scheme_at + 3]);
        rest = &rest[scheme_at + 3..];
        if let Some(at) = rest.find('@') {
            let creds = &rest[..at];
            if let Some(colon) = creds.find(':') {
                out.push_str(&creds[..colon + 1]);
                out.push_str("[redacted]");
                out.push('@');
                rest = &rest[at + 1..];
                continue;
            }
        }
        // sem userinfo — copia até próximo possível
        if let Some(next) = rest.find("://") {
            out.push_str(&rest[..next]);
            rest = &rest[next..];
        } else {
            out.push_str(rest);
            rest = "";
        }
    }
    out.push_str(rest);
    out
}

pub fn append_entry(app_data_dir: &Path, mut entry: AuditEntry) -> Result<(), GitError> {
    let dir = ensure_logs_dir(app_data_dir)?;
    entry.command = sanitize_for_audit(&entry.command);
    if let Some(err) = entry.error.take() {
        entry.error = Some(sanitize_for_audit(&err));
    }
    let path = today_file(&dir);
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)
        .map_err(|e| GitError::Io(format!("Não foi possível abrir log: {e}")))?;
    let line = serde_json::to_string(&entry)
        .map_err(|e| GitError::Io(format!("Falha ao serializar auditoria: {e}")))?;
    writeln!(file, "{line}").map_err(|e| GitError::Io(format!("Falha ao gravar auditoria: {e}")))?;
    Ok(())
}

/// Lê entradas dos últimos `days` (máx. retenção), mais recentes primeiro.
pub fn list_entries(app_data_dir: &Path, days: u32) -> Result<Vec<AuditEntry>, GitError> {
    let _ = purge_old_logs(app_data_dir);
    let dir = ensure_logs_dir(app_data_dir)?;
    let days = days.min(RETENTION_DAYS as u32).max(1);
    let today = Local::now().date_naive();
    let mut entries = Vec::new();

    for offset in 0..days {
        let day = today - Duration::days(offset as i64);
        let path = dir.join(format!(
            "{FILE_PREFIX}{}{FILE_SUFFIX}",
            day.format("%Y-%m-%d")
        ));
        if !path.exists() {
            continue;
        }
        let file = fs::File::open(&path)
            .map_err(|e| GitError::Io(format!("Não foi possível ler {}: {e}", path.display())))?;
        for line in BufReader::new(file).lines().flatten() {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }
            if let Ok(entry) = serde_json::from_str::<AuditEntry>(line) {
                entries.push(entry);
            }
        }
    }

    entries.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
    Ok(entries)
}

pub fn now_timestamp() -> String {
    Local::now().to_rfc3339()
}

/// Utilitário de teste: parseia data do nome do arquivo.
#[cfg(test)]
fn parse_log_date(name: &str) -> Option<NaiveDate> {
    let date_str = name
        .strip_prefix(FILE_PREFIX)?
        .strip_suffix(FILE_SUFFIX)?;
    NaiveDate::parse_from_str(date_str, "%Y-%m-%d").ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::AuditAction;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_data_dir() -> PathBuf {
        let n = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("trilho-audit-{n}"));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn sanitize_redacta_ghp_token() {
        let s = sanitize_for_audit("auth ghp_abcdefghijklmnopqrstuvwxyz123456 error");
        assert!(!s.contains("ghp_abcd"));
        assert!(s.contains("[redacted-token]"));
    }

    #[test]
    fn sanitize_redacta_password_em_url() {
        let s = sanitize_for_audit("https://user:s3cret@github.com/org/repo.git");
        assert!(!s.contains("s3cret"));
        assert!(s.contains("[redacted]"));
    }

    #[test]
    fn append_e_list_roundtrip() {
        let dir = temp_data_dir();
        let entry = AuditEntry {
            timestamp: now_timestamp(),
            action: AuditAction::Commit,
            command: "git commit -m test".into(),
            repo: "C:/tmp/repo".into(),
            branch: Some("main".into()),
            commits: vec![],
            result: AuditResult::Success,
            error: None,
            from_assistant: false,
        };
        append_entry(&dir, entry.clone()).unwrap();
        let listed = list_entries(&dir, 7).unwrap();
        assert_eq!(listed.len(), 1);
        assert_eq!(listed[0].action, AuditAction::Commit);
        assert_eq!(listed[0].result, AuditResult::Success);
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn purge_remove_arquivo_antigo() {
        let dir = temp_data_dir();
        let logs = ensure_logs_dir(&dir).unwrap();
        let old = logs.join("actions-2000-01-01.jsonl");
        fs::write(&old, "{}\n").unwrap();
        let today = today_file(&logs);
        fs::write(&today, "{}\n").unwrap();
        let n = purge_old_logs(&dir).unwrap();
        assert!(n >= 1);
        assert!(!old.exists());
        assert!(today.exists());
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn parse_log_date_ok() {
        assert_eq!(
            parse_log_date("actions-2026-07-09.jsonl"),
            Some(NaiveDate::from_ymd_opt(2026, 7, 9).unwrap())
        );
    }
}
