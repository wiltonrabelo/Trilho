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

## Próxima fase: M2 — Análise

- RF-02 origem da branch (heurística + confiança)
- RF-03 blame (commit / working tree / staging)
