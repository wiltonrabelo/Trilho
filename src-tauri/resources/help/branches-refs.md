# Branches e refs

Painel Refs: Ramos (locais), Remotos, Tags, Pilhas; pesquisa.
Checkout: git switch em local; remota com --track (ou switch local se a branch já existir).
Clique: foca commits exclusivos; duplo clique: checkout.

## Menu de contexto — Ramos (locais)
Botão direito num ramo:
- **Checkout** — desabilitado se já for a branch em checkout.
- **Remover localmente** — `git branch -D` (só local); desabilitado na branch atual.
- **Remover no repositório remoto (…)** — `git push <remote> --delete <branch>`; o rótulo
  deixa claro que remove **no servidor**; desabilitado na branch atual. Se não houver
  tracking listado, usa `origin` (ou o primeiro remoto conhecido).

## Menu de contexto — Remotos
- **Checkout** — desabilitado se já estiver nessa branch; cria tracking se ainda não
  houver local.
- **Remover no repositório remoto (…)** — remove no servidor; desabilitado se for a
  branch em checkout.
- **Remover localmente** — só se existir branch local com o mesmo nome (e não for a atual).

Todas as remoções passam pelo preview RF-08. O assistente **não** propõe delete de
branch — só a UI (Refs).

Comparar branches (RF-14): ícone nos Ramos — escolhe A/B, modo merge-base (`A...B`) ou tips (`A..B`),
lista de arquivos e diff por arquivo; layout lado a lado ou unificado; ordenação por checkouts recentes.
