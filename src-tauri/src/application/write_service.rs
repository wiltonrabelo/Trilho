//! Casos de uso de escrita M3 — preview (RF-08) e execução com gates.

use crate::application::operations::{
    AddRemote, CreateCommit, GitOperation, PullFfOnly, PushSetUpstream, PushUpstream, RevertCommit,
    SetRemoteUrl, Stage, StageAll, StageMany, SwitchBranch, UncommitSoft, UnshallowRemote, Unstage,
    UnstageAll, UnstageMany,
};
use crate::application::write_gates::head_is_local_only;
use crate::application::{GitError, GitWriter, RepoContext};
use crate::domain::{OperationPreview, WriteRequest};
use crate::infrastructure::{
    list_local_branches, list_remote_branches, repo_info, validate_clone_branch,
    validate_git_object_id, validate_remote_name, validate_remote_url, validate_repo_relative_path,
};
use git2::Repository;

/// Extrai o path Git de um rótulo de rename (`old → new`).
fn git_path_from_display(display: &str) -> &str {
    display
        .split_once(" → ")
        .map(|(_, new)| new)
        .unwrap_or(display)
}

fn validate_paths(paths: &[String]) -> Result<Vec<String>, GitError> {
    if paths.is_empty() {
        return Err(GitError::Git("Nenhum arquivo selecionado.".into()));
    }
    paths
        .iter()
        .map(|p| {
            validate_repo_relative_path(git_path_from_display(p))
                .map_err(|e| GitError::Git(e.to_string()))
        })
        .collect()
}

