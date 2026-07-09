//! Casos de uso de escrita M3 — preview (RF-08) e execução com gates.

use crate::application::backup_ref::{backup_ref_preview_command, create_backup_ref};
use crate::application::operations::{
    AddRemote, ApplyReversePatch, AbortCherryPick, AbortMerge, AbortRevert, CherryPickCommit,
    ContinueMerge, CreateCommit, CreateTag, DeleteTag, DiscardWorktree,
    DiscardWorktreeAll, DiscardWorktreeMany, GitOperation, PullFfOnly, PushForceWithLease,
    PushSetUpstream, PushTag,
    PushUpstream, RemoveUntracked, RemoveUntrackedMany, ResetCommit, ResetMode, RevertCommit,
    SetRemoteUrl, SkipCherryPick, SkipRevert, Stage,
    StageAll, StageMany, StashApply, StashDrop, StashPop, StashPush, SwitchBranch, UncommitSoft,
    UnshallowRemote, Unstage, UnstageAll, UnstageMany,
};
use crate::application::write_gates::{head_is_local_only, is_commit_on_remote};
use crate::application::{GitCommand, GitError, GitWriter, RepoContext};
use crate::domain::{FileChangeKind, InProgressKind, OperationPreview, ResetModeDto, WriteRequest};
use crate::infrastructure::{
    list_local_branches, list_remote_branches, list_stashes, repo_info, stash_reference,
    validate_clone_branch, validate_git_object_id, validate_remote_name, validate_remote_url,
    validate_repo_relative_path, validate_tag_name, execute_reword, SafeGitCli,
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
            let blocked = gate_revert(ctx, repo_path)?
                .or(gate_revert_merge(ctx, &sha)?)
                .or(gate_not_head_commit(repo_path, &sha, "reverter")?);
            let op = RevertCommit { sha };
            (ctx.preview_op(&op), op.description().to_string(), blocked)
        }
        WriteRequest::CherryPick {
            commit_id,
            commit_ids,
            record_origin,
        } => {
            let shas = resolve_cherry_pick_shas(&commit_id, &commit_ids)?;
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
            let path = validate_repo_relative_path(git_path_from_display(path))
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
            let paths = match validate_paths(paths) {
                Ok(p) => p,
                Err(GitError::Git(msg)) if msg.contains("Nenhum") => {
                    return Ok(blocked_preview(repo_path, &msg));
                }
                Err(e) => return Err(e),
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
            let path = validate_repo_relative_path(git_path_from_display(path))
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
            let paths = match validate_paths(paths) {
                Ok(p) => p,
                Err(GitError::Git(msg)) if msg.contains("Nenhum") => {
                    return Ok(blocked_preview(repo_path, &msg));
                }
                Err(e) => return Err(e),
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
        WriteRequest::DiscardHunk { path, patch } => {
            let path = validate_repo_relative_path(git_path_from_display(path))
                .map_err(|e| GitError::Git(e.to_string()))?;
            let blocked = gate_discard_worktree(ctx, std::slice::from_ref(&path))?
                .or(gate_reverse_patch(ctx, patch)?);
            let op = ApplyReversePatch {
                patch: patch.clone(),
            };
            (
                ctx.preview_op(&op),
                format!(
                    "Descarta um trecho de «{path}». Esta ação não pode ser desfeita."
                ),
                blocked,
            )
        }
        WriteRequest::ResolveConflictSide { path, side } => {
            let path = validate_repo_relative_path(git_path_from_display(path))
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
            let path = validate_repo_relative_path(git_path_from_display(path))
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
            let reset_mode = reset_mode_from_dto(*mode);
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
            if let Err(e) = ctx.execute_op(&RevertCommit { sha }) {
                // `git revert` falha com conflito, mas deixa REVERT_HEAD — tratar como
                // sucesso parcial para a UI atualizar e mostrar os conflitos.
                if revert_in_progress(ctx.repo_path()) {
                    return Ok(());
                }
                return Err(e);
            }
        }
        WriteRequest::CherryPick {
            commit_id,
            commit_ids,
            record_origin,
        } => {
            let shas = resolve_cherry_pick_shas(&commit_id, &commit_ids)?;
            if let Err(e) = ctx.execute_op(&CherryPickCommit {
                shas,
                record_origin,
            }) {
                if cherry_pick_in_progress(ctx.repo_path()) {
                    return Ok(());
                }
                return Err(e);
            }
        }
        WriteRequest::Push => {
            let op = push_upstream_op(ctx)?;
            ctx.execute_op(&op)?;
            sync_local_upstream_ref(ctx, &op)?;
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
        WriteRequest::StashPush {
            message,
            include_untracked,
        } => {
            ctx.execute_op(&StashPush {
                message,
                include_untracked,
            })?;
        }
        WriteRequest::StashApply { index } => {
            let reference = stash_reference(index)?;
            ctx.execute_op(&StashApply { reference })?;
        }
        WriteRequest::StashPop { index } => {
            let reference = stash_reference(index)?;
            ctx.execute_op(&StashPop { reference })?;
        }
        WriteRequest::StashDrop { index } => {
            let reference = stash_reference(index)?;
            ctx.execute_op(&StashDrop { reference })?;
        }
        WriteRequest::CreateTag {
            name,
            commit_id,
            annotated,
            message,
            push_to_remote,
        } => {
            let name = validate_tag_name(&name)?;
            let commit_id =
                validate_git_object_id(&commit_id).map_err(|e| GitError::Git(e.to_string()))?;
            ctx.execute_op(&CreateTag {
                name: name.clone(),
                commit_id,
                annotated,
                message,
            })?;
            if push_to_remote {
                ctx.execute_op(&PushTag {
                    remote: "origin".into(),
                    name,
                })?;
            }
        }
        WriteRequest::DeleteTag { name } => {
            let name = validate_tag_name(&name)?;
            ctx.execute_op(&DeleteTag { name })?;
        }
        WriteRequest::DiscardWorktree { path } => {
            let path = validate_repo_relative_path(git_path_from_display(&path))
                .map_err(|e| GitError::Git(e.to_string()))?;
            ctx.execute_op(&DiscardWorktree { path })?;
        }
        WriteRequest::DiscardWorktreeMany { paths } => {
            let paths = validate_paths(&paths)?;
            ctx.execute_op(&DiscardWorktreeMany { paths })?;
        }
        WriteRequest::DiscardWorktreeAll => {
            ctx.execute_op(&DiscardWorktreeAll)?;
        }
        WriteRequest::RemoveUntracked { path } => {
            let path = validate_repo_relative_path(git_path_from_display(&path))
                .map_err(|e| GitError::Git(e.to_string()))?;
            ctx.execute_op(&RemoveUntracked { path })?;
        }
        WriteRequest::RemoveUntrackedMany { paths } => {
            let paths = validate_paths(&paths)?;
            ctx.execute_op(&RemoveUntrackedMany { paths })?;
        }
        WriteRequest::DiscardHunk { path, patch } => {
            let _path = validate_repo_relative_path(git_path_from_display(&path))
                .map_err(|e| GitError::Git(e.to_string()))?;
            ctx.execute_op(&ApplyReversePatch { patch })?;
        }
        WriteRequest::ResolveConflictSide { path, side } => {
            let path = validate_repo_relative_path(git_path_from_display(&path))
                .map_err(|e| GitError::Git(e.to_string()))?;
            let side = normalize_conflict_side(&side)?;
            let choice = if side == "ours" {
                crate::infrastructure::ConflictSideChoice::Ours
            } else {
                crate::infrastructure::ConflictSideChoice::Theirs
            };
            crate::infrastructure::resolve_conflict_side(ctx.writer(), &path, choice)?;
        }
        WriteRequest::ResolveConflictContent { path, content } => {
            let path = validate_repo_relative_path(git_path_from_display(&path))
                .map_err(|e| GitError::Git(e.to_string()))?;
            if content.contains("<<<<<<<") || content.contains(">>>>>>>") {
                return Err(GitError::Git(
                    "O conteúdo ainda contém marcadores de conflito.".into(),
                ));
            }
            crate::infrastructure::resolve_conflict_content(
                ctx.repo_path(),
                ctx.writer(),
                &path,
                &content,
            )?;
        }
        WriteRequest::AbortRevert => {
            ctx.execute_op(&AbortRevert)?;
        }
        WriteRequest::ContinueRevert => {
            ctx.writer().finish_revert()?;
        }
        WriteRequest::AbortMerge => {
            ctx.execute_op(&AbortMerge)?;
        }
        WriteRequest::ContinueMerge => {
            ctx.writer().finish_merge()?;
        }
        WriteRequest::AbortCherryPick => {
            ctx.execute_op(&AbortCherryPick)?;
        }
        WriteRequest::ContinueCherryPick => {
            ctx.writer().finish_cherry_pick()?;
        }
        WriteRequest::SkipRevert => {
            ctx.execute_op(&SkipRevert)?;
        }
        WriteRequest::SkipCherryPick => {
            ctx.execute_op(&SkipCherryPick)?;
        }
        WriteRequest::Reword {
            commit_id,
            summary,
            body,
            force_push,
        } => {
            let sha =
                validate_git_object_id(&commit_id).map_err(|e| GitError::Git(e.to_string()))?;
            let message = format_reword_message(&summary, body.as_deref());
            execute_reword(ctx.writer(), &sha, &message)?;
            if force_push && reword_target_on_remote(ctx, &sha)? {
                execute_force_push_with_lease(ctx)?;
            }
        }
        WriteRequest::Reset {
            commit_id,
            mode,
            force_push,
        } => {
            let sha =
                validate_git_object_id(&commit_id).map_err(|e| GitError::Git(e.to_string()))?;
            let reset_mode = reset_mode_from_dto(mode);
            if reset_mode == ResetMode::Hard {
                let branch = branch_name_for_backup(ctx.repo_path())?;
                create_backup_ref(ctx.writer(), &branch)?;
                if has_tracked_worktree_changes(ctx)? {
                    ctx.execute_op(&StashPush {
                        message: Some(RESET_HARD_STASH_MSG.into()),
                        include_untracked: false,
                    })?;
                }
            }
            ctx.execute_op(&ResetCommit {
                sha,
                mode: reset_mode,
            })?;
            if force_push && reset_needs_force_push(ctx, &commit_id)? {
                let branch = branch_name_for_backup(ctx.repo_path())?;
                create_backup_ref(ctx.writer(), &branch)?;
                execute_force_push_with_lease(ctx)?;
            }
        }
        WriteRequest::PushForce => {
            let branch = branch_name_for_backup(ctx.repo_path())?;
            create_backup_ref(ctx.writer(), &branch)?;
            execute_force_push_with_lease(ctx)?;
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
    sync_local_upstream_ref(ctx, &plan.push)?;
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

fn gate_revert(ctx: &RepoContext, repo_path: &str) -> Result<Option<String>, GitError> {
    if let Some(msg) = gate_sequencer_idle(repo_path, "reverter")? {
        return Ok(Some(msg));
    }
    let status = ctx.reader().get_status()?;
    if !status.staged.is_empty() || !status.unstaged.is_empty() || !status.untracked.is_empty() {
        return Ok(Some(
            "Working tree com alterações — faça commit, stash ou descarte antes de reverter um commit."
                .into(),
        ));
    }
    Ok(None)
}

fn resolve_cherry_pick_shas(
    commit_id: &Option<String>,
    commit_ids: &[String],
) -> Result<Vec<String>, GitError> {
    let raw: Vec<String> = if !commit_ids.is_empty() {
        commit_ids.to_vec()
    } else if let Some(id) = commit_id {
        vec![id.clone()]
    } else {
        return Err(GitError::Git(
            "Cherry-pick exige pelo menos um commit.".into(),
        ));
    };
    let mut shas = Vec::with_capacity(raw.len());
    for id in raw {
        shas.push(validate_git_object_id(&id).map_err(|e| GitError::Git(e.to_string()))?);
    }
    Ok(shas)
}

fn gate_cherry_pick_shas(
    ctx: &RepoContext,
    repo_path: &str,
    shas: &[String],
) -> Result<Option<String>, GitError> {
    let anchor = shas.first().map(String::as_str).unwrap_or("");
    if let Some(msg) = gate_cherry_pick(ctx, repo_path, anchor)? {
        return Ok(Some(msg));
    }
    for sha in shas {
        if let Some(msg) = gate_cherry_pick_merge(ctx, sha)? {
            return Ok(Some(msg));
        }
        if let Some(msg) = gate_not_head_commit(repo_path, sha, "cherry-pick")? {
            return Ok(Some(msg));
        }
        if let Some(msg) = gate_cherry_pick_foreign(ctx, sha)? {
            return Ok(Some(msg));
        }
    }
    Ok(None)
}

fn cherry_pick_description(shas: &[String], record_origin: bool) -> String {
    let mut desc = if shas.len() > 1 {
        format!(
            "Aplica {} commits no topo da branch atual (do mais antigo ao mais recente).",
            shas.len()
        )
    } else {
        "Aplica as alterações do commit selecionado no topo da branch atual.".into()
    };
    if record_origin {
        desc.push_str(" Registra a origem de cada commit na mensagem (-x).");
    }
    desc
}

fn gate_cherry_pick(
    ctx: &RepoContext,
    repo_path: &str,
    _sha: &str,
) -> Result<Option<String>, GitError> {
    if let Some(msg) = gate_sequencer_idle(repo_path, "cherry-pick")? {
        return Ok(Some(msg));
    }
    let status = ctx.reader().get_status()?;
    if !status.staged.is_empty() || !status.unstaged.is_empty() || !status.untracked.is_empty() {
        return Ok(Some(
            "Working tree com alterações — faça commit, stash ou descarte antes do cherry-pick."
                .into(),
        ));
    }
    Ok(None)
}

fn gate_sequencer_idle(repo_path: &str, action: &str) -> Result<Option<String>, GitError> {
    let git_dir = std::path::Path::new(repo_path).join(".git");
    if git_dir.join("CHERRY_PICK_HEAD").exists() {
        return Ok(Some(format!(
            "Há um cherry-pick em andamento — finalize com «Continuar cherry-pick» ou cancele antes de {action}."
        )));
    }
    if git_dir.join("REVERT_HEAD").exists() {
        return Ok(Some(format!(
            "Há um revert em andamento — conclua ou aborte antes de {action}."
        )));
    }
    if git_dir.join("MERGE_HEAD").exists() {
        return Ok(Some(format!(
            "Há um merge em andamento — conclua ou aborte antes de {action}."
        )));
    }
    Ok(None)
}

fn gate_not_head_commit(
    repo_path: &str,
    sha: &str,
    action: &str,
) -> Result<Option<String>, GitError> {
    if is_head_commit(repo_path, sha)? {
        return Ok(Some(format!(
            "Este é o último commit (HEAD) — não é possível fazer {action} dele."
        )));
    }
    Ok(None)
}

fn gate_cherry_pick_merge(ctx: &RepoContext, sha: &str) -> Result<Option<String>, GitError> {
    let repo = Repository::discover(ctx.repo_path()).map_err(|e| GitError::Io(e.to_string()))?;
    let oid = git2::Oid::from_str(sha).map_err(|e| GitError::Git(e.to_string()))?;
    let commit = repo
        .find_commit(oid)
        .map_err(|_| GitError::Git("Commit não encontrado no repositório.".into()))?;
    if commit.parent_count() > 1 {
        let summary = commit.summary().unwrap_or("merge");
        return Ok(Some(format!(
            "«{summary}» ({:.7}) é commit de merge — cherry-pick exige escolher qual lado manter \
             (git cherry-pick -m), operação avançada fora do MVP. Desmarque merges e escolha só \
             commits normais.",
            oid
        )));
    }
    Ok(None)
}

/// Cherry-pick só faz sentido para commits fora do histórico da branch atual.
fn gate_cherry_pick_foreign(ctx: &RepoContext, sha: &str) -> Result<Option<String>, GitError> {
    if is_ancestor_of_head(ctx.writer(), sha)? {
        return Ok(Some(
            "Este commit já faz parte do histórico da branch atual — use cherry-pick para trazer \
             commits de outras branches que ainda não estão aqui."
                .into(),
        ));
    }
    Ok(None)
}

fn is_ancestor_of_head(cli: &SafeGitCli, sha: &str) -> Result<bool, GitError> {
    let op = GitCommand {
        args: vec![
            "merge-base".into(),
            "--is-ancestor".into(),
            sha.into(),
            "HEAD".into(),
        ],
    };
    cli.run_bool(&op)
}

fn gate_abort_revert(repo_path: &str) -> Result<Option<String>, GitError> {
    if std::path::Path::new(repo_path)
        .join(".git/REVERT_HEAD")
        .exists()
    {
        Ok(None)
    } else {
        Ok(Some("Não há revert em andamento.".into()))
    }
}

fn gate_skip_revert(repo_path: &str) -> Result<Option<String>, GitError> {
    if std::path::Path::new(repo_path)
        .join(".git/REVERT_HEAD")
        .exists()
    {
        Ok(None)
    } else {
        Ok(Some("Não há revert em andamento para pular.".into()))
    }
}

fn gate_skip_cherry_pick(repo_path: &str) -> Result<Option<String>, GitError> {
    if std::path::Path::new(repo_path)
        .join(".git/CHERRY_PICK_HEAD")
        .exists()
    {
        Ok(None)
    } else {
        Ok(Some("Não há cherry-pick em andamento para pular.".into()))
    }
}

fn gate_continue_revert(repo_path: &str, ctx: &RepoContext) -> Result<Option<String>, GitError> {
    if !std::path::Path::new(repo_path)
        .join(".git/REVERT_HEAD")
        .exists()
    {
        return Ok(Some("Não há revert em andamento.".into()));
    }
    let status = ctx.reader().get_status()?;
    if status
        .staged
        .iter()
        .chain(status.unstaged.iter())
        .any(|f| f.kind == FileChangeKind::Conflicted)
    {
        return Ok(Some(
            "Ainda há conflitos não resolvidos — resolva os arquivos antes de continuar o revert."
                .into(),
        ));
    }
    Ok(None)
}

fn gate_abort_merge(repo_path: &str) -> Result<Option<String>, GitError> {
    if std::path::Path::new(repo_path)
        .join(".git/MERGE_HEAD")
        .exists()
    {
        Ok(None)
    } else {
        Ok(Some("Não há merge em andamento.".into()))
    }
}

fn gate_continue_merge(repo_path: &str, ctx: &RepoContext) -> Result<Option<String>, GitError> {
    if !std::path::Path::new(repo_path)
        .join(".git/MERGE_HEAD")
        .exists()
    {
        return Ok(Some("Não há merge em andamento.".into()));
    }
    let status = ctx.reader().get_status()?;
    if status
        .staged
        .iter()
        .chain(status.unstaged.iter())
        .any(|f| f.kind == FileChangeKind::Conflicted)
    {
        return Ok(Some(
            "Ainda há conflitos não resolvidos — resolva os arquivos antes de continuar o merge."
                .into(),
        ));
    }
    Ok(None)
}

fn gate_abort_cherry_pick(repo_path: &str) -> Result<Option<String>, GitError> {
    if std::path::Path::new(repo_path)
        .join(".git/CHERRY_PICK_HEAD")
        .exists()
    {
        Ok(None)
    } else {
        Ok(Some("Não há cherry-pick em andamento.".into()))
    }
}

fn gate_continue_cherry_pick(repo_path: &str, ctx: &RepoContext) -> Result<Option<String>, GitError> {
    if !std::path::Path::new(repo_path)
        .join(".git/CHERRY_PICK_HEAD")
        .exists()
    {
        return Ok(Some("Não há cherry-pick em andamento.".into()));
    }
    let status = ctx.reader().get_status()?;
    if status
        .staged
        .iter()
        .chain(status.unstaged.iter())
        .any(|f| f.kind == FileChangeKind::Conflicted)
    {
        return Ok(Some(
            "Ainda há conflitos não resolvidos — resolva os arquivos antes de continuar o cherry-pick."
                .into(),
        ));
    }
    Ok(None)
}

fn format_reword_message(summary: &str, body: Option<&str>) -> String {
    let summary = summary.trim();
    match body.map(str::trim).filter(|b| !b.is_empty()) {
        Some(b) => format!("{summary}\n\n{b}"),
        None => summary.to_string(),
    }
}

fn gate_clean_worktree(ctx: &RepoContext) -> Result<Option<String>, GitError> {
    if let Some(op) = &ctx.reader().get_status()?.operation_in_progress {
        return Ok(Some(format!(
            "{} Conclua ou aborte a operação antes de continuar.",
            op.message
        )));
    }
    let status = ctx.reader().get_status()?;
    if !status.staged.is_empty() || !status.unstaged.is_empty() || !status.untracked.is_empty() {
        return Ok(Some(
            "Working tree com alterações — faça commit, stash ou descarte antes de continuar."
                .into(),
        ));
    }
    Ok(None)
}

fn is_head_commit(repo_path: &str, sha: &str) -> Result<bool, GitError> {
    let head = SafeGitCli::new(repo_path).run(&crate::application::GitCommand {
        args: vec!["rev-parse".into(), "HEAD".into()],
    })?;
    Ok(head.trim().eq_ignore_ascii_case(sha))
}

fn gate_reword(
    ctx: &RepoContext,
    repo_path: &str,
    sha: &str,
    summary: &str,
    force_push: bool,
) -> Result<Option<String>, GitError> {
    if summary.trim().is_empty() {
        return Ok(Some("A mensagem do commit não pode ficar vazia.".into()));
    }
    if let Some(msg) = gate_clean_worktree(ctx)? {
        return Ok(Some(msg));
    }
    if is_head_commit(repo_path, sha)? {
        return Ok(Some(
            "Este é o último commit — use Amend em «Alterações locais» para alterar a mensagem."
                .into(),
        ));
    }
    let repo = Repository::discover(repo_path).map_err(|e| GitError::Io(e.to_string()))?;
    let oid = git2::Oid::from_str(sha).map_err(|e| GitError::Git(e.to_string()))?;
    let commit = repo
        .find_commit(oid)
        .map_err(|_| GitError::Git("Commit não encontrado no repositório.".into()))?;
    if commit.parent_count() > 1 {
        return Ok(Some(
            "Commit de merge — reword exige operação avançada fora do MVP.".into(),
        ));
    }
    // Reword reaplica com cherry-pick linear; merges no caminho trazem
    // commits laterais e costumam gerar conflito (ex.: «could not apply …»).
    if range_has_merge_commits(ctx.writer(), sha)? {
        return Ok(Some(
            "Há merges no histórico após este commit — o Trilho ainda não reaplica \
             merges no reword. Escolha um commit mais recente (após o último merge) \
             ou reescreva a mensagem só em histórico linear."
                .into(),
        ));
    }
    let on_remote = reword_target_on_remote(ctx, sha)?;
    if on_remote {
        if !force_push {
            return Ok(Some(
                "Este commit já foi enviado ao remoto — confirme o push forçado para concluir o reword."
                    .into(),
            ));
        }
        if let Some(msg) = gate_force_push_upstream(ctx)? {
            return Ok(Some(msg));
        }
    }
    Ok(None)
}

fn reword_target_on_remote(ctx: &RepoContext, sha: &str) -> Result<bool, GitError> {
    let sync = ctx.reader().get_sync_info()?;
    let Some(upstream) = sync.upstream.as_deref() else {
        return Ok(false);
    };
    is_commit_on_remote(ctx.writer(), upstream, sha)
}

/// `true` se existir algum merge em `sha..HEAD` (histórico não linear).
fn range_has_merge_commits(cli: &dyn GitWriter, sha: &str) -> Result<bool, GitError> {
    let out = cli.run(&GitCommand {
        args: vec![
            "rev-list".into(),
            "--merges".into(),
            "--count".into(),
            format!("{sha}..HEAD"),
        ],
    })?;
    let count: u64 = out.trim().parse().unwrap_or(0);
    Ok(count > 0)
}

fn gate_force_push_upstream(ctx: &RepoContext) -> Result<Option<String>, GitError> {
    let sync = ctx.reader().get_sync_info()?;
    if sync.upstream.is_none() {
        return Ok(Some(
            "Branch sem upstream — configure o remoto antes do push forçado.".into(),
        ));
    }
    Ok(None)
}

fn reset_mode_from_dto(mode: ResetModeDto) -> ResetMode {
    match mode {
        ResetModeDto::Soft => ResetMode::Soft,
        ResetModeDto::Mixed => ResetMode::Mixed,
        ResetModeDto::Hard => ResetMode::Hard,
    }
}

const RESET_HARD_STASH_MSG: &str = "trilho: backup antes de reset --hard";

fn branch_name_for_backup(repo_path: &str) -> Result<String, GitError> {
    repo_info(repo_path)?
        .branch
        .ok_or_else(|| GitError::Git("Branch atual não identificada.".into()))
}

fn has_tracked_worktree_changes(ctx: &RepoContext) -> Result<bool, GitError> {
    let status = ctx.reader().get_status()?;
    Ok(!status.staged.is_empty() || !status.unstaged.is_empty())
}

fn is_likely_protected_branch(branch: &str) -> bool {
    matches!(
        branch,
        "main" | "master" | "develop" | "production" | "release"
    )
}

fn remote_only_commit_short_ids(ctx: &RepoContext) -> Result<Vec<String>, GitError> {
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

/// `true` se o upstream aponta para um commit estritamente posterior ao alvo do reset.
fn reset_needs_force_push(ctx: &RepoContext, sha: &str) -> Result<bool, GitError> {
    let sync = ctx.reader().get_sync_info()?;
    let Some(upstream) = sync.upstream else {
        return Ok(false);
    };
    let upstream_sha = ctx
        .writer()
        .run(&GitCommand {
            args: vec!["rev-parse".into(), upstream.clone()],
        })?
        .trim()
        .to_string();
    if upstream_sha.eq_ignore_ascii_case(sha) {
        return Ok(false);
    }
    is_commit_on_remote(ctx.writer(), &upstream, sha)
}

fn gate_reset(
    ctx: &RepoContext,
    repo_path: &str,
    sha: &str,
    _mode: ResetMode,
    force_push: bool,
    needs_force: bool,
) -> Result<Option<String>, GitError> {
    let info = repo_info(repo_path)?;
    if info.is_detached {
        return Ok(Some(
            "Repositório em detached HEAD — troque para uma branch antes de resetar.".into(),
        ));
    }
    if let Some(msg) = gate_sequencer_idle(repo_path, "resetar")? {
        return Ok(Some(msg));
    }
    if is_head_commit(repo_path, sha)? {
        return Ok(Some(
            "Este já é o último commit (HEAD) — escolha um commit anterior para resetar.".into(),
        ));
    }
    if !is_ancestor_of_head(ctx.writer(), sha)? {
        return Ok(Some(
            "Este commit não faz parte do histórico atual da branch — só é possível resetar \
             para commits ancestrais do HEAD."
                .into(),
        ));
    }
    if needs_force && force_push {
        if let Some(msg) = gate_force_push_upstream(ctx)? {
            return Ok(Some(msg));
        }
    }
    Ok(None)
}

fn gate_force_push_standalone(ctx: &RepoContext) -> Result<Option<String>, GitError> {
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
fn push_upstream_op(ctx: &RepoContext) -> Result<PushSetUpstream, GitError> {
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
fn sync_local_upstream_ref(ctx: &RepoContext, op: &PushSetUpstream) -> Result<(), GitError> {
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

fn gate_pull(ctx: &RepoContext) -> Result<Option<String>, GitError> {
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
fn execute_force_push_with_lease(ctx: &RepoContext) -> Result<(), GitError> {
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

fn gate_stash(ctx: &RepoContext, include_untracked: bool) -> Result<Option<String>, GitError> {
    let status = ctx.reader().get_status()?;
    let has_tracked = !status.staged.is_empty() || !status.unstaged.is_empty();
    let has_untracked = !status.untracked.is_empty();
    if !has_tracked && (!include_untracked || !has_untracked) {
        return Ok(Some("Não há alterações para guardar no stash.".into()));
    }
    Ok(None)
}

fn stash_effect_description(
    ctx: &RepoContext,
    include_untracked: bool,
) -> Result<String, GitError> {
    let status = ctx.reader().get_status()?;
    let mut parts = Vec::new();
    if !status.staged.is_empty() {
        parts.push(format!("{} em stage", status.staged.len()));
    }
    if !status.unstaged.is_empty() {
        parts.push(format!("{} não staged", status.unstaged.len()));
    }
    if include_untracked && !status.untracked.is_empty() {
        parts.push(format!("{} não rastreados", status.untracked.len()));
    }
    let detail = if parts.is_empty() {
        "alterações rastreadas".to_string()
    } else {
        parts.join(", ")
    };
    Ok(format!(
        "Guarda {detail} em uma pilha temporária (stash). A working tree ficará limpa."
    ))
}

fn gate_stash_restore(ctx: &RepoContext) -> Result<Option<String>, GitError> {
    let status = ctx.reader().get_status()?;
    if !status.staged.is_empty() || !status.unstaged.is_empty() || !status.untracked.is_empty() {
        return Ok(Some(
            "Working tree com alterações — commit, stash ou descarte antes de reaplicar.".into(),
        ));
    }
    Ok(None)
}

fn gate_stash_index(repo_path: &str, index: usize) -> Result<Option<String>, GitError> {
    let stashes = list_stashes(repo_path)?;
    if stashes.iter().any(|s| s.index == index) {
        Ok(None)
    } else {
        Ok(Some("Stash não encontrado — a pilha pode ter mudado.".into()))
    }
}

fn stash_label(repo_path: &str, index: usize) -> String {
    list_stashes(repo_path)
        .ok()
        .and_then(|ss| {
            ss.into_iter()
                .find(|s| s.index == index)
                .map(|s| s.message)
        })
        .unwrap_or_else(|| format!("stash@{{{index}}}"))
}

fn gate_tag_exists(repo_path: &str, name: &str) -> Result<Option<String>, GitError> {
    let cli = SafeGitCli::new(repo_path);
    let op = crate::application::GitCommand {
        args: vec![
            "show-ref".into(),
            "--verify".into(),
            "--quiet".into(),
            format!("refs/tags/{name}"),
        ],
    };
    if cli.run_bool(&op)? {
        Ok(Some(format!(
            "Já existe uma tag «{name}» neste repositório."
        )))
    } else {
        Ok(None)
    }
}

fn gate_push_tag(ctx: &RepoContext) -> Result<Option<String>, GitError> {
    let info = repo_info(ctx.repo_path())?;
    if !info.has_remote {
        return Ok(Some(
            "Não há remoto configurado — desmarque «Enviar ao remoto».".into(),
        ));
    }
    Ok(None)
}

fn gate_tag_missing(repo_path: &str, name: &str) -> Result<Option<String>, GitError> {
    match gate_tag_exists(repo_path, name)? {
        Some(_) => Ok(None),
        None => Ok(Some(format!("Tag «{name}» não encontrada."))),
    }
}

fn status_path_matches(display: &str, path: &str) -> bool {
    git_path_from_display(display) == path
}

fn revert_in_progress(repo_path: &str) -> bool {
    std::path::Path::new(repo_path)
        .join(".git/REVERT_HEAD")
        .exists()
}

fn cherry_pick_in_progress(repo_path: &str) -> bool {
    std::path::Path::new(repo_path)
        .join(".git/CHERRY_PICK_HEAD")
        .exists()
}

fn normalize_conflict_side(side: &str) -> Result<&'static str, GitError> {
    match side.trim().to_ascii_lowercase().as_str() {
        "ours" => Ok("ours"),
        "theirs" => Ok("theirs"),
        _ => Err(GitError::Git(
            "Lado inválido — use «ours» (atual) ou «theirs» (entrando).".into(),
        )),
    }
}

fn gate_resolve_conflict(ctx: &RepoContext, path: &str) -> Result<Option<String>, GitError> {
    let status = ctx.reader().get_status()?;
    let conflicted = status
        .staged
        .iter()
        .chain(status.unstaged.iter())
        .any(|f| f.path == path && f.kind == FileChangeKind::Conflicted);
    if !conflicted {
        return Ok(Some(format!(
            "«{path}» não está marcado como conflito no status."
        )));
    }
    Ok(None)
}

fn gate_discard_blocked(ctx: &RepoContext) -> Result<Option<String>, GitError> {
    let status = ctx.reader().get_status()?;
    if let Some(op) = &status.operation_in_progress {
        let abort = match op.kind {
            InProgressKind::Revert => "«Abortar revert»",
            InProgressKind::Merge => "«Abortar merge»",
            InProgressKind::CherryPick => "«Abortar cherry-pick»",
        };
        return Ok(Some(format!(
            "{} Descartar arquivos não cancela a operação — use {abort}.",
            op.message
        )));
    }
    let has_conflicts = status
        .staged
        .iter()
        .chain(status.unstaged.iter())
        .any(|f| f.kind == FileChangeKind::Conflicted);
    if has_conflicts {
        return Ok(Some(
            "Arquivos em conflito — edite para resolver ou aborte a operação em andamento."
                .into(),
        ));
    }
    Ok(None)
}

fn gate_discard_worktree(
    ctx: &RepoContext,
    paths: &[String],
) -> Result<Option<String>, GitError> {
    if let Some(msg) = gate_discard_blocked(ctx)? {
        return Ok(Some(msg));
    }
    let status = ctx.reader().get_status()?;
    for path in paths {
        let in_unstaged = status
            .unstaged
            .iter()
            .any(|f| status_path_matches(&f.path, path));
        if !in_unstaged {
            return Ok(Some(format!(
                "«{path}» não tem alterações fora do stage para descartar — use unstage se estiver só em staging."
            )));
        }
    }
    Ok(None)
}

fn gate_discard_worktree_all(ctx: &RepoContext) -> Result<Option<String>, GitError> {
    if let Some(msg) = gate_discard_blocked(ctx)? {
        return Ok(Some(msg));
    }
    let status = ctx.reader().get_status()?;
    if status.unstaged.is_empty() {
        return Ok(Some(
            "Não há alterações fora do stage para descartar.".into(),
        ));
    }
    Ok(None)
}

fn gate_remove_untracked(
    ctx: &RepoContext,
    paths: &[String],
) -> Result<Option<String>, GitError> {
    let status = ctx.reader().get_status()?;
    for path in paths {
        let in_untracked = status
            .untracked
            .iter()
            .any(|f| status_path_matches(&f.path, path));
        if !in_untracked {
            return Ok(Some(format!(
                "«{path}» não é um arquivo não rastreado — use descartar para alterações em arquivos rastreados."
            )));
        }
    }
    Ok(None)
}

fn gate_reverse_patch(ctx: &RepoContext, patch: &str) -> Result<Option<String>, GitError> {
    if patch.trim().is_empty() {
        return Ok(Some("Nenhum trecho selecionado para descartar.".into()));
    }
    let cmd = crate::application::GitCommand {
        args: vec![
            "apply".into(),
            "--reverse".into(),
            "--check".into(),
            "-".into(),
        ],
    };
    match ctx.writer().run_with_stdin(&cmd, patch.as_bytes()) {
        Ok(_) => Ok(None),
        Err(e) => Ok(Some(format!(
            "O trecho não pode ser revertido automaticamente: {e}"
        ))),
    }
}

fn preview_create_tag(
    ctx: &RepoContext,
    repo_path: &str,
    name: &str,
    commit_id: &str,
    annotated: bool,
    message: Option<&str>,
    push_to_remote: bool,
) -> Result<(Vec<String>, String, Option<String>), GitError> {
    let name = match validate_tag_name(name) {
        Ok(n) => n,
        Err(GitError::Git(msg)) => return Ok((vec![], String::new(), Some(msg))),
        Err(e) => return Err(e),
    };
    let commit_id = match validate_git_object_id(commit_id) {
        Ok(id) => id,
        Err(GitError::Git(msg)) => return Ok((vec![], String::new(), Some(msg))),
        Err(e) => return Err(e),
    };

    let mut blocked = gate_tag_exists(repo_path, &name)?;
    if annotated && message.map(str::trim).filter(|m| !m.is_empty()).is_none() {
        blocked = blocked.or(Some(
            "Tags anotadas precisam de uma mensagem.".into(),
        ));
    }

    let tag_op = CreateTag {
        name: name.clone(),
        commit_id: commit_id.clone(),
        annotated,
        message: message.map(str::to_string),
    };
    let mut commands = ctx.preview_op(&tag_op);
    let kind = if annotated { "anotada" } else { "leve" };
    let short: String = commit_id.chars().take(7).collect();
    let mut description = format!("Cria tag {kind} «{name}» no commit {short}.");

    if push_to_remote {
        blocked = blocked.or(gate_push_tag(ctx)?);
        let push_op = PushTag {
            remote: "origin".into(),
            name: name.clone(),
        };
        commands.extend(ctx.preview_op(&push_op));
        description.push_str(" Em seguida envia a tag ao remoto origin.");
    }

    Ok((commands, description, blocked))
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
    fn reset_para_head_e_bloqueado() {
        let dir = std::env::temp_dir().join(format!("trilho-rsthd-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        init_repo_with_commit(&dir);
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
            &WriteRequest::Reset {
                commit_id: sha,
                mode: crate::domain::ResetModeDto::Mixed,
                force_push: false,
            },
        )
        .expect("preview");
        assert!(preview.blocked.is_some());
        assert!(preview.blocked.unwrap().contains("HEAD"));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn reword_com_merge_no_caminho_e_bloqueado() {
        let dir = std::env::temp_dir().join(format!("trilho-rwmrg-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        init_repo_with_commit(&dir);
        // Commit A (alvo do reword) → branch feat → merge → HEAD
        std::process::Command::new("git")
            .args(["commit", "--allow-empty", "-m", "alvo reword"])
            .current_dir(&dir)
            .output()
            .unwrap();
        let alvo = std::process::Command::new("git")
            .args(["rev-parse", "HEAD"])
            .current_dir(&dir)
            .output()
            .unwrap();
        let alvo = String::from_utf8_lossy(&alvo.stdout).trim().to_string();
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

        let ctx = RepoContext::open(&dir.to_string_lossy()).expect("ctx");
        let preview = preview_write(
            &ctx,
            ctx.repo_path(),
            &WriteRequest::Reword {
                commit_id: alvo,
                summary: "nova mensagem".into(),
                body: None,
                force_push: false,
            },
        )
        .expect("preview");
        assert!(
            preview.blocked.is_some(),
            "reword com merge no caminho deve bloquear"
        );
        assert!(
            preview
                .blocked
                .as_ref()
                .unwrap()
                .to_lowercase()
                .contains("merge"),
            "deve mencionar merges: {:?}",
            preview.blocked
        );
        let _ = std::fs::remove_dir_all(&dir);
    }

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

    #[test]
    fn cherry_pick_de_merge_e_bloqueado() {
        let dir = std::env::temp_dir().join(format!("trilho-cpmrg-{}", std::process::id()));
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
            &WriteRequest::CherryPick {
                commit_id: Some(sha),
                commit_ids: vec![],
                record_origin: false,
            },
        )
        .expect("preview");
        assert!(preview.blocked.is_some());
        assert!(preview.blocked.unwrap().to_lowercase().contains("merge"));
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// Revert com working tree suja deve bloquear no preview (evita conflito parcial).
    #[test]
    fn revert_com_working_tree_suja_e_bloqueado() {
        let dir = std::env::temp_dir().join(format!("trilho-revwt-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        init_repo_with_commit(&dir);
        std::fs::write(dir.join("f.txt"), "dirty").unwrap();
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
            "revert com WT suja deve vir bloqueado"
        );
        assert!(preview
            .blocked
            .unwrap()
            .to_lowercase()
            .contains("working tree"));
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

    #[test]
    fn stash_sem_alteracoes_bloqueado() {
        let dir = std::env::temp_dir().join(format!("trilho-stash-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        init_repo_with_commit(&dir);

        let ctx = RepoContext::open(&dir.to_string_lossy()).expect("ctx");
        let preview = preview_write(
            &ctx,
            ctx.repo_path(),
            &WriteRequest::StashPush {
                message: None,
                include_untracked: false,
            },
        )
        .expect("preview");
        assert!(preview.blocked.is_some());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn stash_com_alteracoes_executa() {
        let dir = std::env::temp_dir().join(format!("trilho-stash-ok-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        init_repo_with_commit(&dir);
        std::fs::write(dir.join("f.txt"), "changed").unwrap();

        let ctx = RepoContext::open(&dir.to_string_lossy()).expect("ctx");
        let preview = preview_write(
            &ctx,
            ctx.repo_path(),
            &WriteRequest::StashPush {
                message: Some("wip".into()),
                include_untracked: false,
            },
        )
        .expect("preview");
        assert!(preview.blocked.is_none(), "{:?}", preview.blocked);
        assert!(preview.commands.iter().any(|c| c.contains("stash")));

        execute_write(
            &ctx,
            WriteRequest::StashPush {
                message: Some("wip".into()),
                include_untracked: false,
            },
        )
        .expect("stash");

        let status = ctx.reader().get_status().expect("status");
        assert!(status.staged.is_empty() && status.unstaged.is_empty());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn create_tag_leve_executa() {
        let dir = std::env::temp_dir().join(format!("trilho-tag-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        init_repo_with_commit(&dir);
        let sha = std::process::Command::new("git")
            .args(["rev-parse", "HEAD"])
            .current_dir(&dir)
            .output()
            .unwrap();
        let commit_id = String::from_utf8(sha.stdout).unwrap().trim().to_string();

        let ctx = RepoContext::open(&dir.to_string_lossy()).expect("ctx");
        let preview = preview_write(
            &ctx,
            ctx.repo_path(),
            &WriteRequest::CreateTag {
                name: "v1.0".into(),
                commit_id: commit_id.clone(),
                annotated: false,
                message: None,
                push_to_remote: false,
            },
        )
        .expect("preview");
        assert!(preview.blocked.is_none(), "{:?}", preview.blocked);
        assert!(preview.commands.iter().any(|c| c.contains("tag")));

        execute_write(
            &ctx,
            WriteRequest::CreateTag {
                name: "v1.0".into(),
                commit_id,
                annotated: false,
                message: None,
                push_to_remote: false,
            },
        )
        .expect("tag");

        let out = std::process::Command::new("git")
            .args(["tag", "-l", "v1.0"])
            .current_dir(&dir)
            .output()
            .unwrap();
        assert!(String::from_utf8_lossy(&out.stdout).contains("v1.0"));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn create_tag_duplicata_bloqueada() {
        let dir = std::env::temp_dir().join(format!("trilho-tag-dup-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        init_repo_with_commit(&dir);
        std::process::Command::new("git")
            .args(["tag", "v1"])
            .current_dir(&dir)
            .output()
            .unwrap();
        let sha = std::process::Command::new("git")
            .args(["rev-parse", "HEAD"])
            .current_dir(&dir)
            .output()
            .unwrap();
        let commit_id = String::from_utf8(sha.stdout).unwrap().trim().to_string();

        let ctx = RepoContext::open(&dir.to_string_lossy()).expect("ctx");
        let preview = preview_write(
            &ctx,
            ctx.repo_path(),
            &WriteRequest::CreateTag {
                name: "v1".into(),
                commit_id,
                annotated: false,
                message: None,
                push_to_remote: false,
            },
        )
        .expect("preview");
        assert!(preview.blocked.is_some());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn delete_tag_executa() {
        let dir = std::env::temp_dir().join(format!("trilho-tag-del-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        init_repo_with_commit(&dir);
        std::process::Command::new("git")
            .args(["tag", "v-del"])
            .current_dir(&dir)
            .output()
            .unwrap();

        let ctx = RepoContext::open(&dir.to_string_lossy()).expect("ctx");
        let preview = preview_write(
            &ctx,
            ctx.repo_path(),
            &WriteRequest::DeleteTag {
                name: "v-del".into(),
            },
        )
        .expect("preview");
        assert!(preview.blocked.is_none());
        execute_write(
            &ctx,
            WriteRequest::DeleteTag {
                name: "v-del".into(),
            },
        )
        .expect("delete");

        let out = std::process::Command::new("git")
            .args(["tag", "-l"])
            .current_dir(&dir)
            .output()
            .unwrap();
        assert!(String::from_utf8_lossy(&out.stdout).trim().is_empty());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn discard_worktree_restaura_arquivo() {
        let dir = std::env::temp_dir().join(format!("trilho-discard-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        init_repo_with_commit(&dir);
        std::fs::write(dir.join("f.txt"), "changed").unwrap();

        let ctx = RepoContext::open(&dir.to_string_lossy()).expect("ctx");
        let preview = preview_write(
            &ctx,
            ctx.repo_path(),
            &WriteRequest::DiscardWorktree {
                path: "f.txt".into(),
            },
        )
        .expect("preview");
        assert!(preview.blocked.is_none(), "{:?}", preview.blocked);

        execute_write(
            &ctx,
            WriteRequest::DiscardWorktree {
                path: "f.txt".into(),
            },
        )
        .expect("discard");

        let content = std::fs::read_to_string(dir.join("f.txt")).unwrap();
        assert_eq!(content, "x");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn discard_all_bloqueado_com_revert_em_andamento() {
        let dir = std::env::temp_dir().join(format!("trilho-discrev-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        init_repo_with_commit(&dir);
        std::fs::write(dir.join("f.txt"), "dirty").unwrap();
        std::fs::write(dir.join(".git/REVERT_HEAD"), "abc\n").unwrap();

        let ctx = RepoContext::open(&dir.to_string_lossy()).expect("ctx");
        let preview = preview_write(&ctx, ctx.repo_path(), &WriteRequest::DiscardWorktreeAll)
            .expect("preview");
        assert!(
            preview.blocked.is_some(),
            "descartar tudo deve bloquear com revert pendente"
        );
        assert!(preview
            .blocked
            .unwrap()
            .contains("Abortar revert"));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn reset_hard_com_wt_suja_nao_bloqueia_e_inclui_stash() {
        let dir = std::env::temp_dir().join(format!("trilho-rsthrd-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        init_repo_with_commit(&dir);
        std::process::Command::new("git")
            .args(["commit", "--allow-empty", "-m", "second"])
            .current_dir(&dir)
            .output()
            .unwrap();
        let parent = std::process::Command::new("git")
            .args(["rev-parse", "HEAD~1"])
            .current_dir(&dir)
            .output()
            .unwrap();
        let parent = String::from_utf8_lossy(&parent.stdout)
            .trim()
            .to_string();
        std::fs::write(dir.join("f.txt"), "dirty").unwrap();

        let ctx = RepoContext::open(&dir.to_string_lossy()).expect("ctx");
        let preview = preview_write(
            &ctx,
            ctx.repo_path(),
            &WriteRequest::Reset {
                commit_id: parent,
                mode: crate::domain::ResetModeDto::Hard,
                force_push: false,
            },
        )
        .expect("preview");
        assert!(
            preview.blocked.is_none(),
            "hard reset com WT suja deve permitir (stash automático): {:?}",
            preview.blocked
        );
        assert!(
            preview.commands.iter().any(|c| c.contains("stash push")),
            "preview deve incluir stash: {:?}",
            preview.commands
        );
        assert!(
            preview
                .commands
                .iter()
                .any(|c| c.contains("refs/trilho/backup")),
            "preview deve incluir backup ref: {:?}",
            preview.commands
        );
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn push_force_bloqueado_sem_remoto_a_frente() {
        let dir = std::env::temp_dir().join(format!("trilho-pfblk-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        init_repo_with_commit(&dir);

        let ctx = RepoContext::open(&dir.to_string_lossy()).expect("ctx");
        let preview =
            preview_write(&ctx, ctx.repo_path(), &WriteRequest::PushForce).expect("preview");
        assert!(
            preview.blocked.is_some(),
            "push force sem upstream/behind deve bloquear"
        );
        let _ = std::fs::remove_dir_all(&dir);
    }
}
