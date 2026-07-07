# Decisões de implementação — Trilho

## M0

### shadcn/ui adiado para M1+
- **Decisão:** não instalar shadcn/ui no M0; **continua adiado** (layout Tailwind puro).
- **Motivo:** componentes atuais são simples; shadcn entra quando o layout estabilizar (M2+).

### Ícone
- **Decisão:** ícone aprovado = locomotiva em movimento (line-art, traços de velocidade).
- **Fonte:** `assets/trilho-icon-1024.png` → `npm run tauri icon assets/trilho-icon-1024.png`.

## M1 + M1-b — Leitura + rede + grafo ✅ (encerrado)

### Escopo entregue
- Repo picker com repositórios recentes (max 8)
- Grafo com **lanes** (gitgraph), curvas suaves, HEAD/merge/local
- Status via `--porcelain=v2 -z` + badges M/A/D/R/U + seções staged/unstaged/untracked
- Diff lado a lado (commit e arquivo)
- Watcher seletivo (RF-19) com debounce e reconciliação
- `fetch` via Git CLI + botão **Conectar** (GCM)
- **Detecção proativa** de credential helper / GCM (`get_credential_status`)
- Banners: detached HEAD, branch sem upstream
- Paleta violeta (PLANO §8), tema claro/escuro
- Revwalk `TOPOLOGICAL | TIME`

### Auth RF-10 parcial ✅
- `GIT_TERMINAL_PROMPT=0` + `GCM_INTERACTIVE=always`
- Erros de auth → mensagem acionável + botão Conectar
- Hint proativo quando GCM não detectado e há upstream

### Pendências fora do M1 (M4 / processo)
- Certificado EV para instalador assinado — iniciar aquisição antes do M4
- Validação manual em repo >5k commits — checklist no README

## Dívidas técnicas — fechar ANTES do M3 (operações de escrita)

> Revisão SOLID/Clean Code (Claude, 2026-07-03). Itens localizados; nenhum bloqueia M2.

### Bloqueador para o M3
- [x] **Command pattern do PLANO §9** — `application/operations.rs` com `GitOperation`; `commands.rs` delega via `RepoContext::execute`.
- [x] **LSP no `SafeGitCli`** — `SafeGitCli::new(path)` + `impl GitWriter` honrando `preview()`/`run()` via `invoke()`.

### Refactors (uma sessão cada, sem prazo rígido)
- [x] **DIP na composição** — `RepoContext::open()` injeta `Arc<dyn GitReader>` + `SafeGitCli`.
- [x] **`App.tsx`** — extraído para `useRepo`, `useCommits`, `useSync` (~250 linhas).
- [x] **DRY upstream** — `infrastructure/upstream.rs` (`resolve_head_upstream`).
- [x] **Testes de integração Rust** — `git2_reader` (repo temp), `upstream`, `app_state::validate_git_repo`.

## M2 — Análise ✅ (encerrado)

### Escopo entregue
- **RF-02** origem da branch: heurística com pontuação (merge-base, first-parent, merge messages)
- Confiança honesta (Alta/Média/Baixa/Indeterminada); reflog só reforça, nunca infla sozinho
- Badge + banner no header (`BranchOriginBadge`)
- **RF-03** blame: três fontes (commit/HEAD, working tree, staging via `--contents -`)
- Parser `--line-porcelain`, painel `BlamePanel`, clique em linha no diff
- IPC: `get_branch_origin`, `get_file_blame`
- Testes: `branch_origin`, `blame_parser`, `blame` (repo temp)

## Pós-M2 — Trilha legível em repositório grande (2026-07-03)

- **Visão padrão "Trilha da branch"** (`--first-parent`, RF-01): linha única da branch atual;
  merges colapsados com badge. Toggle p/ "Grafo completo" (lanes). Motivo: SysPDV (311
  branches) tornava o grafo completo ilegível.