pub fn preview_write(
    ctx: &RepoContext,
    repo_path: &str,
    req: &WriteRequest,
) -> Result<OperationPreview, GitError> {
    let (commands, description, blocked) = match req {
        WriteRequest::Stage { path } => {
            let path = validate_repo_relative_path(git_path_from_display(path))
                .map_err(|e| GitError::Git(e.to_string()))?;
            let op = Stage { path };
            (ctx.preview_op(&op), op.description().to_string(), None)
        }
        WriteRequest::StageMany { paths } => {
            let paths = match validate_paths(paths) {
                Ok(p) => p,
                Err(GitError::Git(msg)) if msg.contains("Nenhum") => {
                    return Ok(blocked_preview(repo_path, &msg));
                }
                Err(e) => return Err(e),
            };
            let count = paths.len();
            let op = StageMany { paths };
            (
                ctx.preview_op(&op),
                format!("{} ({count} arquivo(s)).", op.description()),
                None,
            )
        }
        WriteRequest::StageAll => {
            let op = StageAll;
            (ctx.preview_op(&op), op.description().to_string(), None)
        }
        WriteRequest::Unstage { path } => {
            let path = validate_repo_relative_path(git_path_from_display(path))
                .map_err(|e| GitError::Git(e.to_string()))?;
            let op = Unstage { path };
            (ctx.preview_op(&op), op.description().to_string(), None)
        }
        WriteRequest::UnstageMany { paths } => {
            let paths = match validate_paths(paths) {
                Ok(p) => p,
                Err(GitError::Git(msg)) if msg.contains("Nenhum") => {
                    return Ok(blocked_preview(repo_path, &msg));
                }
                Err(e) => return Err(e),
            };
            let count = paths.len();
            let op = UnstageMany { paths };
            (
                ctx.preview_op(&op),
                format!("{} ({count} arquivo(s)).", op.description()),
                None,
            )
        }
        WriteRequest::UnstageAll => {
            let op = UnstageAll;
            (ctx.preview_op(&op), op.description().to_string(), None)
        }
        WriteRequest::Commit {
            summary,
            body,
            amend,
        } => {
            if summary.trim().is_empty() {
                return Ok(blocked_preview(
                    repo_path,
                    "A mensagem do commit (resumo) não pode ficar vazia.",
                ));
            }
            let op = CreateCommit {
                summary: summary.clone(),
                body: body.clone(),
                amend: *amend,
            };
            let blocked = if *amend { gate_amend(ctx)? } else { None };
            if !*amend {
                let staged_count = ctx.reader().get_status()?.staged.len();
                if staged_count == 0 {
                    return Ok(blocked_preview(
                        repo_path,
                        "Nenhum arquivo em stage — adicione alterações antes de commitar.",
                    ));
                }
            }
            // Honestidade do RF-08: --amend também absorve o que estiver em
            // staging — a descrição não pode falar só em "mensagem".
            let description = if *amend {
                let staged_count = ctx.reader().get_status()?.staged.len();
                if staged_count > 0 {
                    format!(
                        "Altera o último commit (ainda não enviado) e INCLUI o(s) \
                         {staged_count} arquivo(s) em staging nele."
                    )
                } else {
                    op.description().to_string()
                }
            } else {
                op.description().to_string()
            };
            (ctx.preview_op(&op), description, blocked)
        }
        WriteRequest::Uncommit => {
            let op = UncommitSoft;
            (
                ctx.preview_op(&op),
                op.description().to_string(),
                gate_uncommit(ctx)?,
            )
        }
        WriteRequest::Revert { commit_id } => {
            let sha =
                validate_git_object_id(commit_id).map_err(|e| GitError::Git(e.to_string()))?;
            let blocked = gate_revert_merge(ctx, &sha)?;
            let op = RevertCommit { sha };
            (ctx.preview_op(&op), op.description().to_string(), blocked)
        }
        WriteRequest::Push => {
            let op = PushUpstream;
            (
                ctx.preview_op(&op),
                op.description().to_string(),
                gate_push(ctx)?,
            )
        }
        WriteRequest::PullFfOnly => {
            let op = PullFfOnly;
            let blocked = gate_pull(ctx)?;
            (ctx.preview_op(&op), op.description().to_string(), blocked)
        }
        WriteRequest::UnshallowHistory => {
            let op = UnshallowRemote;
            let blocked = gate_unshallow(ctx)?;
            (ctx.preview_op(&op), op.description().to_string(), blocked)
        }
        WriteRequest::SwitchBranch {
            branch,
            track_remote,
        } => {
            let branch = validate_clone_branch(Some(branch))?
                .ok_or_else(|| GitError::Git("Nome de branch inválido.".into()))?;
            let track_remote = resolve_switch_track(repo_path, &branch, track_remote.as_deref())?;
            let op = SwitchBranch {
                branch: branch.clone(),
                track_remote: track_remote.clone(),
            };
            let blocked = gate_switch_branch(ctx, repo_path, &branch, track_remote.as_deref())?;
            let commands = op
                .all_commands()
                .iter()
                .flat_map(|c| GitWriter::preview(ctx.writer(), c))
                .collect();
            (commands, op.effect_description(), blocked)
        }
        WriteRequest::Publish { url } => preview_publish(ctx, url.as_deref())?,
    };

    Ok(OperationPreview {
        commands,
        description,
        repo_path: repo_path.to_string(),
        blocked,
    })
}

