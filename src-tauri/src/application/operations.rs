//! Operações Git encapsuladas (Command pattern — PLANO §9, RF-08).
//!
//! Cada operação define o `GitCommand` executado; preview e run usam o mesmo objeto.

use crate::application::GitCommand;

/// Operação Git de repositório — `preview()` e `run()` derivam de `command()`.
pub trait GitOperation: Send + Sync {
    fn command(&self) -> GitCommand;
    /// Entrada stdin para comandos como `git commit -F -`.
    fn stdin_payload(&self) -> Option<Vec<u8>> {
        None
    }
    /// Efeito em linguagem natural (RF-08).
    fn description(&self) -> &'static str {
        "Executa operação Git no repositório aberto."
    }
}

pub struct FetchRemote;

impl GitOperation for FetchRemote {
    fn command(&self) -> GitCommand {
        GitCommand {
            args: vec!["fetch".into(), "--prune".into()],
        }
    }
}

pub struct StatusPorcelain;

impl GitOperation for StatusPorcelain {
    fn command(&self) -> GitCommand {
        GitCommand {
            args: vec![
                "status".into(),
                "--porcelain=v2".into(),
                "--branch".into(),
                "-z".into(),
            ],
        }
    }
}

pub struct FileDiff {
    pub path: String,
    pub staged: bool,
}

impl GitOperation for FileDiff {
    fn command(&self) -> GitCommand {
        let mut args = vec!["diff".into(), "--no-color".into()];
        if self.staged {
            args.push("--cached".into());
        }
        args.push("--".into());
        args.push(self.path.clone());
        GitCommand { args }
    }
}

pub struct ShowCommit {
    pub sha: String,
}

impl GitOperation for ShowCommit {
    fn command(&self) -> GitCommand {
        GitCommand {
            args: vec![
                "show".into(),
                "--no-color".into(),
                "--format=".into(),
                self.sha.clone(),
            ],
        }
    }
}

/// Diff de um único arquivo dentro de um commit (detalhes de commit, M1).
/// `--first-parent` garante um diff coerente em merges (contra o 1º pai),
/// alinhado com o cálculo de `list_commit_files`.
pub struct CommitFileDiff {
    pub sha: String,
    pub path: String,
}

impl GitOperation for CommitFileDiff {
    fn command(&self) -> GitCommand {
        GitCommand {
            args: vec![
                "show".into(),
                "--no-color".into(),
                "--format=".into(),
                "--first-parent".into(),
                self.sha.clone(),
                "--".into(),
                self.path.clone(),
            ],
        }
    }
}

pub struct RevListAheadBehind {
    pub upstream: String,
}

impl GitOperation for RevListAheadBehind {
    fn command(&self) -> GitCommand {
        GitCommand {
            args: vec![
                "rev-list".into(),
                "--left-right".into(),
                "--count".into(),
                format!("HEAD...{}", self.upstream),
            ],
        }
    }
}

// --- M3: operações de escrita não destrutivas ---

pub struct Stage {
    pub path: String,
}

impl GitOperation for Stage {
    fn command(&self) -> GitCommand {
        GitCommand {
            args: vec!["add".into(), "--".into(), self.path.clone()],
        }
    }
    fn description(&self) -> &'static str {
        "Adiciona o arquivo à staging area para o próximo commit."
    }
}

pub struct StageMany {
    pub paths: Vec<String>,
}

impl GitOperation for StageMany {
    fn command(&self) -> GitCommand {
        let mut args = vec!["add".into(), "--".into()];
        args.extend(self.paths.iter().cloned());
        GitCommand { args }
    }
    fn description(&self) -> &'static str {
        "Adiciona os arquivos selecionados à staging area."
    }
}

pub struct StageAll;

impl GitOperation for StageAll {
    fn command(&self) -> GitCommand {
        GitCommand {
            args: vec!["add".into(), "-A".into()],
        }
    }
    fn description(&self) -> &'static str {
        "Adiciona todas as alterações (modified, deleted, untracked) à staging area."
    }
}

pub struct Unstage {
    pub path: String,
}

impl GitOperation for Unstage {
    fn command(&self) -> GitCommand {
        GitCommand {
            args: vec![
                "restore".into(),
                "--staged".into(),
                "--".into(),
                self.path.clone(),
            ],
        }
    }
    fn description(&self) -> &'static str {
        "Remove o arquivo da staging area; o conteúdo permanece na working tree."
    }
}

pub struct UnstageMany {
    pub paths: Vec<String>,
}

impl GitOperation for UnstageMany {
    fn command(&self) -> GitCommand {
        let mut args = vec!["restore".into(), "--staged".into(), "--".into()];
        args.extend(self.paths.iter().cloned());
        GitCommand { args }
    }
    fn description(&self) -> &'static str {
        "Remove os arquivos selecionados da staging area."
    }
}

pub struct UnstageAll;

impl GitOperation for UnstageAll {
    fn command(&self) -> GitCommand {
        GitCommand {
            args: vec!["restore".into(), "--staged".into(), "--".into(), ".".into()],
        }
    }
    fn description(&self) -> &'static str {
        "Remove todos os arquivos da staging area; nada é descartado da working tree."
    }
}

pub struct CreateCommit {
    pub summary: String,
    pub body: Option<String>,
    pub amend: bool,
}

