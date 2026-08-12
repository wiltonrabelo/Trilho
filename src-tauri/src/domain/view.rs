//! Tipos que atravessam o IPC para o frontend. Ficam no domínio para que o
//! contrato da UI não dependa de qual adaptador de infraestrutura o produz.

use serde::{Deserialize, Serialize};

use crate::domain::FileChangeKind;

/// Referência a branch remota (`origin/feature` → remote=`origin`, branch=`feature`).
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RemoteBranchRef {
    pub remote: String,
    pub branch: String,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct StashEntry {
    pub index: usize,
    /// Ref Git (`stash@{0}`).
    pub reference: String,
    /// Texto após `stash@{n}:`.
    pub message: String,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct TagEntry {
    pub name: String,
    pub commit_id: String,
    pub short_id: String,
}

/// Modo de comparação entre pontas (`A..B`) ou a partir do merge-base (`A...B`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
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

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct GithubAccount {
    pub username: String,
    pub is_active: bool,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct CredentialStatus {
    pub helper_configured: bool,
    pub gcm_available: bool,
    pub helper_summary: Option<String>,
    /// Mensagem acionável quando fetch remoto pode falhar por credencial.
    pub hint: Option<String>,
    /// Credencial GitHub já armazenada no helper (sem abrir GUI).
    pub github_connected: bool,
    /// Usuário retornado pelo helper, quando disponível.
    pub github_username: Option<String>,
    /// Contas HTTPS salvas no GCM (`git credential-manager github list`).
    pub github_accounts: Vec<GithubAccount>,
    /// `credential.https://github.com.useHttpPath` — separa credenciais por repositório.
    pub use_http_path: bool,
    /// Chaves privadas detectadas em `~/.ssh`.
    pub ssh_keys: Vec<SshKeyInfo>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SshKeyInfo {
    /// Nome base da chave (ex.: `id_ed25519`).
    pub name: String,
    pub has_public: bool,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SshTestResult {
    pub success: bool,
    pub username: Option<String>,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct PrSummary {
    pub number: u64,
    pub title: String,
    pub url: String,
    /// Branch de destino do PR (base) — ex.: feature-SPF-1112.
    pub base_branch: String,
}

/// RF-12 — PRs da branch atual no GitHub.
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

/// RF-20 — conteúdo 3-vias do arquivo em conflito.
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

/// Lado escolhido pelo usuário ao resolver um conflito.
#[derive(Debug, Clone, Copy, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum ConflictSideChoice {
    Ours,
    Theirs,
}