pub fn execute_write(ctx: &RepoContext, req: WriteRequest) -> Result<(), GitError> {
    let preview = preview_write(ctx, ctx.repo_path(), &req)?;
    if let Some(msg) = preview.blocked {
        return Err(GitError::Git(msg));
    }

    match req {
        WriteRequest::Stage { path } => {
            let path = validate_repo_relative_path(git_path_from_display(&path))
                .map_err(|e| GitError::Git(e.to_string()))?;
            ctx.execute_op(&Stage { path })?;
        }
        WriteRequest::StageMany { paths } => {
            let paths = validate_paths(&paths)?;
            ctx.execute_op(&StageMany { paths })?;
        }
        WriteRequest::StageAll => {
            ctx.execute_op(&StageAll)?;
        }
        WriteRequest::Unstage { path } => {
            let path = validate_repo_relative_path(git_path_from_display(&path))
                .map_err(|e| GitError::Git(e.to_string()))?;
            ctx.execute_op(&Unstage { path })?;
        }
        WriteRequest::UnstageMany { paths } => {
            let paths = validate_paths(&paths)?;
            ctx.execute_op(&UnstageMany { paths })?;
        }
        WriteRequest::UnstageAll => {
            ctx.execute_op(&UnstageAll)?;
        }
        WriteRequest::Commit {
            summary,
            body,
            amend,
        } => {
            ctx.execute_op(&CreateCommit {
                summary,
                body,
                amend,
            })?;
        }
        WriteRequest::Uncommit => {
            ctx.execute_op(&UncommitSoft)?;
        }
        WriteRequest::Revert { commit_id } => {
            let sha =
                validate_git_object_id(&commit_id).map_err(|e| GitError::Git(e.to_string()))?;
            ctx.execute_op(&RevertCommit { sha })?;
        }
        WriteRequest::Push => {
            ctx.execute_op(&PushUpstream)?;
        }
        WriteRequest::PullFfOnly => {
            ctx.execute_op(&PullFfOnly)?;
        }
        WriteRequest::UnshallowHistory => {
            ctx.execute_op(&UnshallowRemote)?;
        }
        WriteRequest::SwitchBranch {
            branch,
            track_remote,
        } => {
            let branch = validate_clone_branch(Some(&branch))?
                .ok_or_else(|| GitError::Git("Nome de branch inválido.".into()))?;
            let track_remote = resolve_switch_track(ctx.repo_path(), &branch, track_remote.as_deref())?;
            let op = SwitchBranch {
                branch,
                track_remote,
            };
            for cmd in op.all_commands() {
                GitWriter::run(ctx.writer(), &cmd)?;
            }
        }
        WriteRequest::Publish { url } => execute_publish(ctx, url.as_deref())?,
    }
    Ok(())
}

fn blocked_preview(repo_path: &str, msg: &str) -> OperationPreview {
    OperationPreview {
        commands: vec![],
        description: String::new(),
        repo_path: repo_path.to_string(),
        blocked: Some(msg.to_string()),
    }
}

struct PublishPlan {
    /// Passo de remoto: `remote add` (1ª publicação) ou `remote set-url`
    /// (corrigir URL errada) — ambos são `GitOperation`.
    remote_step: Option<Box<dyn GitOperation>>,
    push: PushSetUpstream,
    description: String,
}

fn resolve_primary_remote(ctx: &RepoContext) -> Result<String, GitError> {
    let repo = Repository::discover(ctx.repo_path()).map_err(|e| GitError::Io(e.to_string()))?;
    let remotes = repo.remotes().map_err(|e| GitError::Io(e.to_string()))?;
    for i in 0..remotes.len() {
        if remotes.get(i) == Some("origin") {
            return Ok("origin".into());
        }
    }
    remotes
        .get(0)
        .map(|s| s.to_string())
        .ok_or_else(|| GitError::Git("Nenhum remoto configurado.".into()))
}

fn plan_publish(ctx: &RepoContext, remote_url: Option<&str>) -> Result<PublishPlan, GitError> {
    let info = repo_info(ctx.repo_path())?;
    if info.is_detached {
        return Err(GitError::Git(
            "Repositório em detached HEAD — troque para uma branch antes de publicar.".into(),
        ));
    }
    if info.upstream.is_some() {
        return Err(GitError::Git(
            "Esta branch já está publicada. Use Push para enviar novos commits.".into(),
        ));
    }
    let branch = info
        .branch
        .ok_or_else(|| GitError::Git("Sem branch ativa para publicar.".into()))?;

    let (remote_step, remote_name, description): (Option<Box<dyn GitOperation>>, String, String) =
        if info.has_remote {
            let name = resolve_primary_remote(ctx)?;
            // URL informada e diferente da atual → corrige o remoto antes do
            // push (ex.: 1ª publicação apontou para a conta errada).
            match remote_url {
                Some(url) => {
                    let url = validate_remote_url(url)?;
                    if info.remote_url.as_deref() == Some(url.as_str()) {
                        (
                            None,
                            name,
                            format!("Publica a branch {branch} no remoto e configura o upstream."),
                        )
                    } else {
                        let step: Box<dyn GitOperation> = Box::new(SetRemoteUrl {
                            name: name.clone(),
                            url,
                        });
                        (
                            Some(step),
                            name,
                            format!(
                                "Atualiza a URL do remoto e publica a branch {branch}, \
                                 configurando o upstream."
                            ),
                        )
                    }
                }
                None => (
                    None,
                    name,
                    format!("Publica a branch {branch} no remoto e configura o upstream."),
                ),
            }
        } else {
            let url = match remote_url {
                Some(url) => validate_remote_url(url)?,
                None => {
                    return Err(GitError::Git(
                        "Informe a URL do repositório remoto para publicar.".into(),
                    ));
                }
            };
            let step: Box<dyn GitOperation> = Box::new(AddRemote {
                name: "origin".into(),
                url,
            });
            (
                Some(step),
                "origin".to_string(),
                "Conecta ao remoto e publica a branch pela primeira vez.".to_string(),
            )
        };

    Ok(PublishPlan {
        remote_step,
        push: PushSetUpstream {
            remote: remote_name,
            branch,
        },
        description,
    })
}

