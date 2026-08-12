# Segurança

- RF-08: preview do comando Git real antes de executar; a execução IPC exige
  o token de uso único emitido no preview (A-02) — o `WriteRequest` sozinho é
  rejeitado. Salvar na aba Arquivo faz preview interno + execute com o mesmo token
  (sem modal, mas com vínculo backend). Após falha na execução o token **não** é
  restaurado (pode ter havido efeito parcial) — é preciso novo preview.
- O «Comando Git» do diálogo é **uma linha por operação** (args juntados), não um
  argumento por linha; Cancelar/Confirmar ficam fixos mesmo em telas baixas.
- Spawn com lista de args (sem shell); paths confinados; validação de SHAs/refs.
- Editor interno: recusa symlink/junction e arquivos **> 2 MiB**.
- Timeouts Git (rede ~15 min / local ~2 min) com interrupção da árvore de processos.
- Credenciais no Windows Credential Manager / GCM.
- Assistente: allowlist + saída tratada como não confiável; prompt injection
  em diffs/mensagens é ignorado; destrutivas default-deny via assistente;
  fetch pelo assistente vira proposta RF-08 (não roda sozinho).

## Por que o «Comando Git» do diálogo parece longo? (RF-08)

Não é uma sequência de vários comandos. É **um** `git` (uma linha) com:

1. `-C <caminho-do-repo>` — roda no repositório aberto (sem depender do cwd do processo).
2. Vários `-c chave=valor` — **overrides defensivos** aplicados a **toda** invocação Git
   do Trilho (leitura e escrita), para o resultado ser previsível e não depender da
   config local do usuário (hooks, LFS, fsmonitor, sshCommand, etc.).
3. O verbo da operação — ex.: `add -A` (stage tudo), `commit …`, `push`, etc.

No Git Bash você costuma digitar só o verbo (`git add .`). No Trilho o preview mostra
o comando **como ele realmente será executado**, daí a aparência «cheia».

### Overrides fixos (não inventar outros)

Estes são exatamente os aplicados pelo executor seguro (`defensive_config_args`):

- `core.fsmonitor=false` — evita fsmonitor externo interferindo.
- `core.hooksPath=` — desativa hooks do repo/usuário nesta invocação.
- `core.sshCommand=` — anula `core.sshCommand` hostil da config local.
- `credential.helper=` + helper confiável do SO (`manager` / equivalente) —
  não herda `credential.helper=!…` do repositório.
- `uploadpack.packObjectsHook=` — desliga hook de pack-objects.
- Por remoto do repo: `remote.<nome>.uploadpack=git-upload-pack` e
  `receivepack=git-receive-pack`. Se existir `remote.<nome>.vcs=…`, a operação
  de rede é **recusada** (não dá para anular com `-c vcs=` sem quebrar o HTTPS).
- `gc.auto=0` — não dispara garbage collection automática no meio da operação.
- `protocol.ext.allow=never` — bloqueia protocolo `ext::` (risco de execução).
- `filter.lfs.required=false` e `filter.lfs.process=` / `clean=` / `smudge=` vazios —
  desliga filtros Git LFS nesta invocação.

Equivalência prática: stage tudo no preview ≈ `git add -A` no Bash (semelhante a
`git add .`). Os `-c`/`-C` são só o envelope de segurança do Trilho.

Se a dúvida do usuário não estiver neste catálogo, diga que **não está documentado**
no Trilho — **não invente** flags, motivos ou comportamento.
