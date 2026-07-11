//! RF-12 — status de Pull Request da branch no GitHub.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};
use std::time::{Duration, Instant};

use crate::infrastructure::credential::resolve_github_api_token;
use crate::infrastructure::validation::{parse_github_slug_from_remote, GithubSlug};

const CACHE_TTL: Duration = Duration::from_secs(60);
const USER_AGENT: &str = "Trilho/0.1";
const GITHUB_API_VERSION: &str = "2022-11-28";

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct PrSummary {
    pub number: u64,
    pub title: String,
    pub url: String,
    /// Branch de destino do PR (base) — ex.: feature-SPF-1112.
    pub base_branch: String,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct BranchPrStatus {
    /// `false` = não exibir (não é GitHub, sem credencial, etc.).
    pub visible: bool,
    pub open: Vec<PrSummary>,
    pub merged: Vec<PrSummary>,
    /// Fechado sem merge.
    pub closed: Vec<PrSummary>,
    /// Aviso curto quando a consulta falhou (rede, rate limit).
    pub notice: Option<String>,
}

#[derive(Debug, Deserialize)]
struct GithubPullHead {
    #[serde(rename = "ref")]
    ref_name: String,
    #[serde(default)]
    label: String,
}

#[derive(Debug, Deserialize)]
struct GithubPullBase {
    #[serde(rename = "ref")]
    ref_name: String,
}

#[derive(Debug, Deserialize)]
struct GithubPull {
    number: u64,
    title: String,
    state: String,
    html_url: String,
    merged_at: Option<String>,
    head: GithubPullHead,
    base: GithubPullBase,
}

struct CacheEntry {
    fetched_at: Instant,
    value: BranchPrStatus,
}

fn cache_store() -> &'static Mutex<HashMap<String, CacheEntry>> {
    static STORE: OnceLock<Mutex<HashMap<String, CacheEntry>>> = OnceLock::new();
    STORE.get_or_init(|| Mutex::new(HashMap::new()))
}

fn hidden() -> BranchPrStatus {
    BranchPrStatus {
        visible: false,
        open: Vec::new(),
        merged: Vec::new(),
        closed: Vec::new(),
        notice: None,
    }
}

fn unavailable(notice: impl Into<String>) -> BranchPrStatus {
    BranchPrStatus {
        visible: true,
        open: Vec::new(),
        merged: Vec::new(),
        closed: Vec::new(),
        notice: Some(notice.into()),
    }
}

/// Consulta PRs da branch no GitHub (com cache curto).
pub fn get_branch_pr_status(
    remote_url: &str,
    branch: &str,
    data_dir: Option<&std::path::Path>,
) -> BranchPrStatus {
    let slug = match parse_github_slug_from_remote(remote_url) {
        Some(s) => s,
        None => return hidden(),
    };
    let branch = branch.trim();
    if branch.is_empty() {
        return hidden();
    }
    let ssh_remote = is_ssh_remote(remote_url);

    let cache_key = format!("{}:{}/{}:{branch}", slug.host, slug.owner, slug.repo);
    if let Some(cached) = read_cache(&cache_key) {
        return cached;
    }

    let token = match resolve_github_api_token(&slug, data_dir) {
        Some(t) => t,
        None => {
            if ssh_remote {
                return unavailable(
                    "salve um PAT (ghp_… ou github_pat_…) com escopo repo em Conectar → Token",
                );
            }
            return hidden();
        }
    };

    let status = match fetch_prs(&slug, branch, &token) {
        Ok(status) => status,
        Err(msg) => {
            if ssh_remote && looks_like_repo_access_denied(&msg) {
                unavailable(
                    "token sem acesso à API deste repo — use PAT clássico com escopo repo e autorize SSO da organização",
                )
            } else {
                unavailable(msg)
            }
        }
    };
    if status.notice.is_none() {
        write_cache(cache_key, status.clone());
    }
    status
}