fn preview_publish(
    ctx: &RepoContext,
    remote_url: Option<&str>,
) -> Result<(Vec<String>, String, Option<String>), GitError> {
    match plan_publish(ctx, remote_url) {
        Ok(plan) => {
            let mut commands = Vec::new();
            if let Some(ref op) = plan.remote_step {
                commands.extend(ctx.preview_op(op.as_ref()));
            }
            commands.extend(ctx.preview_op(&plan.push));
            Ok((commands, plan.description, None))
        }
        Err(GitError::Git(msg))
            if msg.contains("Informe a URL") || msg.contains("já está publicada") =>
        {
            Ok((vec![], String::new(), Some(msg)))
        }
        Err(e) => Err(e),
    }
}

fn execute_publish(ctx: &RepoContext, remote_url: Option<&str>) -> Result<(), GitError> {
    let plan = plan_publish(ctx, remote_url)?;
    if let Some(op) = plan.remote_step {
        ctx.execute_op(op.as_ref())?;
    }
    ctx.execute_op(&plan.push)?;
    Ok(())
}

fn gate_amend(ctx: &RepoContext) -> Result<Option<String>, GitError> {
    if head_is_local_only(ctx.reader(), ctx.writer())? {
        Ok(None)
    } else {
        Ok(Some(
            "O último commit já foi enviado — amend não está disponível no MVP.".into(),
        ))
    }
}

/// Revert de commit de MERGE exige `-m <pai>` (fora do MVP): sem gate, o
/// `git revert` falharia com erro críptico depois da confirmação.
fn gate_revert_merge(ctx: &RepoContext, sha: &str) -> Result<Option<String>, GitError> {
    let repo = Repository::discover(ctx.repo_path()).map_err(|e| GitError::Io(e.to_string()))?;
    let oid = git2::Oid::from_str(sha).map_err(|e| GitError::Git(e.to_string()))?;
    let commit = repo
        .find_commit(oid)
        .map_err(|_| GitError::Git("Commit não encontrado no repositório.".into()))?;
    if commit.parent_count() > 1 {
        return Ok(Some(
            "Este é um commit de MERGE — revertê-lo exige escolher qual lado manter \
             (git revert -m), operação avançada fora do MVP. Reverta os commits \
             individuais da branch mesclada, se necessário."
                .into(),
        ));
    }
    Ok(None)
}

fn gate_uncommit(ctx: &RepoContext) -> Result<Option<String>, GitError> {
    if head_is_local_only(ctx.reader(), ctx.writer())? {
        Ok(None)
    } else {
        Ok(Some(
            "O último commit já foi enviado — uncommit só vale para commits locais.".into(),
        ))
    }
}

