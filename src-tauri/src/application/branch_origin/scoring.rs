use crate::domain::OriginConfidence;
use git2::{Oid, Repository, Sort};
use std::collections::HashMap;

use super::ScoredCandidate;
use super::candidates::branch_tip;

/// Cadeia first-parent de `start` como mapa `oid → distância` (com teto).
pub(super) fn first_parent_depths(repo: &Repository, start: Oid, cap: usize) -> HashMap<Oid, usize> {
    let mut map = HashMap::new();
    let mut cursor = Some(start);
    let mut depth = 0usize;
    while let Some(oid) = cursor {
        if map.len() >= cap || map.contains_key(&oid) {
            break;
        }
        map.insert(oid, depth);
        depth += 1;
        cursor = repo.find_commit(oid).ok().and_then(|c| c.parent_id(0).ok());
    }
    map
}

pub(super) fn score_candidate(
    repo: &Repository,
    head_oid: Oid,
    head_fp_depths: &HashMap<Oid, usize>,
    candidate: &str,
    current_branch: &str,
) -> Option<ScoredCandidate> {
    let tip = branch_tip(repo, candidate)?;
    let merge_base = repo.merge_base(head_oid, tip).ok()?;

    // Mesmo commit: só trilha de integração (main/develop) pode ser origem da feature.
    if tip == head_oid {
        let cur_prio = name_priority(current_branch);
        let cand_prio = name_priority(candidate);
        if cand_prio <= cur_prio {
            return None;
        }
        return Some(ScoredCandidate {
            name: candidate.to_string(),
            score: 50.0 + cand_prio as f64,
            structural: 30.0,
            signals: vec![format!(
                "trilha de integração «{candidate}» no mesmo commit que «{current_branch}»"
            )],
            merge_base: tip,
            depth: 0,
        });
    }

    // Candidata à frente da HEAD: branch filha, não origem.
    if merge_base == head_oid {
        return None;
    }

    // Fork verdadeiro: o merge-base precisa estar na trilha first-parent da
    // HEAD. Caso contrário a candidata foi MERGEADA para dentro (convergência),
    // não é a origem. A distância na trilha é a proximidade do fork.
    let depth = *head_fp_depths.get(&merge_base)?;

    let mut score = 0.0;
    let mut structural = 0.0;
    let mut signals = Vec::new();

    if merge_base == tip {
        score += 40.0;
        structural += 40.0;
        signals.push(format!("merge-base coincide com ponta de «{candidate}»"));
    }

    let commits_since = count_commits_since(repo, merge_base, head_oid);
    let recency = (25.0 - commits_since as f64).max(0.0);
    if recency > 0.0 {
        score += recency;
        if recency >= 15.0 {
            structural += recency;
            signals.push(format!(
                "fork recente ({commits_since} commits desde o merge-base)"
            ));
        }
    }

    if first_unique_parent_is_merge_base(repo, head_oid, tip, merge_base) {
        score += 30.0;
        structural += 30.0;
        signals.push("primeiro commit exclusivo tem pai no merge-base".into());
    }

    let (lineage_boost, lineage_signal) = name_lineage_boost(current_branch, candidate);
    if lineage_boost > 0.0 {
        score += lineage_boost;
        structural += lineage_boost;
        signals.push(lineage_signal);
    }

    score += name_priority(candidate) as f64;

    Some(ScoredCandidate {
        name: candidate.to_string(),
        score,
        structural,
        signals,
        merge_base,
        depth,
    })
}

/// Sinal DOMINANTE de proximidade (§7.3): como todos os merge-bases estão na
/// mesma trilha first-parent da HEAD, eles são totalmente ordenados por
/// profundidade. O merge-base MAIS PRÓXIMO da HEAD é o ponto de fork real; os
/// mais fundos são apenas tronco HERDADO (ancestrais desse fork) — jamais a
/// origem direta. Premia o(s) fork(s) mais recente(s) de forma decisiva; isso
/// resolve o caso em que uma branch antiga mergeada-e-divergente (cujo merge-base
/// é o fork ANTIGO dela no tronco) competia com a origem verdadeira.
///
/// Desempate no grupo do fork mais próximo: várias branches-IRMÃS nascem do mesmo
/// ponto do tronco (mesmo merge-base) e empatam. A origem convencional entre elas
/// é a trilha de INTEGRAÇÃO (nome canônico: development/main/...). Sem esse
/// desempate o resultado vira «indeterminado» por quase-empate — inútil na prática.
pub(super) fn apply_merge_base_proximity(scored: &mut [ScoredCandidate]) {
    let Some(min_depth) = scored.iter().map(|s| s.depth).min() else {
        return;
    };

    for c in scored.iter_mut() {
        if c.depth == min_depth {
            c.score += 45.0;
            c.structural += 45.0;
            c.signals
                .push("merge-base mais próximo da HEAD (fork mais recente)".into());
        }
    }

    // Dentro do grupo do fork mais próximo, a branch com nome de tronco mais forte
    // (e ÚNICA nesse posto) é a origem convencional — desempata de forma decisiva.
    let top_name = scored
        .iter()
        .filter(|s| s.depth == min_depth)
        .map(|s| name_priority(&s.name))
        .max()
        .unwrap_or(0);
    // Prioridade < 2 = nome comum (feature/hotfix/...): ninguém no grupo é trilha
    // de integração — ambiguidade honesta, sem desempate por nome.
    if top_name < 2 {
        return;
    }
    let leaders = scored
        .iter()
        .filter(|s| s.depth == min_depth && name_priority(&s.name) == top_name)
        .count();
    if leaders != 1 {
        return;
    }
    if let Some(best) = scored
        .iter_mut()
        .find(|s| s.depth == min_depth && name_priority(&s.name) == top_name)
    {
        best.score += 18.0;
        best.structural += 18.0;
        best.signals
            .push("trilha de integração no ponto de fork (desempate por nome)".into());
    }
}

