# Modelo de ameaças — Trilho

Documento formal (hardening Codex). Complementa `SECURITY.md`.

| Campo | Valor |
|--------|--------|
| Versão | 0.1.x |
| Atualizado | 2026-08-10 |
| Escopo | App desktop Windows (Tauri 2 + WebView2 + Git CLI) |

## 1. Ativos

| Ativo | Por que importa |
|------|-----------------|
| Working tree / índice Git | Integridade do código do usuário |
| Credenciais (GCM, PAT, SSH, API keys LLM) | Acesso a remotes e provedores |
| Histórico / refs locais | Perda ou reescrita irreversível |
| Host (subprocessos) | Execução de código via Git/LLM |

## 2. Superfícies não confiáveis

Tratar como **não confiáveis** por padrão:

1. **Repositório local** — `.git/config`, hooks, fsmonitor, symlinks/junctions, conteúdo de arquivos  
2. **Remoto Git** — refs, objetos, mensagens de commit, URLs  
3. **Diff / blame / status** — texto exibido e enviado ao LLM  
4. **Resposta do LLM** — tool-calls, texto, pedidos de fetch/escrita  
5. **WebView / frontend** — IPC spoofing, XSS se CSP falhar  

## 3. Atores

| Ator | Capacidade |
|------|------------|
| Usuário legítimo | Confirma RF-08; escolhe repo; configura assistente |
| Repo hostil | Config/hooks/symlinks; diffs envenenados |
| Remoto hostil / MITM parcial | Objetos e refs maliciosos (Git + TLS do SO) |
| Prompt injection (LLM) | Induz tools ou pedidos de escrita/fetch |
| Processo local malicioso | Menor foco; assume usuário do mesmo perfil Windows |

## 4. Controles (estado atual)

| Ameaça | Controle | ID Codex |
|--------|----------|----------|
| Hook/fsmonitor/sshCommand/uploadpack/vcs no clone/fetch | Config efêmera (`core.hooksPath=`, `core.sshCommand=`, `uploadpack.packObjectsHook=`, `remote.*.uploadpack/receivepack`, helper OS); `remote.*.vcs` customizado → recusa | C-01 |
| Retry com token após execute parcial | Token A-02 consumido no `take`; falha **não** restaura — novo preview | A-02 |
| LLM faz `git fetch` sozinho | Fetch vira proposta RF-08 | A-01 |
| Execute sem preview / replay | Token one-shot preview→execute (write + clone) | A-02 |
| Editor segue symlink para fora do repo | `symlink_metadata` + rejeição de reparse | A-03 |
| PAT em texto puro | Credential Manager (+ migrate) | A-04 |
| Tools próprias do Claude Code | Allowlist vazia / bloqueio explícito | M-01 |
| Git/LLM pendurados | Timeouts + kill árvore (SafeGitCli, clone, ls-remote) | M-02 |
| Deps vulneráveis | `npm audit` + `cargo audit` no CI | M-03 |
| IPC amplo demais | Capabilities: read / write-propose / write-execute / secrets | M-04 |
| Diff/arquivo/conversa enormes | Truncamento no assistente; limite 2 MiB no editor | hardening |
| Escrita sem olhar comando | Diálogo RF-08 (comando real) + E2E Playwright | RF-08 / B-04 |

## 5. Fora de escopo (aceite residual)

- Bugs no Git for Windows, WebView2 ou SO  
- Assinatura EV do instalador sem certificado corporativo (pipeline prepara checksums; assinatura opcional via secrets)  
- E2E com `tauri-driver` controlando o binário nativo (opcional; cobrimos UI mock + contratos Rust)  
- Ameaça física / malware com mesmo usuário Windows  

## 6. Verificação contínua

| Gate | Onde |
|------|------|
| Lint / Clippy `-D warnings` / testes Rust | CI `quality` |
| `npm audit --audit-level=high` + `cargo audit` | CI `quality` |
| E2E RF-08 (Playwright) | CI `e2e` |
| Contratos de capabilities / configs defensivas | `security_contract` (cargo test) |
| SBOM CycloneDX (npm + cargo) | CI `sbom` |
| SHA-256 dos instaladores | CI `build-installer` |

## 7. Revisão

Revisitar este modelo a cada release que altere: IPC, assistente LLM, clone/auth ou empacotamento.
