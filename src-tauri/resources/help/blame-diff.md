# Diff e Blame

Painel inferior direito para arquivo do working tree ou de commit:

## Working tree
1. Abas **Alterações | Arquivo** (caminho completo do arquivo no topo).
2. Em Alterações: sub-abas **Diff | Blame**.
3. Diff com hunks e **Reverter trecho** (ver `working-tree`).
4. **Destacar diff** — mesma UI em tela cheia (ver abaixo).

## Blame (painel normal)
Tabela por linha: linha, commit (hash curto), autor, conteúdo.
Seletor **Commit | Working tree | Staging** no cabeçalho do Blame (só na aba Blame):
- **Working tree** — quem alterou cada linha na versão **no disco**.
- **Staging** — quem alterou cada linha na versão do **index** (staged).
- **Commit** — não aparece como fonte no painel normal (use o overlay; ver abaixo).
Clique numa linha do diff foca o blame nessa linha.
Linhas ainda não comitadas mostram autor **«Ainda não comitado»** (em vez do texto em inglês do Git).
Hashes do commit **não** são clicáveis no painel normal.

## Seletor Commit | Working tree | Staging (modo Destacar)
No **Destacar diff**, o seletor aparece nas abas **Diff** e **Blame** (barra acima do conteúdo).
Controla **o que o Diff mostra** e **de qual versão o Blame é calculado** (exceto navegação
por commit no overlay — ver abaixo).

| Seletor | Blame | Diff |
|--------|-------|------|
| **Working tree** | Autoria linha a linha do arquivo **no disco** | Alterações locais vs. HEAD; hunks com **Reverter trecho** |
| **Staging** | Autoria da versão no **index** (staged) | Diff do arquivo **como está no stage** |
| **Commit** | Lista **inalterada** (não recarrega ao clicar um hash) | Diff do **commit escolhido** no Blame para este arquivo |

Comportamento **Commit** no overlay:
- Clique num hash na aba Blame → aba **Diff** + seletor em **Commit** + legenda «Diff do commit abc1234».
- O grafo e a seleção global **não mudam** (navegação local ao overlay).
- **Working tree** no seletor → volta o diff das alterações locais; o último commit clicado fica
  memorizado — selecione **Commit** de novo para rever aquele diff.
- **Commit** sem ter clicado um hash antes → mensagem para escolher um commit na aba Blame.
- Linhas **«Ainda não comitado»** (`0000000`) não são clicáveis.

## Blame no modo Destacar diff (recursos extras)
Somente na tela cheia (**Destacar diff**), a aba Blame ganha:
- Coluna **Data** (data/hora da autoria, formato pt-BR).
- Colunas **redimensionáveis** (arraste a borda do cabeçalho); larguras salvas em localStorage.
- **Clique no hash do commit** (coluna Commit) — ver tabela do seletor acima.
- Fluxo típico: Destacar → Blame (WT) → clique no commit → Diff (Commit) → Working tree →
  Commit de novo → Blame → outro commit.

## Commit histórico (grafo)
Ao selecionar arquivo num commit passado: Diff | Blame sem abas Alterações|Arquivo.
No modo Destacar, explorar commits pelo blame usa navegação **local** (overlay), sem alterar
a seleção do grafo.
