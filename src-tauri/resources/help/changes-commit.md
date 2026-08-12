# Alterações e commit

Painel direito superior (**ALTERAÇÕES**): staged / unstaged / untracked.

Operações em lote e por arquivo (somente neste painel):
- Stage selecionados / Stage tudo / Unstage tudo / Descartar tudo / Guardar (stash).
- Por linha: checkbox + ícones no hover (+ stage, lixeira descartar).
- Barra do arquivo selecionado: Stage, Unstage, Descartar, Remover (untracked).

Commit (painel central inferior):
- Resumo + descrição opcional; amend quando permitido.
- Opção «Listar arquivos na descrição» pré-preenche +/~/- dos staged (localStorage).
  O texto começa com **duas linhas em branco** e depois «Arquivos do commit:», para
  sobrar espaço digitar mensagem manual acima da lista.

Uncommit (soft) no Detalhes do HEAD quando o commit ainda é local / elegível.

**Importante:** Stage e Descartar arquivo **não** aparecem no painel do diff — use o painel
Alterações acima. O painel do diff trata de visualização, blame e **reverter trecho**
(ver tópico `working-tree`).

## Por que o comando do preview parece «cheio»?
Stage (e qualquer escrita) abre o diálogo RF-08 com o comando Git **real** que o Trilho
vai executar. O final é o equivalente ao que você digitaria no Git Bash (ex.: stage tudo
→ `git add -A`, perto de `git add .`). Os muitos `-c …` e o `-C <pasta>` **não** são
passos extras: são overrides defensivos aplicados a **toda** escrita. Detalhe completo:
tópico `safety` (ou `comando-git`).
