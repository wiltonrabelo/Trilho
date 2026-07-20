import { ContextMenu, type ContextMenuItem } from "@/components/ContextMenu";
import type { CommitDto } from "@/types";

export type CommitContextMenuItem = ContextMenuItem;

interface CommitContextMenuProps {
  commit: CommitDto;
  x: number;
  y: number;
  items: CommitContextMenuItem[];
  onClose: () => void;
}

/** Menu de contexto do commit — mesmas ações do painel Detalhes. */
export function CommitContextMenu({
  commit,
  x,
  y,
  items,
  onClose,
}: CommitContextMenuProps) {
  return (
    <ContextMenu
      x={x}
      y={y}
      title={`${commit.shortId} · ${commit.summary}`}
      ariaLabel={`Ações do commit ${commit.shortId}`}
      emptyLabel="Nenhuma ação disponível neste commit."
      items={items}
      onClose={onClose}
    />
  );
}
