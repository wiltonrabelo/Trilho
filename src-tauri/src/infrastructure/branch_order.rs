//! Ordenação opcional de branches por checkouts recentes no reflog (RF-14).
//!
//! Fonte primária continua sendo `for-each-ref` / listagem conhecida.
//! O reflog só **reordena** nomes que já existem na lista — nunca inventa refs.

use crate::application::{GitCommand, GitError};
use crate::infrastructure::SafeGitCli;

/// Extrai destinos de checkout do reflog do HEAD, na ordem do mais recente.
/// Só devolve nomes que estão em `known` (interseção segura).
pub fn order_refs_by_recent_checkout(
    cli: &SafeGitCli,
    known: &[String],
) -> Result<Vec<String>, GitError> {
    if known.is_empty() {
        return Ok(vec![]);
    }
    let known_set: std::collections::HashSet<&str> =
        known.iter().map(String::as_str).collect();
    let out = cli
        .run(&GitCommand {
            args: vec![
                "reflog".into(),
                "show".into(),
                "HEAD".into(),
                "-n".into(),
                "80".into(),
                "--format=%gs".into(),
            ],
        })
        .unwrap_or_default();

    let mut ordered = Vec::new();
    let mut seen = std::collections::HashSet::new();
    for line in out.lines() {
        if let Some(dest) = checkout_destination(line) {
            if known_set.contains(dest.as_str()) && seen.insert(dest.clone()) {
                ordered.push(dest);
            }
        }
    }
    for name in known {
        if seen.insert(name.clone()) {
            ordered.push(name.clone());
        }
    }
    Ok(ordered)
}

/// `checkout: moving from X to Y` → `Y` (sem parse genérico de mensagem humana).
fn checkout_destination(subject: &str) -> Option<String> {
    const PREFIX: &str = "checkout: moving from ";
    let rest = subject.strip_prefix(PREFIX)?;
    let (_from, to) = rest.split_once(" to ")?;
    let to = to.trim();
    if to.is_empty() || to == "HEAD" {
        return None;
    }
    Some(to.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extrai_destino_do_checkout() {
        assert_eq!(
            checkout_destination("checkout: moving from main to feature-x").as_deref(),
            Some("feature-x")
        );
        assert_eq!(
            checkout_destination("commit: mensagem qualquer"),
            None
        );
        assert_eq!(checkout_destination("checkout: moving from a to HEAD"), None);
    }
}
