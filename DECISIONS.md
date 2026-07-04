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

## Enhancement (backlog pós-MVP)

- **Trilha comparada de duas branches** (pedido do stakeholder, 2026-07-03): selecionar duas
  branches e ver os commits de ambas na trilha, com divergência e convergência. Não coberto
  pelo PLANO (RF-14 compara **arquivos**, não trilha visual). Primeiro recorte entregue acima
  (branch atual vs base inferida). Evolução: seletor manual de base + linha da base como
  segunda lane + marcação dos merges de convergência.

## Próxima fase: M3 — Operações seguras

- RF-08 preview, RF-05 unstage, RF-15 commit, RF-06 uncommit, RF-07 revert, push
