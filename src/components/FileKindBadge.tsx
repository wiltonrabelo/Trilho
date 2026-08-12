import type { FileChangeKind } from "@/types";

const KIND_BADGE: Record<FileChangeKind, { label: string; className: string }> = {
  modified: {
    label: "M",
    className: "bg-amber-500/15 text-amber-700 dark:text-amber-400",
  },
  added: {
    label: "A",
    className: "bg-emerald-500/15 text-emerald-700 dark:text-emerald-400",
  },
  deleted: {
    label: "D",
    className: "bg-red-500/15 text-red-600 dark:text-red-400",
  },
  renamed: {
    label: "R",
    className: "bg-violet-500/15 text-violet-700 dark:text-violet-300",
  },
  untracked: {
    label: "U",
    className: "bg-muted/25 text-muted",
  },
  conflicted: {
    label: "!",
    className: "bg-orange-500/20 text-orange-700 dark:text-orange-300",
  },
};

/** Letra do status do arquivo (M/A/D/R/U/!) com a cor da categoria. */
export function KindBadge({ kind }: { kind: FileChangeKind }) {
  const b = KIND_BADGE[kind];
  return (
    <span
      className={`inline-flex h-4 w-4 shrink-0 items-center justify-center rounded text-[10px] font-bold ${b.className}`}
    >
      {b.label}
    </span>
  );
}
