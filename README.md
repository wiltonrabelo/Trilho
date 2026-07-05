# Trilho

Cliente Git desktop minimalista para Windows — visualizar a trilha de commits, status,
diff e sincronização com remoto.

**Stack:** Tauri 2 · Rust · React 18 · TypeScript · Tailwind

## Requisitos

- Node.js 20+
- Rust (rustup) + cargo
- WebView2 (Windows 11 já inclui)
- Git for Windows (recomendado — inclui GCM para fetch/push)

## Comandos

```powershell
cd C:\Projetos\Trilho
npm install
npm run dev          # app desktop (Tauri)
npm run dev:web      # browser com mocks
npm run lint
npm run test         # Vitest (frontend)
npm run test:rust    # clippy + cargo test
npm run audit        # npm audit + cargo audit (M4)
npm run build:win    # instalador NSIS/MSI (unsigned)
```

Artefatos do instalador: `src-tauri\target\release\bundle\`

## Fases do MVP

| Fase | Status | Entrega |
|------|--------|---------|
| M0 | ✅ | Scaffolding, tema, arquitetura Git, baseline de segurança |
| M1 + M1-b | ✅ | Leitura, fetch, watcher, grafo com lanes, status/diff |
| M2 | ✅ | Origem da branch, blame |
| M3 | ✅ | Operações seguras (preview, unstage, commit, push/pull, publicar) |
| M4 | 🚧 | CI, SECURITY.md, a11y, instalador, validação SysPDV |

Documentação completa: `C:\Projetos\SysPDV\Docs\git-trail-viewer\MVP.md`

## Checklist de validação M4 (manual)

### Instalador e CI
- [ ] `npm run build:win` gera `.exe` (NSIS) e/ou `.msi`
- [ ] CI no GitHub Actions passa (lint, tsc, vitest, clippy, testes Rust)
- [ ] `SECURITY.md` publicado no repositório

### Repositório grande (SysPDV)
- [ ] Abre em < 3s; grafo paginado fluido com > 5k commits
- [ ] "Carregar mais" sem travar a UI

### Fluxos críticos (E2E manual)
- [ ] Abrir repo → status → stage → commit → push
- [ ] Fetch + ahead/behind + pull --ff-only
- [ ] Publicar branch nova (remote + upstream)
- [ ] Preview (RF-08) idêntico ao comando executado
- [ ] Detached HEAD e repo vazio degradam com aviso (sem crash)

### Acessibilidade (básica)
- [ ] Tab navega botões principais; skip link "Ir para o conteúdo"
- [ ] Diálogos com `role="dialog"` e foco visível
- [ ] Contraste tema claro/escuro (AA)

### Segurança
- [ ] `npm audit` / `cargo audit` sem críticas abertas (ou documentadas)
- [ ] Code signing EV — *lead time externo*; build unsigned OK para teste interno

Decisões e backlog: `DECISIONS.md` · Segurança: `SECURITY.md`
