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
- Credenciais (GCM, PAT no Credential Manager, SSH, chaves LLM)
- Conteúdo malicioso em repositórios abertos (hooks, fsmonitor, symlinks)
- Assistente LLM (prompt injection → tools / escrita)

Fora de escopo: vulnerabilidades no Git, WebView2 ou no sistema operacional, salvo se o Trilho as expuser de forma evitável.

## Modelo de ameaças

Ver **[THREAT_MODEL.md](./THREAT_MODEL.md)** — ativos, superfícies não confiáveis, controles e aceite residual.

## Mitigações baseline

- Config efêmera Git: `core.fsmonitor=false`, `core.hooksPath=`, `core.sshCommand=`, helper de credencial do SO
- Preview RF-08 + token one-shot (A-02) para escrita e clone
- Capabilities Tauri: read / write-propose / write-execute / secrets
- Editor interno rejeita symlink/junction e arquivos > 2 MiB
- Timeouts + kill de árvore em operações Git
- CSP restrita no WebView (`tauri.conf.json`)

## Verificação

| Gate | Comando / CI |
|------|----------------|
| Deps | `npm run audit` (`npm audit` + `cargo audit`) |
| E2E RF-08 | `npm run test:e2e` |
| Contratos Rust | `cargo test --manifest-path src-tauri/Cargo.toml security_contract` |
| SBOM | job CI `sbom` (CycloneDX npm + cargo) |
| Instalador | SHA-256 nos artefatos; assinatura EV se `WINDOWS_CERT_PFX` |

## Política de patch

Correções de segurança **críticas** e **altas** entram em release patch assim que validadas.  
Dependências: `npm audit` e `cargo audit` no CI.
