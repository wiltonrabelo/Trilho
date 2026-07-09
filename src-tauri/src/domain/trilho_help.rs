//! RF-21 — catálogo oficial de ajuda do Trilho (fonte de verdade para o assistente).

/// Índice curto (tópicos) — use quando o usuário pergunta “o que o Trilho faz?”.
pub fn help_index() -> &'static str {
    r#"# Ajuda do Trilho — índice

Trilho é um cliente Git desktop (Windows) focado na trilha de commits, com preview
obrigatório antes de qualquer escrita (RF-08) e log de auditoria (RF-11).

Tópicos (passe `topic` em get_trilho_help):
- overview — visão geral e layout
- open-clone — abrir repo e clonar remoto
- graph — grafo / trilha de commits
- changes-commit — alterações, stage, commit, amend
- sync — fetch, push, pull, force push, publicar
- branches-refs — ramos, remotos, checkout, comparar
- stash-tags — pilhas (stash) e tags
- history-ops — revert, reset, reword, cherry-pick, uncommit
- conflicts — resolução 3 vias
- blame-diff — blame e diff de arquivo
- github — conexão GitHub, PR, SSH/GCM
- audit — histórico de ações
- assistant — o que o assistente pode / não pode fazer
- safety — regras de segurança (preview, default-deny)
"#
}