fn gate_push(ctx: &RepoContext) -> Result<Option<String>, GitError> {
    let sync = ctx.reader().get_sync_info()?;
    if sync.upstream.is_none() {
        return Ok(Some(
            "Branch sem upstream — use «Publicar» no Trilho para enviar esta branch.".into(),
        ));
    }
    if sync.ahead == 0 {
        return Ok(Some("Não há commits locais para enviar.".into()));
    }
    if sync.behind > 0 {
        return Ok(Some(
            "O remoto está à frente. Atualize com «pull --ff-only» antes de enviar.".into(),
        ));
    }
    Ok(None)
}

fn gate_pull(ctx: &RepoContext) -> Result<Option<String>, GitError> {
    let sync = ctx.reader().get_sync_info()?;
    if sync.upstream.is_none() {
        return Ok(Some(
            "Branch sem upstream — use «Publicar» no Trilho antes de puxar.".into(),
        ));
    }
    if sync.behind == 0 {
        return Ok(Some(
            "Já está em dia com o remoto (nada para puxar).".into(),
        ));
    }
    Ok(None)
}

fn gate_unshallow(ctx: &RepoContext) -> Result<Option<String>, GitError> {
    let info = repo_info(ctx.repo_path())?;
    if !info.has_remote {
        return Ok(Some(
            "Sem remoto configurado — não é possível completar o histórico.".into(),
        ));
    }
    if !info.is_shallow {
        return Ok(Some("O histórico local já está completo.".into()));
    }
    Ok(None)
}

fn gate_switch_branch(
    ctx: &RepoContext,
    repo_path: &str,
    branch: &str,
    track_remote: Option<&str>,
) -> Result<Option<String>, GitError> {
    let origin = ctx.reader().get_branch_origin()?;
    if origin.current_branch.as_deref() == Some(branch) {
        return Ok(Some("Você já está nesta branch.".into()));
    }

    let status = ctx.reader().get_status()?;
    if !status.staged.is_empty() || !status.unstaged.is_empty() || !status.untracked.is_empty() {
        return Ok(Some(
            "Working tree com alterações — faça commit, stash ou descarte antes de trocar de branch."
                .into(),
        ));
    }

    match track_remote {
        Some(remote) => {
            validate_remote_name(remote)?;
            let remotes = list_remote_branches(repo_path)?;
            if !remotes
                .iter()
                .any(|r| r.remote == remote && r.branch == branch)
            {
                return Ok(Some(
                    "Branch remota não encontrada — atualize com «Buscar» (fetch) e tente de novo."
                        .into(),
                ));
            }
            Ok(None)
        }
        None => {
            let locals = list_local_branches(repo_path)?;
            if !locals.iter().any(|b| b == branch) {
                return Ok(Some("Branch local não encontrada.".into()));
            }
            Ok(None)
        }
    }
}

