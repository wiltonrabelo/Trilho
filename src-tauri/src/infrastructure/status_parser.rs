//! Parser de `git status --porcelain=v2 -z` (RF-04).
//!
//! Entrada `1`: 9 campos — path em `parts[8]`.
//! Entrada `2` (rename/copy): 10 campos — score (`R100`/`C100`) em `parts[8]`, path destino em `parts[9]`;
//! path origem no segmento NUL seguinte (`-z`).

use crate::application::GitError;
use crate::domain::{FileChange, FileChangeKind, RepoStatus};

pub fn parse_porcelain_v2(raw: &str) -> Result<RepoStatus, GitError> {
    let mut staged = Vec::new();
    let mut unstaged = Vec::new();
    let mut untracked = Vec::new();

    let segments: Vec<&str> = raw.split('\0').filter(|s| !s.is_empty()).collect();
    let mut i = 0;

    while i < segments.len() {
        let entry = segments[i];

        if entry.starts_with('#') {
            i += 1;
            continue;
        }

        if entry.starts_with("? ") {
            untracked.push(FileChange {
                path: entry[2..].to_string(),
                kind: FileChangeKind::Untracked,
                staged: false,
            });
            i += 1;
            continue;
        }

        if entry.starts_with("1 ") {
            parse_v1_entry(entry, &mut staged, &mut unstaged);
            i += 1;
            continue;
        }

        if entry.starts_with("2 ") {
            let orig_from_next = segments.get(i + 1).and_then(|s| {
                is_orphan_path_segment(s).then(|| (*s).to_string())
            });
            let consumed_extra = orig_from_next.is_some();
            parse_v2_rename_entry(entry, orig_from_next.as_deref(), &mut staged, &mut unstaged);
            i += if consumed_extra { 2 } else { 1 };
            continue;
        }

        i += 1;
    }

    Ok(RepoStatus {
        staged,
        unstaged,
        untracked,
    })
}

/// Segmento NUL separado — path de origem após registro `2` (formato `-z` do git).
fn is_orphan_path_segment(s: &str) -> bool {
    !s.starts_with("1 ")
        && !s.starts_with("2 ")
        && !s.starts_with("? ")
        && !s.starts_with('#')
}

fn parse_v1_entry(entry: &str, staged: &mut Vec<FileChange>, unstaged: &mut Vec<FileChange>) {
    let parts: Vec<&str> = entry.splitn(9, ' ').collect();
    if parts.len() < 9 {
        return;
    }
    push_xy_changes(&parts[1], &parts[8], staged, unstaged);
}

fn parse_v2_rename_entry(
    entry: &str,
    orig_path: Option<&str>,
    staged: &mut Vec<FileChange>,
    unstaged: &mut Vec<FileChange>,
) {
    // Campo extra vs entrada `1`: score de rename/copy (ex. R100) antes do path.
    let parts: Vec<&str> = entry.splitn(10, ' ').collect();
    if parts.len() < 10 {
        return;
    }

    let new_path = parts[9];
    let display = match orig_path {
        Some(old) => format!("{old} → {new_path}"),
        None => new_path.to_string(),
    };
    push_xy_changes(&parts[1], &display, staged, unstaged);
}

fn push_xy_changes(
    xy: &str,
    path: &str,
    staged: &mut Vec<FileChange>,
    unstaged: &mut Vec<FileChange>,
) {
    if xy.len() != 2 {
        return;
    }
    let index = xy.chars().next().unwrap_or('.');
    let worktree = xy.chars().nth(1).unwrap_or('.');

    if index != '.' && index != '?' {
        staged.push(FileChange {
            path: path.to_string(),
            kind: xy_to_kind(index),
            staged: true,
        });
    }
    if worktree != '.' && worktree != '?' {
        unstaged.push(FileChange {
            path: path.to_string(),
            kind: xy_to_kind(worktree),
            staged: false,
        });
    }
}

fn xy_to_kind(c: char) -> FileChangeKind {
    match c {
        'A' | '?' => FileChangeKind::Added,
        'D' => FileChangeKind::Deleted,
        'R' | 'C' => FileChangeKind::Renamed,
        _ => FileChangeKind::Modified,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Formato real do git status --porcelain=v2 -z para rename (10 campos + orig NUL).
    const RENAME_V2: &str =
        "2 R. N... 100644 100644 100644 7898192abc7898192 7898192abc7898192 R100 new.ts\0old.ts\0";

    #[test]
    fn parse_staged_unstaged_e_untracked() {
        let raw = "1 M. N... 100644 100644 100644 abc def staged.ts\0\
                   1 .M N... 100644 100644 100644 abc def unstaged.ts\0\
                   ? untracked.txt\0";
        let status = parse_porcelain_v2(raw).expect("parse");
        assert_eq!(status.staged.len(), 1);
        assert_eq!(status.unstaged.len(), 1);
        assert_eq!(status.untracked.len(), 1);
    }

    #[test]
    fn parse_rename_formato_git_real_com_r100() {
        let status = parse_porcelain_v2(RENAME_V2).expect("parse");
        assert_eq!(status.staged.len(), 1);
        assert_eq!(status.staged[0].path, "old.ts → new.ts");
        assert_eq!(status.staged[0].kind, FileChangeKind::Renamed);
    }

    #[test]
    fn parse_rename_sem_origem_mostra_so_destino() {
        let raw = "2 R. N... 100644 100644 100644 abcd1234 abcd1234 R100 new.ts\0";
        let status = parse_porcelain_v2(raw).expect("parse");
        assert_eq!(status.staged[0].path, "new.ts");
    }

    #[test]
    fn origem_orfa_nao_confunde_com_path_2foo() {
        // Path de origem "2foo.ts" não é linha de status (precisa ser "2 ").
        let raw = "2 R. N... 100644 100644 100644 abcd1234 abcd1234 R100 new.ts\02foo.ts\0";
        let status = parse_porcelain_v2(raw).expect("parse");
        assert_eq!(status.staged[0].path, "2foo.ts → new.ts");
    }
}
