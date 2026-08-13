//! B-04 — contratos de segurança verificáveis sem WebView desktop.
//! Complementa o E2E Playwright (RF-08 na UI) e os testes unitários
//! de `write_auth` / `worktree_file` / `git_cli`.

#[cfg(test)]
mod tests {
    use crate::infrastructure::{defensive_config_args, save_worktree_file};
    use std::fs;
    use std::path::PathBuf;
    use std::process::Command;

    fn init_repo(dir: &std::path::Path) {
        let _ = fs::remove_dir_all(dir);
        fs::create_dir_all(dir).unwrap();
        for args in [
            vec!["init"],
            vec!["config", "user.email", "t@t.com"],
            vec!["config", "user.name", "T"],
            vec!["commit", "--allow-empty", "-m", "init"],
        ] {
            let st = Command::new("git")
                .args(&args)
                .current_dir(dir)
                .status()
                .unwrap();
            assert!(st.success(), "git {args:?}");
        }
    }

    #[test]
    fn configs_defensivas_zeram_hooks_e_ssh_command() {
        let args = defensive_config_args();
        let joined = args.join(" ");
        assert!(joined.contains("core.hooksPath="), "{joined}");
        assert!(joined.contains("core.sshCommand="), "{joined}");
        assert!(joined.contains("uploadpack.packObjectsHook="), "{joined}");
        assert!(
            joined.contains("credential.helper=") || joined.contains("credential.helper"),
            "{joined}"
        );
    }

    #[test]
    fn capabilities_nao_usam_permissao_monolitica() {
        let toml = include_str!("../permissions/trilho-commands.toml");
        assert!(
            !toml.contains("allow-repo-commands"),
            "permissão monolítica allow-repo-commands não deve existir (M-04)"
        );
        for id in [
            "allow-repo-read",
            "allow-repo-write-propose",
            "allow-repo-write-execute",
            "allow-secrets",
        ] {
            assert!(toml.contains(id), "falta capability {id}");
        }
        assert!(
            toml.contains("preview_write_operation")
                && toml.contains("execute_write_operation")
                && toml.contains("preview_clone_remote")
                && toml.contains("execute_clone_remote"),
            "preview/execute de write e clone devem estar nas capabilities"
        );
    }

    #[test]
    fn default_capability_agrega_os_quatro_grupos() {
        let json = include_str!("../capabilities/default.json");
        for id in [
            "allow-repo-read",
            "allow-repo-write-propose",
            "allow-repo-write-execute",
            "allow-secrets",
        ] {
            assert!(json.contains(id), "default.json sem {id}");
        }
    }

    /// Registrar no `invoke_handler` não libera o comando: sem entrada na ACL a
    /// chamada morre em runtime com «not allowed by ACL», e nada em tempo de
    /// compilação avisa. Este teste é o aviso.
    #[test]
    fn todo_comando_registrado_esta_em_alguma_permissao() {
        let lib = include_str!("lib.rs");
        let toml = include_str!("../permissions/trilho-commands.toml");
        let registrados: Vec<&str> = lib
            .lines()
            .filter_map(|linha| linha.trim().strip_prefix("commands::"))
            .filter_map(|resto| resto.strip_suffix(','))
            .collect();
        assert!(
            registrados.len() > 30,
            "extração falhou: só {} comandos encontrados em lib.rs",
            registrados.len()
        );
        let sem_permissao: Vec<&str> = registrados
            .iter()
            .copied()
            .filter(|cmd| !toml.contains(&format!("\"{cmd}\"")))
            .collect();
        assert!(
            sem_permissao.is_empty(),
            "comandos sem permissão em trilho-commands.toml: {sem_permissao:?}"
        );
    }

    #[test]
    fn editor_recusa_conteudo_acima_do_limite() {
        let dir: PathBuf =
            std::env::temp_dir().join(format!("trilho-sec-edit-{}", std::process::id()));
        init_repo(&dir);
        let huge = "x".repeat(2 * 1024 * 1024 + 1);
        let err = save_worktree_file(dir.to_str().unwrap(), "big.txt", &huge)
            .unwrap_err()
            .to_string();
        assert!(err.contains("limite"), "{err}");
        let _ = fs::remove_dir_all(&dir);
    }
}