- **Divergência visível**: `BranchOrigin.merge_base_id` exposto; Trilha marca commits da
  branch (cor da lane) vs base (esmaecido), nó âmbar "⑂ divergiu de X" no merge-base.
- Perf: comandos IPC demorados `async` (sync roda na main thread do Tauri 2 e congela UI);
  heurística RF-02 com tetos (40 candidatas, walks limitados) + sinal de merge-base recente.

## Backlog pós-MVP — visão geral

> Itens **fora do MVP** (M0–M4), agrupados por **fluxo de trabalho** (UX). RF-21 por último (F7).

### 1. Entrada e remoto
| Prioridade | ID | Entrega | Fase |
|------------|-----|---------|------|
| Alta | **RF-22** | Clonar repositório remoto | F3 |
| Alta | **RF-10 completo** | OAuth / PAT / SSH + gestão de contas | F3 |

### 2. Navegação e refs (dia a dia)
| Prioridade | ID | Entrega | Fase |
|------------|-----|---------|------|
| Alta | **Checkout branch** | Trocar de branch (`git switch`) com preview RF-08 | F4 |
| Média-alta | **Painel Refs** | Sidebar estilo SourceTree: Ramos, Remotos, Tags, Stashes | F4 |
| Média-alta | **RF-23** | **Stash** (incl. untracked opcional) | F4 |

### 3. Working tree
| Prioridade | ID | Entrega | Fase |
|------------|-----|---------|------|
| Média | RF-18 hunk | Descartar por trecho (hunk) | F5 |

### 4. Histórico e metadados
| Prioridade | ID | Entrega | Fase |
|------------|-----|---------|------|
| Média | **RF-24** | **Criar tag** em commit (+ push opcional) | F4 |
| Média | RF-16 | Reword (editar mensagem de commit antigo) | F5 |

### 5. Operações avançadas
| Prioridade | ID | Entrega | Fase |
|------------|-----|---------|------|
| Média | RF-07 reset, RF-09 force push, RF-13 cherry-pick | Reescrita de histórico | F5 |
| Média | RF-14, RF-12, RF-20 | Diff branches, PR, conflitos 3-vias | F5 |
| Baixa | RF-11 | Log de auditoria (7 dias) | F5 |

### 6. Inteligência e enhancements
| Prioridade | ID | Entrega | Fase |
|------------|-----|---------|------|
| Baixa | **RF-21** | Assistente LLM → ações allowlisted | F7 |
| — | — | Trilha comparada de duas branches | — |

Especificação: `Docs/git-trail-viewer/PLANO.md` (§RF-21, §RF-22, §RF-23, §RF-24).

## Enhancement (backlog pós-MVP)

- **RF-22 — Clonar repositório remoto** (pedido do stakeholder, 2026-07-04): fluxo de entrada
  complementar ao repo picker, referência **SourceTree → Clone** (URL remota, pasta destino, nome
  da pasta, branch opcional, avançado colapsável, barra de progresso). Especificação completa em
  `Docs/git-trail-viewer/PLANO.md` §RF-22.

  **Por que não repetir o Publicar:**
  - Clone usa `git clone`, que já cria `origin` **e** tracking da branch — não depende de
    `remote add` + `push -u` manual.
  - Auth na **1ª vez**: GCM com `GCM_INTERACTIVE=always` durante o clone (mesmo env do
    `SafeGitCli`); hint proativo se GCM ausente; erros → `GitError::Auth` + botão Conectar.
  - **Checklist pós-clone:** `hasRemote`, `upstream` e sync ahead/behind OK sem banner "sem
    upstream"; nunca mensagem `git branch -u`.
  - **IPC:** um campo `url`; testes serde; registrar comando + rebuild (lições do Publicar).

  **Recorte 1:** URL + destino + nome + progresso + abrir repo. **Recorte 2:** ls-remote p/ branch,
  shallow, atalhos GitHub/GitLab.

