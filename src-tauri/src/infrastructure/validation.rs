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

/// URL de repositório remoto (HTTPS, SSH ou file).
pub fn validate_remote_url(url: &str) -> Result<String, GitError> {
    let trimmed = url.trim();
    if trimmed.is_empty() {
        return Err(GitError::Git("URL do repositório remoto vazia.".into()));
    }
    if trimmed.contains('\0') || trimmed.contains('\n') {
        return Err(GitError::Git("URL do repositório remoto inválida.".into()));
    }
    let lower = trimmed.to_lowercase();
    if lower.starts_with("https://")
        || lower.starts_with("http://")
        || lower.starts_with("git@")
        || lower.starts_with("ssh://")
        || lower.starts_with("file://")
    {
        Ok(trimmed.to_string())
    } else {
        Err(GitError::Git(
            "Informe uma URL Git válida (HTTPS ou SSH).".into(),
        ))
    }
}

/// Nome de pasta seguro no Windows (destino do clone).
pub fn validate_folder_name(name: &str) -> Result<String, GitError> {
    let trimmed = name.trim();
    if trimmed.is_empty() {
        return Err(GitError::Git("Nome da pasta vazio.".into()));
    }
    if trimmed == "." || trimmed == ".." {
        return Err(GitError::Git("Nome de pasta inválido.".into()));
    }
    const INVALID: &[char] = &['<', '>', ':', '"', '/', '\\', '|', '?', '*'];
    if trimmed.chars().any(|c| INVALID.contains(&c) || c.is_control()) {
        return Err(GitError::Git(
            "Nome da pasta contém caracteres inválidos no Windows.".into(),
        ));
    }
    Ok(trimmed.to_string())
}

/// Branch inicial do clone (`--branch`).
pub fn validate_clone_branch(branch: Option<&str>) -> Result<Option<String>, GitError> {
    match branch.map(str::trim).filter(|s| !s.is_empty()) {
        None => Ok(None),
        Some(b) if b.contains("..") || b.contains('\\') || b.contains(' ') || b.starts_with('-') => {
            Err(GitError::Git("Nome de branch inválido.".into()))
        }
        Some(b) => Ok(Some(b.to_string())),
    }
}

/// Nome de remoto Git (`origin`, `upstream`, …).
pub fn validate_remote_name(name: &str) -> Result<String, GitError> {
    let trimmed = name.trim();
    if trimmed.is_empty()
        || trimmed.contains('/')
        || trimmed.contains('\\')
        || trimmed.contains(' ')
        || trimmed.starts_with('-')
    {
        return Err(GitError::Git("Nome de remoto inválido.".into()));
    }
    Ok(trimmed.to_string())
}

/// Profundidade shallow (`--depth`).
pub fn validate_clone_depth(depth: Option<u32>) -> Result<Option<u32>, GitError> {
    match depth {
        None => Ok(None),
        Some(0) => Err(GitError::Git(
            "Profundidade do clone deve ser pelo menos 1.".into(),
        )),
        Some(d) => Ok(Some(d)),
    }
}

/// Destino do clone: não pode existir ou deve ser diretório vazio.
pub fn validate_clone_destination(path: &std::path::Path) -> Result<(), GitError> {
    if !path.exists() {
        return Ok(());
    }
    if !path.is_dir() {
        return Err(GitError::Git(
            "Já existe um arquivo com esse nome no destino.".into(),
        ));
    }
    let mut entries = std::fs::read_dir(path)
        .map_err(|e| GitError::Io(format!("Não foi possível ler o destino: {e}")))?;
    if entries.next().is_some() {
        return Err(GitError::Git(
            "A pasta de destino já existe e não está vazia.".into(),
        ));
    }
    Ok(())
}

/// Extrai nome do repositório a partir da URL (último segmento, sem `.git`).
pub fn repo_name_from_url(url: &str) -> Option<String> {
    let trimmed = url.trim().trim_end_matches('/');
    let segment = trimmed
        .rsplit(['/', ':'])
        .next()
        .filter(|s| !s.is_empty())?;
    let name = segment.strip_suffix(".git").unwrap_or(segment);
    if name.is_empty() {
        None
    } else {
        validate_folder_name(name).ok()
    }
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

    #[test]
    fn aceita_url_https() {
        assert!(validate_remote_url("https://github.com/user/repo.git").is_ok());
    }

    #[test]
    fn rejeita_url_invalida() {
        assert!(validate_remote_url("not-a-url").is_err());
    }

    #[test]
    fn extrai_nome_do_repo_da_url() {
        assert_eq!(
            repo_name_from_url("https://github.com/user/Trilho.git").as_deref(),
            Some("Trilho")
        );
    }

    #[test]
    fn rejeita_destino_nao_vazio() {
        let dir = std::env::temp_dir().join(format!("trilho-dest-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("x.txt"), "a").unwrap();
        assert!(validate_clone_destination(&dir).is_err());
        let _ = std::fs::remove_dir_all(&dir);
    }
}
