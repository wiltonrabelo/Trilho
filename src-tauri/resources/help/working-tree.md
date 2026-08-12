# Working tree — Alterações, Arquivo, reverter trecho

Ao selecionar um arquivo alterado, o painel inferior direito oferece:

## Abas Alterações | Arquivo
- **Alterações** — diff unificado com ações por trecho; sub-abas Diff | Blame.
- **Arquivo** — editor do conteúdo atual no disco (working tree).

## Reverter trecho (RF-18)
Na aba Alterações → Diff, cada bloco (hunk) do diff tem botão **«Reverter trecho»**.
- Usa `git apply --reverse` (ou `--cached` se o trecho estiver staged).
- Hunks distantes no mesmo arquivo Git aparecem como **trechos separados**, cada um com
  seu botão — reverter um não desfaz o outro.
- Após reverter, o diff recarrega automaticamente.
- Preview RF-08 antes de executar.

Descartar o **arquivo inteiro** continua no painel **Alterações** (lista de arquivos).

## Editor na aba Arquivo
- Texto editável; indicador «Alterações não salvas» / «Salvo no working tree».
- **Salvar** ou **Ctrl+S** (com foco no editor) grava no disco **sem stage automático**.
- Bloqueado em detached HEAD, arquivo em conflito ou operação Git em andamento.
- Após salvar, status e diff do arquivo são atualizados.

## Destacar diff
Botão **«Destacar diff»** abre tela cheia com as mesmas abas Alterações | Arquivo,
Diff | Blame e reverter trecho — estado compartilhado com o painel normal.
**Restaurar** fecha o overlay e volta ao painel; o conteúdo (commit/arquivo selecionado)
permanece o que estava ao navegar.
