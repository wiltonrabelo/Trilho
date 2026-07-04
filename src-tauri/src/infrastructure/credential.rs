//! Detecção do credential helper (RF-10 parcial — M1).

use serde::Serialize;
use std::process::Command;

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct CredentialStatus {
    pub helper_configured: bool,
    pub gcm_available: bool,
    pub helper_summary: Option<String>,
    /// Mensagem acionável quando fetch remoto pode falhar por credencial.
    pub hint: Option<String>,
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

    let hint = if gcm_available {
        None
    } else if helper_configured {
        Some(
            "Credential helper configurado, mas o Git Credential Manager (GCM) não foi detectado. \
             Instale o Git for Windows ou configure `git config --global credential.helper manager`."
                .into(),
        )
    } else {
        Some(
            "Nenhum credential helper configurado. Para fetch/push, instale o Git for Windows (GCM) \
             ou rode `git config --global credential.helper manager`."
                .into(),
        )
    };

    CredentialStatus {
        helper_configured,
        gcm_available,
        helper_summary,
        hint,
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
}
