//! Heurística de origem da branch (RF-02, PLANO §7.3).

use crate::domain::{BranchOrigin, OriginConfidence};
use git2::{BranchType, Oid, Repository, Sort};
use std::collections::{HashMap, HashSet};

struct ScoredCandidate {
    name: String,
    score: f64,
    structural: f64,
    signals: Vec<String>,
    /// Merge-base com a HEAD — ponto de divergência exposto no resultado.
    merge_base: Oid,
    /// Distância (em passos first-parent) do merge-base até a HEAD. Menor =
    /// fork mais RECENTE = origem mais provável (§7.3). Candidatas com merge-base
    /// mais fundo são pontos do tronco HERDADO (ancestrais do fork real).
    depth: usize,
}

/// Infere a branch de origem da branch atual com pontuação e confiança honesta.
pub fn infer_branch_origin(repo: &Repository) -> BranchOrigin {
    let head = match repo.head() {
        Ok(h) => h,
        Err(_) => return BranchOrigin::indeterminate(None, "Repositório sem HEAD."),
    };

    if !head.is_branch() {
        return BranchOrigin::indeterminate(
            None,
            "HEAD detached — origem da branch indeterminada.",
        );
    }

    let current = head.shorthand().unwrap_or("HEAD").to_string();
    let head_oid = match head.target() {
        Some(o) => o,
        None => {
            return BranchOrigin::indeterminate(
                Some(current),
                "HEAD sem commit — origem indeterminada.",
            );
        }
    };

    let candidates = limit_candidates(repo, collect_candidates(repo, &current));
    if candidates.is_empty() {
        return BranchOrigin {
            current_branch: Some(current),
            candidate: None,
            confidence: OriginConfidence::Indeterminate,
            explanation: "Nenhuma outra branch candidata no repositório.".into(),
            signals: vec![],
            merge_base_id: None,
        };
    }

    // Cadeia first-parent da HEAD (oid → distância): o merge-base de uma origem
    // VERDADEIRA está nela (é o ponto de fork). Branches mergeadas para dentro
    // da trilha têm merge-base fora dessa cadeia (entram como 2º pai) e são
    // descartadas — sem isso, uma branch recém-mergeada vence com confiança alta
    // falsa. A distância também mede a PROXIMIDADE do fork (§7.3).
    let head_fp_depths = first_parent_depths(repo, head_oid, 5000);

    let mut scored: Vec<ScoredCandidate> = candidates
        .into_iter()
        .filter_map(|name| score_candidate(repo, head_oid, &head_fp_depths, &name))
        .collect();

    apply_merge_base_proximity(&mut scored);

    scored.sort_by(|a, b| {
        b.score
            .partial_cmp(&a.score)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| name_priority(&b.name).cmp(&name_priority(&a.name)))
    });

    if scored.is_empty() {
        return BranchOrigin {
            current_branch: Some(current),
            candidate: None,
            confidence: OriginConfidence::Indeterminate,
            explanation: "Nenhuma candidata pontuou — origem indeterminada.".into(),
            signals: vec![],
            merge_base_id: None,
        };
    }

    let best = &scored[0];
    let second_score = scored.get(1).map(|s| s.score).unwrap_or(0.0);
    let confidence = classify_confidence(best.score, second_score, best.structural);

    if confidence == OriginConfidence::Indeterminate {
        return BranchOrigin {
            current_branch: Some(current),
            candidate: None,
            confidence,
            explanation: "Candidatas ambíguas — origem indeterminada.".into(),
            signals: best.signals.clone(),
            merge_base_id: None,
        };
    }

    let explanation = match confidence {
        OriginConfidence::High => format!(
            "Sinais estruturais fortes indicam que «{}» derivou de «{}».",
            current, best.name
        ),
        OriginConfidence::Medium => format!(
            "Heurística sugere «{}» como origem de «{}» — confirme no histórico.",
            best.name, current
        ),
        OriginConfidence::Low => format!(
            "Possível origem «{}» para «{}», com evidência fraca.",
            best.name, current
        ),
        OriginConfidence::Indeterminate => unreachable!(),
    };

    BranchOrigin {
        current_branch: Some(current),
        candidate: Some(best.name.clone()),
        confidence,
        explanation,
        signals: best.signals.clone(),
        merge_base_id: Some(best.merge_base.to_string()),
    }
}

