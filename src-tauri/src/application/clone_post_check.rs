//! Checklist pós-clone (PLANO §RF-22) — anti-regressão Publicar.

use crate::application::write_gates::head_is_local_only;
use crate::application::{GitError, RepoContext};
use crate::infrastructure::repo_info;

/// Valida que o Git configurou remoto, upstream e sync após `git clone`.
pub fn validate_post_clone(repo_path: &str) -> Result<(), GitError> {
    let mut issues: Vec<String> = Vec::new();
    let info = repo_info(repo_path)?;

    if !info.has_remote {
        issues.push("Remoto não configurado (esperado «origin» após clone).".into());
    }
    if info.remote_url.is_none() {
        issues.push("URL do remoto ausente.".into());
    }
    if info.is_detached {
        issues.push("Repositório em detached HEAD após clone.".into());
    }
    if info.upstream.is_none() {
        issues.push(
            "Branch sem upstream — o clone deveria configurar rastreamento automaticamente.".into(),
        );
    }

    let ctx = RepoContext::open(repo_path)?;
    let sync = ctx.reader().get_sync_info()?;
    if sync.upstream.is_none() {
        issues.push("Indicador de sync sem upstream.".into());
    }

    if info.has_commits && head_is_local_only(ctx.reader(), ctx.writer())? {
        issues.push(
            "HEAD tratado como só local — amend/uncommit podem se comportar como após Publicar \
             sem upstream."
                .into(),
        );
    }

    if issues.is_empty() {
        Ok(())
    } else {
        Err(GitError::Git(format!(
            "Clone concluído, mas o repositório não passou na validação pós-clone:\n• {}",
            issues.join("\n• ")
        )))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use std::process::Command;

    fn run_git(cwd: &PathBuf, args: &[&str]) {
        let status = Command::new("git")
            .args(args)
            .current_dir(cwd)
            .status()
            .expect("git");
        assert!(status.success(), "git {:?} em {}", args, cwd.display());
    }

    /// Repo bare + clone local — espelha clone HTTPS bem-sucedido.
    fn cloned_fixture() -> (PathBuf, PathBuf) {
        let root = std::env::temp_dir().join(format!("trilho-post-clone-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        let bare = root.join("origin.git");
        let work = root.join("work");
        let clone = root.join("clone");
        std::fs::create_dir_all(&bare).unwrap();
        std::fs::create_dir_all(&work).unwrap();

        run_git(&bare, &["init", "--bare"]);
        run_git(&work, &["init", "-b", "main"]);
        run_git(&work, &["config", "user.email", "t@t.com"]);
        run_git(&work, &["config", "user.name", "T"]);
        std::fs::write(work.join("f.txt"), "x").unwrap();
        run_git(&work, &["add", "f.txt"]);
        run_git(&work, &["commit", "-m", "init"]);
        run_git(
            &work,
            &["remote", "add", "origin", bare.to_str().unwrap()],
        );
        run_git(&work, &["push", "-u", "origin", "main"]);
        run_git(&bare, &["symbolic-ref", "HEAD", "refs/heads/main"]);

        run_git(
            &root,
            &[
                "clone",
                bare.to_str().unwrap(),
                clone.file_name().unwrap().to_str().unwrap(),
            ],
        );

        (root, clone)
    }

    #[test]
    fn post_clone_checklist_ok_em_clone_local() {
        let (root, clone) = cloned_fixture();
        validate_post_clone(clone.to_str().unwrap()).expect("checklist pós-clone");
        let _ = std::fs::remove_dir_all(&root);
    }
}
