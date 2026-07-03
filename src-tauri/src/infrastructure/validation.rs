//! Validação de argumentos passados à CLI Git (PLANO §11.5).

use crate::application::GitError;

/// SHA-1 completo (40) ou abreviado (7–40), apenas hex.
pub fn validate_git_object_id(id: &str) -> Result<String, GitError> {
    let trimmed = id.trim();
    if trimmed.is_empty() {
        return Err(GitError::Git("Identificador de commit vazio.".into()));
    }
    if trimmed.starts_with('-') {
        return Err(GitError::Git(
            "Identificador de commit inválido (não pode começar com '-').".into(),
        ));
    }
    if trimmed.len() < 7 || trimmed.len() > 40 {
        return Err(GitError::Git(
            "Identificador de commit deve ter entre 7 e 40 caracteres hex.".into(),
        ));
    }
    if !trimmed.chars().all(|c| c.is_ascii_hexdigit()) {
        return Err(GitError::Git(
            "Identificador de commit deve conter apenas dígitos hexadecimais.".into(),
        ));
    }
    Ok(trimmed.to_string())
}

/// Caminho relativo dentro do repositório (evita injeção de flags via `--`).
pub fn validate_repo_relative_path(path: &str) -> Result<String, GitError> {
    let trimmed = path.trim();
    if trimmed.is_empty() {
        return Err(GitError::Git("Caminho de arquivo vazio.".into()));
    }
    if trimmed.starts_with('-') || trimmed.contains('\0') {
        return Err(GitError::Git("Caminho de arquivo inválido.".into()));
    }
    if trimmed.contains("..") {
        return Err(GitError::Git(
            "Caminho de arquivo não pode conter '..'.".into(),
        ));
    }
    Ok(trimmed.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn aceita_sha_valido() {
        assert!(validate_git_object_id("5dc5bb7").is_ok());
        assert!(validate_git_object_id("abcdef0123456789abcdef0123456789abcdef01").is_ok());
    }

    #[test]
    fn rejeita_flag_como_commit() {
        assert!(validate_git_object_id("--help").is_err());
    }

    #[test]
    fn rejeita_path_com_flag() {
        assert!(validate_repo_relative_path("--all").is_err());
    }
}
