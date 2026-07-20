//! RF-21 — catálogo oficial de ajuda do Trilho (fonte de verdade para o assistente).

/// Índice curto (tópicos) — use quando o usuário pergunta “o que o Trilho faz?”.
pub fn help_index() -> &'static str {
    r#"# Ajuda do Trilho — índice

Trilho é um cliente Git desktop (Windows) focado na trilha de commits, com preview
obrigatório antes de qualquer escrita (RF-08) e log de auditoria (RF-11).

Como consultar: aba **Assistente** (com LLM configurado) ou ferramenta `get_trilho_help`.
Não há ainda um botão «Guia» dedicado na UI — o catálogo abaixo é a fonte de verdade.

Tópicos (passe `topic` em get_trilho_help):
- overview — visão geral e layout
- open-clone — abrir repo e clonar remoto
- graph — grafo / trilha de commits / trilha comparada
- changes-commit — alterações, stage, commit, amend, descartar
- working-tree — abas Alterações|Arquivo, editor, reverter trecho
- sync — fetch, push, pull, force push, publicar
- branches-refs — ramos, remotos, checkout, comparar
- stash-tags — pilhas (stash) e tags
- history-ops — revert, reset, reword, cherry-pick, uncommit
- conflicts — resolução 3 vias
- blame-diff — diff, blame, destacar diff
- github — conexão GitHub/GHE, PR, SSH/GCM
- audit — histórico de ações
- assistant — o que o assistente pode / não pode fazer
- safety — regras de segurança (preview, default-deny)
"#
}

