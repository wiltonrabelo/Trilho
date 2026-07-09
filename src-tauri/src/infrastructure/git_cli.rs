//! Executor seguro da Git CLI (escrita e leitura via subprocess).

use crate::application::{GitCommand, GitError, GitWriter};
use std::process::{Command, Stdio};

/// Config defensiva sem `-C` (ex.: `git clone` fora de um repo aberto).
pub fn defensive_config_args() -> Vec<String> {
    vec![
        "-c".into(),
        "core.fsmonitor=false".into(),
        "-c".into(),
        "core.hooksPath=".into(),
        "-c".into(),
        "gc.auto=0".into(),
        "-c".into(),
        "protocol.ext.allow=never".into(),
        "-c".into(),
        "filter.lfs.required=false".into(),
        "-c".into(),
        "filter.lfs.process=".into(),
        "-c".into(),
        "filter.lfs.clean=".into(),
        "-c".into(),
        "filter.lfs.smudge=".into(),
    ]
}

/// Argumentos-base defensivos aplicados a TODA invocação do Git (PLANO §7.7/§11.5).
pub fn defensive_base_args(repo_path: &str) -> Vec<String> {
    let mut args = defensive_config_args();
    args.insert(0, repo_path.into());
    args.insert(0, "-C".into());
    args
}

/// Adaptador Git CLI vinculado a um repositório — honra `GitWriter` (LSP).
#[derive(Clone)]
pub struct SafeGitCli {
    repo_path: String,
}

impl SafeGitCli {
    pub fn new(repo_path: impl Into<String>) -> Self {
        Self {
            repo_path: repo_path.into(),
        }
    }

    #[allow(dead_code)] // M3: diagnóstico / logs de operações
    pub fn repo_path(&self) -> &str {
        &self.repo_path
    }

    pub fn full_args(&self, command: &GitCommand) -> Vec<String> {
        let mut args = defensive_base_args(&self.repo_path);
        args.extend(command.args.iter().cloned());
        args
    }

    pub fn run(&self, command: &GitCommand) -> Result<String, GitError> {
        self.invoke(command)
    }

    /// Comando cuja resposta é booleana via exit code (ex.: `merge-base
    /// --is-ancestor`): exit 0 → `true`; exit 1 sem stderr → `false`; qualquer
    /// outra coisa é ERRO e propaga. Nunca use `run()` + "erro = false" para
    /// gates — falha real viraria resposta e o gate abriria indevidamente.
    pub fn run_bool(&self, command: &GitCommand) -> Result<bool, GitError> {
        let output = self.raw_output(command, None, &[])?;
        if output.status.success() {
            return Ok(true);
        }
        let stderr = String::from_utf8_lossy(&output.stderr);
        if output.status.code() == Some(1) && stderr.trim().is_empty() {
            return Ok(false);
        }
        Err(GitError::from_git_stderr(&stderr))
    }

    fn invoke(&self, command: &GitCommand) -> Result<String, GitError> {
        self.invoke_with_stdin(command, None, &[])
    }

    pub fn run_with_stdin(&self, command: &GitCommand, stdin: &[u8]) -> Result<String, GitError> {
        self.invoke_with_stdin(command, Some(stdin), &[])
    }

    pub fn run_with_env(
        &self,
        command: &GitCommand,
        extra_env: &[(&str, &str)],
    ) -> Result<String, GitError> {
        self.invoke_with_stdin(command, None, extra_env)
    }

