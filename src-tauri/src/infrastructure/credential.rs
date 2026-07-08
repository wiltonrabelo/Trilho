//! Detecção e assistente de credenciais GitHub (RF-10).

use crate::infrastructure::ssh_keys::list_ssh_keys;
use serde::Serialize;
use std::io::Write;
use std::process::{Command, Stdio};

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct GithubAccount {
    pub username: String,
    pub is_active: bool,
}

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
    /// Contas HTTPS salvas no GCM (`git credential-manager github list`).
    pub github_accounts: Vec<GithubAccount>,
    /// `credential.https://github.com.useHttpPath` — separa credenciais por repositório.
    pub use_http_path: bool,
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
    let github_username = probe.username.clone();
    let use_http_path = read_github_use_http_path();
    let github_accounts = list_github_accounts(github_username.as_deref());
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
        github_accounts,
        use_http_path,
        ssh_keys,
    }
}

/// Lista contas GitHub salvas no GCM. Se o comando não existir, devolve só a conta ativa.
pub fn list_github_accounts(active_username: Option<&str>) -> Vec<GithubAccount> {
    let mut usernames = run_gcm_github_list().unwrap_or_default();
    if usernames.is_empty() {
        if let Some(user) = active_username.filter(|u| !u.is_empty() && *u != "git") {
            usernames.push(user.to_string());
        }
    }
    usernames.sort();
    usernames.dedup();
    usernames
        .into_iter()
        .map(|username| GithubAccount {
            is_active: active_username == Some(username.as_str()),
            username,
        })
        .collect()
}

/// Remove conta GitHub do GCM (`git credential-manager github logout`).
pub fn logout_github_account(username: &str) -> Result<(), String> {
    let username = username.trim();
    if username.is_empty() || username == "git" {
        return Err("Informe o usuário GitHub a remover.".into());
    }
    if username.contains('\n') || username.contains(' ') {
        return Err("Nome de usuário inválido.".into());
    }
    let output = run_gcm_github(&["logout", username])?;
    if output.status.success() {
        return Ok(());
    }
    let stderr = String::from_utf8_lossy(&output.stderr);
    let stdout = String::from_utf8_lossy(&output.stdout);
    Err(format!(
        "Não foi possível remover a conta: {}",
        if stderr.trim().is_empty() {
            stdout.trim()
        } else {
            stderr.trim()
        }
    ))
}

pub fn read_github_use_http_path() -> bool {
    Command::new("git")
        .args([
            "config",
            "--global",
            "--get",
            "credential.https://github.com.useHttpPath",
        ])
        .output()
        .ok()
        .filter(|o| o.status.success())
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|v| {
            let v = v.trim().to_lowercase();
            v == "true" || v == "1" || v == "yes"
        })
        .unwrap_or(false)
}

/// Recomendado para múltiplas contas HTTPS no mesmo PC (credencial por caminho do repo).
pub fn enable_github_use_http_path() -> Result<(), String> {
    let output = Command::new("git")
        .args([
            "config",
            "--global",
            "credential.https://github.com.useHttpPath",
            "true",
        ])
        .output()
        .map_err(|e| format!("Não foi possível executar git: {e}"))?;
    if output.status.success() {
        Ok(())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(format!("Falha ao configurar useHttpPath: {stderr}"))
    }
}

fn run_gcm_github_list() -> Result<Vec<String>, String> {
    let output = run_gcm_github(&["list"])?;
    if !output.status.success() {
        return Err(String::from_utf8_lossy(&output.stderr).into_owned());
    }
    Ok(parse_github_list_output(&String::from_utf8_lossy(&output.stdout)))
}

fn run_gcm_github(args: &[&str]) -> Result<std::process::Output, String> {
    let mut last_err = String::new();
    for invocation in [
        ("git", vec!["credential-manager"]),
        ("git-credential-manager", vec![]),
        ("git-credential-manager.exe", vec![]),
    ] {
        let (bin, prefix): (&str, Vec<&str>) = invocation;
        let mut cmd_args: Vec<&str> = prefix.iter().copied().collect();
        cmd_args.push("github");
        cmd_args.extend(args);
        let output = Command::new(bin)
            .args(&cmd_args)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output();
        match output {
            Ok(o) => return Ok(o),
            Err(e) => last_err = format!("{bin}: {e}"),
        }
    }
    Err(last_err)
}

