# Política de segurança — Trilho

## Versões suportadas

| Versão | Suporte |
|--------|---------|
| 0.1.x  | ✅ Atual (MVP) |

## Reportar vulnerabilidade

**Não abra issue pública** para falhas de segurança.

Envie um e-mail para o mantenedor do repositório (GitHub: **wiltonrabelo**) com:

- Descrição do problema e impacto
- Passos para reproduzir
- Versão do Trilho e do Git for Windows (se aplicável)

Responderemos em até **5 dias úteis** com confirmação de recebimento.

## Escopo

- Execução de comandos Git via CLI (injeção de argumentos, path traversal)
- IPC Tauri / permissões do app
- Credenciais (GCM, tokens em memória)
- Conteúdo malicioso em repositórios abertos (hooks, fsmonitor — mitigado por config efêmera defensiva)

Fora de escopo: vulnerabilidades no Git, WebView2 ou no sistema operacional, salvo se o Trilho as expuser de forma evitável.

## Mitigações baseline (MVP)

- Toda invocação Git usa config efêmera: `core.fsmonitor=false`, `core.hooksPath=` (vazio), `gc.auto=0`
- Validação de paths, SHAs e URLs remotas antes de passar ao CLI
- `GIT_TERMINAL_PROMPT=0` + `GCM_INTERACTIVE=always` (auth via GUI, não terminal)
- CSP restrita no WebView (`tauri.conf.json`)

## Política de patch

Correções de segurança **críticas** e **altas** entram em release patch assim que validadas.  
Dependências: `npm audit` e `cargo audit` no CI (M4).
