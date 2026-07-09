//! RF-20 — leitura e resolução de arquivos em conflito (3 vias).

use serde::Serialize;
use std::path::Path;

use crate::application::{GitCommand, GitError};
use crate::infrastructure::git_cli::SafeGitCli;
use crate::infrastructure::validation::validate_repo_relative_path;
use git2::{Index, Repository};

/// Um lado do conflito (conteúdo do blob no índice / working tree).
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ConflictSide {
    pub available: bool,
    pub content: String,
}

/// Região de conflito (ou trecho comum) no working tree.
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ConflictRegion {
    /// `context` | `conflict`
    pub kind: String,
    pub ours: String,
    pub theirs: String,
    /// Só em `context`: texto comum.
    pub text: String,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ConflictFileView {
    pub path: String,
    pub base: ConflictSide,
    pub ours: ConflictSide,
    pub theirs: ConflictSide,
    /// Conteúdo atual do working tree (pode conter marcadores).
    pub worktree: String,
    pub regions: Vec<ConflictRegion>,
    pub conflict_count: u32,
    /// true se o WT ainda tem marcadores `<<<<<<<`.
    pub has_markers: bool,
}

/// Lê stages 1/2/3 + working tree e parseia marcadores.
pub fn get_conflict_file(repo_path: &str, display_path: &str) -> Result<ConflictFileView, GitError> {
    let path = validate_repo_relative_path(display_path)?;
    let git_path = path
        .rsplit(" → ")
        .next()
        .unwrap_or(path.as_str())
        .trim()
        .to_string();

    let repo = Repository::discover(repo_path)
        .map_err(|e| GitError::Io(format!("Não foi possível abrir o repositório: {e}")))?;
    let index = repo
        .index()
        .map_err(|e| GitError::Io(format!("Não foi possível ler o índice: {e}")))?;

    let base = side_from_index(&repo, &index, &git_path, 1);
    let ours = side_from_index(&repo, &index, &git_path, 2);
    let theirs = side_from_index(&repo, &index, &git_path, 3);

    if !base.available && !ours.available && !theirs.available {
        return Err(GitError::Git(format!(
            "Arquivo «{git_path}» não está em conflito no índice."
        )));
    }

    let worktree = read_worktree(&repo, &git_path)?;
    let regions = parse_conflict_markers(&worktree);
    let conflict_count = regions.iter().filter(|r| r.kind == "conflict").count() as u32;
    let has_markers = conflict_count > 0;

    Ok(ConflictFileView {
        path: git_path,
        base,
        ours,
        theirs,
        worktree,
        regions,
        conflict_count,
        has_markers,
    })
}

fn side_from_index(repo: &Repository, index: &Index, path: &str, stage: i32) -> ConflictSide {
    match index.get_path(Path::new(path), stage) {
        Some(entry) => match repo.find_blob(entry.id) {
            Ok(blob) => match std::str::from_utf8(blob.content()) {
                Ok(text) => ConflictSide {
                    available: true,
                    content: text.to_string(),
                },
                Err(_) => ConflictSide {
                    available: true,
                    content: String::from_utf8_lossy(blob.content()).into_owned(),
                },
            },
            Err(_) => ConflictSide {
                available: false,
                content: String::new(),
            },
        },
        None => ConflictSide {
            available: false,
            content: String::new(),
        },
    }
}

fn read_worktree(repo: &Repository, path: &str) -> Result<String, GitError> {
    let workdir = repo
        .workdir()
        .ok_or_else(|| GitError::Git("Repositório bare — sem working tree.".into()))?;
    let full = workdir.join(path);
    if !full.exists() {
        return Ok(String::new());
    }
    let bytes = std::fs::read(&full)
        .map_err(|e| GitError::Io(format!("Não foi possível ler {path}: {e}")))?;
    Ok(String::from_utf8_lossy(&bytes).into_owned())
}

/// Conta blocos de conflito (`<<<<<<<`) sem alocar regiões.
pub fn count_conflict_markers(content: &str) -> u32 {
    content
        .lines()
        .filter(|line| line.starts_with("<<<<<<<"))
        .count() as u32
}

/// Parseia marcadores padrão do Git (`<<<<<<<`, `=======`, `>>>>>>>`).
/// Aceita LF e CRLF (`split_inclusive` preserva o terminador de cada linha).
pub fn parse_conflict_markers(content: &str) -> Vec<ConflictRegion> {
    let lines: Vec<&str> = content.split_inclusive('\n').collect();
    let mut regions = Vec::new();
    let mut i = 0;
    let mut context_buf = String::new();

    while i < lines.len() {
        let line = lines[i];
        if line.starts_with("<<<<<<<") {
            if !context_buf.is_empty() {
                regions.push(ConflictRegion {
                    kind: "context".into(),
                    ours: String::new(),
                    theirs: String::new(),
                    text: std::mem::take(&mut context_buf),
                });
            }
            i += 1;
            let mut ours = String::new();
            while i < lines.len() && !lines[i].starts_with("=======") {
                ours.push_str(lines[i]);
                i += 1;
            }
            if i < lines.len() && lines[i].starts_with("=======") {
                i += 1;
            }
            let mut theirs = String::new();
            while i < lines.len() && !lines[i].starts_with(">>>>>>>") {
                theirs.push_str(lines[i]);
                i += 1;
            }
            if i < lines.len() && lines[i].starts_with(">>>>>>>") {
                i += 1;
            }
            regions.push(ConflictRegion {
                kind: "conflict".into(),
                ours,
                theirs,
                text: String::new(),
            });
        } else {
            context_buf.push_str(line);
            i += 1;
        }
    }

    if !context_buf.is_empty() {
        regions.push(ConflictRegion {
            kind: "context".into(),
            ours: String::new(),
            theirs: String::new(),
            text: context_buf,
        });
    }

    if regions.is_empty() {
        regions.push(ConflictRegion {
            kind: "context".into(),
            ours: String::new(),
            theirs: String::new(),
            text: content.to_string(),
        });
    }

    regions
}

/// Monta o resultado a partir das escolhas por região (`ours` | `theirs` | `both` | `custom`).
pub fn build_resolved_content(
    regions: &[ConflictRegion],
    choices: &[ConflictChoice],
) -> Result<String, GitError> {
    if choices.len() != regions.iter().filter(|r| r.kind == "conflict").count() {
        return Err(GitError::Git(
            "Número de escolhas não corresponde aos blocos em conflito.".into(),
        ));
    }
    let mut out = String::new();
    let mut choice_i = 0;
    for region in regions {
        if region.kind != "conflict" {
            out.push_str(&region.text);
            continue;
        }
        match &choices[choice_i] {
            ConflictChoice::Ours => out.push_str(&region.ours),
            ConflictChoice::Theirs => out.push_str(&region.theirs),
            ConflictChoice::Both => {
                out.push_str(&region.ours);
                out.push_str(&region.theirs);
            }
            ConflictChoice::BothTheirsFirst => {
                out.push_str(&region.theirs);
                out.push_str(&region.ours);
            }
            ConflictChoice::Custom(text) => out.push_str(text),
        }
        choice_i += 1;
    }
    Ok(out)
}

#[derive(Debug, Clone, serde::Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum ConflictChoice {
    Ours,
    Theirs,
    Both,
    BothTheirsFirst,
    Custom(String),
}

