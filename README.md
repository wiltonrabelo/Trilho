# Trilho

Cliente Git desktop minimalista para Windows — visualizar a trilha de commits, status,
diff e sincronização com remoto.

**Stack:** Tauri 2 · Rust · React 18 · TypeScript · Tailwind

## Requisitos

- Node.js 20+
- Rust (rustup) + cargo
- WebView2 (Windows 11 já inclui)
- Git for Windows (recomendado — inclui GCM para fetch)

## Comandos

```powershell
cd C:\Projetos\Trilho
npm install
npm run dev          # app desktop (Tauri)
npm run dev:web      # browser com mocks
npm run lint
npm run test         # Vitest (frontend)
npm run test:rust    # cargo test
```

## Fases do MVP

| Fase | Status | Entrega |
|------|--------|---------|
| M0 | ✅ | Scaffolding, tema, arquitetura Git, baseline de segurança |
| M1 + M1-b | ✅ | Leitura, fetch, watcher, grafo com lanes, status/diff |
| M2 | ✅ | Origem da branch, blame |
| M3 | ✅ | Operações seguras (preview, unstage, commit, push/pull) |
| M4 | ⏳ | Instalador assinado, E2E, a11y |

Documentação completa: `C:\Projetos\SysPDV\Docs\git-trail-viewer\MVP.md`

## Checklist de validação M1 (manual)

Validar em repositório real (ex.: SysPDV):

- [ ] Repo picker abre pasta válida e rejeita pasta sem `.git`
- [ ] Grafo com lanes, HEAD, merge, datas relativas, badge local
- [ ] Paginação "Carregar mais" em repo grande (>5k commits)
- [ ] Alterações: staged / unstaged / untracked + badges M/A/D/R/U
- [ ] Diff lado a lado (arquivo e commit)
- [ ] Fetch + indicador "última sync" + Conectar em falha de auth
- [ ] Watcher: editar arquivo externo reflete na UI em < 1s
- [ ] Banners detached HEAD e branch sem upstream

Decisões: `DECISIONS.md`