- **RF-23 — Stash (guardar alterações)** (pedido do stakeholder, 2026-07-05): PLANO §RF-23.

  **UX (working tree):** botão **«Guardar (stash)»** no painel **Alterações locais** (junto a
  stage/unstage); diálogo com mensagem opcional + checkbox **«Incluir não rastreados»** (`-u`,
  desmarcada por padrão); resumo de quantos arquivos entram; aviso se untracked; preview RF-08.
  **Recorte 1:** `git stash push` de tudo (staged+unstaged+opcional untracked). **Recorte 2:**
  listar/aplicar/excluir stashes.

- **RF-24 — Criar tag** (pedido do stakeholder, 2026-07-05): PLANO §RF-24.

  **UX (histórico):** botão **«Criar tag…»** no `DetailPanel` do **commit selecionado**; diálogo
  com nome, tipo (anotada default / leve), mensagem, checkbox **«Enviar ao remoto»**; preview
  `git tag` + `git push origin <tag>` se marcado. **Recorte 2:** listar/excluir tags.

- **RF-21 — Assistente LLM (linguagem natural → operações Git no app)** (PLANO §RF-21, fase **F7**):
  painel de chat onde o usuário pede em português ("desfaz o último commit", "mostra diff deste
  arquivo", "publica no GitHub") e a LLM **traduz para ações já existentes do Trilho** — **não**
  executa shell/`git` arbitrário.

  **Modelo de execução (obrigatório):**
  - Porta `LlmProvider` plugável (OpenAI, Anthropic, **Ollama local**); chave no Credential Manager.
  - **Tool calling** com allowlist 1:1 → `WriteRequest` / leitura (`list_commits`, `get_status`,
    `get_file_diff`, `preview_write_operation`, etc.).
  - **Toda ação passa por RF-08** (preview do comando Git real) + confirmação humana; destrutivas
    com confirmação reforçada; **default-deny** de reset/force via assistente (opt-in explícito).
  - Opt-in global, **desligado por padrão**; controle do que vai ao provedor (metadados vs diffs).
  - Auditoria RF-11: entradas marcadas "originada pelo assistente".
  - Testes de prompt injection (§11.9 PLANO).

  **Pré-requisitos:** M3 estável (Command pattern + preview fiel); idealmente RF-10 completo e
  RF-22 antes, para cobrir clone/auth na allowlist.

  **Recorte 1:** chat + allowlist leitura + stage/unstage/commit/fetch. **Recorte 2:** push/pull,
  revert, cherry-pick quando existirem. **Recorte 3:** contexto de grafo/blame na conversa.

- **Trilha comparada de duas branches** (pedido do stakeholder, 2026-07-03): selecionar duas
  branches e ver os commits de ambas na trilha, com divergência e convergência. Não coberto
  pelo PLANO (RF-14 compara **arquivos**, não trilha visual). Primeiro recorte entregue acima
  (branch atual vs base inferida). Evolução: seletor manual de base + linha da base como
  segunda lane + marcação dos merges de convergência.

- **Checkout branch** (pedido do stakeholder, 2026-07-06): trocar a branch ativa sem sair do
  Trilho. Referência **SourceTree → Ramos** (duplo-clique / menu → Checkout).

  **Comando:** `git switch <branch>` (PLANO §7.7 — não `checkout` sobrecarregado).
  **Gates:** working tree suja → aviso; preview RF-08; detached HEAD ao trocar para commit.
  **Recorte 1:** lista de branches locais + checkout. **Recorte 2:** checkout de
  `origin/<branch>` (cria tracking local se necessário).

- **Painel Refs — organização estilo SourceTree** (referência UX, 2026-07-06): na barra lateral
  esquerda, **abaixo do repo picker**, seções colapsáveis com campo **Pesquisar** no topo:

  | SourceTree (PT) | Conteúdo | Trilho (proposto) |
  |-----------------|----------|-------------------|
  | **RAMOS** | Branches locais; checkout ao selecionar | Lista `refs/heads/*`; destaque na branch atual (HEAD) |
  | **REMOTOS** | Agrupado por remoto (`origin/…`) | `refs/remotes/origin/*`; hint se só `main` no fetch refspec |
  | **TAGS** | Tags locais e remotas | `refs/tags/*`; clique → seleciona commit no grafo |
  | **PILHAS** | Stashes salvos | Lista `git stash list` (RF-23 recorte 2) |

  **WORKSPACE SourceTree** (tabs Status / History / Search) **não** copiar literalmente — o Trilho
  já separa: **Alterações locais** (status) + **Trilha de commits** (history). O painel Refs
  complementa o header (hoje só mostra o nome da branch) e os chips de ref no grafo.

  **Ordem sugerida de implementação:** checkout (recorte 1) → painel Ramos/Remotos → RF-23
  (Pilhas) → RF-24 (Tags no painel + criar tag no commit).

## M3 — Operações seguras ✅ (encerrado)

### Escopo entregue
- **RF-08** preview antes de toda escrita: modal com comando copiável + confirmação
- **RF-05** unstage por arquivo e unstage all (`restore --staged`)
- **RF-15** commit com resumo/corpo + amend (só HEAD local)
- **RF-06** uncommit soft (só HEAD local)
- **RF-07** revert de commit selecionado
- Push upstream + pull `--ff-only` com gates (ahead/behind, upstream)
- Backend: `WriteRequest`, `write_service`, `write_gates`, `GitOperation` (M3 ops)
- IPC: `preview_write_operation`, `execute_write_operation` + evento `repo-changed`
- UI: `OperationDialog`, `CommitForm`, ações em `StatusPanel`, `DetailPanel`, `SyncIndicator`

### Gates
- Amend/uncommit bloqueados se HEAD já está no remoto
- Push bloqueado se `behind > 0`, `ahead == 0` ou sem upstream
- Pull bloqueado se `behind == 0`
- Operações de escrita desabilitadas em detached HEAD

## M4 — Empacotamento e qualidade ✅ (código; validação manual pendente)

### Escopo (MVP.md §M4 + §7)
- [x] **CI** — `.github/workflows/ci.yml` (lint, tsc, vitest, clippy, testes Rust; build instalador em push master)
- [x] **SECURITY.md** — canal de reporte e baseline documentada
- [x] **Scripts** — `npm run audit`, `npm run build:win`
- [x] **A11y básica** — skip link, `aria-label` em repo picker / commit / sync; diálogos já tinham `role="dialog"`
- [x] **Instalador (unsigned)** — `npm run build:win` OK: `Trilho_0.1.0_x64-setup.exe` + `.msi`
- [ ] **Instalador assinado** — certificado EV (lead time; unsigned para teste interno)
- [x] **E2E smoke (web)** — Playwright `e2e/smoke.spec.ts` + job CI (`npm run test:e2e`)
- [ ] **E2E desktop** — tauri-driver (opcional pós-MVP)
- [ ] **Validação SysPDV** — repo >5k commits (checklist README — manual)
- [x] **Hints amend/reword** — mensagens UX quando amend indisponível / reword RF-16
- [x] **Fixes revisão 2026-07-05** — `run_bool` fail-closed, preview amend+staging, gate revert merge (+3 testes)
- [x] **Paginação por cursor** — `list_commits(after)` + cache de refs (M4 perf >5k)
- [x] **A11y diálogos** — Escape, foco inicial, `useDialogA11y`
- [x] **Dependabot** — `.github/dependabot.yml`
- [x] **CI rustfmt + audit obrigatório**

### Próximo passo M4 (manual / processo)
1. Checklist README — SysPDV >5k commits + fluxos críticos
2. Certificado EV para instalador assinado

## Backlog F3 — RF-22 Clone remoto ✅

- [x] **Recorte 1** — URL + destino + nome + `git clone --progress` + preview RF-08 + abrir repo + progresso (`clone-progress`)
- [x] **Recorte 2** — seletor de branch (`ls-remote`), shallow clone, atalhos GitHub/GitLab
- [x] **Completar histórico** — `fetch --unshallow` na barra de sync (clone raso)
- [x] **Checklist pós-clone** — validar upstream/hasRemote após clone (`validate_post_clone` + teste + aviso na UI)

## Backlog F3 — RF-10 Conexão GitHub ✅

- [x] **Recorte 1** — `ConnectDialog` (login GCM + PAT), `configure_gcm_helper`, status `githubConnected` / `githubUsername`, botão «Conectar» no sync e header
- [x] **Recorte 2** — detecção de chaves `~/.ssh`, aba SSH no assistente, copiar `.pub`, `test_github_ssh` (`ssh -T git@github.com`)
- [x] **Recorte 3** — listar/remover contas GCM (`github list` / `logout`), «Adicionar conta», ativar `useHttpPath` para múltiplas contas HTTPS

## Backlog F4 — Navegação e refs 🚧

- [x] **Checkout branch — recorte 1** — listar branches locais + `git switch` + preview RF-08 + gate WT suja
- [x] **Checkout branch — recorte 2** — checkout de branch remota (`git switch --track origin/<branch>`)
- [x] **Painel Refs — recorte 1** — sidebar colapsável: Ramos + Remotos + pesquisa (estilo SourceTree)
- [x] **Painel Refs — recorte 2** — seção Tags listar/excluir + clique → commit no grafo (Pilhas ✅)

## Backlog F4 — RF-24 Tags ✅

- [x] **Recorte 1** — criar tag no commit selecionado (diálogo + preview RF-08 + push opcional)
- [x] **Recorte 2** — listar/excluir tags na seção Tags do painel Refs

## Backlog F5 — RF-18 Descartar alterações ✅

- [x] **Recorte 1** — descartar arquivo inteiro (`git restore --worktree`) + remover não rastreado (`git clean`)
- [x] **Recorte 2** — descartar hunk (`git apply --reverse`) no painel de diff
- [x] **Conflitos de revert** — parser `u`, abortar revert, gates de descarte
- [x] **Continuar revert/merge/cherry-pick** — após resolver conflitos (`--continue`), mensagem correta quando WT limpa mas `*_HEAD` persiste

## Backlog F5 — RF-16 Reword ✅

- [x] **Recorte 1** — reword de commit local não-HEAD via cherry-pick automatizado + diálogo
- [x] **Recorte 2** — reword de commit já enviado + `push --force-with-lease` (RF-09)

## Backlog F4 — RF-23 Stash ⏳

- [x] **Recorte 1** — `git stash push` (mensagem, `-u` opcional, preview RF-08) no painel Alterações
- [x] **Recorte 2** — listar/aplicar/excluir stashes (seção **Pilhas** no painel Refs)

## Backlog F5 — RF-13 Cherry-pick 🚧

- [x] **Recorte 1** — cherry-pick de um commit no painel do commit + preview RF-08 + gates (WT limpa, não-HEAD, não merge) + conflitos via continue/abort existentes
- [ ] **Recorte 2** — cherry-pick múltiplo + flag `-x`

## Dívidas técnicas pós-revisão M2 — ✅ fechadas (2026-07-05)

### 1. ISP — `GitReader` ✅
- Extraídos `TrailReader` (grafo/trilha) e `BlameProvider` (RF-03).
- `GitReader: TrailReader + BlameProvider` — status, sync, origem da branch.

### 2. `branch_origin` ✅
- Split em `branch_origin/{mod,candidates,scoring}.rs`.

### 3. `App.tsx` ✅
- Multi-seleção extraída para `hooks/useFileSelection.ts`.

### 4. Clippy ✅
- `npm run test:rust` inclui clippy `-D warnings`.