#[derive(Debug, Clone, Copy, serde::Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum ConflictSideChoice {
    Ours,
    Theirs,
}

/// `git checkout --ours|--theirs -- <path>` + `git add`.
pub fn resolve_conflict_side(
    cli: &SafeGitCli,
    path: &str,
    side: ConflictSideChoice,
) -> Result<(), GitError> {
    let path = validate_repo_relative_path(path)?;
    let flag = match side {
        ConflictSideChoice::Ours => "--ours",
        ConflictSideChoice::Theirs => "--theirs",
    };
    cli.run(&GitCommand {
        args: vec![
            "checkout".into(),
            flag.into(),
            "--".into(),
            path.clone(),
        ],
    })?;
    cli.run(&GitCommand {
        args: vec!["add".into(), "--".into(), path],
    })?;
    Ok(())
}

/// Grava conteúdo resolvido no working tree e `git add`.
pub fn resolve_conflict_content(
    repo_path: &str,
    cli: &SafeGitCli,
    path: &str,
    content: &str,
) -> Result<(), GitError> {
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
    cli.run(&GitCommand {
        args: vec!["add".into(), "--".into(), path],
    })?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parseia_marcadores_simples() {
        let content = "antes\n<<<<<<< HEAD\nours\n=======\ntheirs\n>>>>>>> branch\ndepois\n";
        let regions = parse_conflict_markers(content);
        assert_eq!(regions.len(), 3);
        assert_eq!(regions[0].kind, "context");
        assert!(regions[0].text.contains("antes"));
        assert_eq!(regions[1].kind, "conflict");
        assert_eq!(regions[1].ours.trim(), "ours");
        assert_eq!(regions[1].theirs.trim(), "theirs");
        assert_eq!(regions[2].kind, "context");
    }

    #[test]
    fn build_resolved_ours() {
        let regions = parse_conflict_markers(
            "a\n<<<<<<<\no\n=======\nt\n>>>>>>>\nb\n",
        );
        let out = build_resolved_content(&regions, &[ConflictChoice::Ours]).unwrap();
        assert_eq!(out, "a\no\nb\n");
    }

    #[test]
    fn build_resolved_both() {
        let regions = parse_conflict_markers("<<<<<<<\no\n=======\nt\n>>>>>>>");
        let out = build_resolved_content(&regions, &[ConflictChoice::Both]).unwrap();
        assert_eq!(out, "o\nt\n");
    }

    #[test]
    fn parseia_marcadores_com_crlf() {
        let content = "antes\r\n<<<<<<< HEAD\r\nours\r\n=======\r\ntheirs\r\n>>>>>>> branch\r\ndepois\r\n";
        let regions = parse_conflict_markers(content);
        assert_eq!(regions.len(), 3);
        assert_eq!(regions[0].kind, "context");
        assert!(regions[0].text.contains("antes"));
        assert!(regions[0].text.contains('\r'));
        assert_eq!(regions[1].kind, "conflict");
        assert_eq!(regions[1].ours, "ours\r\n");
        assert_eq!(regions[1].theirs, "theirs\r\n");
        assert_eq!(count_conflict_markers(content), 1);
    }

    #[test]
    fn count_conflict_markers_varios_blocos() {
        let content = "<<<<<<< a\nx\n=======\ny\n>>>>>>>\n<<<<<<< b\np\n=======\nq\n>>>>>>>";
        assert_eq!(count_conflict_markers(content), 2);
    }

    #[test]
    fn build_resolved_preserva_crlf() {
        let regions = parse_conflict_markers(
            "a\r\n<<<<<<<\r\no\r\n=======\r\nt\r\n>>>>>>>\r\nb\r\n",
        );
        let out = build_resolved_content(&regions, &[ConflictChoice::Ours]).unwrap();
        assert_eq!(out, "a\r\no\r\nb\r\n");
    }
}
