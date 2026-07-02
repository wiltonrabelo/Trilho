//! Camada de Infraestrutura — adaptadores concretos das portas da aplicação.
//!
//! No M0 há apenas mocks e o esqueleto do wrapper seguro da CLI. A leitura real
//! (git2) e a escrita real (Git CLI) entram nas fases seguintes.

// `SafeGitCli`/`defensive_base_args` são exercitados pelos testes e usados nas
// próximas fases; mantidos desde o M0 para fixar o baseline de segurança.
#![allow(dead_code)]

use crate::application::{GitCommand, GitError, GitReader};
use crate::domain::Commit;

/// Argumentos-base defensivos aplicados a TODA invocação do Git em repositórios
/// não confiáveis (PLANO §7.7/§11.5). Neutralizam execução de código já na
/// LEITURA (fsmonitor, hooks, filtros) e evitam auto-gc em background (watcher).
///
/// Precede qualquer subcomando: `git <base> <subcomando> ...`.
pub fn defensive_base_args(repo_path: &str) -> Vec<String> {
    vec![
        // Repositório explícito (determinístico, thread-safe).
        "-C".into(),
        repo_path.into(),
        // Sem monitor de FS (pode invocar binário externo).
        "-c".into(),
        "core.fsmonitor=false".into(),
        // Sem hooks do repositório.
        "-c".into(),
        "core.hooksPath=".into(),
        // Sem auto-gc (evita tempestade de eventos no watcher — RF-19).
        "-c".into(),
        "gc.auto=0".into(),
        // Restringe protocolos externos perigosos.
        "-c".into(),
        "protocol.ext.allow=never".into(),
        // Neutraliza filtros clean/smudge (LFS e drivers comuns — §7.7/§11.5).
        // Repositórios marcados como confiáveis podem omitir estes flags no M1+.
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

/// Adaptador de LEITURA de exemplo (M0). Será substituído por um `Git2Reader`.
pub struct MockGitReader;

impl MockGitReader {
    pub fn new() -> Self {
        Self
    }
}

impl Default for MockGitReader {
    fn default() -> Self {
        Self::new()
    }
}

impl GitReader for MockGitReader {
    fn list_commits(&self, limit: usize) -> Result<Vec<Commit>, GitError> {
        let sample = vec![
            Commit {
                id: "9f3a1c2e5b7d0a4f6e8c1b2d3a4f5e6c7d8b9a0f".into(),
                short_id: "9f3a1c2".into(),
                summary: "feat: estrutura inicial do Trilho (M0)".into(),
                author_name: "Você".into(),
                authored_at: "2026-07-02T14:10:00-03:00".into(),
                is_local_only: true,
            },
            Commit {
                id: "1b2c3d4e5f60718293a4b5c6d7e8f90a1b2c3d4e".into(),
                short_id: "1b2c3d4".into(),
                summary: "chore: configuração de tema claro/escuro".into(),
                author_name: "Você".into(),
                authored_at: "2026-07-02T11:05:00-03:00".into(),
                is_local_only: false,
            },
            Commit {
                id: "abcdef0123456789abcdef0123456789abcdef01".into(),
                short_id: "abcdef0".into(),
                summary: "docs: plano e MVP aprovados".into(),
                author_name: "Você".into(),
                authored_at: "2026-07-01T18:42:00-03:00".into(),
                is_local_only: false,
            },
        ];
        Ok(sample.into_iter().take(limit).collect())
    }
}

/// Esqueleto do executor seguro da CLI (escrita). No M0 apenas monta e
/// pré-visualiza o comando; a execução real chega nas fases de operação.
pub struct SafeGitCli;

impl SafeGitCli {
    /// Monta a linha de comando completa (base defensiva + subcomando).
    pub fn full_args(repo_path: &str, command: &GitCommand) -> Vec<String> {
        let mut args = defensive_base_args(repo_path);
        args.extend(command.args.iter().cloned());
        args
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::GitReader;

    #[test]
    fn defensive_base_args_neutraliza_vetores_de_execucao() {
        let args = defensive_base_args("C:/repo");
        // Precede tudo com -C <repo>.
        assert_eq!(args[0], "-C");
        assert_eq!(args[1], "C:/repo");
        // Neutraliza fsmonitor, hooks e auto-gc (§7.7/§11.5, RF-19).
        assert!(args.contains(&"core.fsmonitor=false".to_string()));
        assert!(args.contains(&"core.hooksPath=".to_string()));
        assert!(args.contains(&"gc.auto=0".to_string()));
        assert!(args.contains(&"protocol.ext.allow=never".to_string()));
        // Filtros clean/smudge desabilitados (LFS).
        assert!(args.contains(&"filter.lfs.required=false".to_string()));
        assert!(args.contains(&"filter.lfs.process=".to_string()));
        assert!(args.contains(&"filter.lfs.clean=".to_string()));
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

    #[test]
    fn mock_reader_respeita_limite() {
        let reader = MockGitReader::new();
        let commits = reader.list_commits(2).expect("deve listar");
        assert_eq!(commits.len(), 2);
    }
}
