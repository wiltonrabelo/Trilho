//! Executor seguro da Git CLI (escrita e leitura via subprocess).

use crate::application::{GitCommand, GitError, GitWriter};
use std::process::Command;

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

pub struct SafeGitCli;

impl SafeGitCli {
    pub fn full_args(repo_path: &str, command: &GitCommand) -> Vec<String> {
        let mut args = defensive_base_args(repo_path);
        args.extend(command.args.iter().cloned());
        args
    }

    pub fn run(repo_path: &str, command: &GitCommand) -> Result<String, GitError> {
        let args = Self::full_args(repo_path, command);
        let output = Command::new("git")
            .args(&args)
            .env("GIT_TERMINAL_PROMPT", "0")
            // GCM no Windows intercepta credenciais independentemente do prompt de terminal.
            .env("GCM_INTERACTIVE", "always")
            .output()
            .map_err(|e| GitError::Io(format!("Não foi possível executar git: {e}")))?;

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
        line.extend(command.args.clone());
        line
    }

    fn run(&self, _command: &GitCommand) -> Result<String, GitError> {
        Err(GitError::Io(
            "SafeGitCli::run requer repo_path — use SafeGitCli::run(path, cmd).".into(),
        ))
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
        assert!(args.contains(&"filter.lfs.smudge=".to_string()));
    }

    #[test]
    fn full_args_concatena_base_e_subcomando() {
        let cmd = GitCommand {
            args: vec!["status".into(), "--porcelain=v2".into()],
        };
        let full = SafeGitCli::full_args("C:/repo", &cmd);
        assert_eq!(full[0], "-C");
        assert_eq!(full.last().unwrap(), "--porcelain=v2");
    }
}