impl GitOperation for CreateCommit {
    fn command(&self) -> GitCommand {
        let mut args = vec!["commit".into(), "-F".into(), "-".into()];
        if self.amend {
            args.push("--amend".into());
        }
        GitCommand { args }
    }
    fn stdin_payload(&self) -> Option<Vec<u8>> {
        let mut msg = self.summary.trim().to_string();
        if let Some(body) = self.body.as_ref().filter(|b| !b.trim().is_empty()) {
            msg.push_str("\n\n");
            msg.push_str(body.trim());
        }
        msg.push('\n');
        Some(msg.into_bytes())
    }
    fn description(&self) -> &'static str {
        if self.amend {
            "Altera a mensagem do último commit (ainda não enviado)."
        } else {
            "Cria um novo commit com os arquivos em staging."
        }
    }
}

pub struct UncommitSoft;

impl GitOperation for UncommitSoft {
    fn command(&self) -> GitCommand {
        GitCommand {
            args: vec!["reset".into(), "--soft".into(), "HEAD~1".into()],
        }
    }
    fn description(&self) -> &'static str {
        "Move o último commit de volta para a staging area (nada é perdido)."
    }
}

pub struct RevertCommit {
    pub sha: String,
}

impl GitOperation for RevertCommit {
    fn command(&self) -> GitCommand {
        GitCommand {
            args: vec!["revert".into(), "--no-edit".into(), self.sha.clone()],
        }
    }
    fn description(&self) -> &'static str {
        "Cria um commit reverso que desfaz as alterações do commit selecionado."
    }
}

pub struct PushSetUpstream {
    pub remote: String,
    pub branch: String,
}

impl GitOperation for PushSetUpstream {
    fn command(&self) -> GitCommand {
        GitCommand {
            args: vec![
                "push".into(),
                "-u".into(),
                self.remote.clone(),
                self.branch.clone(),
            ],
        }
    }
    fn description(&self) -> &'static str {
        "Publica a branch no remoto e define o upstream."
    }
}

pub struct AddRemote {
    pub name: String,
    pub url: String,
}

impl GitOperation for AddRemote {
    fn command(&self) -> GitCommand {
        GitCommand {
            args: vec![
                "remote".into(),
                "add".into(),
                self.name.clone(),
                self.url.clone(),
            ],
        }
    }
    fn description(&self) -> &'static str {
        "Conecta o repositório local ao remoto."
    }
}

/// Corrige a URL de um remoto já configurado (ex.: publicação apontou para a
/// conta errada) — sem isso o usuário fica preso no terminal.
pub struct SetRemoteUrl {
    pub name: String,
    pub url: String,
}

impl GitOperation for SetRemoteUrl {
    fn command(&self) -> GitCommand {
        GitCommand {
            args: vec![
                "remote".into(),
                "set-url".into(),
                self.name.clone(),
                self.url.clone(),
            ],
        }
    }
    fn description(&self) -> &'static str {
        "Atualiza a URL do remoto."
    }
}

pub struct PushUpstream;

impl GitOperation for PushUpstream {
    fn command(&self) -> GitCommand {
        GitCommand {
            args: vec!["push".into()],
        }
    }
    fn description(&self) -> &'static str {
        "Envia commits locais ao remoto configurado (push)."
    }
}

pub struct PullFfOnly;

impl GitOperation for PullFfOnly {
    fn command(&self) -> GitCommand {
        GitCommand {
            args: vec!["pull".into(), "--ff-only".into()],
        }
    }
    fn description(&self) -> &'static str {
        "Atualiza a branch local com fast-forward apenas (sem merge automático)."
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fetch_preview_estavel() {
        let op = FetchRemote;
        assert_eq!(op.command().args, vec!["fetch", "--prune"]);
    }

    #[test]
    fn file_diff_staged_inclui_cached() {
        let op = FileDiff {
            path: "src/a.ts".into(),
            staged: true,
        };
        assert!(op.command().args.contains(&"--cached".to_string()));
    }

    #[test]
    fn stage_many_inclui_todos_os_paths() {
        let op = StageMany {
            paths: vec!["a.ts".into(), "b.ts".into()],
        };
        assert_eq!(op.command().args, vec!["add", "--", "a.ts", "b.ts"]);
    }

    #[test]
    fn stage_usa_git_add() {
        let op = Stage {
            path: "src/a.ts".into(),
        };
        assert_eq!(op.command().args, vec!["add", "--", "src/a.ts"]);
    }

    #[test]
    fn unstage_usa_restore_staged() {
        let op = Unstage {
            path: "src/a.ts".into(),
        };
        assert_eq!(
            op.command().args,
            vec!["restore", "--staged", "--", "src/a.ts"]
        );
    }

    #[test]
    fn commit_amend_inclui_flag() {
        let op = CreateCommit {
            summary: "fix".into(),
            body: None,
            amend: true,
        };
        assert!(op.command().args.contains(&"--amend".to_string()));
        assert!(op.stdin_payload().is_some());
    }

    #[test]
    fn push_set_upstream_inclui_u() {
        let op = PushSetUpstream {
            remote: "origin".into(),
            branch: "master".into(),
        };
        assert_eq!(op.command().args, vec!["push", "-u", "origin", "master"]);
    }
}
