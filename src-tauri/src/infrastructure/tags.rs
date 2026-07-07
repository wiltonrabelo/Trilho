//! Listagem de tags (`git for-each-ref refs/tags`).

use serde::Serialize;

use crate::application::{GitCommand, GitError};
use crate::infrastructure::SafeGitCli;

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct TagEntry {
    pub name: String,
    pub commit_id: String,
    pub short_id: String,
}

pub fn list_tags(repo_path: &str) -> Result<Vec<TagEntry>, GitError> {
    let cli = SafeGitCli::new(repo_path);
    let cmd = GitCommand {
        args: vec![
            "for-each-ref".into(),
            "refs/tags".into(),
            "--format=%(refname:short)|%(*objectname)".into(),
        ],
    };
    let out = cli.run(&cmd)?;
    let mut tags: Vec<TagEntry> = out
        .lines()
        .filter_map(parse_tag_line)
        .collect();
    tags.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(tags)
}

fn parse_tag_line(line: &str) -> Option<TagEntry> {
    let line = line.trim();
    if line.is_empty() {
        return None;
    }
    let (name, commit_id) = line.split_once('|')?;
    let name = name.trim();
    let commit_id = commit_id.trim();
    if name.is_empty() || commit_id.is_empty() {
        return None;
    }
    let short_id: String = commit_id.chars().take(7).collect();
    Some(TagEntry {
        name: name.to_string(),
        commit_id: commit_id.to_string(),
        short_id,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_linha_for_each_ref() {
        let line = "v1.0.0|a8900a6246eb160ef433b6aeddde7c3a8c47eeb6";
        let entry = parse_tag_line(line).expect("tag");
        assert_eq!(entry.name, "v1.0.0");
        assert_eq!(entry.short_id, "a8900a6");
        assert_eq!(entry.commit_id.len(), 40);
    }
}
