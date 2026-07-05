//! Casos de uso de escrita M3 — preview (RF-08) e execução com gates.

use crate::application::operations::{
    AddRemote, CreateCommit, GitOperation, PullFfOnly, PushSetUpstream, PushUpstream, RevertCommit,
    SetRemoteUrl, Stage, StageAll, StageMany, UncommitSoft, Unstage, UnstageAll, UnstageMany,
};
use crate::application::write_gates::head_is_local_only;
use crate::application::{GitError, RepoContext};
use crate::domain::{OperationPreview, WriteRequest};
use crate::infrastructure::{
    repo_info, validate_git_object_id, validate_remote_url, validate_repo_relative_path,
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
            (ctx.preview_op(&op), op.description().to_string(), blocked)
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
            let op = RevertCommit { sha };
            (ctx.preview_op(&op), op.description().to_string(), None)
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
