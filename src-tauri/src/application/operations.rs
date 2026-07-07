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

/// RF-18 — descarta alterações não staged de um arquivo (`git restore`).
pub struct DiscardWorktree {
    pub path: String,
}

impl GitOperation for DiscardWorktree {
    fn command(&self) -> GitCommand {
        GitCommand {
            args: vec![
                "restore".into(),
                "--worktree".into(),
                "--".into(),
                self.path.clone(),
            ],
        }
    }
    fn description(&self) -> &'static str {
        "Descarta alterações não staged do arquivo (irreversível — não há commit)."
    }
}

pub struct DiscardWorktreeMany {
    pub paths: Vec<String>,
}

impl GitOperation for DiscardWorktreeMany {
    fn command(&self) -> GitCommand {
        let mut args = vec!["restore".into(), "--worktree".into(), "--".into()];
        args.extend(self.paths.iter().cloned());
        GitCommand { args }
    }
    fn description(&self) -> &'static str {
        "Descarta alterações não staged dos arquivos selecionados."
    }
}

pub struct DiscardWorktreeAll;

impl GitOperation for DiscardWorktreeAll {
    fn command(&self) -> GitCommand {
        GitCommand {
            args: vec![
                "restore".into(),
                "--worktree".into(),
                "--".into(),
                ".".into(),
            ],
        }
    }
    fn description(&self) -> &'static str {
        "Descarta todas as alterações não staged da working tree."
    }
}

/// RF-18 — remove arquivo não rastreado (`git clean`).
pub struct RemoveUntracked {
    pub path: String,
}

impl GitOperation for RemoveUntracked {
    fn command(&self) -> GitCommand {
        GitCommand {
            args: vec![
                "clean".into(),
                "-fd".into(),
                "--".into(),
                self.path.clone(),
            ],
        }
    }
    fn description(&self) -> &'static str {
        "Remove arquivo ou pasta não rastreada do disco (irreversível)."
    }
}

pub struct RemoveUntrackedMany {
    pub paths: Vec<String>,
}

impl GitOperation for RemoveUntrackedMany {
    fn command(&self) -> GitCommand {
        let mut args = vec!["clean".into(), "-fd".into(), "--".into()];
        args.extend(self.paths.iter().cloned());
        GitCommand { args }
    }
    fn description(&self) -> &'static str {
        "Remove arquivos/pastas não rastreados selecionados do disco."
    }
}

/// RF-18 — descarta hunk via `git apply --reverse`.
pub struct ApplyReversePatch {
    pub patch: String,
}

impl GitOperation for ApplyReversePatch {
    fn command(&self) -> GitCommand {
        GitCommand {
            args: vec!["apply".into(), "--reverse".into(), "-".into()],
        }
    }
    fn stdin_payload(&self) -> Option<Vec<u8>> {
        Some(self.patch.as_bytes().to_vec())
    }
    fn description(&self) -> &'static str {
        "Descarta o trecho selecionado do arquivo (irreversível)."
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

/// RF-13 — aplica commit em outra branch no topo da branch atual.
pub struct CherryPickCommit {
    pub sha: String,
}

impl GitOperation for CherryPickCommit {
    fn command(&self) -> GitCommand {
        GitCommand {
            args: vec![
                "cherry-pick".into(),
                "--no-edit".into(),
                self.sha.clone(),
            ],
        }
    }
    fn description(&self) -> &'static str {
        "Aplica as alterações do commit selecionado no topo da branch atual."
    }
}

pub struct AbortRevert;

impl GitOperation for AbortRevert {
    fn command(&self) -> GitCommand {
        GitCommand {
            args: vec!["revert".into(), "--abort".into()],
        }
    }
    fn description(&self) -> &'static str {
        "Cancela o revert em andamento e restaura o estado anterior."
    }
}

pub struct ContinueRevert;

