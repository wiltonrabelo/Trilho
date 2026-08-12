# Operações de histórico

No painel Detalhes do commit selecionado (e no menu de contexto do grafo, quando couber):

## Reverter (`git revert`)
- Cria um **novo commit** que desfaz o selecionado; **não** apaga o histórico.
- **Permitido no HEAD (último commit)** — útil quando a empresa precisa desfazer o
  último envio com um commit de revert (planejamento / rollback seguro).
- **Não** disponível em commit de **merge** nesta versão (use os commits individuais
  da branch mesclada).
- Working tree precisa estar limpa; o resultado fica local até o usuário dar Push.
- Diferente de **Uncommit** (soft): uncommit remove o último commit e mantém alterações
  no working tree / stage, sem criar commit de revert.

## Outras ações
- Resetar para aqui — soft/mixed/hard; hard com backup/stash se WT suja. Só em commit
  que **não** é o HEAD.
- Cherry-pick — um ou vários (visão de branch); flag -x opcional; não no HEAD.
- Editar mensagem — **Amend** no HEAD ainda local; **reword** no HEAD já enviado
  (com force-with-lease) ou em commit anterior; reword já enviado exige confirmar
  o push forçado.
- Uncommit (soft) — desfaz o último commit mantendo alterações (quando elegível / local).
- Criar tag…

Via Assistente: pode propor revert (inclui HEAD), cherry-pick, push, pull, uncommit,
tags, stash… NÃO pode propor reset/force/reword (reescrevem histórico — só UI).
