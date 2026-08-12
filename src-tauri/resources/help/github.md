# GitHub / conexão (RF-10, RF-12)

Botão GitHub / Conectar: GCM (login), PAT, SSH (listar .pub, testar ssh -T),
múltiplas contas (useHttpPath), logout.

## Status de PR (RF-12)
Badge/chips no header quando há credencial HTTPS e remoto GitHub:
- PR aberto / mergeado / fechado com link para o navegador.
- **github.com** e **GitHub Enterprise** (`github.*` + API `{host}/api/v3`).
- Token por host no Credential Manager.
- ≤2 PRs na branch: chips individuais; >2 PRs: menu dropdown.
- Cache ~60s; falha graciosa (rate limit, rede).
