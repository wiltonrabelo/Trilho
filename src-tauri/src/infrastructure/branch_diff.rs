//! RF-14 — diff entre duas refs (branches locais ou remotas).

use serde::Serialize;

use crate::application::{GitCommand, GitError};
use crate::domain::FileChangeKind;
use crate::infrastructure::SafeGitCli;

/// Modo de comparação entre pontas (`A..B`) ou a partir do merge-base (`A...B`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum BranchDiffMode {
    /// Diferença direta entre as pontas (`A..B`).
    Tips,
    /// O que B tem desde que divergiu de A (`A...B`) — padrão RF-14.
    MergeBase,
}

impl BranchDiffMode {
    pub fn range_spec(self, left: &str, right: &str) -> String {
        match self {
            BranchDiffMode::Tips => format!("{left}..{right}"),
            BranchDiffMode::MergeBase => format!("{left}...{right}"),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BranchDiffFile {
    pub path: String,
    pub kind: FileChangeKind,
    pub additions: u32,
    pub deletions: u32,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BranchDiffSummary {
    pub left: String,
    pub right: String,
    pub mode: BranchDiffMode,
    pub range: String,
    pub files: Vec<BranchDiffFile>,
}

/// Lista arquivos alterados entre `left` e `right` no modo escolhido.
pub fn list_branch_diff(
    cli: &SafeGitCli,
    left: &str,
    right: &str,
    mode: BranchDiffMode,
) -> Result<BranchDiffSummary, GitError> {
    let range = mode.range_spec(left, right);
    let name_status = cli.run(&GitCommand {
        args: vec![
            "diff".into(),
            "--name-status".into(),
            "-z".into(),
            range.clone(),
        ],
    })?;
    let numstat = cli.run(&GitCommand {
        args: vec!["diff".into(), "--numstat".into(), range.clone()],
    })?;
    let stats = parse_numstat(&numstat);
    let files = parse_name_status_z(&name_status)
        .into_iter()
        .map(|(path, kind)| {
            let (additions, deletions) = stats.get(&path).copied().unwrap_or((0, 0));
            BranchDiffFile {
                path,
                kind,
                additions,
                deletions,
            }
        })
        .collect();
    Ok(BranchDiffSummary {
        left: left.to_string(),
        right: right.to_string(),
        mode,
        range,
        files,
    })
}

/// Diff unificado de um arquivo entre as refs.
pub fn get_branch_file_diff(
    cli: &SafeGitCli,
    left: &str,
    right: &str,
    mode: BranchDiffMode,
    path: &str,
) -> Result<String, GitError> {
    let range = mode.range_spec(left, right);
    cli.run(&GitCommand {
        args: vec![
            "diff".into(),
            "--no-color".into(),
            range,
            "--".into(),
            path.to_string(),
        ],
    })
}

fn parse_name_status_z(raw: &str) -> Vec<(String, FileChangeKind)> {
    let parts: Vec<&str> = raw.split('\0').filter(|s| !s.is_empty()).collect();
    let mut out = Vec::new();
    let mut i = 0;
    while i < parts.len() {
        let status = parts[i];
        if status.is_empty() {
            break;
        }
        let code = status.chars().next().unwrap_or('M');
        match code {
            'R' | 'C' => {
                // status\0old\0new
                if i + 2 >= parts.len() {
                    break;
                }
                let new_path = parts[i + 2].to_string();
                out.push((new_path, FileChangeKind::Renamed));
                i += 3;
            }
            'A' => {
                if i + 1 >= parts.len() {
                    break;
                }
                out.push((parts[i + 1].to_string(), FileChangeKind::Added));
                i += 2;
            }
            'D' => {
                if i + 1 >= parts.len() {
                    break;
                }
                out.push((parts[i + 1].to_string(), FileChangeKind::Deleted));
                i += 2;
            }
            _ => {
                if i + 1 >= parts.len() {
                    break;
                }
                out.push((parts[i + 1].to_string(), FileChangeKind::Modified));
                i += 2;
            }
        }
    }
    out
}

fn parse_numstat(raw: &str) -> std::collections::HashMap<String, (u32, u32)> {
    let mut map = std::collections::HashMap::new();
    for line in raw.lines() {
        let mut parts = line.splitn(3, '\t');
        let (Some(adds_s), Some(dels_s), Some(path)) =
            (parts.next(), parts.next(), parts.next())
        else {
            continue;
        };
        // Renome: "old => new" ou só new após status R no name-status.
        let path = path
            .rsplit_once(" => ")
            .map(|(_, new)| new)
            .unwrap_or(path)
            .to_string();
        map.insert(path, (parse_count(adds_s), parse_count(dels_s)));
    }
    map
}

fn parse_count(s: &str) -> u32 {
    if s == "-" {
        0
    } else {
        s.parse().unwrap_or(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn range_specs() {
        assert_eq!(
            BranchDiffMode::Tips.range_spec("main", "feat"),
            "main..feat"
        );
        assert_eq!(
            BranchDiffMode::MergeBase.range_spec("main", "feat"),
            "main...feat"
        );
    }

    #[test]
    fn parse_name_status_basico() {
        let raw = "M\0a.txt\0A\0b.txt\0D\0c.txt\0";
        let parsed = parse_name_status_z(raw);
        assert_eq!(parsed.len(), 3);
        assert_eq!(parsed[0].1, FileChangeKind::Modified);
        assert_eq!(parsed[1].0, "b.txt");
        assert_eq!(parsed[1].1, FileChangeKind::Added);
        assert_eq!(parsed[2].1, FileChangeKind::Deleted);
    }

    #[test]
    fn parse_name_status_rename() {
        let raw = "R100\0old.txt\0new.txt\0";
        let parsed = parse_name_status_z(raw);
        assert_eq!(parsed.len(), 1);
        assert_eq!(parsed[0].0, "new.txt");
        assert_eq!(parsed[0].1, FileChangeKind::Renamed);
    }
}