fn parse_github_list_output(stdout: &str) -> Vec<String> {
    let mut out = Vec::new();
    for line in stdout.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let lower = line.to_lowercase();
        if lower.contains("no accounts")
            || lower.contains("nenhuma conta")
            || lower.starts_with("usage:")
            || lower.starts_with("description:")
        {
            continue;
        }
        let user = line
            .strip_prefix('@')
            .or_else(|| line.strip_prefix("login:").map(str::trim))
            .or_else(|| line.strip_prefix("username:").map(str::trim))
            .unwrap_or(line);
        let user = user.split_whitespace().next().unwrap_or(user).trim();
        if user.is_empty() || user == "git" || user.contains('/') {
            continue;
        }
        out.push(user.to_string());
    }
    out
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
    let fill = run_credential_fill(b"protocol=https\nhost=github.com\n\n");
    GithubCredentialProbe {
        connected: fill.password.as_ref().is_some_and(|p| !p.is_empty()),
        username: fill.username,
    }
}

/// Token HTTPS para a API GitHub (password do credential helper).
pub fn get_github_api_token(credential_path: Option<&str>) -> Option<String> {
    let mut input = String::from("protocol=https\nhost=github.com\n");
    if read_github_use_http_path() {
        if let Some(path) = credential_path.filter(|p| !p.is_empty()) {
            let path = path.trim_start_matches('/');
            input.push_str(&format!("path={path}\n"));
        }
    }
    input.push('\n');
    let fill = run_credential_fill(input.as_bytes());
    fill.password.filter(|p| !p.is_empty())
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct CredentialFill {
    username: Option<String>,
    password: Option<String>,
}

fn run_credential_fill(input: &[u8]) -> CredentialFill {
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
            return CredentialFill {
                username: None,
                password: None,
            }
        }
    };

    if let Some(mut stdin) = child.stdin.take() {
        let _ = stdin.write_all(input);
    }

    let output = match child.wait_with_output() {
        Ok(o) => o,
        Err(_) => {
            return CredentialFill {
                username: None,
                password: None,
            }
        }
    };

    if !output.status.success() {
        return CredentialFill {
            username: None,
            password: None,
        };
    }

    parse_credential_fill_full(&String::from_utf8_lossy(&output.stdout))
}

fn parse_credential_fill_full(stdout: &str) -> CredentialFill {
    let mut username: Option<String> = None;
    let mut password: Option<String> = None;

    for line in stdout.lines() {
        if let Some(u) = line.strip_prefix("username=") {
            if !u.is_empty() {
                username = Some(u.to_string());
            }
        }
        if let Some(p) = line.strip_prefix("password=") {
            if !p.is_empty() {
                password = Some(p.to_string());
            }
        }
    }

    CredentialFill { username, password }
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
        let fill = parse_credential_fill_full(
            "protocol=https\nhost=github.com\nusername=octocat\npassword=secret\n",
        );
        assert_eq!(fill.password.as_deref(), Some("secret"));
        assert_eq!(fill.username.as_deref(), Some("octocat"));
    }

    #[test]
    fn parse_credential_fill_sem_password() {
        let fill = parse_credential_fill_full("protocol=https\nhost=github.com\n");
        assert!(fill.password.is_none());
    }

    #[test]
    fn parse_github_list_output_ignora_vazio() {
        assert!(parse_github_list_output("").is_empty());
        assert!(parse_github_list_output("No accounts found.\n").is_empty());
    }

    #[test]
    fn parse_github_list_output_usernames() {
        let list = parse_github_list_output("octocat\n@hubot\nlogin: devuser\n");
        assert_eq!(list, vec!["octocat", "hubot", "devuser"]);
    }

    #[test]
    fn store_pat_rejeita_vazio() {
        assert!(store_github_pat("  ").is_err());
    }
}
