//! Detecção e assistente de credenciais GitHub (RF-10).

use crate::infrastructure::ssh_keys::list_ssh_keys;
use serde::Serialize;
use std::io::Write;
use std::process::{Command, Stdio};

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct CredentialStatus {
    pub helper_configured: bool,
    pub gcm_available: bool,
    pub helper_summary: Option<String>,
    /// Mensagem acionável quando fetch remoto pode falhar por credencial.
    pub hint: Option<String>,
    /// Credencial GitHub já armazenada no helper (sem abrir GUI).
    pub github_connected: bool,
    /// Usuário retornado pelo helper, quando disponível.
    pub github_username: Option<String>,
    /// Chaves privadas detectadas em `~/.ssh`.
    pub ssh_keys: Vec<crate::infrastructure::SshKeyInfo>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct GithubCredentialProbe {
    connected: bool,
    username: Option<String>,
}

pub fn detect_credential_status() -> CredentialStatus {
    let helpers = read_credential_helpers();
    let helper_configured = !helpers.is_empty();
    let helper_summary = if helpers.is_empty() {
        None
    } else {
        Some(helpers.join(", "))
    };

    let gcm_in_config = helpers.iter().any(|h| helper_implies_gcm(h));
    let gcm_binary = probe_gcm_binary();
    let gcm_available = gcm_in_config || gcm_binary;

    let probe = probe_github_credential();
    let github_connected = probe.connected;
    let github_username = probe.username;
    let ssh_keys = list_ssh_keys();

    let hint = if gcm_available {
        None
    } else if helper_configured {
        Some(
            "Credential helper configurado, mas o Git Credential Manager (GCM) não foi detectado. \
             Instale o Git for Windows ou use «Configurar GCM» no assistente de conexão."
                .into(),
        )
    } else {
        Some(
            "Nenhum credential helper configurado. Instale o Git for Windows (GCM) \
             ou use «Configurar GCM» no assistente de conexão."
                .into(),
        )
    };

    CredentialStatus {
        helper_configured,
        gcm_available,
        helper_summary,
        hint,
        github_connected,
        github_username,
        ssh_keys,
    }
}

/// Configura `credential.helper manager` globalmente quando ausente.
pub fn ensure_gcm_configured() -> Result<(), String> {
    let status = detect_credential_status();
    if status.gcm_available {
        return Ok(());
    }
    let output = Command::new("git")
        .args(["config", "--global", "credential.helper", "manager"])
        .output()
        .map_err(|e| format!("Não foi possível executar git: {e}"))?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Falha ao configurar GCM: {stderr}"));
    }
    Ok(())
}

/// Abre o fluxo de login do GCM (OAuth/device flow) para GitHub.
pub fn trigger_github_login(remote_url: Option<&str>) -> Result<(), String> {
    ensure_gcm_configured()?;

    if try_gcm_github_login() {
        return Ok(());
    }

    let probe_url = remote_url
        .map(str::trim)
        .filter(|u| !u.is_empty() && u.contains("github"))
        .map(str::to_string)
        .unwrap_or_else(|| "https://github.com/octocat/Hello-World.git".into());

    let output = Command::new("git")
        .args(crate::infrastructure::git_cli::defensive_config_args())
        .arg("ls-remote")
        .arg("--heads")
        .arg(&probe_url)
        .env("GIT_TERMINAL_PROMPT", "0")
        .env("GCM_INTERACTIVE", "always")
        .output()
        .map_err(|e| format!("Não foi possível executar git: {e}"))?;

    if output.status.success() {
        return Ok(());
    }

    let stderr = String::from_utf8_lossy(&output.stderr);
    let lower = stderr.to_lowercase();
    if lower.contains("cancel") || lower.contains("user cancelled") {
        return Err("Login cancelado.".into());
    }
    if lower.contains("authentication failed") || lower.contains("invalid credentials") {
        return Err("Autenticação negada — tente novamente.".into());
    }

    // Repo inexistente ou sem permissão, mas credencial pode ter sido salva.
    let probe = probe_github_credential();
    if probe.connected {
        return Ok(());
    }

    Err(format!(
        "Não foi possível concluir o login. {}",
        stderr.trim()
    ))
}

/// Armazena PAT do GitHub via `git credential approve` (Windows Credential Manager).
pub fn store_github_pat(pat: &str) -> Result<(), String> {
    let pat = pat.trim();
    if pat.is_empty() {
        return Err("Informe o token de acesso pessoal.".into());
    }
    if pat.contains('\n') || pat.contains('\r') {
        return Err("Token inválido.".into());
    }

    ensure_gcm_configured()?;

    let input = format!(
        "protocol=https\nhost=github.com\nusername=git\npassword={pat}\n\n"
    );
    let mut child = Command::new("git")
        .args(["credential", "approve"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("Não foi possível executar git: {e}"))?;

    if let Some(mut stdin) = child.stdin.take() {
        stdin
            .write_all(input.as_bytes())
            .map_err(|e| format!("Falha ao enviar credencial: {e}"))?;
    }

    let output = child
        .wait_with_output()
        .map_err(|e| format!("Falha ao aguardar git: {e}"))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Não foi possível salvar o token: {stderr}"));
    }

    if !probe_github_credential().connected {
        return Err(
            "Token salvo, mas o helper não confirmou a credencial — verifique o token.".into(),
        );
    }

    Ok(())
}

