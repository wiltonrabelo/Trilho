//! Dispatcher de preview (RF-08) — monta comandos, descrição e gates por
//! variant de `WriteRequest`.

use super::branch::{
    gate_delete_local_branch, gate_delete_remote_branch, gate_switch_branch, resolve_switch_track,
};
use super::commit_history::{
    cherry_pick_description, gate_amend, gate_cherry_pick_shas, gate_reset, gate_revert,
    gate_revert_merge, gate_reword, gate_uncommit, reset_needs_force_push,
    resolve_cherry_pick_shas, reword_target_on_remote, RESET_HARD_STASH_MSG,
};
use super::conflict_sequencer::{
    gate_abort_cherry_pick, gate_abort_merge, gate_abort_revert, gate_continue_cherry_pick,
    gate_continue_merge, gate_continue_revert, gate_resolve_conflict, gate_skip_cherry_pick,
    gate_skip_revert, normalize_conflict_side,
};
use super::gates::{branch_name_for_backup, has_tracked_worktree_changes};
use super::staging::{
    gate_discard_hunk, gate_discard_worktree, gate_discard_worktree_all, gate_remove_untracked,
    gate_save_worktree_file,
};
use super::stash_tag::{
    gate_stash, gate_stash_index, gate_stash_restore, gate_tag_missing, preview_create_tag,
    stash_effect_description, stash_label,
};
use super::sync_remote::{
    gate_force_push_standalone, gate_pull, gate_push, gate_unshallow, is_likely_protected_branch,
    preview_publish, push_upstream_op, remote_only_commit_short_ids,
};
use super::{
    validate_paths, validate_paths_de_indice, validate_paths_de_indice_muitos, PathsValidados,
    MSG_SELECAO_VAZIA,
};
use crate::domain::{caminho_git_do_rotulo, ResetMode};
use crate::application::backup_ref::backup_ref_preview_command;
use crate::application::operations::{
    AbortCherryPick, AbortMerge, AbortRevert, ApplyReversePatch, CherryPickCommit, ContinueMerge,
    CreateCommit, DeleteLocalBranch, DeleteRemoteBranch, DeleteTag, DiscardWorktree,
    DiscardWorktreeAll, DiscardWorktreeMany, GitOperation, PullFfOnly, PushUpstream,
    RemoveUntracked, RemoveUntrackedMany, ResetCommit, RevertCommit, SkipCherryPick,
    SkipRevert, Stage, StageAll, StageMany, StashApply, StashDrop, StashPop, StashPush,
    SwitchBranch, UncommitSoft, UnshallowRemote, Unstage, UnstageAll, UnstageMany,
};
use crate::application::{GitError, GitWriter, RepoContext};
use crate::domain::{OperationPreview, WriteRequest};
use crate::infrastructure::{
    stash_reference, validate_clone_branch, validate_git_object_id, validate_remote_name,
    validate_repo_relative_path, validate_tag_name,
};

fn blocked_preview(repo_path: &str, msg: &str) -> OperationPreview {
    OperationPreview {
        commands: vec![],
        description: String::new(),
        repo_path: repo_path.to_string(),
        blocked: Some(msg.to_string()),
        authorization: None,
    }
}

