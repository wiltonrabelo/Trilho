# Assistente LLM (RF-21)

Aba Assistente (centro-baixo). Opt-in desligado por padrão; provedores Ollama /
OpenAI (API key) / **Codex CLI (ChatGPT)** / Anthropic (API key).
Chaves OpenAI/Anthropic no Credential Manager (nunca no código).

### Codex CLI (sem API key no Trilho)
- **Codex CLI:** `codex login` (ChatGPT com acesso Codex). Modelo típico: `gpt-5.4-mini`.
  Invoca `codex exec --sandbox read-only` em cwd neutro; remove `OPENAI_API_KEY`/
  `CODEX_API_KEY` do subprocesso para preferir auth ChatGPT. Não exige `codex` no PATH
  se a extensão VS Code/Cursor `openai.chatgpt-*` estiver instalada (usa
  `bin/windows-x86_64/codex.exe`).

Usa a **mesma allowlist** que Ollama/OpenAI/Anthropic: o adaptador interpreta o
bloco textual `<<<TRILHO_TOOL_CALLS>>>`…`<<<END_TRILHO_TOOL_CALLS>>>` e o runtime
executa as tools. Não há Bash/`git` livre do agent do CLI no repo do usuário.
Revisão por pacote de diffs continua **sem** tools.

Toda escrita proposta → preview RF-08 + confirmação humana (nunca executa sozinho).

## Pode (leitura)
status, sync (ahead/behind), branches locais/remotas, stashes, tags, origem da branch,
trilha comparada (dual trail), diff entre branches (lista de arquivos), status de PR,
leitura 3 vias de conflito, blame, fetch; **get_trilho_help** para dúvidas do produto.

### Commits: listar vs contar
- **`list_commits`** — amostra recente do grafo (resumo, autor, refs). **Máx. 30**.
  Não use para «quantos commits no total».
- **`count_commits`** — número exato via `git rev-list --count` (só o número, sem listar).
  - Sem args: total alcançável de **HEAD**.
  - Com `exclude` (ex.: `main`, `origin/master`): conta `exclude..ref` — commits na tip
    que não estão na base («quantos desta branch desde main»).
  - `ref` opcional (branch/tag/SHA/HEAD); refs validadas pelo Trilho.

Com **«Enviar diffs ao provedor»** ligado (necessário para revisão de código):
- Pedidos de revisão/bugs: o Trilho **coleta diffs no runtime** (branch vs base,
  commit selecionado ou working tree) e envia um pacote ao modelo **sem tool-calling**
  — modelos locais pequenos costumam falhar se dependerem só de tools.
- Tools ainda disponíveis no chat geral: `get_file_diff`, `get_commit_file_diff`,
  `get_branch_file_diff`, `show_file_at_ref`, `list_branch_diff_files`, `list_commit_files`.

### Limitações da revisão
- Só o que o pacote/tools trouxerem — **não** indexa o repositório inteiro.
- Diffs/conteúdos grandes são **truncados** (orçamento por pedido).
- Achados são **sugestões**; não substituem testes, CI nem revisão humana.
- Sem «Enviar diffs», a revisão **não** está disponível — o assistente pede para marcar
  a opção e repetir o pedido (sem inventar JSON nem revisão fictícia).

## Pode (propor → confirmação)
stage/unstage (1, vários ou all), commit/amend, uncommit, push, pull --ff-only,
**fetch (refs remotas)**, unshallow, publish, switch branch (+ track remoto),
stash push/apply/pop/drop, criar/excluir tag, **revert (incluindo HEAD; não merge)**,
cherry-pick, abort/continue/skip de revert|merge|cherry-pick, aceitar lado ours/theirs
em conflito.

Fetch pelo assistente **não** roda sozinho — vira `propose_fetch_remote` + preview RF-08
(o botão Fetch da UI continua sendo ação humana direta, com Git endurecido).

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
- **remover branch local/remota**: só no painel Refs (menu de contexto) — preview RF-08.
- **abrir Terminal / Git Bash**: só o botão Terminal no header.
- **shell / git arbitrário**: bloqueado por segurança.

Exemplos: «quantos commits tem esta branch?» → `count_commits`;
«quantos desde main?» → `count_commits` com `exclude=main`;
«revise esta branch contra master», «compare o arquivo X com o commit Y»,
«como funciona reverter trecho?», «posso reverter o HEAD?», «onde fica o Terminal?»
(tópicos `assistant`, `history-ops`, `overview`, `branches-refs`, `safety`).

## Não inventar
Responda dúvidas do produto **somente** com o retorno de `get_trilho_help` (e dados
das ferramentas de leitura). Se o catálogo não cobrir a pergunta, diga explicitamente
que isso **não está documentado** no Trilho. Não invente flags Git, motivos de UI,
atalhos ou comportamento.