fn try_gcm_github_login() -> bool {
    let output = Command::new("git")
        .args(["credential-manager", "github", "login"])
        .env("GCM_INTERACTIVE", "always")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output();

    match output {
        Ok(o) => o.status.success(),
        Err(_) => false,
    }
}

fn probe_github_credential() -> GithubCredentialProbe {
    let input = b"protocol=https\nhost=github.com\n\n";
    let mut child = match Command::new("git")
        .args([
            "-c",
            "credential.interactive=false",
            "credential",
            "fill",
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .env("GIT_TERMINAL_PROMPT", "0")
        .env("GCM_INTERACTIVE", "never")
        .spawn()
    {
        Ok(c) => c,
        Err(_) => {
            return GithubCredentialProbe {
                connected: false,
                username: None,
            }
        }
    };

    if let Some(mut stdin) = child.stdin.take() {
        let _ = stdin.write_all(input);
    }

    let output = match child.wait_with_output() {
        Ok(o) => o,
        Err(_) => {
            return GithubCredentialProbe {
                connected: false,
                username: None,
            }
        }
    };

    if !output.status.success() {
        return GithubCredentialProbe {
            connected: false,
            username: None,
        };
    }

    parse_credential_fill(&String::from_utf8_lossy(&output.stdout))
}

fn parse_credential_fill(stdout: &str) -> GithubCredentialProbe {
    let mut username: Option<String> = None;
    let mut has_password = false;

    for line in stdout.lines() {
        if let Some(u) = line.strip_prefix("username=") {
            if !u.is_empty() {
                username = Some(u.to_string());
            }
        }
        if let Some(p) = line.strip_prefix("password=") {
            has_password = !p.is_empty();
        }
    }

    GithubCredentialProbe {
        connected: has_password,
        username: if has_password { username } else { None },
    }
}

fn read_credential_helpers() -> Vec<String> {
    let output = Command::new("git")
        .args(["config", "--global", "--get-all", "credential.helper"])
        .output()
        .ok();
    let Some(output) = output else {
        return Vec::new();
    };
    if !output.status.success() {
        return Vec::new();
    }
    String::from_utf8_lossy(&output.stdout)
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty())
        .map(|s| s.to_string())
        .collect()
}

fn helper_implies_gcm(helper: &str) -> bool {
    let lower = helper.to_lowercase();
    lower.contains("manager")
        || lower.contains("gcm")
        || lower.contains("wincred")
        || lower.contains("osxkeychain")
        || lower.contains("libsecret")
}

fn probe_gcm_binary() -> bool {
    if command_exists("git-credential-manager") {
        return true;
    }
    if command_exists("git-credential-manager.exe") {
        return true;
    }
    #[cfg(windows)]
    {
        for path in windows_gcm_paths() {
            if std::path::Path::new(&path).exists() {
                return true;
            }
        }
    }
    false
}

fn command_exists(name: &str) -> bool {
    #[cfg(windows)]
    let probe = Command::new("where").arg(name).output();
    #[cfg(not(windows))]
    let probe = Command::new("which").arg(name).output();
    probe.map(|o| o.status.success()).unwrap_or(false)
}

#[cfg(windows)]
fn windows_gcm_paths() -> Vec<String> {
    let mut paths = Vec::new();
    if let Ok(pf) = std::env::var("ProgramFiles") {
        paths.push(format!(r"{pf}\Git\mingw64\bin\git-credential-manager.exe"));
        paths.push(format!(
            r"{pf}\Git\mingw64\libexec\git-core\git-credential-manager.exe"
        ));
    }
    if let Ok(pf86) = std::env::var("ProgramFiles(x86)") {
        paths.push(format!(
            r"{pf86}\Git\mingw64\bin\git-credential-manager.exe"
        ));
    }
    paths
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn helper_implies_gcm_reconhece_manager() {
        assert!(helper_implies_gcm("manager"));
        assert!(helper_implies_gcm("C:\\...\\git-credential-manager.exe"));
        assert!(!helper_implies_gcm("cache"));
    }

    #[test]
    fn detect_retorna_estrutura_valida() {
        let status = detect_credential_status();
        if status.gcm_available {
            assert!(status.hint.is_none());
        } else {
            assert!(status.hint.is_some());
        }
    }

    #[test]
    fn parse_credential_fill_detecta_password() {
        let probe = parse_credential_fill(
            "protocol=https\nhost=github.com\nusername=octocat\npassword=secret\n",
        );
        assert!(probe.connected);
        assert_eq!(probe.username.as_deref(), Some("octocat"));
    }

    #[test]
    fn parse_credential_fill_sem_password() {
        let probe = parse_credential_fill("protocol=https\nhost=github.com\n");
        assert!(!probe.connected);
    }

    #[test]
    fn store_pat_rejeita_vazio() {
        assert!(store_github_pat("  ").is_err());
    }
}