/// Texto completo de um tópico, ou índice se `topic` vazio / desconhecido.
pub fn help_for_topic(topic: &str) -> String {
    let key = topic.trim().to_lowercase().replace('_', "-").replace(' ', "-");
    let body = match key.as_str() {
        "" | "index" | "ajuda" | "help" => help_index(),
        "overview" | "geral" | "visao" | "visão" => HELP_OVERVIEW,
        "open-clone" | "clone" | "abrir" | "repo" => HELP_OPEN_CLONE,
        "graph" | "grafo" | "trilha" | "commits" => HELP_GRAPH,
        "changes-commit" | "commit" | "stage" | "alteracoes" | "alterações" => HELP_CHANGES,
        "sync" | "push" | "pull" | "fetch" | "publicar" => HELP_SYNC,
        "branches-refs" | "branch" | "ramos" | "refs" | "checkout" => HELP_BRANCHES,
        "stash-tags" | "stash" | "tag" | "tags" | "pilhas" => HELP_STASH_TAGS,
        "history-ops" | "revert" | "reset" | "reword" | "cherry-pick" | "uncommit" => {
            HELP_HISTORY
        }
        "conflicts" | "conflito" | "conflitos" => HELP_CONFLICTS,
        "blame-diff" | "blame" | "diff" => HELP_BLAME_DIFF,
        "github" | "pr" | "conectar" | "ssh" | "gcm" => HELP_GITHUB,
        "audit" | "auditoria" | "acoes" | "ações" | "historico" | "histórico" => HELP_AUDIT,
        "assistant" | "assistente" | "llm" => HELP_ASSISTANT,
        "safety" | "seguranca" | "segurança" | "preview" | "rf-08" => HELP_SAFETY,
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

const HELP_OVERVIEW: &str = r#"# Visão geral

Layout principal:
1. Esquerda — Repo picker / recentes; painel Refs (Ramos, Remotos, Tags, Pilhas).
2. Centro — grafo de commits (topo) + painel Detalhes | Assistente (baixo).
3. Direita — Alterações locais / arquivos do commit (topo) + Diff | Blame (baixo).
4. Header — branch, origem, badge de PR, sync (fetch/push/pull), GitHub, Ações, tema.

Princípios:
- Toda escrita passa por pré-visualização do comando Git real (RF-08) + confirmação.
- Detached HEAD: grafo em leitura; escritas desabilitadas.
- Assistente (aba Assistente) traduz pedidos em ações allowlisted; nunca shell arbitrário.
"#;

const HELP_OPEN_CLONE: &str = r#"# Abrir e clonar

Abrir: escolha uma pasta Git existente no Repo picker (ou recente).
Clonar (RF-22): botão Clonar — URL, pasta destino, nome, branch opcional, shallow opcional,
progresso, abre o repo ao terminar. Auth HTTPS via GCM na 1ª vez.
Pós-clone: origin + tracking já configurados; se clone raso, use «Completar histórico»
(unshallow) na barra de sync.
"#;

const HELP_GRAPH: &str = r#"# Grafo / trilha

O grafo mostra a trilha de commits com lanes, refs e badge «local» quando ainda não
foi enviado. Clique num commit para ver Detalhes; «Working copy» mostra alterações locais.
Paginação por cursor em repos grandes. Visão de branch focada e trilha dupla (origem)
quando aplicável. Load more carrega histórico mais antigo.
"#;

const HELP_CHANGES: &str = r#"# Alterações e commit

Painel direito (working copy): staged / unstaged / untracked.
- Stage / Unstage (arquivo, vários ou todos).
- Commit: resumo + descrição opcional; amend quando permitido.
- Opção «Listar arquivos na descrição» pré-preenche +/~/- dos staged.
- Descartar arquivo / remover untracked / descartar hunk no diff (RF-18).
- Guardar (stash) no painel de alterações (RF-23).
Uncommit (soft) no Detalhes do HEAD quando o commit ainda é local / elegível.
"#;

const HELP_SYNC: &str = r#"# Sync (fetch / push / pull)

Barra de sync no header:
- Fetch — atualiza refs remotas.
- Push — envia commits locais.
- Pull — apenas --ff-only (sem merge automático).
- Force push — quando remoto está à frente (behind > 0); usa --force-with-lease + backup.
- Publicar — 1ª vez: remote + push -u (quando não há upstream).
- Completar histórico — fetch --unshallow em clone raso.
Erros de auth abrem o fluxo Conectar (GCM/PAT/SSH).
"#;

const HELP_BRANCHES: &str = r#"# Branches e refs

Painel Refs: Ramos (locais), Remotos, Tags, Pilhas; pesquisa.
Checkout: git switch em local; remota com --track.
Comparar branches (RF-14): ícone nos Ramos — escolhe A/B, modo merge-base ou tips,
lista de arquivos e diff.
"#;

const HELP_STASH_TAGS: &str = r#"# Stash e tags

Stash (RF-23): «Guardar» nas alterações (mensagem, incluir untracked opcional).
Pilhas no Refs: aplicar / pop / excluir.
Tags (RF-24): «Criar tag…» no commit; anotada ou leve; push opcional.
Listar/excluir na seção Tags do Refs; clique na tag foca o commit no grafo.
"#;

const HELP_HISTORY: &str = r#"# Operações de histórico

No painel Detalhes do commit selecionado:
- Reverter — não no HEAD; não em merge (nesta versão).
- Resetar para aqui — soft/mixed/hard; hard com backup/stash se WT suja.
- Cherry-pick — um ou vários (visão de branch); flag -x opcional.
- Editar mensagem — amend no HEAD ou reword (RF-16) em commit local; reword já
  enviado pode exigir force-with-lease.
- Uncommit (soft) — desfaz o último commit mantendo alterações.
- Criar tag…

Via Assistente: pode propor revert, cherry-pick, push, pull. NÃO pode propor reset
nem force push (use o UI manual).
"#;

const HELP_CONFLICTS: &str = r#"# Conflitos (RF-20)

Em merge/revert/cherry-pick: lista com contagem de blocos; painel Diff com 3 vias.
Por arquivo: aceitar atual/entrando; por bloco: atual/entrando/ambos/editar;
marcar resolvido (git add). Continuar / Abortar / Pular (--skip) quando aplicável.
"#;

const HELP_BLAME_DIFF: &str = r#"# Diff e Blame

Painel inferior direito: abas Diff | Blame (mesmo padrão Detalhes | Assistente).
Diff do arquivo selecionado (working tree ou commit). Blame por linha com fonte
commit / working tree / staging; clique na linha foca o blame.
"#;

const HELP_GITHUB: &str = r#"# GitHub / conexão (RF-10, RF-12)

Botão GitHub / Conectar: GCM (login), PAT, SSH (listar .pub, testar ssh -T),
múltiplas contas (useHttpPath), logout.
Badge de PR no header quando remoto é github.com + credencial HTTPS: aberto /
mergeado / fechado com link. GitHub Enterprise / menu multi-PR ainda opcional.
"#;

const HELP_AUDIT: &str = r#"# Auditoria (RF-11)

Botão «Ações» no header → Histórico de ações (7 dias).
Registra stage/commit/push/force/reset/revert/cherry-pick/reword em JSONL local,
com sanitização de segredos. Entradas do assistente marcadas «assistente».
"#;

const HELP_ASSISTANT: &str = r#"# Assistente LLM (RF-21)

Aba Assistente (centro-baixo). Opt-in desligado por padrão; provedores Ollama /
OpenAI / Anthropic; chaves no Credential Manager (nunca no código).
Pode: ler status/commits/blame/sync; propor stage/unstage/commit/push/pull/revert/
cherry-pick; responder dúvidas sobre o Trilho via get_trilho_help.
Não pode: reset, force push, reword, discard, publish, shell arbitrário.
Toda escrita → preview RF-08 + confirmação humana.
"#;

const HELP_SAFETY: &str = r#"# Segurança

- RF-08: preview do comando Git real antes de executar.
- Spawn com lista de args (sem shell); paths confinados; validação de SHAs/refs.
- Credenciais no Windows Credential Manager / GCM.
- Assistente: allowlist + saída tratada como não confiável; prompt injection
  em diffs/mensagens é ignorado; destrutivas default-deny via assistente.
"#;

const HELP_ALL: &str = r#"# Ajuda completa do Trilho

(Concatenação dos tópicos principais.)

## overview
Trilho = cliente Git desktop com grafo, preview RF-08 e auditoria RF-11.
Layout: Refs | grafo+Detalhes/Assistente | alterações+Diff/Blame. Header: sync, GitHub, Ações.

## open-clone
Abrir pasta Git; Clonar URL+destino+branch/shallow; unshallow na sync.

## graph
Trilha de commits, working copy, paginação, refs/local badge.

## changes-commit
Stage/unstage/commit/amend; listar arquivos na descrição; discard/hunk; stash.

## sync
Fetch, push, pull --ff-only, force-with-lease, publicar, unshallow.

## branches-refs
Ramos/Remotos/Tags/Pilhas; switch; comparar branches.

## stash-tags
Stash push/apply/drop; criar/listar/excluir tags.

## history-ops
Revert, reset, cherry-pick, reword, uncommit, criar tag — no Detalhes.
Assistente: revert/cherry-pick/push/pull sim; reset/force não.

## conflicts
3 vias, aceitar lados/blocos, continue/abort/skip.

## blame-diff
Abas Diff|Blame; fontes commit/WT/staging.

## github
GCM/PAT/SSH; badge PR.

## audit
Histórico 7 dias; marca assistente.

## assistant
Opt-in; Ollama/OpenAI/Anthropic; allowlist; get_trilho_help para dúvidas do produto.

## safety
Preview, sem shell, cofre de credenciais, default-deny destrutivas no assistente.
"#;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn indice_lista_topicos() {
        let idx = help_index();
        assert!(idx.contains("overview"));
        assert!(idx.contains("assistant"));
        assert!(idx.contains("safety"));
    }

    #[test]
    fn topico_commit_responde() {
        let t = help_for_topic("commit");
        assert!(t.to_lowercase().contains("stage") || t.to_lowercase().contains("commit"));
    }

    #[test]
    fn topico_desconhecido_volta_indice() {
        let t = help_for_topic("xyzzy");
        assert!(t.contains("não encontrado") || t.contains("índice") || t.contains("indice") || t.contains("Tópicos"));
    }
}