impl GitOperation for ContinueRevert {
    fn command(&self) -> GitCommand {
        GitCommand {
            args: vec![
                "revert".into(),
                "--continue".into(),
                "--no-edit".into(),
            ],
        }
    }
    fn description(&self) -> &'static str {
        "Finaliza o revert em andamento (cria o commit reverso após resolver conflitos)."
    }
}

pub struct AbortMerge;

impl GitOperation for AbortMerge {
    fn command(&self) -> GitCommand {
        GitCommand {
            args: vec!["merge".into(), "--abort".into()],
        }
    }
    fn description(&self) -> &'static str {
        "Cancela o merge em andamento e restaura o estado anterior."
    }
}

pub struct ContinueMerge;

impl GitOperation for ContinueMerge {
    fn command(&self) -> GitCommand {
        GitCommand {
            args: vec!["merge".into(), "--continue".into(), "--no-edit".into()],
        }
    }
    fn description(&self) -> &'static str {
        "Finaliza o merge em andamento após resolver conflitos."
    }
}

pub struct AbortCherryPick;

impl GitOperation for AbortCherryPick {
    fn command(&self) -> GitCommand {
        GitCommand {
            args: vec!["cherry-pick".into(), "--abort".into()],
        }
    }
    fn description(&self) -> &'static str {
        "Cancela o cherry-pick em andamento e restaura o estado anterior."
    }
}

pub struct ContinueCherryPick;

impl GitOperation for ContinueCherryPick {
    fn command(&self) -> GitCommand {
        GitCommand {
            args: vec![
                "cherry-pick".into(),
                "--continue".into(),
                "--no-edit".into(),
            ],
        }
    }
    fn description(&self) -> &'static str {
        "Finaliza o cherry-pick em andamento após resolver conflitos."
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

pub struct SwitchBranch {
    pub branch: String,
    pub track_remote: Option<String>,
}

impl SwitchBranch {
    pub fn effect_description(&self) -> String {
        match &self.track_remote {
            Some(remote) => format!(
                "Cria a branch local «{}» a partir de {}/{} e configura o upstream.",
                self.branch, remote, self.branch
            ),
            None => "Troca para outra branch local.".into(),
        }
    }

    /// Comandos Git — `switch --track` falha quando a ref remota não é
    /// reconhecida como branch (ex.: após fetch com refspec explícito).
    pub fn all_commands(&self) -> Vec<GitCommand> {
        match &self.track_remote {
            Some(remote) => {
                let remote_ref = format!("{remote}/{}", self.branch);
                let branch_cfg = format!("branch.{}", self.branch);
                vec![
                    GitCommand {
                        args: vec![
                            "switch".into(),
                            "-c".into(),
                            self.branch.clone(),
                            remote_ref,
                        ],
                    },
                    GitCommand {
                        args: vec![
                            "config".into(),
                            format!("{branch_cfg}.remote"),
                            remote.clone(),
                        ],
                    },
                    GitCommand {
                        args: vec![
                            "config".into(),
                            format!("{branch_cfg}.merge"),
                            format!("refs/heads/{}", self.branch),
                        ],
                    },
                ]
            }
            None => vec![GitCommand {
                args: vec!["switch".into(), self.branch.clone()],
            }],
        }
    }
}

impl GitOperation for SwitchBranch {
    fn command(&self) -> GitCommand {
        self.all_commands().into_iter().next().expect("switch tem comando")
    }
    fn description(&self) -> &'static str {
        if self.track_remote.is_some() {
            "Cria branch local rastreando o remoto e troca para ela."
        } else {
            "Troca para outra branch local."
        }
    }
}

pub struct StashPush {
    pub message: Option<String>,
    pub include_untracked: bool,
}

impl GitOperation for StashPush {
    fn command(&self) -> GitCommand {
        let mut args = vec!["stash".into(), "push".into()];
        if self.include_untracked {
            args.push("-u".into());
        }
        if let Some(msg) = self.message.as_ref().map(|m| m.trim()).filter(|m| !m.is_empty()) {
            args.push("-m".into());
            args.push(msg.to_string());
        }
        GitCommand { args }
    }
    fn description(&self) -> &'static str {
        "Guarda as alterações da working tree em uma pilha (stash) temporária."
    }
}

