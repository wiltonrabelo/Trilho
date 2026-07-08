//! RF-12 — status de Pull Request da branch no GitHub.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};
use std::time::{Duration, Instant};

use crate::infrastructure::credential::get_github_api_token;
use crate::infrastructure::validation::{
    github_credential_path, parse_github_slug_from_remote, GithubSlug,
};

const CACHE_TTL: Duration = Duration::from_secs(60);
const USER_AGENT: &str = "Trilho/0.1";

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct PrSummary {
    pub number: u64,
    pub title: String,
    pub url: String,
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
struct GithubPull {
    number: u64,
    title: String,
    state: String,
    html_url: String,
    merged_at: Option<String>,
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
pub fn get_branch_pr_status(remote_url: &str, branch: &str) -> BranchPrStatus {
    let slug = match parse_github_slug_from_remote(remote_url) {
        Some(s) => s,
        None => return hidden(),
    };
    let branch = branch.trim();
    if branch.is_empty() {
        return hidden();
    }

    let cache_key = format!("{}/{}:{branch}", slug.owner, slug.repo);
    if let Some(cached) = read_cache(&cache_key) {
        return cached;
    }

    let token = match get_github_api_token(Some(&github_credential_path(&slug))) {
        Some(t) => t,
        None => return hidden(),
    };

    let status = match fetch_prs(&slug, branch, &token) {
        Ok(status) => status,
        Err(msg) => unavailable(msg),
    };
    write_cache(cache_key, status.clone());
    status
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

fn fetch_prs(slug: &GithubSlug, branch: &str, token: &str) -> Result<BranchPrStatus, String> {
    let head = format!("{}:{branch}", slug.owner);
    let url = format!(
        "https://api.github.com/repos/{}/{}/pulls?head={head}&state=all&per_page=20",
        slug.owner, slug.repo
    );

    let response = ureq::get(&url)
        .set("User-Agent", USER_AGENT)
        .set("Accept", "application/vnd.github+json")
        .set("Authorization", &format!("Bearer {token}"))
        .call()
        .map_err(|e| match e {
            ureq::Error::Status(403, resp) => {
                let remaining = resp.header("x-ratelimit-remaining");
                if remaining == Some("0") {
                    "rate limit GitHub".to_string()
                } else {
                    format!("acesso negado ({})", resp.status())
                }
            }
            ureq::Error::Status(code, _) => format!("HTTP {code}"),
            ureq::Error::Transport(t) => t.to_string(),
        })?;

    let pulls: Vec<GithubPull> = response
        .into_json()
        .map_err(|e| format!("resposta inválida: {e}"))?;

    let mut open = Vec::new();
    let mut merged = Vec::new();
    let mut closed = Vec::new();

    for pr in pulls {
        let summary = PrSummary {
            number: pr.number,
            title: pr.title,
            url: pr.html_url,
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

    Ok(BranchPrStatus {
        visible: true,
        open,
        merged,
        closed,
        notice: None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hidden_quando_nao_e_github() {
        let status = get_branch_pr_status("https://gitlab.com/u/r.git", "main");
        assert!(!status.visible);
    }

    #[test]
    fn classifica_estados_do_json() {
        let pulls = r#"[
          {"number":1,"title":"Open","state":"open","html_url":"https://github.com/o/r/pull/1","merged_at":null},
          {"number":2,"title":"Merged","state":"closed","html_url":"https://github.com/o/r/pull/2","merged_at":"2026-01-01T00:00:00Z"},
          {"number":3,"title":"Closed","state":"closed","html_url":"https://github.com/o/r/pull/3","merged_at":null}
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
    }
}
