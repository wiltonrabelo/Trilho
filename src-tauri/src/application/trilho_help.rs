//! RF-21 — catálogo oficial de ajuda do Trilho (fonte de verdade para o
//! assistente). O texto vive em `resources/help/*.md`; aqui fica só o
//! roteamento de tópico para conteúdo.

const HELP_INDEX: &str = include_str!("../../resources/help/index.md");
const HELP_OVERVIEW: &str = include_str!("../../resources/help/overview.md");
const HELP_OPEN_CLONE: &str = include_str!("../../resources/help/open-clone.md");
const HELP_GRAPH: &str = include_str!("../../resources/help/graph.md");
const HELP_CHANGES: &str = include_str!("../../resources/help/changes-commit.md");
const HELP_WORKING_TREE: &str = include_str!("../../resources/help/working-tree.md");
const HELP_SYNC: &str = include_str!("../../resources/help/sync.md");
const HELP_BRANCHES: &str = include_str!("../../resources/help/branches-refs.md");
const HELP_STASH_TAGS: &str = include_str!("../../resources/help/stash-tags.md");
const HELP_HISTORY: &str = include_str!("../../resources/help/history-ops.md");
const HELP_CONFLICTS: &str = include_str!("../../resources/help/conflicts.md");
const HELP_BLAME_DIFF: &str = include_str!("../../resources/help/blame-diff.md");
const HELP_GITHUB: &str = include_str!("../../resources/help/github.md");
const HELP_AUDIT: &str = include_str!("../../resources/help/audit.md");
const HELP_ASSISTANT: &str = include_str!("../../resources/help/assistant.md");
const HELP_SAFETY: &str = include_str!("../../resources/help/safety.md");
const HELP_ALL: &str = include_str!("../../resources/help/all.md");

/// Índice curto (tópicos) — use quando o usuário pergunta “o que o Trilho faz?”.
pub fn help_index() -> &'static str {
    HELP_INDEX
}

/// Texto completo de um tópico, ou índice se `topic` vazio / desconhecido.
pub fn help_for_topic(topic: &str) -> String {
    let key = topic.trim().to_lowercase().replace(['_', ' '], "-");
    let body = match key.as_str() {
        "" | "index" | "ajuda" | "help" | "guia" => help_index(),
        "overview" | "geral" | "visao" | "visão" | "terminal" | "git-bash" | "menu-contexto" => {
            HELP_OVERVIEW
        }
        "open-clone" | "clone" | "abrir" | "repo" | "recentes" => HELP_OPEN_CLONE,
        "graph" | "grafo" | "trilha" | "commits" | "trilha-comparada" | "dual-trail" => {
            HELP_GRAPH
        }
        "changes-commit" | "commit" | "stage" | "alteracoes" | "alterações" => HELP_CHANGES,
        "working-tree" | "arquivo" | "editor" | "reverter-trecho" | "hunk" | "descartar" => {
            HELP_WORKING_TREE
        }
        "sync" | "push" | "pull" | "fetch" | "publicar" => HELP_SYNC,
        "branches-refs" | "branch" | "ramos" | "refs" | "checkout" => HELP_BRANCHES,
        "stash-tags" | "stash" | "tag" | "tags" | "pilhas" => HELP_STASH_TAGS,
        "history-ops" | "revert" | "reset" | "reword" | "cherry-pick" | "uncommit" => {
            HELP_HISTORY
        }
        "conflicts" | "conflito" | "conflitos" => HELP_CONFLICTS,
        "blame-diff" | "blame" | "diff" | "destacar" | "blame-commit" | "navegar-blame"
        | "fonte-blame" | "working-tree-blame" | "staging-blame" => HELP_BLAME_DIFF,
        "github" | "pr" | "conectar" | "ssh" | "gcm" | "ghe" | "enterprise" => HELP_GITHUB,
        "audit" | "auditoria" | "acoes" | "ações" | "historico" | "histórico" => HELP_AUDIT,
        "assistant" | "assistente" | "llm" => HELP_ASSISTANT,
        "safety" | "seguranca" | "segurança" | "preview" | "rf-08"
        | "comando-git" | "comando" | "git-cli" | "preview-comando" | "porque-comando"
        | "por-que-comando" | "porquê-comando" | "defensive" | "overrides" => HELP_SAFETY,
        "all" | "tudo" | "completo" => HELP_ALL,
        _ => {
            return format!(
                "Tópico «{topic}» não encontrado.\n\n{}",
                help_index()
            );
        }
    };
    body.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn indice_lista_topicos() {
        let idx = help_index();
        assert!(idx.contains("overview"));
        assert!(idx.contains("working-tree"));
        assert!(idx.contains("assistant"));
        assert!(idx.contains("safety"));
    }

    #[test]
    fn topico_commit_responde() {
        let t = help_for_topic("commit");
        assert!(t.to_lowercase().contains("stage") || t.to_lowercase().contains("commit"));
    }

    #[test]
    fn topico_reverter_trecho() {
        let t = help_for_topic("reverter-trecho");
        assert!(t.contains("Reverter trecho"));
        assert!(t.contains("git apply --reverse"));
    }

    #[test]
    fn topico_guia_volta_indice() {
        let t = help_for_topic("guia");
        assert!(t.contains("índice") || t.contains("indice"));
    }

    #[test]
    fn topico_blame_destacar_navegacao() {
        let t = help_for_topic("blame-diff");
        assert!(t.contains("Destacar diff"));
        assert!(t.contains("Ainda não comitado"));
        assert!(t.contains("Clique no hash do commit"));
        assert!(t.contains("Working tree"));
        assert!(t.contains("Staging"));
        assert!(t.contains("Reverter trecho"));
    }

    #[test]
    fn topico_fonte_blame_alias() {
        let t = help_for_topic("fonte-blame");
        assert!(t.contains("Seletor Commit"));
    }

    #[test]
    fn topico_comando_git_explica_overrides() {
        let t = help_for_topic("comando-git");
        assert!(t.contains("core.fsmonitor=false"));
        assert!(t.contains("core.hooksPath="));
        assert!(t.contains("core.sshCommand="));
        assert!(t.contains("uploadpack.packObjectsHook="));
        assert!(t.contains("git-upload-pack"));
        assert!(t.contains("filter.lfs.required=false"));
        assert!(t.contains("add -A"));
        assert!(t.contains("uma linha"));
        assert!(t.contains("2 MiB"));
        assert!(t.contains("não") && t.contains("restaurado"));
        assert!(t.to_lowercase().contains("não invente") || t.to_lowercase().contains("não inventar"));
    }

    #[test]
    fn topico_revert_permite_head() {
        let t = help_for_topic("revert");
        assert!(t.contains("Permitido no HEAD") || t.contains("incluindo o HEAD"));
        assert!(t.to_lowercase().contains("merge"));
    }

    #[test]
    fn topico_terminal_e_ramos_menu() {
        let overview = help_for_topic("terminal");
        assert!(overview.contains("Git Bash"));
        assert!(overview.contains("espaço vazio"));
        let branches = help_for_topic("branches-refs");
        assert!(branches.contains("Remover no repositório remoto"));
        assert!(branches.contains("Remover localmente"));
        let changes = help_for_topic("changes-commit");
        assert!(changes.contains("duas linhas em branco"));
    }

    #[test]
    fn topico_desconhecido_volta_indice() {
        let t = help_for_topic("xyzzy");
        assert!(t.contains("não encontrado") || t.contains("índice") || t.contains("indice") || t.contains("Tópicos"));
    }
}
