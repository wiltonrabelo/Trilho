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
    let start = start_line.max(1);
    let end = end_line.max(start).min(start + MAX_BLAME_LINES - 1);
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
            let index_blob = cli.run(&GitCommand {
                args: vec!["show".into(), format!(":{path}")],
            })?;
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
}
