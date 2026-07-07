//! Execução de blame nas três fontes (RF-03, PLANO §7.4).

use crate::application::{GitCommand, GitError};
use crate::domain::{BlameLine, BlameSource};
use crate::infrastructure::blame_parser::parse_line_porcelain;
use crate::infrastructure::git_cli::SafeGitCli;

const MAX_BLAME_LINES: u32 = 200;

pub fn blame_file(
    cli: &SafeGitCli,
    path: &str,
    source: BlameSource,
    commit_id: Option<&str>,
    start_line: u32,
    end_line: u32,
) -> Result<Vec<BlameLine>, GitError> {
    let line_count = blob_line_count(cli, path, source, commit_id)?;
    if line_count == 0 {
        return Ok(vec![]);
    }

    let start = start_line.max(1).min(line_count);
    let end = end_line.max(start).min(line_count).min(start + MAX_BLAME_LINES - 1);
    let range = format!("{start},{end}");

    let output = match source {
        BlameSource::Commit => {
            let rev = commit_id.unwrap_or("HEAD");
            cli.run(&GitCommand {
                args: vec![
                    "blame".into(),
                    "--line-porcelain".into(),
                    "-L".into(),
                    range,
                    rev.into(),
                    "--".into(),
                    path.into(),
                ],
            })?
        }
        BlameSource::WorkingTree => cli.run(&GitCommand {
            args: vec![
                "blame".into(),
                "--line-porcelain".into(),
                "-L".into(),
                range,
                "--".into(),
                path.into(),
            ],
        })?,
        BlameSource::Staging => {
            let index_blob = match cli.run(&GitCommand {
                args: vec!["show".into(), format!(":{path}")],
            }) {
                Ok(blob) => blob,
                Err(e) if is_unmerged_blame_error(&e) => {
                    return blame_file(cli, path, BlameSource::WorkingTree, commit_id, start_line, end_line);
                }
                Err(e) => return Err(e),
            };
            cli.run_with_stdin(
                &GitCommand {
                    args: vec![
                        "blame".into(),
                        "--line-porcelain".into(),
                        "--contents".into(),
                        "-".into(),
                        "-L".into(),
                        range,
                        "--".into(),
                        path.into(),
                    ],
                },
                index_blob.as_bytes(),
            )?
        }
    };

    parse_line_porcelain(&output)
}

fn blob_line_count(
    cli: &SafeGitCli,
    path: &str,
    source: BlameSource,
    commit_id: Option<&str>,
) -> Result<u32, GitError> {
    let content = match source {
        BlameSource::Commit => {
            let rev = commit_id.unwrap_or("HEAD");
            cli.run(&GitCommand {
                args: vec!["show".into(), format!("{rev}:{path}")],
            })?
        }
        BlameSource::WorkingTree => std::fs::read_to_string(
            std::path::Path::new(cli.repo_path()).join(path),
        )
        .map_err(|e| GitError::Io(e.to_string()))?,
        BlameSource::Staging => match cli.run(&GitCommand {
            args: vec!["show".into(), format!(":{path}")],
        }) {
            Ok(blob) => blob,
            Err(e) if is_unmerged_blame_error(&e) => {
                return blob_line_count(cli, path, BlameSource::WorkingTree, commit_id);
            }
            Err(e) => return Err(e),
        },
    };
    Ok(count_lines(&content))
}

fn is_unmerged_blame_error(err: &GitError) -> bool {
    let msg = err.to_string().to_ascii_lowercase();
    msg.contains("not at stage 0") || msg.contains("unmerged")
}

fn count_lines(content: &str) -> u32 {
    if content.is_empty() {
        return 0;
    }
    content.matches('\n').count() as u32 + 1
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::process::Command;

    #[test]
    fn blame_working_tree() {
        let dir = std::env::temp_dir().join(format!("trilho-blame-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        Command::new("git")
            .args(["init"])
            .current_dir(&dir)
            .output()
            .unwrap();
        Command::new("git")
            .args(["config", "user.email", "t@t.com"])
            .current_dir(&dir)
            .output()
            .unwrap();
        Command::new("git")
            .args(["config", "user.name", "T"])
            .current_dir(&dir)
            .output()
            .unwrap();
        fs::create_dir_all(dir.join("src")).unwrap();
        fs::write(dir.join("src/a.ts"), "const x = 1;\n").unwrap();
        Command::new("git")
            .args(["add", "src/a.ts"])
            .current_dir(&dir)
            .output()
            .unwrap();
        Command::new("git")
            .args(["commit", "-m", "init"])
            .current_dir(&dir)
            .output()
            .unwrap();

        let cli = SafeGitCli::new(dir.to_string_lossy());
        let lines =
            blame_file(&cli, "src/a.ts", BlameSource::WorkingTree, None, 1, 1).expect("blame");
        assert_eq!(lines.len(), 1);
        assert!(lines[0].content.contains('1'));
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn blame_arquivo_vazio_no_commit_retorna_vazio() {
        let dir = std::env::temp_dir().join(format!("trilho-blame0-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        Command::new("git")
            .args(["init"])
            .current_dir(&dir)
            .output()
            .unwrap();
        Command::new("git")
            .args(["config", "user.email", "t@t.com"])
            .current_dir(&dir)
            .output()
            .unwrap();
        Command::new("git")
            .args(["config", "user.name", "T"])
            .current_dir(&dir)
            .output()
            .unwrap();
        fs::write(dir.join("empty.txt"), "").unwrap();
        Command::new("git")
            .args(["add", "empty.txt"])
            .current_dir(&dir)
            .output()
            .unwrap();
        Command::new("git")
            .args(["commit", "-m", "empty file"])
            .current_dir(&dir)
            .output()
            .unwrap();
        let sha = Command::new("git")
            .args(["rev-parse", "HEAD"])
            .current_dir(&dir)
            .output()
            .unwrap();
        let sha = String::from_utf8_lossy(&sha.stdout).trim().to_string();

        let cli = SafeGitCli::new(dir.to_string_lossy());
        let lines = blame_file(
            &cli,
            "empty.txt",
            BlameSource::Commit,
            Some(&sha),
            1,
            10,
        )
        .expect("blame vazio");
        assert!(lines.is_empty());
        let _ = fs::remove_dir_all(&dir);
    }
}
