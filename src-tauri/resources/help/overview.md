# Visão geral

Layout principal:
1. Esquerda — Repo picker / recentes; painel Refs (Ramos, Remotos, Tags, Pilhas).
2. Centro — grafo de commits (topo) + painel Detalhes | Assistente (baixo).
3. Direita — Alterações locais (topo) + diff/blame/editor (baixo).
4. Header — branch, origem, badge de PR, sync (fetch/push/pull), GitHub, **Terminal**
   (abre Git Bash no cwd do repo aberto), Ações, tema.

## Menus de contexto (botão direito)
- Em itens (ramos, tags, pilhas, commits, arquivos, recentes): menu **do Trilho**
  (Checkout, Remover, etc.) — não o menu do navegador.
- Em **espaço vazio** da UI: o clique direito **não faz nada** (menu nativo do
  WebView bloqueado de propósito).

## Terminal (Git Bash)
Com repositório aberto, botão **Terminal** no header abre o **Git Bash** já com
cwd na pasta do repo selecionado (`git-bash.exe --cd=…`). Exige Git for Windows
instalado. Não é um shell embutido no app; o assistente **não** pode abrir o
terminal nem rodar git arbitrário.

Painel inferior direito (arquivo do working tree):
- Abas **Alterações | Arquivo** no topo do diff.
- Sub-abas **Diff | Blame** dentro de Alterações.
- **Stage / Unstage / Descartar arquivo** ficam só no painel **Alterações** (lista de
  arquivos), não repetidos no painel do diff.

Princípios:
- Toda escrita passa por pré-visualização do comando Git real (RF-08) + confirmação.
- Exceção: salvar na aba **Arquivo** grava direto no working tree (sem stage automático).
- Detached HEAD: grafo em leitura; escritas desabilitadas.
- Ajuda do produto: catálogo embutido (`get_trilho_help`) — use a aba Assistente ou peça
  «como funciona X?» com LLM configurado.
