# Grafo / trilha

O grafo mostra a trilha de commits com lanes, refs e badge «local» quando ainda não
foi enviado. Clique num commit para ver Detalhes; «Alterações locais» no topo da trilha
mostra o working copy.

Paginação por cursor em repos grandes. Visão de branch focada (commits exclusivos de um ramo).
Load more carrega histórico mais antigo.

## Trilha comparada (dual trail)
Seletor **«Comparar com»** no grafo:
- **Auto** — usa origem inferida da branch ou base manual salva por repositório (localStorage).
- **Manual** — escolha outra ref local/remota como base.
- Layout dual: lane da branch atual + lane da base + trecho compartilhado.
- Badge de divergência no merge-base; badge «convergência» em merges da lane atual.

Diferente de **Comparar branches** (RF-14), que é diff de **arquivos** entre duas refs.