/// Texto completo de um tópico, ou índice se `topic` vazio / desconhecido.
pub fn help_for_topic(topic: &str) -> String {
    let key = topic.trim().to_lowercase().replace('_', "-").replace(' ', "-");
    let body = match key.as_str() {
        "" | "index" | "ajuda" | "help" | "guia" => help_index(),
        "overview" | "geral" | "visao" | "visão" => HELP_OVERVIEW,
        "open-clone" | "clone" | "abrir" | "repo" => HELP_OPEN_CLONE,
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
3. Direita — Alterações locais (topo) + diff/blame/editor (baixo).
4. Header — branch, origem, badge de PR, sync (fetch/push/pull), GitHub, Ações, tema.

Painel inferior direito (arquivo do working tree):
- Abas **Alterações | Arquivo** no topo do diff.
- Sub-abas **Diff | Blame** dentro de Alterações.
- **Stage / Unstage / Descartar arquivo** ficam só no painel **Alterações** (lista de
  arquivos), não repetidos no painel do diff.

Princípios:
- Toda escrita passa por pré-visualização do comando Git real (RF-08) + confirmação.
- Exceção: salvar na aba **Arquivo** grava direto no working tree (sem stage automático).
- Detached HEAD: grafo em leitura; escritas desabilitadas.
- Ajuda do produto: catálogo embutido (`get_trilho_help`) — use a aba Assistente ou peça
  «como funciona X?» com LLM configurado.
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
foi enviado. Clique num commit para ver Detalhes; «Alterações locais» no topo da trilha
mostra o working copy.

Paginação por cursor em repos grandes. Visão de branch focada (commits exclusivos de um ramo).
Load more carrega histórico mais antigo.

## Trilha comparada (dual trail)
Seletor **«Comparar com»** no grafo:
- **Auto** — usa origem inferida da branch ou base manual salva por repositório (localStorage).
- **Manual** — escolha outra ref local/remota como base.
- Layout dual: lane da branch atual + lane da base + trecho compartilhado.
- Badge de divergência no merge-base; badge «convergência» em merges da lane atual.

Diferente de **Comparar branches** (RF-14), que é diff de **arquivos** entre duas refs.
"#;

const HELP_CHANGES: &str = r#"# Alterações e commit

Painel direito superior (**ALTERAÇÕES**): staged / unstaged / untracked.

Operações em lote e por arquivo (somente neste painel):
- Stage selecionados / Stage tudo / Unstage tudo / Descartar tudo / Guardar (stash).
- Por linha: checkbox + ícones no hover (+ stage, lixeira descartar).
- Barra do arquivo selecionado: Stage, Unstage, Descartar, Remover (untracked).

Commit (painel central inferior):
- Resumo + descrição opcional; amend quando permitido.
- Opção «Listar arquivos na descrição» pré-preenche +/~/- dos staged (localStorage).

Uncommit (soft) no Detalhes do HEAD quando o commit ainda é local / elegível.

**Importante:** Stage e Descartar arquivo **não** aparecem no painel do diff — use o painel
Alterações acima. O painel do diff trata de visualização, blame e **reverter trecho**
(ver tópico `working-tree`).
"#;

const HELP_WORKING_TREE: &str = r#"# Working tree — Alterações, Arquivo, reverter trecho

Ao selecionar um arquivo alterado, o painel inferior direito oferece:

## Abas Alterações | Arquivo
- **Alterações** — diff unificado com ações por trecho; sub-abas Diff | Blame.
- **Arquivo** — editor do conteúdo atual no disco (working tree).

## Reverter trecho (RF-18)
Na aba Alterações → Diff, cada bloco (hunk) do diff tem botão **«Reverter trecho»**.
- Usa `git apply --reverse` (ou `--cached` se o trecho estiver staged).
- Hunks distantes no mesmo arquivo Git aparecem como **trechos separados**, cada um com
  seu botão — reverter um não desfaz o outro.
- Após reverter, o diff recarrega automaticamente.
- Preview RF-08 antes de executar.

Descartar o **arquivo inteiro** continua no painel **Alterações** (lista de arquivos).

## Editor na aba Arquivo
- Texto editável; indicador «Alterações não salvas» / «Salvo no working tree».
- **Salvar** ou **Ctrl+S** (com foco no editor) grava no disco **sem stage automático**.
- Bloqueado em detached HEAD, arquivo em conflito ou operação Git em andamento.
- Após salvar, status e diff do arquivo são atualizados.

## Destacar diff
Botão **«Destacar diff»** abre tela cheia com as mesmas abas Alterações | Arquivo,
Diff | Blame e reverter trecho — estado compartilhado com o painel normal.
**Restaurar** fecha o overlay e volta ao painel; o conteúdo (commit/arquivo selecionado)
permanece o que estava ao navegar.
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

Comparar branches (RF-14): ícone nos Ramos — escolhe A/B, modo merge-base (`A...B`) ou tips (`A..B`),
lista de arquivos e diff por arquivo; layout lado a lado ou unificado; ordenação por checkouts recentes.
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

Via Assistente: pode propor revert, cherry-pick, push, pull, uncommit, tags, stash…
NÃO pode propor reset/force/reword (reescrevem histórico — só UI).
"#;

const HELP_CONFLICTS: &str = r#"# Conflitos (RF-20)

Em merge/revert/cherry-pick: lista com contagem de blocos; painel Diff com 3 vias.
Por arquivo: aceitar atual/entrando; por bloco: atual/entrando/ambos/editar;
marcar resolvido (git add). Continuar / Abortar / Pular (--skip) quando aplicável.

Arquivos em conflito não permitem reverter trecho nem salvar na aba Arquivo — use o resolvedor.
"#;

const HELP_BLAME_DIFF: &str = r#"# Diff e Blame

Painel inferior direito para arquivo do working tree ou de commit:

## Working tree
1. Abas **Alterações | Arquivo** (caminho completo do arquivo no topo).
2. Em Alterações: sub-abas **Diff | Blame**.
3. Diff com hunks e **Reverter trecho** (ver `working-tree`).
4. **Destacar diff** — mesma UI em tela cheia (ver abaixo).

## Blame (painel normal)
Tabela por linha: linha, commit (hash curto), autor, conteúdo.
Seletor **Commit | Working tree | Staging** no cabeçalho do Blame (só na aba Blame):
- **Working tree** — quem alterou cada linha na versão **no disco**.
- **Staging** — quem alterou cada linha na versão do **index** (staged).
- **Commit** — não aparece como fonte no painel normal (use o overlay; ver abaixo).
Clique numa linha do diff foca o blame nessa linha.
Linhas ainda não comitadas mostram autor **«Ainda não comitado»** (em vez do texto em inglês do Git).
Hashes do commit **não** são clicáveis no painel normal.

## Seletor Commit | Working tree | Staging (modo Destacar)
No **Destacar diff**, o seletor aparece nas abas **Diff** e **Blame** (barra acima do conteúdo).
Controla **o que o Diff mostra** e **de qual versão o Blame é calculado** (exceto navegação
por commit no overlay — ver abaixo).

| Seletor | Blame | Diff |
|--------|-------|------|
| **Working tree** | Autoria linha a linha do arquivo **no disco** | Alterações locais vs. HEAD; hunks com **Reverter trecho** |
| **Staging** | Autoria da versão no **index** (staged) | Diff do arquivo **como está no stage** |
| **Commit** | Lista **inalterada** (não recarrega ao clicar um hash) | Diff do **commit escolhido** no Blame para este arquivo |

Comportamento **Commit** no overlay:
- Clique num hash na aba Blame → aba **Diff** + seletor em **Commit** + legenda «Diff do commit abc1234».
- O grafo e a seleção global **não mudam** (navegação local ao overlay).
- **Working tree** no seletor → volta o diff das alterações locais; o último commit clicado fica
  memorizado — selecione **Commit** de novo para rever aquele diff.
- **Commit** sem ter clicado um hash antes → mensagem para escolher um commit na aba Blame.
- Linhas **«Ainda não comitado»** (`0000000`) não são clicáveis.

## Blame no modo Destacar diff (recursos extras)
Somente na tela cheia (**Destacar diff**), a aba Blame ganha:
- Coluna **Data** (data/hora da autoria, formato pt-BR).
- Colunas **redimensionáveis** (arraste a borda do cabeçalho); larguras salvas em localStorage.
- **Clique no hash do commit** (coluna Commit) — ver tabela do seletor acima.
- Fluxo típico: Destacar → Blame (WT) → clique no commit → Diff (Commit) → Working tree →
  Commit de novo → Blame → outro commit.

## Commit histórico (grafo)
Ao selecionar arquivo num commit passado: Diff | Blame sem abas Alterações|Arquivo.
No modo Destacar, explorar commits pelo blame usa navegação **local** (overlay), sem alterar
a seleção do grafo.
"#;

const HELP_GITHUB: &str = r#"# GitHub / conexão (RF-10, RF-12)

Botão GitHub / Conectar: GCM (login), PAT, SSH (listar .pub, testar ssh -T),
múltiplas contas (useHttpPath), logout.

## Status de PR (RF-12)
Badge/chips no header quando há credencial HTTPS e remoto GitHub:
- PR aberto / mergeado / fechado com link para o navegador.
- **github.com** e **GitHub Enterprise** (`github.*` + API `{host}/api/v3`).
- Token por host no Credential Manager.
- ≤2 PRs na branch: chips individuais; >2 PRs: menu dropdown.
- Cache ~60s; falha graciosa (rate limit, rede).
"#;

const HELP_AUDIT: &str = r#"# Auditoria (RF-11)

Botão «Ações» no header → Histórico de ações (7 dias).
Registra stage/commit/push/force/reset/revert/cherry-pick/reword/descartar/salvar arquivo
em JSONL local, com sanitização de segredos. Entradas do assistente marcadas «assistente».
"#;

const HELP_ASSISTANT: &str = r#"# Assistente LLM (RF-21)

Aba Assistente (centro-baixo). Opt-in desligado por padrão; provedores Ollama /
OpenAI / Anthropic; chaves no Credential Manager (nunca no código).

Toda escrita proposta → preview RF-08 + confirmação humana (nunca executa sozinho).

## Pode (leitura)
status, commits, arquivos do commit, sync, branches locais/remotas, stashes, tags,
origem da branch, trilha comparada (dual trail), diff entre branches (lista de arquivos),
status de PR, leitura 3 vias de conflito, blame, fetch; diff de arquivo se
«enviar diffs» estiver ligado; **get_trilho_help** para dúvidas do produto.

## Pode (propor → confirmação)
stage/unstage (1, vários ou all), commit/amend, uncommit, push, pull --ff-only,
unshallow, publish, switch branch (+ track remoto), stash push/apply/pop/drop,
criar/excluir tag, revert, cherry-pick, abort/continue/skip de revert|merge|cherry-pick,
aceitar lado ours/theirs em conflito.

## Não pode (e por quê) — use a UI manual
- **reset** (soft/mixed/hard): reescreve HEAD; risco de perda — painel do commit → Reset.
- **force push**: sobrescreve histórico remoto — Sync → Force push.
- **reword**: altera SHA e descendentes — «Editar mensagem» no commit.
- **discard / clean / reverter trecho**: apaga trabalho não commitado — Alterações ou Diff.
- **salvar aba Arquivo**: grava conteúdo arbitrário — editor do painel de diff.
- **resolver conflito com texto gerado pela LLM**: risco de corromper merge — resolvedor 3 vias
  (ou propor ours/theirs).
- **clone remoto**: chat exige repo já aberto — diálogo Clonar.
- **Conectar GitHub / GCM / SSH / PAT / chaves LLM**: só nos diálogos do app.
- **shell / git arbitrário**: bloqueado por segurança.

Exemplos: «como funciona reverter trecho?», «onde faço stage?», «o que é trilha comparada?»
"#;

const HELP_SAFETY: &str = r#"# Segurança

- RF-08: preview do comando Git real antes de executar (exceto salvar aba Arquivo, que
  valida gates mas não abre modal se usar fluxo direto após preview interno).
- Spawn com lista de args (sem shell); paths confinados; validação de SHAs/refs.
- Credenciais no Windows Credential Manager / GCM.
- Assistente: allowlist + saída tratada como não confiável; prompt injection
  em diffs/mensagens é ignorado; destrutivas default-deny via assistente.
"#;

const HELP_ALL: &str = r#"# Ajuda completa do Trilho

(Concatenação dos tópicos principais — use tópicos individuais para detalhe.)

## overview
Cliente Git desktop: grafo, preview RF-08, auditoria RF-11.
Layout: Refs | grafo+Detalhes/Assistente | Alterações+diff/editor. Stage/descartar na lista
Alterações; diff com reverter trecho e aba Arquivo editável.

## open-clone
Abrir pasta Git; Clonar URL+destino+branch/shallow; unshallow na sync.

## graph
Trilha de commits, alterações locais, paginação, trilha comparada (dual trail),
seletor Comparar com, badge convergência.

## changes-commit
Stage/unstage/commit/amend na lista Alterações; listar arquivos na descrição; stash;
uncommit. Sem stage/descartar no painel do diff.

## working-tree
Abas Alterações|Arquivo; reverter trecho por hunk; editor com Salvar/Ctrl+S; destacar diff.

## sync
Fetch, push, pull --ff-only, force-with-lease, publicar, unshallow.

## branches-refs
Ramos/Remotos/Tags/Pilhas; switch; comparar branches (diff de arquivos).

## stash-tags
Stash push/apply/drop; criar/listar/excluir tags.

## history-ops
Revert, reset, cherry-pick, reword, uncommit, criar tag — no Detalhes.
Assistente: revert/cherry-pick/push/pull/uncommit/tags/stash sim; reset/force/reword não.

## conflicts
3 vias, aceitar lados/blocos, continue/abort/skip.

## blame-diff
Diff|Blame; Alterações|Arquivo no WT; destacar diff; seletor Commit|WT|Staging no overlay
(WT=alterações locais, Staging=index, Commit=diff do hash clicado no Blame); blame com data/
colunas redimensionáveis; «Ainda não comitado»; navegação local sem mudar o grafo.

## github
GCM/PAT/SSH; badge PR (github.com + GHE, multi-PR).

## audit
Histórico 7 dias; marca assistente.

## assistant
Opt-in; allowlist ampla (stage…stash/tags/switch/conflitos); default-deny em
reset/force/reword/discard; get_trilho_help topic=assistant.

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
    fn topico_desconhecido_volta_indice() {
        let t = help_for_topic("xyzzy");
        assert!(t.contains("não encontrado") || t.contains("índice") || t.contains("indice") || t.contains("Tópicos"));
    }
}
