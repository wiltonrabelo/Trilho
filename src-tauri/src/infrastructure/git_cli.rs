//! Executor seguro da Git CLI (escrita e leitura via subprocess).

use crate::application::{GitCommand, GitError, GitWriter};
use std::process::{Command, Stdio};

/// Argumentos-base defensivos aplicados a TODA invocação do Git (PLANO §7.7/§11.5).
pub fn defensive_base_args(repo_path: &str) -> Vec<String> {
    vec![
        "-C".into(),
        repo_path.into(),
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

    fn invoke(&self, command: &GitCommand) -> Result<String, GitError> {
        self.invoke_with_stdin(command, None)
    }

    pub fn run_with_stdin(&self, command: &GitCommand, stdin: &[u8]) -> Result<String, GitError> {
        self.invoke_with_stdin(command, Some(stdin))
    }

    fn invoke_with_stdin(
        &self,
        command: &GitCommand,
        stdin: Option<&[u8]>,
    ) -> Result<String, GitError> {
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

        let stderr = String::from_utf8_lossy(&output.stderr);
        if !output.status.success() {
            return Err(GitError::from_git_stderr(&stderr));
        }
        Ok(String::from_utf8_lossy(&output.stdout).into_owned())
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
