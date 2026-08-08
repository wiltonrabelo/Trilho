# Trilho — Arquitetura e guia de manutenção

Documento para humanos e IAs que forem manter o Trilho: o que é o app, como o código está organizado, padrões, onde mexer e armadilhas.

**Repositório:** `C:\Projetos\Trilho`  
**Specs de produto (externas):** `C:\Projetos\SysPDV\Docs\git-trail-viewer\` (`MVP.md`, `PLANO.md`)  
**Decisões / segurança:** `DECISIONS.md`, `SECURITY.md`, `README.md`

---

## 1. Para que serve

**Trilho** é um cliente Git desktop minimalista para Windows, focado na **trilha de commits** (grafo), status do working tree, diff, sincronização com remoto e operações de escrita **sempre com pré-visualização** (RF-08).

Não é um IDE nem um substituto genérico do Git CLI: é um visualizador/operador seguro da história e do working tree, com assistente LLM opcional (RF-21).

---

## 2. Stack tecnológica

| Camada | Tecnologia |
|--------|------------|
| Shell desktop | **Tauri 2** |
| UI | React 18, TypeScript, Vite 6, Tailwind, lucide-react |
| Bridge FE↔Rust | `@tauri-apps/api` v2 (`invoke`), `tauri-plugin-dialog` |
| Backend | Rust 2021 (`rust-version = "1.77"`), crate lib `trilho_lib` |
| Git | `git2` (leitura) + **CLI Git** via `SafeGitCli` (escrita / ops sensíveis) |
| LLM HTTP | `ureq` (Ollama / OpenAI / Anthropic) |
| LLM plano Claude / ChatGPT | subprocesso **Claude Code** / **Codex CLI** (não API key) |
| Testes FE | Vitest + jsdom |
| Testes E2E | Playwright (`e2e/`) |
| Testes Rust | `cargo test` + `clippy -D warnings` |

---

## 3. Layout do repositório

```
Trilho/
  README.md, DECISIONS.md, SECURITY.md, ARCHITECTURE.md
  package.json, vite.config.ts, vitest.config.ts, playwright.config.ts
  src/                         # Frontend React/TS
    App.tsx                    # Orquestração principal da UI
    main.tsx
    components/                # Painéis, diálogos, grafo, Assistente…
    hooks/                     # useRepo, useCommits, useSync, useOperations…
    lib/                       # api.ts (invoke), graph/, mocks, theme…
    types.ts                   # DTOs camelCase (espelham serde)
  src-tauri/
    Cargo.toml
    src/
      main.rs                  # entry binário
      lib.rs                   # registra módulos, AppState, commands
      commands.rs              # #[tauri::command] finos
      domain/                  # tipos puros + help embutido
      application/             # ports, serviços, gates, orquestração
      infrastructure/          # adapters (git2, CLI, LLM, credenciais…)
  e2e/, assets/, public/