pub fn preview_write(
    ctx: &RepoContext,
    repo_path: &str,
    req: &WriteRequest,
) -> Result<OperationPreview, GitError> {
    let (commands, description, blocked) = match req {
        WriteRequest::Stage { path } => {
            let path = validate_repo_relative_path(caminho_git_do_rotulo(path))
                .map_err(|e| GitError::Git(e.to_string()))?;
            let op = Stage { path };
            (ctx.preview_op(&op), op.description().to_string(), None)
        }
        WriteRequest::StageMany { paths } => {
            let paths = match validate_paths(paths)? {
                PathsValidados::Validos(p) => p,
                PathsValidados::SelecaoVazia => {
                    return Ok(blocked_preview(repo_path, MSG_SELECAO_VAZIA));
                }
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
            let op = Unstage {
                paths: validate_paths_de_indice(path)?,
            };
            (ctx.preview_op(&op), op.description().to_string(), None)
        }
        WriteRequest::UnstageMany { paths } => {
            // `count` conta arquivos escolhidos; `paths` pode ser maior, porque
            // cada renomeação ocupa duas entradas no índice.
            let count = paths.len();
            let paths = match validate_paths_de_indice_muitos(paths)? {
                PathsValidados::Validos(p) => p,
                PathsValidados::SelecaoVazia => {
                    return Ok(blocked_preview(repo_path, MSG_SELECAO_VAZIA));
                }
            };
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
            let blocked = gate_revert(ctx, repo_path)?
                .or(gate_revert_merge(ctx, &sha)?);
            let op = RevertCommit { sha };
            (ctx.preview_op(&op), op.description().to_string(), blocked)
        }
        WriteRequest::CherryPick {
            commit_id,
            commit_ids,
            record_origin,
        } => {
            let shas = resolve_cherry_pick_shas(commit_id, commit_ids)?;
            let blocked = gate_cherry_pick_shas(ctx, repo_path, &shas)?;
            let op = CherryPickCommit {
                shas: shas.clone(),
                record_origin: *record_origin,
            };
            (
                ctx.preview_op(&op),
                cherry_pick_description(&shas, *record_origin),
                blocked,
            )
        }
        WriteRequest::Push => {
            let blocked = gate_push(ctx)?;
            let (commands, description) = if blocked.is_some() {
                let op = PushUpstream;
                (ctx.preview_op(&op), op.description().to_string())
            } else {
                let op = push_upstream_op(ctx)?;
                let sync = ctx.reader().get_sync_info()?;
                (
                    ctx.preview_op(&op),
                    format!(
                        "Envia {} commit(s) locais para {}.",
                        sync.ahead,
                        sync.upstream.as_deref().unwrap_or("remoto")
                    ),
                )
            };
            (commands, description, blocked)
        }
        WriteRequest::PullFfOnly => {
            let op = PullFfOnly;
            let blocked = gate_pull(ctx)?;
            (ctx.preview_op(&op), op.description().to_string(), blocked)
        }
        WriteRequest::FetchRemote => {
            let commands =
                crate::infrastructure::preview_fetch_all_remote_branch_refs(repo_path)?;
            (
                commands,
                "Atualiza refs remotas (fetch + prune) de todos os remotos.".into(),
                None,
            )
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
        WriteRequest::DeleteLocalBranch { branch } => {
            let branch = validate_clone_branch(Some(branch))?
                .ok_or_else(|| GitError::Git("Nome de branch inválido.".into()))?;
            let blocked = gate_delete_local_branch(ctx, repo_path, &branch)?;
            let op = DeleteLocalBranch {
                branch: branch.clone(),
            };
            let description = format!(
                "Remove a branch local «{branch}» (git branch -D). Não altera o remoto."
            );
            (ctx.preview_op(&op), description, blocked)
        }
        WriteRequest::DeleteRemoteBranch { remote, branch } => {
            let remote = match validate_remote_name(remote) {
                Ok(r) => r,
                Err(GitError::Git(msg)) => return Ok(blocked_preview(repo_path, &msg)),
                Err(e) => return Err(e),
            };
            let branch = validate_clone_branch(Some(branch))?
                .ok_or_else(|| GitError::Git("Nome de branch inválido.".into()))?;
            let blocked = gate_delete_remote_branch(repo_path, &remote, &branch)?;
            let op = DeleteRemoteBranch {
                remote: remote.clone(),
                branch: branch.clone(),
            };
            let description = format!(
                "Remove a branch «{branch}» no remoto «{remote}» (git push {remote} --delete {branch}). Esta ação afeta o repositório no servidor."
            );
            (ctx.preview_op(&op), description, blocked)
        }
        WriteRequest::StashPush {
            message,
            include_untracked,
        } => {
            let op = StashPush {
                message: message.clone(),
                include_untracked: *include_untracked,
            };
            let blocked = gate_stash(ctx, *include_untracked)?;
            let description = stash_effect_description(ctx, *include_untracked)?;
            (ctx.preview_op(&op), description, blocked)
        }
        WriteRequest::StashApply { index } => {
            let reference = stash_reference(*index)?;
            let op = StashApply {
                reference: reference.clone(),
            };
            let blocked = gate_stash_restore(ctx)?
                .or(gate_stash_index(repo_path, *index)?);
            let description = format!(
                "Reaplica «{reference}» ({}) na working tree.",
                stash_label(repo_path, *index)
            );
            (ctx.preview_op(&op), description, blocked)
        }
        WriteRequest::StashPop { index } => {
            let reference = stash_reference(*index)?;
            let op = StashPop {
                reference: reference.clone(),
            };
            let blocked = gate_stash_restore(ctx)?
                .or(gate_stash_index(repo_path, *index)?);
            let description = format!(
                "Reaplica e remove «{reference}» ({}) da pilha.",
                stash_label(repo_path, *index)
            );
            (ctx.preview_op(&op), description, blocked)
        }
        WriteRequest::StashDrop { index } => {
            let reference = stash_reference(*index)?;
            let op = StashDrop {
                reference: reference.clone(),
            };
            let blocked = gate_stash_index(repo_path, *index)?;
            let description = format!(
                "Remove «{reference}» ({}) sem reaplicar.",
                stash_label(repo_path, *index)
            );
            (ctx.preview_op(&op), description, blocked)
        }
        WriteRequest::CreateTag {
            name,
            commit_id,
            annotated,
            message,
            push_to_remote,
        } => preview_create_tag(
            ctx,
            repo_path,
            name,
            commit_id,
            *annotated,
            message.as_deref(),
            *push_to_remote,
        )?,
        WriteRequest::DeleteTag { name } => {
            let name = match validate_tag_name(name) {
                Ok(n) => n,
                Err(GitError::Git(msg)) => {
                    return Ok(blocked_preview(repo_path, &msg));
                }
                Err(e) => return Err(e),
            };
            let blocked = gate_tag_missing(repo_path, &name)?;
            let op = DeleteTag {
                name: name.clone(),
            };
            let description = format!("Remove a tag local «{name}».");
            (ctx.preview_op(&op), description, blocked)
        }
        WriteRequest::DiscardWorktree { path } => {
            let path = validate_repo_relative_path(caminho_git_do_rotulo(path))
                .map_err(|e| GitError::Git(e.to_string()))?;
            let blocked = gate_discard_worktree(ctx, std::slice::from_ref(&path))?;
            let op = DiscardWorktree { path };
            (
                ctx.preview_op(&op),
                format!("{} Esta ação não pode ser desfeita.", op.description()),
                blocked,
            )
        }
        WriteRequest::DiscardWorktreeMany { paths } => {
            let paths = match validate_paths(paths)? {
                PathsValidados::Validos(p) => p,
                PathsValidados::SelecaoVazia => {
                    return Ok(blocked_preview(repo_path, MSG_SELECAO_VAZIA));
                }
            };
            let blocked = gate_discard_worktree(ctx, &paths)?;
            let count = paths.len();
            let op = DiscardWorktreeMany { paths };
            (
                ctx.preview_op(&op),
                format!(
                    "{} ({count} arquivo(s)). Esta ação não pode ser desfeita.",
                    op.description()
                ),
                blocked,
            )
        }
        WriteRequest::DiscardWorktreeAll => {
            let blocked = gate_discard_worktree_all(ctx)?;
            let op = DiscardWorktreeAll;
            (
                ctx.preview_op(&op),
                format!(
                    "{} Esta ação não pode ser desfeita.",
                    op.description()
                ),
                blocked,
            )
        }
        WriteRequest::RemoveUntracked { path } => {
            let path = validate_repo_relative_path(caminho_git_do_rotulo(path))
                .map_err(|e| GitError::Git(e.to_string()))?;
            let blocked = gate_remove_untracked(ctx, std::slice::from_ref(&path))?;
            let op = RemoveUntracked { path };
            (
                ctx.preview_op(&op),
                format!(
                    "{} O arquivo será apagado do disco.",
                    op.description()
                ),
                blocked,
            )
        }
        WriteRequest::RemoveUntrackedMany { paths } => {
            let paths = match validate_paths(paths)? {
                PathsValidados::Validos(p) => p,
                PathsValidados::SelecaoVazia => {
                    return Ok(blocked_preview(repo_path, MSG_SELECAO_VAZIA));
                }
            };
            let blocked = gate_remove_untracked(ctx, &paths)?;
            let count = paths.len();
            let op = RemoveUntrackedMany { paths };
            (
                ctx.preview_op(&op),
                format!(
                    "{} ({count} item(ns)). Serão apagados do disco.",
                    op.description()
                ),
                blocked,
            )
        }
        WriteRequest::DiscardHunk { path, patch, staged } => {
            let path = validate_repo_relative_path(caminho_git_do_rotulo(path))
                .map_err(|e| GitError::Git(e.to_string()))?;
            let blocked = gate_discard_hunk(ctx, &path, *staged, patch)?;
            let op = ApplyReversePatch {
                patch: patch.clone(),
                cached: *staged,
            };
            let alvo = if *staged { "no stage" } else { "no arquivo" };
            (
                ctx.preview_op(&op),
                format!(
                    "Descarta um trecho de «{path}» {alvo}. Esta ação não pode ser desfeita."
                ),
                blocked,
            )
        }
        WriteRequest::ResolveConflictSide { path, side } => {
            let path = validate_repo_relative_path(caminho_git_do_rotulo(path))
                .map_err(|e| GitError::Git(e.to_string()))?;
            let side_norm = normalize_conflict_side(side)?;
            let blocked = gate_resolve_conflict(ctx, &path)?;
            let flag = if side_norm == "ours" {
                "--ours"
            } else {
                "--theirs"
            };
            (
                vec![
                    format!("git checkout {flag} -- {path}"),
                    format!("git add -- {path}"),
                ],
                format!(
                    "Resolve o conflito em «{path}» aceitando o lado {side_norm} \
                     e marca o arquivo como resolvido."
                ),
                blocked,
            )
        }
        WriteRequest::ResolveConflictContent { path, content } => {
            let path = validate_repo_relative_path(caminho_git_do_rotulo(path))
                .map_err(|e| GitError::Git(e.to_string()))?;
            let blocked = gate_resolve_conflict(ctx, &path)?;
            if content.contains("<<<<<<<") || content.contains(">>>>>>>") {
                return Ok(OperationPreview {
                    commands: vec![],
                    description: String::new(),
                    repo_path: repo_path.to_string(),
                    blocked: Some(
                        "O conteúdo ainda contém marcadores de conflito — \
                         resolva todos os blocos antes de marcar como resolvido."
                            .into(),
                    ),
                    authorization: None,
                });
            }
            (
                vec![
                    format!("# grava conteúdo resolvido em {path}"),
                    format!("git add -- {path}"),
                ],
                format!(
                    "Grava a resolução manual de «{path}» e marca o arquivo como resolvido."
                ),
                blocked,
            )
        }
        WriteRequest::SaveWorktreeFile { path, content } => {
            let path = validate_repo_relative_path(caminho_git_do_rotulo(path))
                .map_err(|e| GitError::Git(e.to_string()))?;
            let blocked = gate_save_worktree_file(ctx, &path)?;
            let bytes = content.len();
            (
                vec![format!("# grava {bytes} bytes em {path}")],
                format!(
                    "Salva as edições de «{path}» no working tree (sem stage automático)."
                ),
                blocked,
            )
        }
        WriteRequest::AbortRevert => {
            let op = AbortRevert;
            (
                ctx.preview_op(&op),
                op.description().to_string(),
                gate_abort_revert(repo_path)?,
            )
        }
        WriteRequest::ContinueRevert => {
            let blocked = gate_continue_revert(repo_path, ctx)?;
            (
                vec![
                    "git revert --continue --no-edit".into(),
                    "# se não houver alterações: git revert --skip".into(),
                ],
                "Finaliza o revert em andamento. Se a resolução dos conflitos não \
                 gerou alterações, o patch é pulado automaticamente."
                    .into(),
                blocked,
            )
        }
        WriteRequest::AbortMerge => {
            let op = AbortMerge;
            (
                ctx.preview_op(&op),
                op.description().to_string(),
                gate_abort_merge(repo_path)?,
            )
        }
        WriteRequest::ContinueMerge => {
            let op = ContinueMerge;
            (
                ctx.preview_op(&op),
                op.description().to_string(),
                gate_continue_merge(repo_path, ctx)?,
            )
        }
        WriteRequest::AbortCherryPick => {
            let op = AbortCherryPick;
            (
                ctx.preview_op(&op),
                op.description().to_string(),
                gate_abort_cherry_pick(repo_path)?,
            )
        }
        WriteRequest::ContinueCherryPick => {
            let blocked = gate_continue_cherry_pick(repo_path, ctx)?;
            (
                vec![
                    "git cherry-pick --continue --no-edit".into(),
                    "# se não houver alterações: git cherry-pick --skip".into(),
                ],
                "Finaliza o cherry-pick em andamento. Se a resolução dos conflitos não \
                 gerou alterações, o patch é pulado automaticamente."
                    .into(),
                blocked,
            )
        }
        WriteRequest::SkipRevert => {
            let op = SkipRevert;
            (
                ctx.preview_op(&op),
                op.description().to_string(),
                gate_skip_revert(repo_path)?,
            )
        }
        WriteRequest::SkipCherryPick => {
            let op = SkipCherryPick;
            (
                ctx.preview_op(&op),
                op.description().to_string(),
                gate_skip_cherry_pick(repo_path)?,
            )
        }
        WriteRequest::Reword {
            commit_id,
            summary,
            body: _body,
            force_push,
        } => {
            let sha =
                validate_git_object_id(commit_id).map_err(|e| GitError::Git(e.to_string()))?;
            let on_remote = reword_target_on_remote(ctx, &sha)?;
            let blocked = gate_reword(ctx, repo_path, &sha, summary, *force_push)?;
            let short = &sha[..sha.len().min(7)];
            let mut description = format!(
                "Reescreve a mensagem do commit {} e reaplica os commits seguintes — \
                 cada um receberá um novo SHA.",
                short
            );
            if on_remote && *force_push {
                description.push_str(
                    " Em seguida envia o histórico reescrito ao remoto com push forçado \
                     (--force-with-lease).",
                );
            }
            let mut commands = vec![
                format!("git checkout --detach {short}^"),
                format!("git cherry-pick -n {short} && git commit -F -  # «{summary}»"),
                "# cherry-pick dos commits seguintes até HEAD".into(),
                "git branch -f <branch-atual> HEAD && git checkout <branch-atual>".into(),
            ];
            if on_remote && *force_push {
                commands.push("git push --force-with-lease".into());
            }
            (commands, description, blocked)
        }
        WriteRequest::Reset {
            commit_id,
            mode,
            force_push,
        } => {
            let sha =
                validate_git_object_id(commit_id).map_err(|e| GitError::Git(e.to_string()))?;
            let reset_mode = *mode;
            let needs_force = reset_needs_force_push(ctx, &sha)?;
            let blocked = gate_reset(ctx, repo_path, &sha, reset_mode, *force_push, needs_force)?;
            let short = &sha[..sha.len().min(7)];
            let op = ResetCommit {
                sha: sha.clone(),
                mode: reset_mode,
            };
            let mut commands = ctx.preview_op(&op);
            let mut description = format!(
                "Move o HEAD para o commit {short} (modo {}). Commits mais recentes na branch \
                 deixam de fazer parte do histórico local.",
                reset_mode.label()
            );
            if reset_mode == ResetMode::Hard {
                let branch = branch_name_for_backup(repo_path)?;
                commands.insert(0, backup_ref_preview_command(&branch));
                description.push_str(
                    " Cria backup local (ref trilho/backup) do HEAD atual.",
                );
                if has_tracked_worktree_changes(ctx)? {
                    commands.insert(
                        1,
                        format!("git stash push -m \"{RESET_HARD_STASH_MSG}\""),
                    );
                    description.push_str(
                        " Alterações locais em arquivos rastreados serão guardadas em stash \
                         antes do reset.",
                    );
                } else {
                    description.push_str(
                        " Alterações não commitadas em arquivos rastreados serão descartadas.",
                    );
                }
            }
            if needs_force {
                description.push_str(
                    " Commits posteriores já estão no remoto — o reset é só local. Quando \
                     quiser publicar o histórico novo, use «Force push» no sync \
                     (--force-with-lease); não é feito automaticamente.",
                );
                if *force_push {
                    commands.push("git push --force-with-lease".into());
                }
            }
            (commands, description, blocked)
        }
        WriteRequest::PushForce => {
            let blocked = gate_force_push_standalone(ctx)?;
            let branch = branch_name_for_backup(repo_path)?;
            let remote_commits = remote_only_commit_short_ids(ctx)?;
            let mut commands = vec!["git push --force-with-lease".into()];
            commands.insert(0, backup_ref_preview_command(&branch));
            commands.insert(
                1,
                format!("git fetch origin +refs/heads/{branch}:refs/remotes/origin/{branch}"),
            );
            let mut description = String::from(
                "Reescreve a branch remota com o HEAD local (--force-with-lease). Operação \
                 irreversível para quem já baseou trabalho nos commits que existirem só no remoto. \
                 Atualiza o tracking remoto e cria backup local (ref trilho/backup) do HEAD atual.",
            );
            if !remote_commits.is_empty() {
                description.push_str(&format!(
                    " Commits que deixarão de fazer parte da branch no remoto: {}.",
                    remote_commits.join(", ")
                ));
            }
            if is_likely_protected_branch(&branch) {
                description.push_str(
                    " ATENÇÃO: branch sensível (main/master) — confirme com a equipe.",
                );
            }
            (commands, description, blocked)
        }
        WriteRequest::Publish { url } => preview_publish(ctx, url.as_deref())?,
    };

    Ok(OperationPreview {
        commands,
        description,
        repo_path: repo_path.to_string(),
        blocked,
        authorization: None,
    })
}