pub struct StashApply {
    pub reference: String,
}

impl GitOperation for StashApply {
    fn command(&self) -> GitCommand {
        GitCommand {
            args: vec![
                "stash".into(),
                "apply".into(),
                self.reference.clone(),
            ],
        }
    }
    fn description(&self) -> &'static str {
        "Reaplica o stash na working tree (mantém na pilha)."
    }
}

pub struct StashPop {
    pub reference: String,
}

impl GitOperation for StashPop {
    fn command(&self) -> GitCommand {
        GitCommand {
            args: vec!["stash".into(), "pop".into(), self.reference.clone()],
        }
    }
    fn description(&self) -> &'static str {
        "Reaplica o stash e remove da pilha."
    }
}

pub struct StashDrop {
    pub reference: String,
}

impl GitOperation for StashDrop {
    fn command(&self) -> GitCommand {
        GitCommand {
            args: vec!["stash".into(), "drop".into(), self.reference.clone()],
        }
    }
    fn description(&self) -> &'static str {
        "Remove o stash da pilha sem reaplicar."
    }
}

/// RF-24 — cria tag em commit (`git tag` / `git tag -a`).
pub struct CreateTag {
    pub name: String,
    pub commit_id: String,
    pub annotated: bool,
    pub message: Option<String>,
}

impl GitOperation for CreateTag {
    fn command(&self) -> GitCommand {
        if self.annotated {
            GitCommand {
                args: vec![
                    "tag".into(),
                    "-a".into(),
                    self.name.clone(),
                    self.commit_id.clone(),
                    "-F".into(),
                    "-".into(),
                ],
            }
        } else {
            GitCommand {
                args: vec![
                    "tag".into(),
                    self.name.clone(),
                    self.commit_id.clone(),
                ],
            }
        }
    }
    fn stdin_payload(&self) -> Option<Vec<u8>> {
        if !self.annotated {
            return None;
        }
        let msg = self
            .message
            .as_ref()
            .map(|m| m.trim())
            .filter(|m| !m.is_empty())?;
        Some(format!("{msg}\n").into_bytes())
    }
    fn description(&self) -> &'static str {
        "Cria uma tag apontando para o commit selecionado."
    }
}

/// RF-24 — envia tag ao remoto (`git push origin <tag>`).
pub struct PushTag {
    pub remote: String,
    pub name: String,
}

impl GitOperation for PushTag {
    fn command(&self) -> GitCommand {
        GitCommand {
            args: vec![
                "push".into(),
                self.remote.clone(),
                self.name.clone(),
            ],
        }
    }
    fn description(&self) -> &'static str {
        "Envia a tag ao remoto configurado."
    }
}

/// RF-24 — remove tag local (`git tag -d`).
pub struct DeleteTag {
    pub name: String,
}

impl GitOperation for DeleteTag {
    fn command(&self) -> GitCommand {
        GitCommand {
            args: vec!["tag".into(), "-d".into(), self.name.clone()],
        }
    }
    fn description(&self) -> &'static str {
        "Remove a tag do repositório local (não remove do remoto)."
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

/// RF-09 / RF-16 recorte 2 — reescreve histórico no remoto com proteção de lease.
pub struct PushForceWithLease;

impl GitOperation for PushForceWithLease {
    fn command(&self) -> GitCommand {
        GitCommand {
            args: vec!["push".into(), "--force-with-lease".into()],
        }
    }
    fn description(&self) -> &'static str {
        "Envia o histórico reescrito ao remoto (push forçado com lease)."
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

/// Completa clone raso — baixa todo o histórico do remoto (RF-22).
pub struct UnshallowRemote;

impl GitOperation for UnshallowRemote {
    fn command(&self) -> GitCommand {
        GitCommand {
            args: vec![
                "fetch".into(),
                "--unshallow".into(),
                "--prune".into(),
            ],
        }
    }
    fn description(&self) -> &'static str {
        "Baixa o histórico completo do remoto (completa clone raso)."
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cherry_pick_usa_no_edit() {
        let op = CherryPickCommit {
            sha: "abc123".into(),
        };
        assert_eq!(
            op.command().args,
            vec!["cherry-pick", "--no-edit", "abc123"]
        );
    }

