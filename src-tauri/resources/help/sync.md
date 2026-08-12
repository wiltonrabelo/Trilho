# Sync (fetch / push / pull)

Barra de sync no header:
- Fetch — atualiza refs remotas.
- Push — envia commits locais.
- Pull — apenas --ff-only (sem merge automático).
- Force push — quando remoto está à frente (behind > 0); usa --force-with-lease + backup.
- Publicar — 1ª vez: remote + push -u (quando não há upstream).
- Completar histórico — fetch --unshallow em clone raso.
Erros de auth abrem o fluxo Conectar (GCM/PAT/SSH).
