import type { ContextMenuItem } from "@/components/ContextMenu";
import type { RemoteBranchRefDto } from "@/types";

/** Qual ref recebeu o clique direito no painel de refs, e onde. */
export type RefsMenuState =
  | {
      kind: "local";
      branch: string;
      x: number;
      y: number;
      active: boolean;
    }
  | {
      kind: "remote";
      remote: string;
      branch: string;
      x: number;
      y: number;
      active: boolean;
      hasLocal: boolean;
    }
  | {
      kind: "tag";
      name: string;
      commitId: string;
      x: number;
      y: number;
    }
  | {
      kind: "stash";
      index: number;
      reference: string;
      message: string;
      x: number;
      y: number;
    };

export interface RefsMenuAcoes {
  /** Alguma escrita em andamento: desabilita tudo que altera o repo. */
  busy: boolean;
  remoteBranches: RemoteBranchRefDto[];
  onSwitchLocal: (branch: string) => void;
  onSwitchRemote: (remote: string, branch: string) => void;
  onDeleteLocal?: (branch: string) => void;
  onDeleteRemote?: (remote: string, branch: string) => void;
  onStashApply: (index: number) => void;
  onStashPop: (index: number) => void;
  onStashDrop: (index: number) => void;
  onTagSelect: (commitId: string) => void;
  onTagDelete: (name: string) => void;
}

/** Itens do menu de contexto conforme a ref clicada. */
export function refsMenuItems(
  menu: RefsMenuState | null,
  acoes: RefsMenuAcoes
): ContextMenuItem[] {
  if (!menu) return [];
  switch (menu.kind) {
    case "tag":
      return itensDeTag(menu, acoes);
    case "stash":
      return itensDeStash(menu, acoes);
    case "local":
      return itensDeBranchLocal(menu, acoes);
    case "remote":
      return itensDeBranchRemota(menu, acoes);
  }
}

function itensDeTag(
  menu: Extract<RefsMenuState, { kind: "tag" }>,
  { busy, onTagSelect, onTagDelete }: RefsMenuAcoes
): ContextMenuItem[] {
  return [
    {
      id: "goto",
      label: "Ir para o commit",
      primary: true,
      onSelect: () => onTagSelect(menu.commitId),
    },
    {
      id: "delete-tag",
      label: "Excluir tag",
      separatorBefore: true,
      disabled: busy,
      onSelect: () => onTagDelete(menu.name),
    },
  ];
}

function itensDeStash(
  menu: Extract<RefsMenuState, { kind: "stash" }>,
  { busy, onStashApply, onStashPop, onStashDrop }: RefsMenuAcoes
): ContextMenuItem[] {
  return [
    {
      id: "apply",
      label: "Aplicar",
      primary: true,
      disabled: busy,
      onSelect: () => onStashApply(menu.index),
    },
    {
      id: "pop",
      label: "Pop (aplicar e remover)",
      disabled: busy,
      onSelect: () => onStashPop(menu.index),
    },
    {
      id: "drop",
      label: "Excluir",
      separatorBefore: true,
      disabled: busy,
      onSelect: () => onStashDrop(menu.index),
    },
  ];
}

function itensDeBranchLocal(
  menu: Extract<RefsMenuState, { kind: "local" }>,
  acoes: RefsMenuAcoes
): ContextMenuItem[] {
  const { busy, remoteBranches, onSwitchLocal, onDeleteLocal, onDeleteRemote } = acoes;
  const items: ContextMenuItem[] = [
    {
      id: "checkout",
      label: "Checkout",
      disabled: menu.active || busy,
      primary: !menu.active,
      onSelect: () => onSwitchLocal(menu.branch),
    },
  ];

  if (onDeleteLocal) {
    items.push({
      id: "delete-local",
      label: "Remover localmente",
      separatorBefore: true,
      disabled: menu.active || busy,
      onSelect: () => onDeleteLocal(menu.branch),
    });
  }

  if (onDeleteRemote) {
    const remotesToShow = remotesParaBranch(remoteBranches, menu.branch);
    for (const remote of remotesToShow) {
      items.push({
        id: `delete-remote-${remote}`,
        label: `Remover no repositório remoto (${remote})`,
        separatorBefore: !onDeleteLocal && remote === remotesToShow[0],
        disabled: menu.active || busy,
        onSelect: () => onDeleteRemote(remote, menu.branch),
      });
    }
  }

  return items;
}

function itensDeBranchRemota(
  menu: Extract<RefsMenuState, { kind: "remote" }>,
  { busy, onSwitchLocal, onSwitchRemote, onDeleteLocal, onDeleteRemote }: RefsMenuAcoes
): ContextMenuItem[] {
  const items: ContextMenuItem[] = [
    {
      id: "checkout",
      label: "Checkout",
      disabled: menu.active || busy,
      primary: !menu.active,
      onSelect: () => {
        if (menu.hasLocal) {
          onSwitchLocal(menu.branch);
        } else {
          onSwitchRemote(menu.remote, menu.branch);
        }
      },
    },
  ];

  if (onDeleteRemote) {
    items.push({
      id: "delete-remote",
      label: `Remover no repositório remoto (${menu.remote})`,
      separatorBefore: true,
      disabled: menu.active || busy,
      onSelect: () => onDeleteRemote(menu.remote, menu.branch),
    });
  }

  if (onDeleteLocal && menu.hasLocal) {
    items.push({
      id: "delete-local",
      label: "Remover localmente",
      disabled: menu.active || busy,
      onSelect: () => onDeleteLocal(menu.branch),
    });
  }

  return items;
}

/**
 * Remotos onde a branch existe. Sem nenhum, oferece `origin` (ou o primeiro
 * remoto conhecido) para que a opção de remover no remoto não suma.
 */
function remotesParaBranch(
  remoteBranches: RemoteBranchRefDto[],
  branch: string
): string[] {
  const comABranch = [
    ...new Set(remoteBranches.filter((r) => r.branch === branch).map((r) => r.remote)),
  ];
  if (comABranch.length > 0) return comABranch;

  const todos = [...new Set(remoteBranches.map((r) => r.remote))];
  if (todos.includes("origin")) return ["origin"];
  return todos.length > 0 ? [todos[0]!] : ["origin"];
}
