//! Listagem de stashes (`git stash list`).

use serde::Serialize;

use crate::application::{GitCommand, GitError};
use crate::infrastructure::SafeGitCli;

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct StashEntry {
    pub index: usize,
    /// Ref Git (`stash@{0}`).
    pub reference: String,
    /// Texto após `stash@{n}:`.
    pub message: String,
}

pub fn list_stashes(repo_path: &str) -> Result<Vec<StashEntry>, GitError> {
    let cli = SafeGitCli::new(repo_path);
    let cmd = GitCommand {
        args: vec!["stash".into(), "list".into()],
    };
    let out = cli.run(&cmd)?;
    Ok(parse_stash_list(&out))
}

pub fn stash_reference(index: usize) -> Result<String, GitError> {
    if index > 9999 {
        return Err(GitError::Git("Índice de stash inválido.".into()));
    }
    Ok(format!("stash@{{{index}}}"))
}

pub fn parse_stash_list(output: &str) -> Vec<StashEntry> {
    output.lines().filter_map(parse_stash_line).collect()
}

fn parse_stash_line(line: &str) -> Option<StashEntry> {
    let line = line.trim();
    let rest = line.strip_prefix("stash@{")?;
    let (idx_part, after) = rest.split_once("}:")?;
    let index = idx_part.parse().ok()?;
    Some(StashEntry {
        index,
        reference: format!("stash@{{{index}}}"),
        message: after.trim().to_string(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_linhas_de_stash_list() {
        let out = "stash@{0}: WIP on main: abc123 msg\nstash@{1}: On master: wip\n";
        let list = parse_stash_list(out);
        assert_eq!(list.len(), 2);
        assert_eq!(list[0].index, 0);
        assert_eq!(list[0].reference, "stash@{0}");
        assert_eq!(list[0].message, "WIP on main: abc123 msg");
        assert_eq!(list[1].index, 1);
    }

    #[test]
    fn lista_vazia_sem_stash() {
        assert!(parse_stash_list("").is_empty());
    }
}
