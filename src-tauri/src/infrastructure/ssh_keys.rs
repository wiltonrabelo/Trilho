//! Detecção de chaves SSH e teste de autenticação GitHub (RF-10 recorte 2).

use crate::application::GitError;
use serde::Serialize;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SshKeyInfo {
    /// Nome base da chave (ex.: `id_ed25519`).
    pub name: String,
    pub has_public: bool,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SshTestResult {
    pub success: bool,
    pub username: Option<String>,
    pub message: String,
}

const SKIP_SSH_FILES: &[&str] = &[
    "known_hosts",
    "known_hosts.old",
    "config",
    "authorized_keys",
    "authorized_keys2",
    "environment",
    "rc",
];

pub fn list_ssh_keys() -> Vec<SshKeyInfo> {
    let Some(dir) = ssh_home_dir() else {
        return Vec::new();
    };
    if !dir.is_dir() {
        return Vec::new();
    }

    let mut keys = Vec::new();
    let entries = match std::fs::read_dir(&dir) {
        Ok(e) => e,
        Err(_) => return keys,
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        let Some(file_name) = path.file_name().and_then(|n| n.to_str()) else {
            continue;
        };
        if file_name.ends_with(".pub") || SKIP_SSH_FILES.contains(&file_name) {
            continue;
        }
        if file_name.starts_with('.') {
            continue;
        }
        let public = dir.join(format!("{file_name}.pub"));
        keys.push(SshKeyInfo {
            name: file_name.to_string(),
            has_public: public.is_file(),
        });
    }

    keys.sort_by(|a, b| a.name.cmp(&b.name));
    keys
}

pub fn read_ssh_public_key(name: &str) -> Result<String, GitError> {
    let trimmed = name.trim();
    if trimmed.is_empty()
        || trimmed.contains('/')
        || trimmed.contains('\\')
        || trimmed.contains("..")
        || trimmed.ends_with(".pub")
    {
        return Err(GitError::Git("Nome de chave SSH inválido.".into()));
    }
    let dir = ssh_home_dir().ok_or_else(|| GitError::Git("Pasta .ssh não encontrada.".into()))?;
    let path = dir.join(format!("{trimmed}.pub"));
    if !path.is_file() {
        return Err(GitError::Git(format!(
            "Chave pública não encontrada: {}",
            path.display()
        )));
    }
    let content = std::fs::read_to_string(&path)
        .map_err(|e| GitError::Io(format!("Não foi possível ler a chave pública: {e}")))?;
    let line = content
        .lines()
        .find(|l| !l.trim().is_empty())
        .unwrap_or("")
        .trim()
        .to_string();
    if line.is_empty() {
        return Err(GitError::Git("Arquivo de chave pública vazio.".into()));
    }
    Ok(line)
}

/// `ssh -T git@github.com` — GitHub responde exit 1 com mensagem de sucesso.
pub fn test_github_ssh() -> SshTestResult {
    let output = match std::process::Command::new("ssh")
        .args([
            "-T",
            "-o",
            "BatchMode=yes",
            "-o",
            "StrictHostKeyChecking=accept-new",
            "-o",
            "ConnectTimeout=12",
            "git@github.com",
        ])
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .output()
    {
        Ok(o) => o,
        Err(e) => {
            return SshTestResult {
                success: false,
                username: None,
                message: format!(
                    "OpenSSH não disponível neste PC ({e}). Instale o cliente OpenSSH do Windows."
                ),
            };
        }
    };

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let combined = format!("{stdout}{stderr}");

    if let Some(user) = parse_github_ssh_hi(&combined) {
        return SshTestResult {
            success: true,
            username: Some(user.clone()),
            message: format!("Autenticado no GitHub via SSH como @{user}."),
        };
    }

    let lower = combined.to_lowercase();
    if lower.contains("permission denied (publickey)") {
        return SshTestResult {
            success: false,
            username: None,
            message: "Nenhuma chave aceita pelo GitHub — adicione a chave pública em \
                      github.com/settings/keys e teste novamente."
                .into(),
        };
    }
    if lower.contains("could not resolve hostname") || lower.contains("connection timed out") {
        return SshTestResult {
            success: false,
            username: None,
            message: "Sem conexão com o GitHub — verifique a internet.".into(),
        };
    }

    SshTestResult {
        success: false,
        username: None,
        message: if combined.trim().is_empty() {
            "Não foi possível testar SSH com o GitHub.".into()
        } else {
            combined.trim().to_string()
        },
    }
}

fn parse_github_ssh_hi(text: &str) -> Option<String> {
    for line in text.lines() {
        let trimmed = line.trim();
        if let Some(rest) = trimmed.strip_prefix("Hi ") {
            if let Some(user) = rest.split('!').next() {
                let user = user.trim();
                if !user.is_empty() && !user.contains(' ') {
                    return Some(user.to_string());
                }
            }
        }
    }
    None
}

fn ssh_home_dir() -> Option<PathBuf> {
  home_dir().map(|h| h.join(".ssh"))
}

fn home_dir() -> Option<PathBuf> {
    #[cfg(windows)]
    {
        std::env::var_os("USERPROFILE").map(PathBuf::from)
    }
    #[cfg(not(windows))]
    {
        std::env::var_os("HOME").map(PathBuf::from)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_hi_github() {
        assert_eq!(
            parse_github_ssh_hi("Hi octocat! You've successfully authenticated.\n"),
            Some("octocat".into())
        );
        assert!(parse_github_ssh_hi("Permission denied (publickey).").is_none());
    }

    #[test]
    fn read_public_rejeita_path_traversal() {
        assert!(read_ssh_public_key("../id_rsa").is_err());
    }

    #[test]
    fn list_ssh_keys_retorna_vetor() {
        let keys = list_ssh_keys();
        for k in &keys {
            assert!(!k.name.is_empty());
        }
    }
}
