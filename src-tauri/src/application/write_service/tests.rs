use super::sync_remote::{plan_publish, PlanoPublicacao, PublishPlan};
use super::{execute_write, preview_write};
use crate::domain::{caminho_git_do_rotulo, rotulo_renomeacao};
use crate::application::RepoContext;
use crate::domain::WriteRequest;

#[test]
fn git_path_from_rename_display() {
    assert_eq!(caminho_git_do_rotulo("a.ts → b.ts"), "b.ts");
    assert_eq!(caminho_git_do_rotulo("plain.ts"), "plain.ts");
}

/// Regressão: renomeação são duas entradas no índice. Desfazer o stage citando
/// só o destino deixava a exclusão da origem staged (`D f.txt` + `?? g.txt`).
#[test]
fn unstage_de_renomeacao_limpa_as_duas_entradas() {
    let dir = std::env::temp_dir().join(format!("trilho-unstg-rn-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    init_repo_with_commit(&dir);
    std::process::Command::new("git")
        .args(["mv", "f.txt", "g.txt"])
        .current_dir(&dir)
        .output()
        .unwrap();

    let ctx = RepoContext::open(&dir.to_string_lossy()).expect("ctx");
    let rotulo = rotulo_renomeacao("f.txt", "g.txt");
    let preview = preview_write(
        &ctx,
        ctx.repo_path(),
        &WriteRequest::Unstage {
            path: rotulo.clone(),
        },
    )
    .expect("preview");
    assert!(preview.blocked.is_none(), "{:?}", preview.blocked);
    let joined = preview.commands.join(" ");
    assert!(
        joined.contains("f.txt") && joined.contains("g.txt"),
        "preview deve citar origem e destino: {joined}"
    );

    execute_write(&ctx, WriteRequest::Unstage { path: rotulo }).expect("unstage");

    let saida = std::process::Command::new("git")
        .args(["status", "--porcelain"])
        .current_dir(&dir)
        .output()
        .unwrap();
    let status = String::from_utf8_lossy(&saida.stdout).to_string();
    assert!(
        !status.contains("D  f.txt"),
        "exclusão da origem continuou no índice: {status}"
    );
    assert!(
        status.contains("?? g.txt"),
        "destino deveria ficar como untracked: {status}"
    );
    let _ = std::fs::remove_dir_all(&dir);
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
            mode: crate::domain::ResetMode::Mixed,
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
fn plano_pronto(ctx: &RepoContext, url: &str) -> PublishPlan {
    match plan_publish(ctx, Some(url)).expect("plan") {
        PlanoPublicacao::Pronto(plan) => plan,
        PlanoPublicacao::Bloqueado(motivo) => panic!("publicação bloqueada: {motivo}"),
    }
}

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
    let plan = plano_pronto(&ctx, "git@github.com:certa/repo.git");
    let step = plan.remote_step.expect("deve ter passo de remoto");
    let args = step.command().args;
    assert!(args.contains(&"set-url".to_string()), "args: {args:?}");
    assert!(args.contains(&"git@github.com:certa/repo.git".to_string()));

    // Mesma URL → sem passo de remoto (só push).
    let plan = plano_pronto(&ctx, "git@github.com:errada/repo.git");
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
            mode: crate::domain::ResetMode::Hard,
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