fn count_commits_since(repo: &Repository, from: Oid, to: Oid) -> usize {
    let mut revwalk = match repo.revwalk() {
        Ok(r) => r,
        Err(_) => return 0,
    };
    let _ = revwalk.hide(from);
    let _ = revwalk.push(to);
    // A pontuação de recência zera a partir de 25 commits — não há razão para
    // contar além disso (em repo grande seriam dezenas de milhares).
    revwalk.take(26).count()
}

fn first_unique_parent_is_merge_base(
    repo: &Repository,
    head: Oid,
    candidate_tip: Oid,
    merge_base: Oid,
) -> bool {
    let mut revwalk = match repo.revwalk() {
        Ok(r) => r,
        Err(_) => return false,
    };
    let _ = revwalk.hide(candidate_tip);
    let _ = revwalk.push(head);
    let _ = revwalk.set_sorting(Sort::TOPOLOGICAL);

    // O revwalk topológico emite do mais novo para o mais antigo; o "primeiro
    // commit exclusivo" da branch é o ÚLTIMO emitido — é o pai dele que deve
    // apontar para o merge-base. Teto de 500: além disso o fork é antigo e o
    // sinal deixa de ser relevante (e o walk completo custa caro em repo grande).
    const MAX_UNIQUE_WALK: usize = 500;
    let mut count = 0usize;
    let mut oldest = None;
    for oid in revwalk {
        let Ok(oid) = oid else { continue };
        count += 1;
        if count > MAX_UNIQUE_WALK {
            return false;
        }
        oldest = Some(oid);
    }
    if let Some(oid) = oldest {
        if let Ok(commit) = repo.find_commit(oid) {
            return commit.parent_ids().any(|p| p == merge_base);
        }
    }
    false
}

/// «main_teste_3_1» costuma derivar de «main_teste_3» — sinal forte de linhagem.
fn name_lineage_boost(current: &str, candidate: &str) -> (f64, String) {
    if current == candidate {
        return (0.0, String::new());
    }
    if current.starts_with(candidate) {
        let rest = &current[candidate.len()..];
        if rest.starts_with('_') || rest.starts_with('/') {
            return (
                40.0,
                format!("nome «{current}» indica derivação de «{candidate}»"),
            );
        }
    }
    (0.0, String::new())
}

/// Prioridade do nome para fins de ORIGEM. Working branches (feature/bugfix/
/// release) saem quase sempre da trilha de INTEGRAÇÃO — por isso develop/
/// development têm prioridade MAIOR que main/master aqui (o oposto de
/// "importância" do branch): num empate de ponto de fork, a origem provável é a
/// integração, não a linha principal (de onde só hotfix costuma sair).
pub(super) fn name_priority(name: &str) -> i32 {
    let n = name.to_lowercase();
    let base = n.rsplit('/').next().unwrap_or(&n);
    match base {
        "develop" | "development" | "dev" => 3,
        "main" | "master" | "trunk" => 2,
        _ => 1,
    }
}

pub(super) fn classify_confidence(best: f64, second: f64, structural: f64) -> OriginConfidence {
    if best < 15.0 || (second > 0.0 && best - second < 5.0) {
        return OriginConfidence::Indeterminate;
    }
    let gap = best - second;
    if structural >= 30.0 && best >= 50.0 && gap >= 12.0 {
        OriginConfidence::High
    } else if best >= 30.0 && gap >= 5.0 {
        OriginConfidence::Medium
    } else if best >= 15.0 {
        OriginConfidence::Low
    } else {
        OriginConfidence::Indeterminate
    }
}
