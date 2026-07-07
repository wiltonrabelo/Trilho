//! Reword via cherry-pick (RF-16 recorte 1) — evita `rebase -i` interativo no Windows.

use crate::application::{GitCommand, GitError};
use crate::infrastructure::SafeGitCli;

/// Reescreve a mensagem de um commit não-HEAD reaplicando os descendentes.
pub fn execute_reword(cli: &SafeGitCli, target_sha: &str, new_message: &str) -> Result<(), GitError> {
    let branch = cli
        .run(&GitCommand {
            args: vec!["branch".into(), "--show-current".into()],
        })?
        .trim()
        .to_string();
    if branch.is_empty() {
        return Err(GitError::Git(
            "Reword exige estar em uma branch — não em detached HEAD.".into(),
        ));
    }

    let parent = match cli.run(&GitCommand {
        args: vec!["rev-parse".into(), format!("{target_sha}^")],
    }) {
        Ok(p) => p.trim().to_string(),
        Err(_) => {
            return Err(GitError::Git(
                "Reword do primeiro commit da branch ainda não suportado.".into(),
            ));
        }
    };

    let mut chain = cli
        .run(&GitCommand {
            args: vec![
                "log".into(),
                "--reverse".into(),
                "--format=%H".into(),
                format!("{target_sha}..HEAD"),
            ],
        })?
        .lines()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_string)
        .collect::<Vec<_>>();
    chain.insert(0, target_sha.to_string());

    cli.run(&GitCommand {
        args: vec!["checkout".into(), "--detach".into(), parent.clone()],
    })?;

    let replay = || -> Result<(), GitError> {
        for sha in &chain {
            if sha == target_sha {
                let _ = cli.run(&GitCommand {
                    args: vec![
                        "cherry-pick".into(),
                        "-n".into(),
                        "--empty=keep".into(),
                        sha.clone(),
                    ],
                });
                let has_index = cli
                    .run(&GitCommand {
                        args: vec!["diff".into(), "--cached".into(), "--quiet".into()],
                    })
                    .is_err();
                let mut args = vec!["commit".into(), "-F".into(), "-".into()];
                if !has_index {
                    args.insert(1, "--allow-empty".into());
                }
                cli.run_with_stdin(&GitCommand { args }, new_message.as_bytes())?;
            } else {
                cli.run(&GitCommand {
                    args: vec![
                        "cherry-pick".into(),
                        "--empty=keep".into(),
                        sha.clone(),
                    ],
                })?;
            }
        }
        Ok(())
    };

    if let Err(e) = replay() {
        let _ = cli.run(&GitCommand {
            args: vec!["cherry-pick".into(), "--abort".into()],
        });
        let _ = cli.run(&GitCommand {
            args: vec!["checkout".into(), branch.clone()],
        });
        return Err(e);
    }

    cli.run(&GitCommand {
        args: vec!["branch".into(), "-f".into(), branch.clone()],
    })?;
    cli.run(&GitCommand {
        args: vec!["checkout".into(), branch],
    })?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn init_repo_with_commits(dir: &std::path::Path, n: usize) {
        let _ = std::fs::remove_dir_all(dir);
        std::fs::create_dir_all(dir).unwrap();
        for args in [
            vec!["init"],
            vec!["config", "user.email", "t@t.com"],
            vec!["config", "user.name", "T"],
        ] {
            std::process::Command::new("git")
                .args(&args)
                .current_dir(dir)
                .output()
                .unwrap();
        }
        std::fs::write(dir.join("f.txt"), "x").unwrap();
        std::process::Command::new("git")
            .args(["add", "f.txt"])
            .current_dir(dir)
            .output()
            .unwrap();
        std::process::Command::new("git")
            .args(["commit", "-m", "first"])
            .current_dir(dir)
            .output()
            .unwrap();
        for i in 2..=n {
            std::process::Command::new("git")
                .args(["commit", "--allow-empty", "-m", &format!("commit {i}")])
                .current_dir(dir)
                .output()
                .unwrap();
        }
    }

    #[test]
    fn reword_altera_mensagem_do_commit_alvo() {
        let dir = std::env::temp_dir().join(format!("trilho-rw-{}", std::process::id()));
        init_repo_with_commits(&dir, 3);
        let middle = std::process::Command::new("git")
            .args(["rev-parse", "HEAD~1"])
            .current_dir(&dir)
            .output()
            .unwrap();
        let middle = String::from_utf8_lossy(&middle.stdout)
            .trim()
            .to_string();

        let cli = SafeGitCli::new(dir.to_string_lossy());
        execute_reword(&cli, &middle, "mensagem reescrita\n\ncom corpo")
            .expect("reword");

        let log = std::process::Command::new("git")
            .args(["log", "-1", "--format=%B", "HEAD~1"])
            .current_dir(&dir)
            .output()
            .unwrap();
        let body = String::from_utf8_lossy(&log.stdout);
        assert!(body.contains("mensagem reescrita"));
        assert!(body.contains("com corpo"));
        let _ = std::fs::remove_dir_all(&dir);
    }
}