    #[test]
    fn continue_revert_usa_continue_no_edit() {
        let op = ContinueRevert;
        assert_eq!(
            op.command().args,
            vec!["revert", "--continue", "--no-edit"]
        );
    }

    #[test]
    fn push_force_with_lease_preview_estavel() {
        let op = PushForceWithLease;
        assert_eq!(op.command().args, vec!["push", "--force-with-lease"]);
    }

    #[test]
    fn unshallow_preview_estavel() {
        let op = UnshallowRemote;
        assert_eq!(
            op.command().args,
            vec!["fetch", "--unshallow", "--prune"]
        );
    }

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
    fn stash_drop_usa_drop() {
        let op = StashDrop {
            reference: "stash@{0}".into(),
        };
        assert_eq!(
            op.command().args,
            vec!["stash", "drop", "stash@{0}"]
        );
    }

    #[test]
    fn create_tag_anotada_usa_a_e_f() {
        let op = CreateTag {
            name: "v1".into(),
            commit_id: "abcdef0".into(),
            annotated: true,
            message: Some("release".into()),
        };
        assert_eq!(
            op.command().args,
            vec!["tag", "-a", "v1", "abcdef0", "-F", "-"]
        );
        assert!(op.stdin_payload().is_some());
    }

    #[test]
    fn create_tag_leve_sem_mensagem() {
        let op = CreateTag {
            name: "v1".into(),
            commit_id: "abcdef0".into(),
            annotated: false,
            message: None,
        };
        assert_eq!(op.command().args, vec!["tag", "v1", "abcdef0"]);
        assert!(op.stdin_payload().is_none());
    }

    #[test]
    fn push_tag_envia_para_remoto() {
        let op = PushTag {
            remote: "origin".into(),
            name: "v1".into(),
        };
        assert_eq!(op.command().args, vec!["push", "origin", "v1"]);
    }

    #[test]
    fn delete_tag_usa_d() {
        let op = DeleteTag {
            name: "v1".into(),
        };
        assert_eq!(op.command().args, vec!["tag", "-d", "v1"]);
    }

    #[test]
    fn discard_worktree_usa_restore_worktree() {
        let op = DiscardWorktree {
            path: "src/a.ts".into(),
        };
        assert_eq!(
            op.command().args,
            vec!["restore", "--worktree", "--", "src/a.ts"]
        );
    }

    #[test]
    fn remove_untracked_usa_clean_fd() {
        let op = RemoveUntracked {
            path: "tmp.txt".into(),
        };
        assert_eq!(
            op.command().args,
            vec!["clean", "-fd", "--", "tmp.txt"]
        );
    }

    #[test]
    fn push_set_upstream_inclui_u() {
        let op = PushSetUpstream {
            remote: "origin".into(),
            branch: "master".into(),
        };
        assert_eq!(op.command().args, vec!["push", "-u", "origin", "master"]);
    }

    #[test]
    fn switch_branch_usa_git_switch() {
        let op = SwitchBranch {
            branch: "feature".into(),
            track_remote: None,
        };
        assert_eq!(op.command().args, vec!["switch", "feature"]);
    }

    #[test]
    fn switch_branch_remota_usa_switch_c_e_config_upstream() {
        let op = SwitchBranch {
            branch: "feature".into(),
            track_remote: Some("origin".into()),
        };
        let cmds = op.all_commands();
        assert_eq!(
            cmds[0].args,
            vec!["switch", "-c", "feature", "origin/feature"]
        );
        assert_eq!(
            cmds[1].args,
            vec!["config", "branch.feature.remote", "origin"]
        );
        assert_eq!(
            cmds[2].args,
            vec!["config", "branch.feature.merge", "refs/heads/feature"]
        );
    }
}