    fn invoke_with_stdin(
        &self,
        command: &GitCommand,
        stdin: Option<&[u8]>,
        extra_env: &[(&str, &str)],
    ) -> Result<String, GitError> {
        let output = self.raw_output(command, stdin, extra_env)?;
        let stderr = String::from_utf8_lossy(&output.stderr);
        if !output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let detail = if stderr.trim().is_empty() {
                stdout.as_ref()
            } else {
                stderr.as_ref()
            };
            return Err(GitError::from_git_stderr(detail));
        }
        Ok(String::from_utf8_lossy(&output.stdout).into_owned())
    }

    fn raw_output(
        &self,
        command: &GitCommand,
        stdin: Option<&[u8]>,
        extra_env: &[(&str, &str)],
    ) -> Result<std::process::Output, GitError> {
        let args = self.full_args(command);
        let mut cmd = Command::new("git");
        cmd.args(&args)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .env("GIT_TERMINAL_PROMPT", "0")
            .env("GCM_INTERACTIVE", "always")
            // Impede que leituras (status, blame) reescrevam `.git/index`,
            // o que dispararia o watcher (RF-19) em laço — flicker infinito.
            .env("GIT_OPTIONAL_LOCKS", "0");
        for (key, value) in extra_env {
            cmd.env(key, value);
        }
        if stdin.is_some() {
            cmd.stdin(Stdio::piped());
        }
        let mut child = cmd
            .spawn()
            .map_err(|e| GitError::Io(format!("Não foi possível executar git: {e}")))?;

        // Escreve o stdin em thread própria: escrever tudo antes de ler o stdout
        // deadlocka quando os dois pipes enchem (entrada grande + git emitindo).
        let writer = stdin.map(|data| {
            let data = data.to_vec();
            let stdin_pipe = child.stdin.take();
            std::thread::spawn(move || {
                use std::io::Write;
                if let Some(mut pipe) = stdin_pipe {
                    // Erro de escrita (ex.: git encerrou cedo) não é fatal aqui;
                    // o status/stderr do processo reporta a falha real.
                    let _ = pipe.write_all(&data);
                }
            })
        });

        let output = child
            .wait_with_output()
            .map_err(|e| GitError::Io(format!("Não foi possível executar git: {e}")))?;

        if let Some(handle) = writer {
            let _ = handle.join();
        }

        Ok(output)
    }

    /// Finaliza revert após resolver conflitos. Se não houver alterações para
    /// commitar (`nothing to commit`), usa `git revert --skip` — mesmo fluxo do Git
    /// quando a resolução já deixou o working tree igual ao resultado esperado.
    pub fn finish_revert(&self) -> Result<(), GitError> {
        self.finish_sequencer(
            &["revert", "--continue", "--no-edit"],
            Some(&["revert", "--skip"]),
        )
    }

    pub fn finish_cherry_pick(&self) -> Result<(), GitError> {
        self.finish_sequencer(
            &["cherry-pick", "--continue", "--no-edit"],
            Some(&["cherry-pick", "--skip"]),
        )
    }

    pub fn finish_merge(&self) -> Result<(), GitError> {
        self.finish_sequencer(&["merge", "--continue", "--no-edit"], None)
    }

    fn finish_sequencer(
        &self,
        continue_args: &[&str],
        skip_args: Option<&[&str]>,
    ) -> Result<(), GitError> {
        let continue_cmd = GitCommand {
            args: continue_args.iter().map(|s| (*s).to_string()).collect(),
        };
        let output = self.raw_output(&continue_cmd, None, &[])?;
        if output.status.success() {
            return Ok(());
        }
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        let combined = format!("{stderr}\n{stdout}");
        let lower = combined.to_lowercase();
        if let Some(skip) = skip_args {
            let no_changes = lower.contains("nothing to commit")
                || lower.contains("no changes added to commit");
            let clean_after_failed_continue = output.status.code() == Some(1)
                && !self.has_unmerged_paths()?;
            if no_changes || clean_after_failed_continue {
                let skip_cmd = GitCommand {
                    args: skip.iter().map(|s| (*s).to_string()).collect(),
                };
                self.run(&skip_cmd)?;
                return Ok(());
            }
        }
        let detail = if stderr.trim().is_empty() {
            combined
        } else {
            stderr.into_owned()
        };
        Err(GitError::from_git_stderr(&detail))
    }

    /// Há entradas não mescladas no `git status --porcelain`.
    fn has_unmerged_paths(&self) -> Result<bool, GitError> {
        let out = self.run(&GitCommand {
            args: vec!["status".into(), "--porcelain=1".into()],
        })?;
        Ok(out.lines().any(|line| {
            let line = line.trim_start();
            if line.is_empty() {
                return false;
            }
            if line.starts_with('u') {
                return true;
            }
            if line.len() >= 2 {
                matches!(
                    &line[..2],
                    "UU" | "AA" | "DD" | "AU" | "UA" | "DU" | "UD"
                )
            } else {
                false
            }
        }))
    }
}

