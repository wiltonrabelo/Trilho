use git2::{BranchType, Oid, Repository};
use std::collections::HashSet;

use super::scoring::name_priority;

/// Teto de candidatas pontuadas — em repositórios com centenas de branches
/// (SysPDV: 300+), pontuar todas custa minutos. Mantém as prioritárias
/// (main/master/develop/...) e as N de commit mais recente.
pub(super) const MAX_SCORED_CANDIDATES: usize = 40;

pub(super) fn collect_candidates(repo: &Repository, current: &str) -> Vec<String> {
    let locals = branch_names(repo, BranchType::Local);
    let remotes = branch_names(repo, BranchType::Remote);

    let mut names: HashSet<String> = locals
        .iter()
        .filter(|n| n.as_str() != current)
        .cloned()
        .collect();

    for remote in remotes {
        if remote.ends_with("/HEAD") {
            continue;
        }
        let base = remote_base_name(&remote);
        // A contraparte remota da própria branch (origin/feature para feature)
        // não é origem — é a mesma branch no remoto.
        if base == current {
            continue;
        }
        // Dedup local × remota da mesma branch (master vs origin/master): sem
        // isso, as duas empatam em score e o resultado vira "indeterminado" falso.
        if locals.contains(base) {
            continue;
        }
        names.insert(remote);
    }

    let mut list: Vec<_> = names.into_iter().collect();
    list.sort();
    list
}

pub(super) fn limit_candidates(repo: &Repository, candidates: Vec<String>) -> Vec<String> {
    if candidates.len() <= MAX_SCORED_CANDIDATES {
        return candidates;
    }
    let mut with_time: Vec<(String, i64)> = candidates
        .into_iter()
        .map(|name| {
            let time = branch_tip(repo, &name)
                .and_then(|oid| repo.find_commit(oid).ok())
                .map(|c| c.time().seconds())
                .unwrap_or(0);
            (name, time)
        })
        .collect();
    // Prioritárias primeiro, depois as mais recentes.
    with_time.sort_by(|a, b| {
        name_priority(&b.0)
            .cmp(&name_priority(&a.0))
            .then_with(|| b.1.cmp(&a.1))
    });
    with_time.truncate(MAX_SCORED_CANDIDATES);
    with_time.into_iter().map(|(name, _)| name).collect()
}

fn branch_names(repo: &Repository, kind: BranchType) -> HashSet<String> {
    let mut names = HashSet::new();
    if let Ok(branches) = repo.branches(Some(kind)) {
        for item in branches.flatten() {
            if let Ok(Some(name)) = item.0.name() {
                names.insert(name.to_string());
            }
        }
    }
    names
}

/// Nome da branch sem o prefixo do remoto ("origin/feature/x" → "feature/x").
/// O shorthand de remota é sempre `<remote>/<branch>`; o primeiro segmento é o
/// remoto, o restante (que pode conter '/') é a branch.
fn remote_base_name(name: &str) -> &str {
    name.split_once('/').map(|(_, rest)| rest).unwrap_or(name)
}

/// Resolve a ponta de uma branch por nome (local, remota ou rev). Pública
/// porque a trilha dupla (RF-01) também precisa localizar a base.
pub(super) fn branch_tip(repo: &Repository, name: &str) -> Option<Oid> {
    if let Ok(reference) = repo.find_reference(&format!("refs/heads/{name}")) {
        return reference.target();
    }
    if let Ok(reference) = repo.find_reference(&format!("refs/remotes/{name}")) {
        return reference.target();
    }
    if let Ok(reference) = repo.find_reference(&format!("refs/remotes/origin/{name}")) {
        return reference.target();
    }
    repo.revparse_single(name)
        .ok()
        .and_then(|obj| obj.peel_to_commit().ok())
        .map(|c| c.id())
}