/// Se a branch local já existe, troca localmente em vez de `--track`.
fn resolve_switch_track(
    repo_path: &str,
    branch: &str,
    track_remote: Option<&str>,
) -> Result<Option<String>, GitError> {
    let Some(remote) = track_remote else {
        return Ok(None);
    };
    let remote = validate_remote_name(remote)?;
    let locals = list_local_branches(repo_path)?;
    if locals.iter().any(|b| b == branch) {
        Ok(None)
    } else {
        Ok(Some(remote))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn git_path_from_rename_display() {
        assert_eq!(git_path_from_display("a.ts → b.ts"), "b.ts");
        assert_eq!(git_path_from_display("plain.ts"), "plain.ts");
    }

    fn init_repo_with_commit(dir: &std::path::Path) {
        std::fs::create_dir_all(dir).unwrap();
        for args in [
            vec!["init"],
            vec!["config", "user.email", "t@t.com"],
            vec!["config", "user.name", "T"],
        ] {
            std::process::Command::new("git")
                .args(&args)
                .current_dir(dir)
                .output()
                .unwrap();
        }
        std::fs::write(dir.join("f.txt"), "x").unwrap();
        std::process::Command::new("git")
            .args(["add", "f.txt"])
            .current_dir(dir)
            .output()
            .unwrap();
        std::process::Command::new("git")
            .args(["commit", "-m", "init"])
            .current_dir(dir)
            .output()
            .unwrap();
    }

    /// Regressão fix 3: revert de merge é bloqueado no preview (não explode
    /// depois da confirmação com o erro críptico do `git revert`).
    #[test]
    fn revert_de_merge_e_bloqueado() {
        let dir = std::env::temp_dir().join(format!("trilho-revmrg-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        init_repo_with_commit(&dir);
        for args in [
            vec!["checkout", "-b", "feat"],
            vec!["commit", "--allow-empty", "-m", "feat work"],
            vec!["checkout", "-"],
            vec!["commit", "--allow-empty", "-m", "avanca"],
            vec!["merge", "--no-ff", "feat", "-m", "merge feat"],
        ] {
            std::process::Command::new("git")
                .args(&args)
                .current_dir(&dir)
                .output()
                .unwrap();
        }
        let sha = std::process::Command::new("git")
            .args(["rev-parse", "HEAD"])
            .current_dir(&dir)
            .output()
            .unwrap();
        let sha = String::from_utf8_lossy(&sha.stdout).trim().to_string();

        let ctx = RepoContext::open(&dir.to_string_lossy()).expect("ctx");
        let preview = preview_write(
            &ctx,
            ctx.repo_path(),
            &WriteRequest::Revert { commit_id: sha },
        )
        .expect("preview");
        assert!(
            preview.blocked.is_some(),
            "revert de merge deve vir bloqueado"
        );
        assert!(preview.blocked.unwrap().to_lowercase().contains("merge"));
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// Regressão fix 2: amend com arquivos em staging avisa que os inclui.
    #[test]
    fn preview_do_amend_avisa_staging() {
        let dir = std::env::temp_dir().join(format!("trilho-amend-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        init_repo_with_commit(&dir);
        std::fs::write(dir.join("extra.txt"), "y").unwrap();
        std::process::Command::new("git")
            .args(["add", "extra.txt"])
            .current_dir(&dir)
            .output()
            .unwrap();

        let ctx = RepoContext::open(&dir.to_string_lossy()).expect("ctx");
        let preview = preview_write(
            &ctx,
            ctx.repo_path(),
            &WriteRequest::Commit {
                summary: "msg nova".into(),
                body: None,
                amend: true,
            },
        )
        .expect("preview");
        assert!(
            preview.description.contains("INCLUI"),
            "descrição deve avisar staging: {}",
            preview.description
        );
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn commit_sem_stage_bloqueado() {
        let dir = std::env::temp_dir().join(format!("trilho-commit-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        init_repo_with_commit(&dir);

        let ctx = RepoContext::open(&dir.to_string_lossy()).expect("ctx");
        let preview = preview_write(
            &ctx,
            ctx.repo_path(),
            &WriteRequest::Commit {
                summary: "vazio".into(),
                body: None,
                amend: false,
            },
        )
        .expect("preview");
        assert!(
            preview.blocked.is_some(),
            "commit sem stage deve bloquear preview"
        );
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn switch_branch_bloqueado_com_wt_suja() {
        let dir = std::env::temp_dir().join(format!("trilho-switch-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        init_repo_with_commit(&dir);
        std::process::Command::new("git")
            .args(["branch", "outra"])
            .current_dir(&dir)
            .output()
            .unwrap();
        std::fs::write(dir.join("dirty.txt"), "x").unwrap();

        let ctx = RepoContext::open(&dir.to_string_lossy()).expect("ctx");
        let preview = preview_write(
            &ctx,
            ctx.repo_path(),
            &WriteRequest::SwitchBranch {
                branch: "outra".into(),
                track_remote: None,
            },
        )
        .expect("preview");
        assert!(
            preview.blocked.is_some(),
            "switch com WT suja deve bloquear"
        );
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn switch_remoto_preview_usa_track() {
        let dir = std::env::temp_dir().join(format!("trilho-switch-remote-{}", std::process::id()));
        let work = dir.join("work");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&work).unwrap();

        let bare = dir.join("origin.git");
        std::process::Command::new("git")
            .args(["init", "--bare", "-b", "main"])
            .arg(&bare)
            .output()
            .unwrap();
        std::fs::write(work.join("f.txt"), "a").unwrap();
        std::process::Command::new("git")
            .args(["init", "-b", "main"])
            .current_dir(&work)
            .output()
            .unwrap();
        std::process::Command::new("git")
            .args(["add", "f.txt"])
            .current_dir(&work)
            .output()
            .unwrap();
        std::process::Command::new("git")
            .args(["commit", "-m", "init"])
            .current_dir(&work)
            .output()
            .unwrap();
        std::process::Command::new("git")
            .args(["remote", "add", "origin"])
            .arg(&bare)
            .current_dir(&work)
            .output()
            .unwrap();
        std::process::Command::new("git")
            .args(["push", "-u", "origin", "main"])
            .current_dir(&work)
            .output()
            .unwrap();
        std::process::Command::new("git")
            .args(["branch", "only-remote"])
            .current_dir(&work)
            .output()
            .unwrap();
        std::process::Command::new("git")
            .args(["push", "origin", "only-remote"])
            .current_dir(&work)
            .output()
            .unwrap();
        std::process::Command::new("git")
            .args(["switch", "main"])
            .current_dir(&work)
            .output()
            .unwrap();
        std::process::Command::new("git")
            .args(["branch", "-D", "only-remote"])
            .current_dir(&work)
            .output()
            .unwrap();
        std::process::Command::new("git")
            .args(["fetch", "origin"])
            .current_dir(&work)
            .output()
            .unwrap();

        let ctx = RepoContext::open(&work.to_string_lossy()).expect("ctx");
        let preview = preview_write(
            &ctx,
            ctx.repo_path(),
            &WriteRequest::SwitchBranch {
                branch: "only-remote".into(),
                track_remote: Some("origin".into()),
            },
        )
        .expect("preview");
        assert!(preview.blocked.is_none(), "{:?}", preview.blocked);
        let joined = preview.commands.join(" ");
        assert!(
            joined.contains("switch") && joined.contains("-c") && joined.contains("only-remote"),
            "preview deve usar switch -c: {joined}"
        );
        assert!(
            joined.contains("branch.only-remote.remote"),
            "preview deve configurar upstream via config: {joined}"
        );

        let result = execute_write(
            &ctx,
            WriteRequest::SwitchBranch {
                branch: "only-remote".into(),
                track_remote: Some("origin".into()),
            },
        );
        assert!(result.is_ok(), "{result:?}");
        let current = std::process::Command::new("git")
            .args(["branch", "--show-current"])
            .current_dir(&work)
            .output()
            .unwrap();
        assert_eq!(
            String::from_utf8_lossy(&current.stdout).trim(),
            "only-remote"
        );
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// Regressão: publicação com remoto já configurado e URL NOVA deve
    /// corrigir a URL (`remote set-url`) antes do push — sem isso, quem
    /// publicou apontando para a conta errada ficava preso no terminal.
    #[test]
    fn publish_com_url_nova_gera_set_url() {
        let dir = std::env::temp_dir().join(format!("trilho-pub-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        init_repo_with_commit(&dir);
        std::process::Command::new("git")
            .args(["remote", "add", "origin", "git@github.com:errada/repo.git"])
            .current_dir(&dir)
            .output()
            .unwrap();

        let ctx = RepoContext::open(&dir.to_string_lossy()).expect("ctx");
        let plan = plan_publish(&ctx, Some("git@github.com:certa/repo.git")).expect("plan");
        let step = plan.remote_step.expect("deve ter passo de remoto");
        let args = step.command().args;
        assert!(args.contains(&"set-url".to_string()), "args: {args:?}");
        assert!(args.contains(&"git@github.com:certa/repo.git".to_string()));

        // Mesma URL → sem passo de remoto (só push).
        let plan = plan_publish(&ctx, Some("git@github.com:errada/repo.git")).expect("plan");
        assert!(plan.remote_step.is_none());
        let _ = std::fs::remove_dir_all(&dir);
    }
}
