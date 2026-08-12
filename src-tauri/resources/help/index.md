# Ajuda do Trilho — índice

Trilho é um cliente Git desktop (Windows) focado na trilha de commits, com preview
obrigatório antes de qualquer escrita (RF-08) e log de auditoria (RF-11).

Como consultar: aba **Assistente** (com LLM configurado) ou ferramenta `get_trilho_help`.
Não há ainda um botão «Guia» dedicado na UI — o catálogo abaixo é a fonte de verdade.

Tópicos (passe `topic` em get_trilho_help):
- overview — visão geral, Terminal (Git Bash), menus de contexto
- open-clone — abrir repo, clonar remoto, recentes
- graph — grafo / trilha de commits / trilha comparada
- changes-commit — alterações, stage, commit, lista de arquivos na descrição
- working-tree — abas Alterações|Arquivo, editor, reverter trecho
- sync — fetch, push, pull, force push, publicar
- branches-refs — ramos, remotos, checkout, remoção local/remota, comparar
- stash-tags — pilhas (stash) e tags (menus de contexto)
- history-ops — revert (inclui HEAD), reset, reword, cherry-pick, uncommit
- conflicts — resolução 3 vias
- blame-diff — diff, blame, destacar diff
- github — conexão GitHub/GHE, PR, SSH/GCM
- audit — histórico de ações
- assistant — o que o assistente pode / não pode fazer
- safety — regras de segurança (preview RF-08, comando Git longo, default-deny)

Sobre o produto: responda só com o que este catálogo afirma. Se o tópico não
cobrir a dúvida, diga que não está documentado — não invente.
