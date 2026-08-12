//! Sincronização com o remoto — publish, push, pull, fetch, unshallow e
//! force push.

use super::gates::{gate_force_push_upstream, gate_sequencer_idle};
use crate::application::operations::{
    AddRemote, GitOperation, PushForceWithLease, PushSetUpstream, SetRemoteUrl,
};
use crate::application::{GitCommand, GitError, RepoContext};
use crate::infrastructure::{primary_remote, repo_info, validate_remote_url};

/// Saída de `plan_publish`. Condições que impedem publicar mas não são falha
/// (falta a URL, branch já publicada) viram `Bloqueado` — o preview mostra o
/// motivo em vez de erro, e a execução recusa.
pub(super) enum PlanoPublicacao {
    Pronto(PublishPlan),
    Bloqueado(String),
}

pub(super) struct PublishPlan {
    /// Passo de remoto: `remote add` (1ª publicação) ou `remote set-url`
    /// (corrigir URL errada) — ambos são `GitOperation`.
    pub(super) remote_step: Option<Box<dyn GitOperation>>,
    push: PushSetUpstream,
    description: String,
}

fn resolve_primary_remote(ctx: &RepoContext) -> Result<String, GitError> {
    primary_remote(ctx.repo_path())
}

pub(super) fn plan_publish(
    ctx: &RepoContext,
    remote_url: Option<&str>,
) -> Result<PlanoPublicacao, GitError> {
    let info = repo_info(ctx.repo_path())?;
    if info.is_detached {
        return Err(GitError::Git(
            "Repositório em detached HEAD — troque para uma branch antes de publicar.".into(),
        ));
    }
    if info.upstream.is_some() {
        return Ok(PlanoPublicacao::Bloqueado(
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
                    return Ok(PlanoPublicacao::Bloqueado(
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

    Ok(PlanoPublicacao::Pronto(PublishPlan {
        remote_step,
        push: PushSetUpstream {
            remote: remote_name,
            branch,
        },
        description,
    }))
}

pub(super) fn preview_publish(
    ctx: &RepoContext,
    remote_url: Option<&str>,
) -> Result<(Vec<String>, String, Option<String>), GitError> {
    match plan_publish(ctx, remote_url)? {
        PlanoPublicacao::Pronto(plan) => {
            let mut commands = Vec::new();
            if let Some(ref op) = plan.remote_step {
                commands.extend(ctx.preview_op(op.as_ref()));
            }
            commands.extend(ctx.preview_op(&plan.push));
            Ok((commands, plan.description, None))
        }
        PlanoPublicacao::Bloqueado(motivo) => Ok((vec![], String::new(), Some(motivo))),
    }
}

pub(super) fn execute_publish(ctx: &RepoContext, remote_url: Option<&str>) -> Result<(), GitError> {
    let plan = match plan_publish(ctx, remote_url)? {
        PlanoPublicacao::Pronto(plan) => plan,
        PlanoPublicacao::Bloqueado(motivo) => return Err(GitError::Git(motivo)),
    };
    if let Some(op) = plan.remote_step {
        ctx.execute_op(op.as_ref())?;
    }
    ctx.execute_op(&plan.push)?;
    sync_local_upstream_ref(ctx, &plan.push)?;
    Ok(())
}

pub(super) fn gate_push(ctx: &RepoContext) -> Result<Option<String>, GitError> {
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
        if sync.ahead > 0 {
            return Ok(Some(
                "Históricos local e remoto divergiram — push normal não funciona. \
                 Se você reescreveu commits e quer sobrescrever o remoto, use «Force push»."
                    .into(),
            ));
        }
        return Ok(Some(
            "O remoto está à frente. Atualize com «pull --ff-only» antes de enviar.".into(),
        ));
    }
    Ok(None)
}

/// `git push -u <remote> <branch>` — explícito quando o tracking local está incompleto.
pub(super) fn push_upstream_op(ctx: &RepoContext) -> Result<PushSetUpstream, GitError> {
    let sync = ctx.reader().get_sync_info()?;
    let upstream = sync.upstream.ok_or_else(|| {
        GitError::Git("Branch sem upstream — use «Publicar» no Trilho.".into())
    })?;
    let (remote, branch) = upstream.split_once('/').ok_or_else(|| {
        GitError::Git(format!("Upstream inválido: {upstream}"))
    })?;
    Ok(PushSetUpstream {
        remote: remote.to_string(),
        branch: branch.to_string(),
    })
}

/// Atualiza `refs/remotes/<remote>/<branch>` após push — refspec restrito (só `main`)
/// não atualiza o tracking da branch atual no `git fetch` padrão.
pub(super) fn sync_local_upstream_ref(
    ctx: &RepoContext,
    op: &PushSetUpstream,
) -> Result<(), GitError> {
    let head = ctx
        .writer()
        .run(&GitCommand {
            args: vec!["rev-parse".into(), "HEAD".into()],
        })?
        .trim()
        .to_string();
    let tracking = format!("refs/remotes/{}/{}", op.remote, op.branch);
    ctx.writer().run(&GitCommand {
        args: vec!["update-ref".into(), tracking, head],
    })?;
    Ok(())
}

pub(super) fn gate_pull(ctx: &RepoContext) -> Result<Option<String>, GitError> {
    let sync = ctx.reader().get_sync_info()?;
    if sync.upstream.is_none() {
        return Ok(Some(
            "Branch sem upstream — use «Publicar» no Trilho antes de puxar.".into(),
        ));
    }
    if sync.ahead > 0 && sync.behind > 0 {
        return Ok(Some(
            "Históricos local e remoto divergiram — pull --ff-only não resolve. \
             Se você reescreveu commits (reword/reset) e quer sobrescrever o remoto, \
             use «Force push». Caso contrário, resolva com merge/rebase fora do Trilho."
                .into(),
        ));
    }
    if sync.behind == 0 {
        return Ok(Some(
            "Já está em dia com o remoto (nada para puxar).".into(),
        ));
    }
    Ok(None)
}

/// Atualiza o ref de tracking do upstream e envia com `--force-with-lease`.
/// Evita `stale info` quando o fetch padrão do remoto não cobre esta branch
/// (refspec restrito, ex.: só `main`).
pub(super) fn execute_force_push_with_lease(ctx: &RepoContext) -> Result<(), GitError> {
    let (remote, branch, expect_sha) = refresh_upstream_tracking_ref(ctx)?;
    ctx.execute_op(&PushForceWithLease {
        remote: remote.clone(),
        branch: branch.clone(),
        expect_sha,
    })?;
    // Alinha o tracking local ao HEAD enviado (fetch padrão pode não cobrir).
    let head = ctx
        .writer()
        .run(&GitCommand {
            args: vec!["rev-parse".into(), "HEAD".into()],
        })?
        .trim()
        .to_string();
    let _ = ctx.writer().run(&GitCommand {
        args: vec![
            "update-ref".into(),
            format!("refs/remotes/{remote}/{branch}"),
            head,
        ],
    });
    Ok(())
}

/// Fetch explícito do branch de tracking; retorna (remote, branch, tip_sha).
fn refresh_upstream_tracking_ref(ctx: &RepoContext) -> Result<(String, String, String), GitError> {
    let sync = ctx.reader().get_sync_info()?;
    let upstream = sync.upstream.ok_or_else(|| {
        GitError::Git("Branch sem upstream — configure o remoto antes do push forçado.".into())
    })?;
    let (remote, branch) = upstream.split_once('/').ok_or_else(|| {
        GitError::Git(format!("Upstream inválido: {upstream}"))
    })?;
    let remote = remote.to_string();
    let branch = branch.to_string();
    let tracking = format!("refs/remotes/{remote}/{branch}");
    let spec = format!("+refs/heads/{branch}:{tracking}");
    ctx.writer().run(&GitCommand {
        args: vec!["fetch".into(), remote.clone(), spec],
    })?;
    let expect_sha = ctx
        .writer()
        .run(&GitCommand {
            args: vec!["rev-parse".into(), tracking],
        })?
        .trim()
        .to_string();
    // Config explícita (não depende de `branch --set-upstream-to`, que falha
    // quando o refspec do remoto não lista esta branch como tracking «válida»).
    let _ = ctx.writer().run(&GitCommand {
        args: vec![
            "config".into(),
            format!("branch.{branch}.remote"),
            remote.clone(),
        ],
    });
    let _ = ctx.writer().run(&GitCommand {
        args: vec![
            "config".into(),
            format!("branch.{branch}.merge"),
            format!("refs/heads/{branch}"),
        ],
    });
    Ok((remote, branch, expect_sha))
}

pub(super) fn gate_unshallow(ctx: &RepoContext) -> Result<Option<String>, GitError> {
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

pub(super) fn gate_force_push_standalone(ctx: &RepoContext) -> Result<Option<String>, GitError> {
    let info = repo_info(ctx.repo_path())?;
    if info.is_detached {
        return Ok(Some(
            "Repositório em detached HEAD — troque para uma branch antes do push forçado.".into(),
        ));
    }
    if let Some(msg) = gate_sequencer_idle(ctx.repo_path(), "enviar com push forçado")? {
        return Ok(Some(msg));
    }
    if let Some(msg) = gate_force_push_upstream(ctx)? {
        return Ok(Some(msg));
    }
    let sync = ctx.reader().get_sync_info()?;
    if sync.behind == 0 {
        return Ok(Some(
            "O remoto não está à frente — use o push normal ou confirme o push forçado \
             no fluxo de reset/reword se reescreveu o histórico local."
                .into(),
        ));
    }
    Ok(None)
}

pub(super) fn is_likely_protected_branch(branch: &str) -> bool {
    matches!(
        branch,
        "main" | "master" | "develop" | "production" | "release"
    )
}

pub(super) fn remote_only_commit_short_ids(ctx: &RepoContext) -> Result<Vec<String>, GitError> {
    let sync = ctx.reader().get_sync_info()?;
    let Some(upstream) = sync.upstream else {
        return Ok(vec![]);
    };
    let out = ctx.writer().run(&GitCommand {
        args: vec![
            "rev-list".into(),
            "--max-count=10".into(),
            upstream,
            "--not".into(),
            "HEAD".into(),
        ],
    })?;
    Ok(out
        .lines()
        .filter(|line| !line.is_empty())
        .map(|sha| sha[..sha.len().min(7)].to_string())
        .collect())
}