fn is_ssh_remote(url: &str) -> bool {
    let u = url.trim();
    u.starts_with("git@") || u.starts_with("ssh://")
}

fn looks_like_repo_access_denied(msg: &str) -> bool {
    let m = msg.to_lowercase();
    m.contains("404")
        || m.contains("não encontrado")
        || m.contains("sem acesso")
        || m.contains("403")
        || m.contains("negado")
}

fn read_cache(key: &str) -> Option<BranchPrStatus> {
    let store = cache_store().lock().ok()?;
    let entry = store.get(key)?;
    if entry.fetched_at.elapsed() > CACHE_TTL {
        return None;
    }
    Some(entry.value.clone())
}

fn write_cache(key: String, value: BranchPrStatus) {
    if let Ok(mut store) = cache_store().lock() {
        store.insert(
            key,
            CacheEntry {
                fetched_at: Instant::now(),
                value,
            },
        );
    }
}

/// Limpa cache de PR (ex.: após salvar novo PAT).
pub fn clear_branch_pr_cache() {
    if let Ok(mut store) = cache_store().lock() {
        store.clear();
    }
}

fn fetch_prs(slug: &GithubSlug, branch: &str, token: &str) -> Result<BranchPrStatus, String> {
    match fetch_prs_by_head(slug, branch, token) {
        Ok(status) if !status.open.is_empty() || !status.merged.is_empty() || !status.closed.is_empty() => {
            Ok(status)
        }
        Ok(_) | Err(_) => fetch_prs_list_filter(slug, branch, token),
    }
}

fn fetch_prs_by_head(slug: &GithubSlug, branch: &str, token: &str) -> Result<BranchPrStatus, String> {
    let head = format!("{}:{}", slug.owner, branch);
    let head_q = percent_encode_query(&head);
    let url = format!(
        "{}/repos/{}/{}/pulls?head={head_q}&state=all&per_page=20",
        slug.api_base_url(),
        slug.owner,
        slug.repo
    );
    let pulls = github_get_json::<Vec<GithubPull>>(&url, token)?;
    Ok(classify_pulls(pulls))
}

fn fetch_prs_list_filter(slug: &GithubSlug, branch: &str, token: &str) -> Result<BranchPrStatus, String> {
    let url = format!(
        "{}/repos/{}/{}/pulls?state=all&per_page=100&sort=updated&direction=desc",
        slug.api_base_url(),
        slug.owner,
        slug.repo
    );
    let pulls = github_get_json::<Vec<GithubPull>>(&url, token)?;
    let filtered: Vec<GithubPull> = pulls
        .into_iter()
        .filter(|pr| pull_matches_branch(pr, branch))
        .collect();
    Ok(classify_pulls(filtered))
}

fn pull_matches_branch(pr: &GithubPull, branch: &str) -> bool {
    pr.head.ref_name == branch
        || pr.head.label.ends_with(&format!(":{branch}"))
        || pr.head.label == branch
}

fn github_get_json<T: for<'de> Deserialize<'de>>(url: &str, token: &str) -> Result<T, String> {
    let response = ureq::get(url)
        .set("User-Agent", USER_AGENT)
        .set("Accept", "application/vnd.github+json")
        .set("X-GitHub-Api-Version", GITHUB_API_VERSION)
        .set("Authorization", &format!("Bearer {token}"))
        .call()
        .map_err(map_github_error)?;

    response
        .into_json()
        .map_err(|e| format!("resposta inválida: {e}"))
}

fn map_github_error(e: ureq::Error) -> String {
    match e {
        ureq::Error::Status(401, _) => {
            "token GitHub inválido ou expirado — salve um PAT em Conectar → Token".into()
        }
        ureq::Error::Status(403, resp) => {
            let remaining = resp.header("x-ratelimit-remaining");
            if remaining == Some("0") {
                "rate limit GitHub — aguarde alguns minutos".into()
            } else {
                let body = resp.into_string().unwrap_or_default();
                if body.to_lowercase().contains("sso") {
                    "autorize o token SSO da organização em github.com/settings/tokens".into()
                } else {
                    "acesso negado à API (403) — verifique escopo repo no PAT".into()
                }
            }
        }
        ureq::Error::Status(404, _) => {
            "repositório privado inacessível via API — salve PAT com escopo repo (Conectar → Token)".into()
        }
        ureq::Error::Status(code, _) => format!("HTTP {code}"),
        ureq::Error::Transport(t) => t.to_string(),
    }
}

