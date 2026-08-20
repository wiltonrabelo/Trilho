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
| LLM plano ChatGPT | subprocesso **Codex CLI** (não API key) |
| Testes FE | Vitest + jsdom |
| Testes E2E | Playwright (`e2e/` — smoke + RF-08); contratos Rust `security_contract` |
| Segurança | `SECURITY.md`, `THREAT_MODEL.md`; SBOM + audit no CI |
| Testes Rust | `cargo test` + `clippy -D warnings` |

---

## 3. Layout do repositório

```
Trilho/
  README.md, DECISIONS.md, SECURITY.md, THREAT_MODEL.md, ARCHITECTURE.md
  package.json, vite.config.ts, vitest.config.ts, playwright.config.ts
  src/                         # Frontend React/TS
    App.tsx                    # Orquestração principal da UI
    main.tsx
    components/                # Painéis, diálogos, grafo, Assistente…
                               # OperationDialog, StatusPanel, ResizableBottomSection…
    hooks/                     # useRepo, useCommits, useSync, useOperations…
    lib/                       # api.ts (invoke), graph/, mocks, theme…
    types.ts                   # DTOs camelCase (espelham serde)
  src-tauri/
    Cargo.toml
    permissions/               # capabilities granulares (read / propose / execute / secrets)
    capabilities/default.json
    src/
      main.rs                  # entry binário
      lib.rs                   # registra módulos, AppState, commands
      commands.rs              # #[tauri::command] finos
      domain/                  # tipos puros + help embutido (`trilho_help.rs`)
      application/             # ports, serviços, gates, orquestração
      infrastructure/          # adapters (git2, CLI, LLM, credenciais…)
      security_contract.rs     # testes de contrato (capabilities / configs / limites)
  e2e/                         # Playwright: smoke + security-rf08
  assets/, public/
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
- `llm/` — Ollama / OpenAI / Anthropic / **Codex CLI**
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
2. `preview_write_operation` monta argv + resumo e, se não bloqueado, emite **token de uso único** (A-02)
3. Usuário confirma no `OperationDialog`
4. `execute_write_operation` recebe **só o token** (não o request do cliente), consome atomicamente, recalcula preview/gates e exige argv idêntico ao autorizado

`SafeGitCli::preview` devolve **uma string por comando** (argv juntado com espaços / aspas), não um elemento por argumento — o diálogo mostra uma linha legível. O `OperationDialog` tem rodapé fixo (Cancelar/Confirmar) e área do comando rolável (importante em 1024×768).

**Nunca** executar escrita “silenciosa” pulando o preview. **Nunca** aceitar `WriteRequest` solto no execute IPC.

### Assistente (RF-21) — allowlist default-deny

- Só tools em `allowlisted_tools`
- Sem shell / git arbitrário
- Escrita só via `propose_*` → vira `WriteRequest` → ainda passa RF-08
- **`propose_fetch_remote`** (não `fetch_remote` automático) — altera refs; exige confirmação
- Bloqueados no assistente (UI manual): reset, force push, reword, discard/hunk, etc. (`denied_tool_reason`)

Leituras úteis (não exaustivo):

| Tool | Papel |
|------|--------|
| `list_commits` | Amostra do grafo (**máx. 30**) — não é contagem total |
| `count_commits` | `git rev-list --count`; opcional `ref` + `exclude` → `exclude..ref` |
| `get_sync_info` | Ahead/behind vs upstream |
| `get_trilho_help` | Catálogo de produto (`domain/trilho_help.rs`) |

### Credenciais

- **Nunca** em JSON de settings
- OpenAI/Anthropic API keys: Credential Manager via `git credential`, host `trilho.llm.{provider}`
- PAT GitHub (API de PRs): mesmo mecanismo, host `trilho.llm.github.api` (migra/apaga arquivo legado `github_api_pat`)
- Preferências sem segredo: `{app_data_dir}/assistant_settings.json`

### Git endurecido (rede / RCE via config local)

- Toda invocação via `SafeGitCli` aplica `-c` defensivo: hooks off, `protocol.ext` never, LFS filters off, **`core.sshCommand=`**, **`credential.helper=`** + helper confiável do SO (`manager` / `manager-core` / `osxkeychain`, preferindo o global allowlisted), **`uploadpack.packObjectsHook=`**
- Por remoto do repo: força `remote.<n>.uploadpack=git-upload-pack` e `receivepack=git-receive-pack`; se `remote.<n>.vcs` estiver definido, a op de rede é **recusada** (não usar `-c vcs=` vazio — quebra o remote helper HTTPS)
- Timeout por classe (rede 15 min / local 2 min) com kill da árvore de processos — também em `run_unbound_git` (`ls-remote`) e `wait_child_status_with_timeout` (`git clone` com progresso)
- `fetch_all_remote_branch_refs` usa `SafeGitCli` (não `Command::new("git")` cru)
- Clone remoto também exige token A-02 (`preview_clone_remote` → `execute_clone_remote`)
- Token A-02 é **one-shot**: após `take` no execute, falha **não** restaura o token (evita retry sobre operação parcial); UI pede novo preview
- Capabilities Tauri: `allow-repo-read` / `allow-repo-write-propose` / `allow-repo-write-execute` / `allow-secrets` (sem permissão monolítica `allow-repo-commands`)
- `save_worktree_file` rejeita symlink/junction no caminho e conteúdo **> 2 MiB** (limite do editor interno)
- Verificação contínua: `npm audit` + `cargo audit`, E2E RF-08, SBOM CycloneDX, `THREAT_MODEL.md`

---

## 8. Assistente LLM — como funciona

### Provedores (`LlmProviderKind`)

| Valor serde | Uso |
|-------------|-----|
| `ollama` | Local / Cloud via app Ollama (`ollamaBaseUrl`) |
| `openAi` | API key (`api.openai.com`) |
| `codexCli` | Codex CLI (`codex login` / ChatGPT) — **≠** API key OpenAI |
| `anthropic` | API key (Console) |

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
| Codex CLI | Protocolo textual no adaptador — **mesma** allowlist |

O Codex CLI não usa tool-calling HTTP da API no subprocesso. O adaptador
(`codex_cli.rs` + `cli_protocol.rs`) instrui o modelo a emitir:

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

### Codex CLI (`codexCli`)

- Arquivo: `infrastructure/llm/codex_cli.rs`
- Comando: `codex exec --ephemeral --sandbox read-only --skip-git-repo-check` (+ `--ask-for-approval never` se a build aceitar) `-`
- Auth: `codex login` / extensão ChatGPT (`~/.codex/auth.json`); subprocesso **remove** `OPENAI_API_KEY` / `CODEX_API_KEY` para não forçar API key
- Resolve binário (nesta ordem): extensão `openai.chatgpt-*/bin/windows-x86_64/codex.exe` → `%APPDATA%\npm` → PATH; no Windows só PE (MZ), ignora `bin/linux-*` da extensão
- cwd neutro + sandbox read-only; **sem Bash/`git` livre do agent** — só allowlist do Trilho
- Protocolo textual de tools (`cli_protocol.rs`)
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

1. **Nunca pular RF-08** — preview emite token; execute consome só o token e exige o mesmo argv.
2. Gates **fail-closed** — preferir `SafeGitCli::run_bool`.
3. Assistente **default-deny** — sem shell; reset/force-push/reword/discard só na UI dedicada.
4. **`send_diffs` off por padrão** — não enviar diffs sem opt-in.
5. **Sem API keys em arquivo** — só Credential Manager.
6. **Codex CLI ≠ OpenAI API key** — CLI autenticado no plano ChatGPT; Anthropic só via API key do Console.
7. Commands finos; lógica na application.
8. Manter mocks de `dev:web` alinhados com `api.ts`.
9. Windows WebView: há keepalive de ícone e recovery de página em branco em `lib.rs` / frontend — não remover sem motivo.
10. UI e erros em **português**, tom curto e claro.
11. Após mudança sensível (gates, tools, LLM): testes unitários Rust no mesmo módulo.
12. Serde FE/Rust: **camelCase** nos DTOs públicos.
13. `writer()` em `RepoContext` é o **runner CLI** (`SafeGitCli`), não necessariamente “lock de escrita Git”; leituras via `rev-parse` usam esse runner quando o `Git2Reader` não expõe a API.
14. Pacotes grandes para Codex CLI: stdin + threads drenando stdout/stderr **antes**/em paralelo ao write (evitar deadlock de pipe).

---

## 13. Visão mental do app (UI)

```
┌─────────────┬──────────────────────────────┬────────────────────┐
│ RefsPanel   │  CommitGraph (trilha)        │  StatusPanel       │
│ branches,   │  + DetailPanel / Assistente  │  Alterações + Diff │
│ tags,stash  │                              │  / editor          │
└─────────────┴──────────────────────────────┴────────────────────┘
         Sync / Connect / diálogos de operação (RF-08)
                              ↓
                    CommitForm (ResizableBottomSection)
```

Centro-baixo: abas **Detalhes** | **Assistente**.

Coluna direita em telas baixas (ex. 1024×768): seções **Staged / Unstaged / Untracked** recolhíveis no `StatusPanel`; formulário de commit abaixo com altura redimensionável (`ResizableBottomSection`); `OperationDialog` com rodapé fixo.

---

## 14. Checklist para uma IA antes de abrir PR

- [ ] Mudança respeita RF-08 se tocar escrita?
- [ ] Tools novas estão na allowlist e têm teste de allow/deny?
- [ ] Diffs só com `send_diffs`?
- [ ] Types TS + commands + `lib.rs` sincronizados?
- [ ] `npm run test` / `npm run test:rust` (ou o subset relevante) passando?
- [ ] Mensagens de erro claras (sem JSON cru de provedor, quando houver mapeamento)?
- [ ] Help embutido (`trilho_help.rs`) e este arquivo atualizados se mudar RF-08 / assistente / segurança?

---

*Última atualização orientativa: parecer Codex fechado (A-02, timeouts, capabilities, symlink/2 MiB, E2E RF-08, SBOM, THREAT_MODEL); preview RF-08 em uma linha; UI Alterações recolhível + commit redimensionável.*
