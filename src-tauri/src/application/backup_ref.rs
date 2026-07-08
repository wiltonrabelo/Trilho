//! Backup local via refs `refs/trilho/backup/<branch>/<timestamp>` (RF-07/RF-09).

use crate::application::{GitCommand, GitError, GitWriter};

const MAX_BACKUPS_PER_BRANCH: usize = 20;
const BACKUP_PREFIX: &str = "refs/trilho/backup/";

/// Normaliza nome de branch para uso seguro em ref names.
pub fn sanitize_branch_for_ref(branch: &str) -> String {
    branch
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '-' || c == '_' || c == '.' {
                c
            } else {
                '-'
            }
        })
        .collect()
}

/// Comando exibido no preview RF-08 (timestamp real só na execução).
pub fn backup_ref_preview_command(branch: &str) -> String {
    format!(
        "git update-ref refs/trilho/backup/{}/<timestamp> HEAD",
        sanitize_branch_for_ref(branch)
    )
}

/// Grava backup do HEAD atual e remove refs antigas além do limite por branch.
pub fn create_backup_ref(cli: &dyn GitWriter, branch: &str) -> Result<String, GitError> {
    let head = cli
        .run(&GitCommand {
            args: vec!["rev-parse".into(), "HEAD".into()],
        })?
        .trim()
        .to_string();
    let ts = chrono::Local::now().format("%Y%m%d-%H%M%S");
    let ref_name = format!(
        "{}{}/{}",
        BACKUP_PREFIX,
        sanitize_branch_for_ref(branch),
        ts
    );
    cli.run(&GitCommand {
        args: vec!["update-ref".into(), ref_name.clone(), head],
    })?;
    prune_old_backup_refs(cli, branch)?;
    Ok(ref_name)
}

fn prune_old_backup_refs(cli: &dyn GitWriter, branch: &str) -> Result<(), GitError> {
    let prefix = format!("{}{}/", BACKUP_PREFIX, sanitize_branch_for_ref(branch));
    let out = cli
        .run(&GitCommand {
            args: vec![
                "for-each-ref".into(),
                "--format=%(refname)".into(),
                prefix,
            ],
        })
        .unwrap_or_default();
    let mut refs: Vec<String> = out
        .lines()
        .filter(|line| !line.is_empty())
        .map(str::to_string)
        .collect();
    refs.sort();
    if refs.len() <= MAX_BACKUPS_PER_BRANCH {
        return Ok(());
    }
    let excess = refs.len() - MAX_BACKUPS_PER_BRANCH;
    for old in refs.into_iter().take(excess) {
        cli.run(&GitCommand {
            args: vec!["update-ref".into(), "-d".into(), old],
        })?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sanitize_substitui_caracteres_invalidos() {
        assert_eq!(sanitize_branch_for_ref("feature/foo"), "feature-foo");
        assert_eq!(sanitize_branch_for_ref("main"), "main");
    }

    #[test]
    fn preview_inclui_prefixo_trilho() {
        let cmd = backup_ref_preview_command("main");
        assert!(cmd.contains("refs/trilho/backup/main/"));
        assert!(cmd.contains("<timestamp>"));
    }
}
