# Ajuda completa do Trilho

(Concatenação dos tópicos principais — use tópicos individuais para detalhe.)

## overview
Cliente Git desktop: grafo, preview RF-08, auditoria RF-11.
Layout: Refs | grafo+Detalhes/Assistente | Alterações+diff/editor.
Header com **Terminal** (Git Bash no cwd do repo). Menus de contexto nos itens;
espaço vazio sem menu do navegador. Stage/descartar na lista Alterações.

## open-clone
Abrir pasta Git; Clonar URL+destino+branch/shallow; unshallow na sync; recentes com
menu (trocar / remover).

## graph
Trilha de commits, alterações locais, paginação, trilha comparada (dual trail),
seletor Comparar com, badge convergência.

## changes-commit
Stage/unstage/commit/amend na lista Alterações; listar arquivos na descrição (com
duas linhas em branco antes de «Arquivos do commit:»); stash; uncommit.
Sem stage/descartar no painel do diff.

## working-tree
Abas Alterações|Arquivo; reverter trecho por hunk; editor com Salvar/Ctrl+S; destacar diff.

## sync
Fetch, push, pull --ff-only, force-with-lease, publicar, unshallow.

## branches-refs
Ramos/Remotos; checkout; menu: remover local / remover no repositório remoto;
comparar branches (diff de arquivos). Delete de branch só na UI.

## stash-tags
Stash e tags via menu de contexto (aplicar/pop/excluir; ir ao commit / excluir tag).

## history-ops
Revert (inclui HEAD; não merge), reset (não-HEAD), cherry-pick, reword (HEAD
enviado ou commit anterior; force-with-lease se remoto), uncommit, criar tag.
Assistente: revert/cherry-pick/push/pull/uncommit/tags/stash sim; reset/force/reword/
delete-branch/Terminal não.

## conflicts
3 vias, aceitar lados/blocos, continue/abort/skip.

## blame-diff
Diff|Blame; Alterações|Arquivo no WT; destacar diff; seletor Commit|WT|Staging no overlay
(WT=alterações locais, Staging=index, Commit=diff do hash clicado no Blame); blame com data/
colunas redimensionáveis; «Ainda não comitado»; navegação local sem mudar o grafo.

## github
GCM/PAT/SSH; badge PR (github.com + GHE, multi-PR).

## audit
Histórico 7 dias; marca assistente.

## assistant
Opt-in; Ollama/OpenAI/Anthropic/Claude Code/Codex CLI (mesmo loop de tools; CLIs via
protocolo textual). Leituras: list_commits (máx. 30), **count_commits** (rev-list
--count; exclude=base para branch desde main), sync, blame, etc. Com send_diffs:
revisão determinística sem tools + tools de diff no chat geral. Default-deny em
reset/force/reword/discard/shell; get_trilho_help topic=assistant.

## safety
Preview RF-08 + token A-02 one-shot (falha não restaura token); comando = **uma linha**
(`-C` + `-c` defensivos + uploadpack/receivepack + verbo); editor sem symlink e
≤2 MiB; timeouts Git; cofre de credenciais; default-deny destrutivas no assistente;
não inventar fora do catálogo.