fn classify_pulls(pulls: Vec<GithubPull>) -> BranchPrStatus {
    let mut open = Vec::new();
    let mut merged = Vec::new();
    let mut closed = Vec::new();

    for pr in pulls {
        let summary = PrSummary {
            number: pr.number,
            title: pr.title,
            url: pr.html_url,
            base_branch: pr.base.ref_name,
        };
        if pr.state.eq_ignore_ascii_case("open") {
            open.push(summary);
            continue;
        }
        if pr.merged_at.is_some() {
            merged.push(summary);
        } else {
            closed.push(summary);
        }
    }

    BranchPrStatus {
        visible: true,
        open,
        merged,
        closed,
        notice: None,
    }
}

fn percent_encode_query(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    for b in value.bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(b as char);
            }
            _ => out.push_str(&format!("%{b:02X}")),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hidden_quando_nao_e_github() {
        let status = get_branch_pr_status("https://gitlab.com/u/r.git", "main", None);
        assert!(!status.visible);
    }

    #[test]
    fn classifica_estados_do_json() {
        let pulls = r#"[
          {"number":1,"title":"Open","state":"open","html_url":"https://github.com/o/r/pull/1","merged_at":null,"head":{"ref":"feature-x","label":"o:feature-x"},"base":{"ref":"main"}},
          {"number":2,"title":"Merged","state":"closed","html_url":"https://github.com/o/r/pull/2","merged_at":"2026-01-01T00:00:00Z","head":{"ref":"feature-y","label":"o:feature-y"},"base":{"ref":"develop"}},
          {"number":3,"title":"Closed","state":"closed","html_url":"https://github.com/o/r/pull/3","merged_at":null,"head":{"ref":"feature-z","label":"o:feature-z"},"base":{"ref":"main"}}
        ]"#;
        let parsed: Vec<GithubPull> = serde_json::from_str(pulls).expect("json");
        let mut open = Vec::new();
        let mut merged = Vec::new();
        let mut closed = Vec::new();
        for pr in parsed {
            let summary = PrSummary {
                number: pr.number,
                title: pr.title,
                url: pr.html_url,
                base_branch: pr.base.ref_name,
            };
            if pr.state.eq_ignore_ascii_case("open") {
                open.push(summary);
            } else if pr.merged_at.is_some() {
                merged.push(summary);
            } else {
                closed.push(summary);
            }
        }
        assert_eq!(open.len(), 1);
        assert_eq!(merged.len(), 1);
        assert_eq!(closed.len(), 1);
        assert_eq!(open[0].base_branch, "main");
    }

    #[test]
    fn pull_matches_branch_por_ref_ou_label() {
        let pr = GithubPull {
            number: 9898,
            title: "t".into(),
            state: "open".into(),
            html_url: "https://github.com/casamagalhaes/syspdv/pull/9898".into(),
            merged_at: None,
            head: GithubPullHead {
                ref_name: "feature-SPF-1122".into(),
                label: "casamagalhaes:feature-SPF-1122".into(),
            },
            base: GithubPullBase {
                ref_name: "feature-SPF-1112".into(),
            },
        };
        assert!(pull_matches_branch(&pr, "feature-SPF-1122"));
    }

    #[test]
    fn percent_encode_codifica_dois_pontos() {
        assert_eq!(
            percent_encode_query("casamagalhaes:feature-SPF-1122"),
            "casamagalhaes%3Afeature-SPF-1122"
        );
    }
}
