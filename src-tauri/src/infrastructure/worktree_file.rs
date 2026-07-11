//! Leitura/gravação de arquivos no working tree (editor interno).

use git2::Repository;
use std::path::Path;

use crate::application::GitError;
use crate::infrastructure::validation::validate_repo_relative_path;

/// Grava conteúdo no working tree sem alterar o stage.
pub fn save_worktree_file(repo_path: &str, path: &str, content: &str) -> Result<(), GitError> {
    let path = validate_repo_relative_path(path)?;
    let repo = Repository::discover(repo_path)
        .map_err(|e| GitError::Io(format!("Não foi possível abrir o repositório: {e}")))?;
    let workdir = repo
        .workdir()
        .ok_or_else(|| GitError::Git("Repositório bare — sem working tree.".into()))?;
    let full = workdir.join(&path);
    if let Some(parent) = full.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| GitError::Io(format!("Não foi possível criar pasta: {e}")))?;
    }
    std::fs::write(&full, content)
        .map_err(|e| GitError::Io(format!("Não foi possível gravar {path}: {e}")))?;
    Ok(())
}

/// Verifica se o path existe no disco dentro do working tree.
pub fn worktree_file_exists(repo_path: &str, path: &str) -> Result<bool, GitError> {
    let path = validate_repo_relative_path(path)?;
    let repo = Repository::discover(repo_path)
        .map_err(|e| GitError::Io(format!("Não foi possível abrir o repositório: {e}")))?;
    let workdir = repo
        .workdir()
        .ok_or_else(|| GitError::Git("Repositório bare — sem working tree.".into()))?;
    let full = workdir.join(Path::new(&path));
    Ok(full.is_file())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::infrastructure::git_cli::SafeGitCli;
    use std::fs;
    use std::process::Command;

    fn init_repo(dir: &std::path::Path) {
        Command::new("git")
            .args(["init", dir.to_str().unwrap()])
            .output()
            .unwrap();
        Command::new("git")
            .args(["config", "user.email", "t@t.com"])
            .current_dir(dir)
            .output()
            .unwrap();
        Command::new("git")
            .args(["config", "user.name", "t"])
            .current_dir(dir)
            .output()
            .unwrap();
    }

    #[test]
    fn salva_arquivo_no_working_tree() {
        let dir = std::env::temp_dir().join(format!("trilho-wt-save-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        init_repo(&dir);
        fs::write(dir.join("foo.txt"), "antes\n").unwrap();
        let cli = SafeGitCli::new(dir.to_str().unwrap());
        cli.run(&crate::application::GitCommand {
            args: vec!["add".into(), "foo.txt".into()],
        })
        .unwrap();
        cli.run(&crate::application::GitCommand {
            args: vec!["commit".into(), "-m".into(), "init".into()],
        })
        .unwrap();

        save_worktree_file(dir.to_str().unwrap(), "foo.txt", "depois\n").unwrap();
        let disk = fs::read_to_string(dir.join("foo.txt")).unwrap();
        assert_eq!(disk, "depois\n");
    }
}