fn collect_candidates(repo: &Repository, current: &str) -> Vec<String> {
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

/// Teto de candidatas pontuadas — em repositórios com centenas de branches
/// (SysPDV: 300+), pontuar todas custa minutos. Mantém as prioritárias
/// (main/master/develop/...) e as N de commit mais recente.
const MAX_SCORED_CANDIDATES: usize = 40;

fn limit_candidates(repo: &Repository, candidates: Vec<String>) -> Vec<String> {
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

/// Cadeia first-parent de `start` como mapa `oid → distância` (com teto).
fn first_parent_depths(repo: &Repository, start: Oid, cap: usize) -> HashMap<Oid, usize> {
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

fn score_candidate(
    repo: &Repository,
    head_oid: Oid,
    head_fp_depths: &HashMap<Oid, usize>,
    candidate: &str,
) -> Option<ScoredCandidate> {
    let tip = branch_tip(repo, candidate)?;
    let merge_base = repo.merge_base(head_oid, tip).ok()?;

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
fn apply_merge_base_proximity(scored: &mut [ScoredCandidate]) {
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

/// Resolve a ponta de uma branch por nome (local, remota ou rev). Pública
/// porque a trilha dupla (RF-01) também precisa localizar a base.
pub fn branch_tip(repo: &Repository, name: &str) -> Option<Oid> {
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

/// Prioridade do nome para fins de ORIGEM. Working branches (feature/bugfix/
/// release) saem quase sempre da trilha de INTEGRAÇÃO — por isso develop/
/// development têm prioridade MAIOR que main/master aqui (o oposto de
/// "importância" do branch): num empate de ponto de fork, a origem provável é a
/// integração, não a linha principal (de onde só hotfix costuma sair).
fn name_priority(name: &str) -> i32 {
    let n = name.to_lowercase();
    let base = n.rsplit('/').next().unwrap_or(&n);
    match base {
        "develop" | "development" | "dev" => 3,
        "main" | "master" | "trunk" => 2,
        _ => 1,
    }
}

fn classify_confidence(best: f64, second: f64, structural: f64) -> OriginConfidence {
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

/// Reforço fraco via reflog — nunca eleva sozinho acima de Low (PLANO §7.3).
pub fn apply_reflog_hint(mut origin: BranchOrigin, reflog_branch: Option<&str>) -> BranchOrigin {
    let Some(candidate) = origin.candidate.clone() else {
        return origin;
    };
    let Some(reflog) = reflog_branch else {
        return origin;
    };
    let lower = reflog.to_lowercase();
    let needle = candidate.to_lowercase();
    if !lower.contains(&needle) {
        return origin;
    }

    origin
        .signals
        .push("reflog menciona candidata (sinal fraco)".into());

    if origin.confidence == OriginConfidence::Indeterminate && origin.signals.len() >= 2 {
        origin.confidence = OriginConfidence::Low;
        origin.explanation =
            format!("Reflog reforça «{candidate}», mas evidência estrutural é limitada.");
    }
    origin
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::process::Command;

    fn init_repo(path: &std::path::Path) {
        fs::create_dir_all(path).unwrap();
        Command::new("git")
            .args(["init"])
            .current_dir(path)
            .output()
            .unwrap();
        Command::new("git")
            .args(["config", "user.email", "t@t.com"])
            .current_dir(path)
            .output()
            .unwrap();
        Command::new("git")
            .args(["config", "user.name", "T"])
            .current_dir(path)
            .output()
            .unwrap();
    }

    fn commit_file(path: &std::path::Path, file: &str, msg: &str) {
        fs::write(path.join(file), msg).unwrap();
        Command::new("git")
            .args(["add", file])
            .current_dir(path)
            .output()
            .unwrap();
        Command::new("git")
            .args(["commit", "-m", msg])
            .current_dir(path)
            .output()
            .unwrap();
    }

    #[test]
    fn detecta_origem_de_feature_branch() {
        let dir = std::env::temp_dir().join(format!("trilho-origin-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        init_repo(&dir);
        commit_file(&dir, "a.txt", "base");
        Command::new("git")
            .args(["branch", "feature"])
            .current_dir(&dir)
            .output()
            .unwrap();
        Command::new("git")
            .args(["checkout", "feature"])
            .current_dir(&dir)
            .output()
            .unwrap();
        commit_file(&dir, "b.txt", "feature work");

        let repo = Repository::discover(&dir).unwrap();
        let origin = infer_branch_origin(&repo);
        assert!(origin.candidate.is_some());
        assert_ne!(origin.confidence, OriginConfidence::Indeterminate);
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn upstream_da_propria_branch_nao_e_candidata() {
        let dir = std::env::temp_dir().join(format!("trilho-origin-up-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        init_repo(&dir);
        commit_file(&dir, "a.txt", "base");
        Command::new("git")
            .args(["checkout", "-b", "feature"])
            .current_dir(&dir)
            .output()
            .unwrap();
        commit_file(&dir, "b.txt", "feature work");
        // Simula a branch sincronizada no remoto (estado normal pós-push).
        Command::new("git")
            .args(["update-ref", "refs/remotes/origin/feature", "HEAD"])
            .current_dir(&dir)
            .output()
            .unwrap();

        let repo = Repository::discover(&dir).unwrap();
        let origin = infer_branch_origin(&repo);
        assert_ne!(
            origin.candidate.as_deref(),
            Some("origin/feature"),
            "a contraparte remota da própria branch não pode ser a origem"
        );
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn local_e_remota_da_mesma_branch_nao_empatam() {
        let dir = std::env::temp_dir().join(format!("trilho-origin-dup-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        init_repo(&dir);
        commit_file(&dir, "a.txt", "base");
        let default = Command::new("git")
            .args(["rev-parse", "--abbrev-ref", "HEAD"])
            .current_dir(&dir)
            .output()
            .unwrap();
        let default = String::from_utf8_lossy(&default.stdout).trim().to_string();
        // Remota sincronizada da branch padrão (master vs origin/master).
        Command::new("git")
            .args([
                "update-ref",
                &format!("refs/remotes/origin/{default}"),
                "HEAD",
            ])
            .current_dir(&dir)
            .output()
            .unwrap();
        Command::new("git")
            .args(["checkout", "-b", "feature"])
            .current_dir(&dir)
            .output()
            .unwrap();
        commit_file(&dir, "b.txt", "feature work");

        let repo = Repository::discover(&dir).unwrap();
        let origin = infer_branch_origin(&repo);
        assert_eq!(
            origin.candidate.as_deref(),
            Some(default.as_str()),
            "dedupe local×remota deve apontar a local, não empatar em indeterminado"
        );
        assert_ne!(origin.confidence, OriginConfidence::Indeterminate);
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn branch_mergeada_para_dentro_nao_e_origem() {
        let dir = std::env::temp_dir().join(format!("trilho-origin-mrg-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        init_repo(&dir);
        commit_file(&dir, "a.txt", "base");
        let default = Command::new("git")
            .args(["rev-parse", "--abbrev-ref", "HEAD"])
            .current_dir(&dir)
            .output()
            .unwrap();
        let default = String::from_utf8_lossy(&default.stdout).trim().to_string();

        // Branch "draevon" mergeada para dentro da default.
        Command::new("git")
            .args(["checkout", "-b", "draevon"])
            .current_dir(&dir)
            .output()
            .unwrap();
        commit_file(&dir, "d.txt", "draevon work");
        Command::new("git")
            .args(["checkout", &default])
            .current_dir(&dir)
            .output()
            .unwrap();
        commit_file(&dir, "m.txt", "avanca default"); // força merge não-ff
        Command::new("git")
            .args(["merge", "--no-ff", "draevon", "-m", "merge draevon"])
            .current_dir(&dir)
            .output()
            .unwrap();

        // Feature nasce da default DEPOIS do merge.
        Command::new("git")
            .args(["checkout", "-b", "feature"])
            .current_dir(&dir)
            .output()
            .unwrap();
        commit_file(&dir, "f.txt", "feature work");

        let repo = Repository::discover(&dir).unwrap();
        let origin = infer_branch_origin(&repo);
        assert_eq!(
            origin.candidate.as_deref(),
            Some(default.as_str()),
            "origem deve ser a default; branch mergeada para dentro (draevon) não é fork-source"
        );
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn merge_de_pr_herdado_da_base_nao_pontua_candidata() {
        let dir = std::env::temp_dir().join(format!("trilho-origin-inh-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        init_repo(&dir);
        commit_file(&dir, "a.txt", "base");
        let default = Command::new("git")
            .args(["rev-parse", "--abbrev-ref", "HEAD"])
            .current_dir(&dir)
            .output()
            .unwrap();
        let default = String::from_utf8_lossy(&default.stdout).trim().to_string();

        // "bugfix-x" mergeada na default ANTES do fork da feature (merge de PR
        // herdado, mencionando a branch na mensagem).
        Command::new("git")
            .args(["checkout", "-b", "bugfix-x"])
            .current_dir(&dir)
            .output()
            .unwrap();
        commit_file(&dir, "b.txt", "bug work");
        Command::new("git")
            .args(["checkout", &default])
            .current_dir(&dir)
            .output()
            .unwrap();
        commit_file(&dir, "c.txt", "avanca");
        Command::new("git")
            .args([
                "merge",
                "--no-ff",
                "bugfix-x",
                "-m",
                "Merge pull request #1 from org/bugfix-x",
            ])
            .current_dir(&dir)
            .output()
            .unwrap();

        // bugfix-x ainda existe como ref e diverge da default (branch reusada).
        Command::new("git")
            .args(["checkout", "bugfix-x"])
            .current_dir(&dir)
            .output()
            .unwrap();
        commit_file(&dir, "b2.txt", "bug extra");
        Command::new("git")
            .args(["checkout", &default])
            .current_dir(&dir)
            .output()
            .unwrap();

        // Feature nasce da default; nunca mergeou bugfix-x.
        Command::new("git")
            .args(["checkout", "-b", "feature"])
            .current_dir(&dir)
            .output()
            .unwrap();
        commit_file(&dir, "f.txt", "feature work");

        let repo = Repository::discover(&dir).unwrap();
        let origin = infer_branch_origin(&repo);
        assert_ne!(
            origin.candidate.as_deref(),
            Some("bugfix-x"),
            "merge de PR herdado da base não pode eleger a branch mencionada"
        );
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn hotfix_mergeado_e_divergente_nao_vence_a_origem_verdadeira() {
        // Reproduz o caso real (SysPDV): feature nasce de «development»; um
        // hotfix antigo forkou de development num ponto MAIS FUNDO e foi mergeado
        // de volta (merge de PR que menciona a branch). O merge herdado não pode
        // eleger o hotfix — a origem é «development» (fork mais próximo da HEAD).
        let dir = std::env::temp_dir().join(format!("trilho-origin-hf-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        init_repo(&dir);
        commit_file(&dir, "a.txt", "base");

        // Trilha de integração.
        Command::new("git")
            .args(["checkout", "-b", "development"])
            .current_dir(&dir)
            .output()
            .unwrap();
        commit_file(&dir, "d1.txt", "dev 1"); // ponto de fork antigo do hotfix

        // hotfix nasce de development em d1 e diverge (commit próprio, não mergeado).
        Command::new("git")
            .args(["checkout", "-b", "hotfix-SPF-1101"])
            .current_dir(&dir)
            .output()
            .unwrap();
        commit_file(&dir, "h1.txt", "hotfix work");

        // development avança e "mergeia o PR do hotfix": merge --no-ff com
        // mensagem que menciona a branch (o 2º pai é outra ramificação, simulando
        // squash/rebase — os commits reais do hotfix não ficam compartilhados).
        Command::new("git")
            .args(["checkout", "development"])
            .current_dir(&dir)
            .output()
            .unwrap();
        commit_file(&dir, "d2.txt", "dev 2");
        Command::new("git")
            .args(["checkout", "-b", "_pr", "development"])
            .current_dir(&dir)
            .output()
            .unwrap();
        commit_file(&dir, "p1.txt", "pr work");
        Command::new("git")
            .args(["checkout", "development"])
            .current_dir(&dir)
            .output()
            .unwrap();
        Command::new("git")
            .args([
                "merge",
                "--no-ff",
                "_pr",
                "-m",
                "Merge pull request #9 from org/hotfix-SPF-1101",
            ])
            .current_dir(&dir)
            .output()
            .unwrap();
        commit_file(&dir, "d3.txt", "dev 3");

        // feature nasce de development DEPOIS do merge herdado.
        Command::new("git")
            .args(["checkout", "-b", "feature-SPF-867", "development"])
            .current_dir(&dir)
            .output()
            .unwrap();
        commit_file(&dir, "f1.txt", "feature work");

        let repo = Repository::discover(&dir).unwrap();
        let origin = infer_branch_origin(&repo);
        assert_eq!(
            origin.candidate.as_deref(),
            Some("development"),
            "origem deve ser development (fork mais recente), não o hotfix mergeado e divergente"
        );
        assert_ne!(origin.confidence, OriginConfidence::Indeterminate);
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn origem_prefere_integracao_entre_irmas_e_master_no_mesmo_fork() {
        // Muitas branches nascem do MESMO ponto do tronco (mesmo merge-base):
        // development, master e irmãs (bugfix/feature). A proximidade sozinha as
        // empata → «indeterminado». O desempate deve eleger a trilha de
        // INTEGRAÇÃO (development), não master nem as irmãs.
        let dir = std::env::temp_dir().join(format!("trilho-origin-tie-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        init_repo(&dir);
        commit_file(&dir, "a.txt", "base"); // linha principal @ a
        let default = Command::new("git")
            .args(["rev-parse", "--abbrev-ref", "HEAD"])
            .current_dir(&dir)
            .output()
            .unwrap();
        let default = String::from_utf8_lossy(&default.stdout).trim().to_string();

        Command::new("git")
            .args(["checkout", "-b", "development"])
            .current_dir(&dir)
            .output()
            .unwrap();
        commit_file(&dir, "d1.txt", "dev 1"); // ponto de fork comum

        // Irmã nasce do mesmo ponto.
        Command::new("git")
            .args(["checkout", "-b", "bugfix-irma", "development"])
            .current_dir(&dir)
            .output()
            .unwrap();
        commit_file(&dir, "s1.txt", "irma work");

        // HEAD nasce do mesmo ponto (development @ d1).
        Command::new("git")
            .args(["checkout", "development"])
            .current_dir(&dir)
            .output()
            .unwrap();
        Command::new("git")
            .args(["checkout", "-b", "feature-SPF-867"])
            .current_dir(&dir)
            .output()
            .unwrap();
        commit_file(&dir, "f1.txt", "feature work");

        // development avança (para o merge-base não coincidir com a ponta dele).
        Command::new("git")
            .args(["checkout", "development"])
            .current_dir(&dir)
            .output()
            .unwrap();
        commit_file(&dir, "d2.txt", "dev 2");

        // A linha principal absorve o tronco (release) — passa a compartilhar o
        // mesmo fork, mas com ponta própria (não ganha o bônus "ponta == merge-base").
        Command::new("git")
            .args(["checkout", &default])
            .current_dir(&dir)
            .output()
            .unwrap();
        Command::new("git")
            .args(["merge", "--no-ff", "development", "-m", "release into main"])
            .current_dir(&dir)
            .output()
            .unwrap();

        Command::new("git")
            .args(["checkout", "feature-SPF-867"])
            .current_dir(&dir)
            .output()
            .unwrap();

        let repo = Repository::discover(&dir).unwrap();
        let origin = infer_branch_origin(&repo);
        assert_eq!(
            origin.candidate.as_deref(),
            Some("development"),
            "entre irmãs e master no mesmo fork, a origem é a trilha de integração"
        );
        assert_ne!(origin.confidence, OriginConfidence::Indeterminate);
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn detached_head_e_indeterminado() {
        let dir = std::env::temp_dir().join(format!("trilho-origin-det-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        init_repo(&dir);
        commit_file(&dir, "a.txt", "base");
        let sha = Command::new("git")
            .args(["rev-parse", "HEAD"])
            .current_dir(&dir)
            .output()
            .unwrap();
        let sha = String::from_utf8_lossy(&sha.stdout).trim().to_string();
        Command::new("git")
            .args(["checkout", &sha])
            .current_dir(&dir)
            .output()
            .unwrap();

        let repo = Repository::discover(&dir).unwrap();
        let origin = infer_branch_origin(&repo);
        assert_eq!(origin.confidence, OriginConfidence::Indeterminate);
        let _ = fs::remove_dir_all(&dir);
    }
}
