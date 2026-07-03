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
- [ ] **Command pattern do PLANO §9 não materializado** — `commands.rs` monta `GitCommand { args }` inline
  (`get_file_diff`, `get_commit_diff`, `fetch_remote`). O RF-08 (preview fiel) exige objetos de operação
  com `preview()`/`run()` no mesmo lugar. Criar os Commands **antes** da primeira operação de escrita,
  senão o preview nasce desacoplado do que executa.
- [ ] **LSP quebrado no `SafeGitCli`** — implementa `GitWriter`, mas `run()` do trait retorna erro fixo
  ("use o método estático"). Corrigir dando `repo_path` ao construtor (`SafeGitCli::new(path)`) e
  honrando o trait — ou remover o `impl GitWriter` até lá.

### Refactors (uma sessão cada, sem prazo rígido)
- [ ] **DIP na composição** — `reader_for()` retorna `Git2Reader` concreto; nada injeta as portas.
  Ao introduzir os Commands, compor via traits (facilita mock nos testes de caso de uso).
- [ ] **`App.tsx` (~555 linhas)** — god component. Extrair hooks: `useRepo`, `useCommits`, `useSync`.
- [ ] **Duplicação em `git2_reader`** — resolução de upstream repetida 3× (`upstream_oid`,
  `get_sync_info`, `repo_info`). Extrair helper único.
- [ ] **Testes de integração Rust** (PLANO §12) — `git2_reader` e `app_state` sem testes; cobrir com
  repositórios temporários (`tempfile`) exercitando git real.

## Próxima fase: M2 — Análise

- RF-02 origem da branch (heurística + confiança)
- RF-03 blame (commit / working tree / staging)