```

---

## 4. Arquitetura Rust (camadas)

Fluxo típico:

```
UI (React) → invoke → commands.rs → application/* → infrastructure/* → Git / OS / LLM
                              ↑
                           domain/* (tipos compartilhados)
```

### `domain/` — o quê é

Tipos e regras sem I/O:

- Commits, trilha (`TrailEntry`), status, sync, blame
- `WriteRequest` / preview de operação
- Assistente: `LlmProviderKind`, `AssistantSettings`, DTOs de chat
- `trilho_help.rs` — catálogo de ajuda do produto (lido pela tool `get_trilho_help`)

### `application/` — como orquestra

- **Ports:** `GitReader`, `GitWriter`, `LlmProvider`, `TrailReader`, `BlameProvider` (`mod.rs`, `llm_provider.rs`)
- **`RepoContext`:** injeta reader (`Arc<dyn GitReader>`) + `SafeGitCli` (writer/CLI)
- **`write_service` / `operations` / `write_gates`:** preview + execute + gates de segurança (RF-08+)
- **`assistant_service`:** chat, allowlist de tools, revisão determinística, gates `send_diffs`
- **`clone_service`, `branch_origin`, `audit_service`, `AppState`**

### `infrastructure/` — adapters

- `git2_reader.rs` — leitura rápida (commits, status, origem…)
- `git_cli.rs` — **`SafeGitCli`**: único caminho preferido para Git CLI com config defensiva
- `llm/` — Ollama / OpenAI / Anthropic / **Claude Code** / **Codex CLI**
- `llm_credentials.rs`, `assistant_settings.rs`, `credential.rs`, `ssh_keys.rs`
- `branch_diff.rs`, `conflict.rs`, `reword.rs`, `repo_watcher.rs`, `github_pr.rs`…

### `commands.rs` + `lib.rs`

Commands devem ser **finos**: validar args, pegar `AppState` / `RepoContext`, chamar application, mapear erro para `String`. Registrar novos commands em `lib.rs` (`generate_handler!`).

---

## 5. Frontend

| Área | Onde |
|------|------|
| Shell | `src/App.tsx`, `src/main.tsx` |
| UI | `src/components/*` |
| Estado por domínio | `src/hooks/*` |
| IPC | `src/lib/api.ts` — `invoke("nome_comando", …)` |
| Tipos | `src/types.ts` (camelCase; Rust usa `serde(rename_all = "camelCase")`) |
| Grafo | `src/lib/graph/*`, `CommitGraph.tsx`, `GraphCanvas.tsx` |
| Dev sem Tauri | `npm run dev:web` + mocks em `src/lib/mock-data.ts` |

Padrão: hook chama `api.ts` → command Rust → atualiza estado → componentes reagem.

Assistente: `AssistantChat.tsx` (opt-in, provedores, chat, botão copiar por resposta do assistente).

---

## 6. Mapa de requisitos (RF) → código

| RF | Tema | Âncoras principais |
|----|------|--------------------|
| RF-01/02 | Trilha / dual trail / origem da branch | `domain/`, `application/branch_origin/`, `useCommits`, `CommitGraph` |
| RF-03 | Blame | `infrastructure/blame*.rs`, `useBlame`, `BlamePanel` |
| RF-04 | Status / diff WT | `status_parser`, `git2_reader`, `StatusPanel`, `DiffViewer` |
| RF-05–09,15–16,18 | Escritas (stage, commit, push/pull, revert, reset, force-push, reword, discard…) | `write_service`, `operations`, `write_gates`, diálogos |
| **RF-08** | Preview obrigatório antes de escrever | `preview_write_operation`, `OperationDialog` |
| RF-10 | Auth GCM / SSH / PAT | `credential.rs`, `ConnectDialog`, `useConnect` |
| RF-11 | Audit log | `audit_*`, `AuditLogDialog` |
| RF-12 | Status de PR | `github_pr.rs`, `PrStatusBadge` |
| RF-14 | Diff entre branches | `branch_diff.rs`, `BranchCompareDialog` |
| RF-19 | Watcher do repo | `repo_watcher.rs`, `useRepoChanged` |
| RF-20 | Conflitos 3 vias | `conflict.rs`, `Conflict*` |
| **RF-21** | Assistente LLM | `assistant_service.rs`, `infrastructure/llm/*`, `AssistantChat.tsx` |
| RF-22–24 | Clone, stash, tags | `clone_service`, diálogos correspondentes |

---

## 7. Padrões importantes

### Ports & adapters

Traits na application; implementações na infrastructure. UI e commands não falam com `git2`/`ureq` direto.

### SafeGitCli

- Toda invocação Git CLI passa por config defensiva (`protocol.ext.allow=never`, etc.).
- Gates de segurança usam **`run_bool` fail-closed** (erro ⇒ “não seguro”, não “liberar”).

### RF-08 — preview antes de escrever

1. UI/assistente propõe `WriteRequest`
2. `preview_write_operation` monta argv + resumo
3. Usuário confirma no `OperationDialog`
4. `execute_write_operation` roda **os mesmos** comandos pré-visualizados

**Nunca** executar escrita “silenciosa” pulando o preview.

### Assistente (RF-21) — allowlist default-deny

- Só tools em `allowlisted_tools`
- Sem shell / git arbitrário
- Escrita só via `propose_*` → vira `WriteRequest` → ainda passa RF-08
- Bloqueados no assistente (UI manual): reset, force push, reword, discard/hunk, etc. (`denied_tool_reason`)

Leituras úteis (não exaustivo):

| Tool | Papel |
|------|--------|
| `list_commits` | Amostra do grafo (**máx. 30**) — não é contagem total |
| `count_commits` | `git rev-list --count`; opcional `ref` + `exclude` → `exclude..ref` |
| `get_sync_info` | Ahead/behind vs upstream |
| `get_trilho_help` | Catálogo de produto (`domain/trilho_help.rs`) |

### Credenciais LLM

- **Nunca** em JSON de settings
- OpenAI/Anthropic: Windows Credential Manager via `git credential`, host `trilho.llm.{provider}`
- Preferências sem segredo: `{app_data_dir}/assistant_settings.json`

---

## 8. Assistente LLM — como funciona

### Provedores (`LlmProviderKind`)

| Valor serde | Uso |
|-------------|-----|
| `ollama` | Local / Cloud via app Ollama (`ollamaBaseUrl`) |
| `openAi` | API key (`api.openai.com`) |
| `codexCli` | Codex CLI (`codex login` / ChatGPT) — **≠** API key OpenAI |
| `anthropic` | API key (Console) — **≠** plano claude.ai |
| `claudeCode` | CLI/extensão Claude Code já logado no PC (plano Pro/Max) |

### Gates

- `enabled` — opt-in (default **off**)
- `send_diffs` — default **off**; necessário para revisão de código / tools de diff
- `send_metadata` — repo/branch/status/UI; o **gate de `send_diffs`** continua sendo informado mesmo com metadados off (intencional)

### Chat com tools (todos os provedores)

`run_chat` → até `MAX_TOOL_ROUNDS` → `run_tool` → leituras voltam ao modelo; `propose_*` acumula `pending_writes`.

| Provedor | Como as tools chegam ao runtime |
|----------|----------------------------------|
| Ollama / OpenAI | Tool-calling estilo OpenAI no HTTP |
| Anthropic (API key) | `tool_use` da Messages API |
| Claude Code / Codex CLI | Protocolo textual no adaptador — **mesma** allowlist |

CLIs não usam tool-calling HTTP da API no subprocesso. Os adaptadores
(`claude_code.rs`, `codex_cli.rs`) instruem o modelo a emitir:

```
<<<TRILHO_TOOL_CALLS>>>
[{"name":"count_commits","arguments":{}}]
<<<END_TRILHO_TOOL_CALLS>>>
```

Só nomes presentes na allowlist do request são aceitos (máx. 4 por rodada); o resto
é ignorado. O loop em `assistant_service` é idêntico aos outros provedores.

### Revisão determinística (quando `send_diffs` + pedido de review)

1. Trilho monta pacote de diffs no Rust (`build_code_review_packet`)
2. Chama o LLM **sem tool-calling** (`tools: []`; sem `context_preamble` completo)
3. Valida resposta (anti JSON falso, anti arquivos inventados, meta-review, etc.)
4. Se inválida: mensagem genérica; hint “modelo local / llama3.2” **só** se o provedor for Ollama

Isso existe porque modelos pequenos (ex. llama3.2) inventam tools/arquivos.

### Claude Code (`claudeCode`)

- Arquivo: `infrastructure/llm/claude_code.rs`
- Resolve binário: PATH → `~/.local/bin` → extensão VS Code/Cursor `anthropic.claude-code-*/resources/native-binary/claude.exe` (semver numérico)
- **Não** usar `--bare` (ignora OAuth do plano)
- Prompt grande via **stdin**; `--permission-mode dontAsk` (modo documentado)
- cwd neutro em temp (evita carregar `CLAUDE.md` do repo do Trilho **e** impede o agent do CLI de operar no working tree do usuário)
- **Sem Bash/`git` arbitrário do Claude Code** — só o que o Trilho executar na allowlist
- Chat geral: allowlist + protocolo `<<<TRILHO_TOOL_CALLS>>>`
- Timeout ~300s com kill

**Claude Desktop (chat) ≠ Claude Code CLI.**

### Codex CLI (`codexCli`)

- Arquivo: `infrastructure/llm/codex_cli.rs`
- Comando: `codex exec --ephemeral --sandbox read-only --skip-git-repo-check` (+ `--ask-for-approval never` se a build aceitar) `-`
- Auth: `codex login` / extensão ChatGPT (`~/.codex/auth.json`); subprocesso **remove** `OPENAI_API_KEY` / `CODEX_API_KEY` para não forçar API key
- Resolve binário (nesta ordem): extensão `openai.chatgpt-*/bin/windows-x86_64/codex.exe` → `%APPDATA%\npm` → PATH; no Windows só PE (MZ), ignora `bin/linux-*` da extensão
- cwd neutro + sandbox read-only; **sem Bash/`git` livre do agent** — só allowlist do Trilho
- Mesmo protocolo textual de tools que Claude Code
- Timeout ~300s com kill

**OpenAI (API key) ≠ Codex CLI (plano ChatGPT).**

Ajuda de produto para o modelo: `get_trilho_help` → `domain/trilho_help.rs` (manter alinhado a estas regras).

---

## 9. Onde ficam settings e estado

| Dado | Onde |
|------|------|
| Data dir do app | Tauri `app_data_dir()` (`AppState::data_dir`) |
| Preferências do Assistente | `{data_dir}/assistant_settings.json` |
| Repos recentes | `{data_dir}/recent_repos.json` |
| Audit | sob `data_dir` (purge na subida) |
| API keys LLM | Credential Manager (`trilho.llm.*`) |
| Tema / prefs de commit | `localStorage` (`lib/theme.ts`, etc.) |

---

## 10. Build, run e testes

```powershell
cd C:\Projetos\Trilho
npm install
npm run dev          # Tauri + Vite (desktop)
npm run dev:web      # só browser + mocks
npm run lint
npm run test         # Vitest
npm run test:rust    # clippy + cargo test
npm run test:e2e     # Playwright
npm run build:win    # instalador → src-tauri/target/release/bundle/
```

Requisitos: Node 20+, Rust/cargo, WebView2, Git for Windows (GCM recomendado).

Portas Vite: `1420` / `1421`. Se “address already in use”, encerrar `trilho.exe` / node na porta e subir de novo.

---

## 11. Onde mexer (mapa rápido)

| Quero… | Mexer em… |
|--------|-----------|
| Novo comando de leitura IPC | `commands.rs` + `lib.rs` + `src/lib/api.ts` + `types.ts` + hook/UI |
| Nova operação de escrita | `domain/operation.rs` (`WriteRequest`) → `write_service` / `operations` → preview RF-08 → gates → diálogo |
| Grafo / trilha | `git2_reader` + `src/lib/graph/*` + `CommitGraph` |
| Origem da branch | `application/branch_origin/*` |
| Nova tool do Assistente | `allowlisted_tools` + `run_tool` + testes em `assistant_service.rs` (+ help em `trilho_help.rs` se for produto) |
| Novo provedor LLM | `LlmProviderKind` + adapter em `infrastructure/llm` + `build_provider` + UI `AssistantChat` |
| Auth / sync | `credential.rs`, `ConnectDialog`, `SyncIndicator`, `useSync` |
| Conflitos | `infrastructure/conflict.rs` + componentes `Conflict*` |
| Texto de ajuda do Assistente | `domain/trilho_help.rs` |
| Decisão de produto | `DECISIONS.md` / specs em `Docs/git-trail-viewer/` |

---

## 12. Convenções e armadilhas (leia antes de mudar)

1. **Nunca pular RF-08** — preview e execute devem usar o mesmo argv.
2. Gates **fail-closed** — preferir `SafeGitCli::run_bool`.
3. Assistente **default-deny** — sem shell; reset/force-push/reword/discard só na UI dedicada.
4. **`send_diffs` off por padrão** — não enviar diffs sem opt-in.
5. **Sem API keys em arquivo** — só Credential Manager.
6. **Claude Code ≠ Anthropic API key**; **Codex CLI ≠ OpenAI API key** — CLIs autenticados no plano; sem `--bare` no Claude.
7. Commands finos; lógica na application.
8. Manter mocks de `dev:web` alinhados com `api.ts`.
9. Windows WebView: há keepalive de ícone e recovery de página em branco em `lib.rs` / frontend — não remover sem motivo.
10. UI e erros em **português**, tom curto e claro.
11. Após mudança sensível (gates, tools, LLM): testes unitários Rust no mesmo módulo.
12. Serde FE/Rust: **camelCase** nos DTOs públicos.
13. `writer()` em `RepoContext` é o **runner CLI** (`SafeGitCli`), não necessariamente “lock de escrita Git”; leituras via `rev-parse` usam esse runner quando o `Git2Reader` não expõe a API.
14. Pacotes grandes para Claude Code: stdin + threads drenando stdout/stderr **antes**/em paralelo ao write (evitar deadlock de pipe).

---

## 13. Visão mental do app (UI)

```
┌─────────────┬──────────────────────────────┬────────────────────┐
│ RefsPanel   │  CommitGraph (trilha)        │  StatusPanel       │
│ branches,   │  + DetailPanel / Assistente  │  Alterações + Diff │
│ tags,stash  │                              │  / editor          │
└─────────────┴──────────────────────────────┴────────────────────┘
         Sync / Connect / diálogos de operação (RF-08)
```

Centro-baixo: abas **Detalhes** | **Assistente**.

---

## 14. Checklist para uma IA antes de abrir PR

- [ ] Mudança respeita RF-08 se tocar escrita?
- [ ] Tools novas estão na allowlist e têm teste de allow/deny?
- [ ] Diffs só com `send_diffs`?
- [ ] Types TS + commands + `lib.rs` sincronizados?
- [ ] `npm run test` / `npm run test:rust` (ou o subset relevante) passando?
- [ ] Mensagens de erro claras (sem JSON cru de provedor, quando houver mapeamento)?

---

*Última atualização orientativa: RF-21 com tools em todos os provedores (Claude Code + Codex CLI via protocolo textual), resolução PE do Codex na extensão VS Code, `count_commits`, revisão determinística e SafeGitCli/RF-08.*