impl GitWriter for SafeGitCli {
    fn preview(&self, command: &GitCommand) -> Vec<String> {
        let mut line = vec!["git".to_string()];
        line.extend(self.full_args(command));
        line
    }

    fn run(&self, command: &GitCommand) -> Result<String, GitError> {
        self.invoke(command)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defensive_base_args_neutraliza_vetores_de_execucao() {
        let args = defensive_base_args("C:/repo");
        assert_eq!(args[0], "-C");
        assert_eq!(args[1], "C:/repo");
        assert!(args.contains(&"core.fsmonitor=false".to_string()));
    }

    #[test]
    fn run_with_stdin_aceita_entrada_grande_sem_travar() {
        let dir = std::env::temp_dir().join(format!("trilho-stdin-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        std::process::Command::new("git")
            .args(["init"])
            .current_dir(&dir)
            .output()
            .unwrap();

        // > buffer típico de pipe (64 KiB) para exercitar a escrita em thread.
        let big = vec![b'x'; 4 * 1024 * 1024];
        let cli = SafeGitCli::new(dir.to_string_lossy());
        let out = cli
            .run_with_stdin(
                &GitCommand {
                    args: vec!["hash-object".into(), "--stdin".into()],
                },
                &big,
            )
            .expect("hash-object com stdin grande");
        assert_eq!(out.trim().len(), 40);
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// Regressão do gate fail-open: exit 1 = false; ERRO real propaga (nunca
    /// vira "não é ancestral").
    #[test]
    fn run_bool_distingue_nao_de_erro() {
        let dir = std::env::temp_dir().join(format!("trilho-bool-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        for args in [
            vec!["init"],
            vec!["config", "user.email", "t@t.com"],
            vec!["config", "user.name", "T"],
            vec!["commit", "--allow-empty", "-m", "a"],
            vec!["commit", "--allow-empty", "-m", "b"],
        ] {
            std::process::Command::new("git")
                .args(&args)
                .current_dir(&dir)
                .output()
                .unwrap();
        }
        let cli = SafeGitCli::new(dir.to_string_lossy());
        let anc = |a: &str, b: &str| {
            cli.run_bool(&GitCommand {
                args: vec![
                    "merge-base".into(),
                    "--is-ancestor".into(),
                    a.into(),
                    b.into(),
                ],
            })
        };
        assert_eq!(anc("HEAD~1", "HEAD").unwrap(), true);
        assert_eq!(anc("HEAD", "HEAD~1").unwrap(), false);
        // SHA inexistente = erro real → propaga, não vira false.
        assert!(anc("deadbeefdeadbeefdeadbeefdeadbeefdeadbeef", "HEAD").is_err());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn git_writer_run_honra_contrato() {
        let cli = SafeGitCli::new("C:/repo");
        let cmd = GitCommand {
            args: vec!["status".into()],
        };
        let preview = cli.preview(&cmd);
        assert_eq!(preview[0], "git");
        assert!(preview.contains(&"-C".to_string()));
        // Sem git real em C:/repo — run falha, mas não com erro de trait quebrado
        let err = cli.run(&cmd).expect_err("repo inexistente");
        assert!(!err.to_string().contains("use o método estático"));
    }
}
