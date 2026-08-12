import { ChevronDown, ChevronRight, Plus, Trash2, Undo2 } from "lucide-react";
import { useState, type MouseEvent } from "react";

import { KindBadge } from "@/components/FileKindBadge";
import type {
  WorktreeFileContext,
  WorktreeFileSection,
} from "@/components/statusPanelTypes";
import { fileCheckKey, type FileCheckSection } from "@/lib/fileCheck";
import type { FileChangeDto } from "@/types";

/** Uma seção recolhível da working tree (staged, unstaged ou untracked). */
export function FileList({
  title,
  files,
  staged,
  checkSection,
  menuSection,
  selectedPath,
  selectedStaged,
  checkedPaths,
  onSelect,
  onToggleCheck,
  onContextMenu,
  onStage,
  onUnstage,
  onDiscard,
  onRemoveUntracked,
}: {
  title: string;
  files: FileChangeDto[];
  staged: boolean;
  checkSection: FileCheckSection;
  menuSection: WorktreeFileSection;
  selectedPath: string | null;
  selectedStaged: boolean | null;
  checkedPaths: ReadonlySet<string>;
  onSelect: (
    path: string,
    staged: boolean,
    meta?: { ctrlKey?: boolean; shiftKey?: boolean }
  ) => void;
  onToggleCheck: (path: string, section: FileCheckSection) => void;
  onContextMenu?: (ctx: WorktreeFileContext) => void;
  onStage?: (path: string) => void;
  onUnstage?: (path: string) => void;
  onDiscard?: (path: string) => void;
  onRemoveUntracked?: (path: string) => void;
}) {
  const [collapsed, setCollapsed] = useState(false);

  function handleContextMenu(e: MouseEvent, f: FileChangeDto) {
    e.preventDefault();
    e.stopPropagation();
    onSelect(f.path, staged);
    onContextMenu?.({
      path: f.path,
      section: menuSection,
      kind: f.kind,
      clientX: e.clientX,
      clientY: e.clientY,
    });
  }

  return (
    <section className="mb-4 border-b border-border/60 pb-3 last:mb-0 last:border-0">
      <button
        type="button"
        onClick={() => setCollapsed((c) => !c)}
        className="mb-1.5 flex w-full items-center justify-between px-1 py-0.5 text-left hover:bg-surface/60"
        aria-expanded={!collapsed}
        title={collapsed ? "Expandir" : "Recolher"}
      >
        <span className="flex items-center gap-1 text-[10px] font-semibold uppercase tracking-wide text-muted">
          {collapsed ? <ChevronRight size={12} /> : <ChevronDown size={12} />}
          {title}
        </span>
        <span className="text-[10px] tabular-nums text-muted">{files.length}</span>
      </button>
      {collapsed ? null : files.length === 0 ? (
        <p className="px-2 py-1 text-xs text-muted/70">—</p>
      ) : (
        <ul className="space-y-0.5" onContextMenu={(e) => e.preventDefault()}>
          {files.map((f) => {
            const isSelected = selectedPath === f.path && selectedStaged === staged;
            const isChecked = checkedPaths.has(fileCheckKey(checkSection, f.path));
            const showStage = !staged && onStage && f.kind !== "conflicted";
            const showUnstage = staged && onUnstage && f.kind !== "conflicted";
            const showDiscard = !staged && onDiscard && f.kind !== "conflicted";
            const showRemove = !staged && onRemoveUntracked;
            return (
              <li
                key={`${staged}-${f.kind}-${f.path}`}
                className="group flex items-center gap-0.5"
              >
                <input
                  type="checkbox"
                  checked={isChecked}
                  onChange={() => onToggleCheck(f.path, checkSection)}
                  title="Selecionar para stage/unstage em lote"
                  className="ml-1 shrink-0 rounded border-border"
                />
                <button
                  type="button"
                  onClick={(e) =>
                    onSelect(f.path, staged, {
                      ctrlKey: e.ctrlKey || e.metaKey,
                      shiftKey: e.shiftKey,
                    })
                  }
                  onContextMenu={(e) => handleContextMenu(e, f)}
                  className={`flex min-w-0 flex-1 items-start gap-2 rounded-md px-2 py-1 text-left focus:outline-none focus-visible:ring-2 focus-visible:ring-accent/30 ${
                    isSelected || isChecked
                      ? "bg-surface ring-1 ring-border"
                      : "hover:bg-surface/60"
                  }`}
                  title={f.path}
                >
                  <KindBadge kind={f.kind} />
                  <span className="min-w-0 flex-1 break-all font-mono text-xs text-text">
                    {f.path}
                  </span>
                  {f.kind === "conflicted" &&
                    typeof f.conflictBlocks === "number" &&
                    f.conflictBlocks > 0 && (
                      <span
                        className="shrink-0 rounded bg-orange-500/15 px-1.5 py-0.5 text-[10px] font-medium tabular-nums text-orange-800 dark:text-orange-200"
                        title={`${f.conflictBlocks} bloco(s) em conflito`}
                      >
                        {f.conflictBlocks} bloco{f.conflictBlocks === 1 ? "" : "s"}
                      </span>
                    )}
                </button>
                {showStage && (
                  <button
                    type="button"
                    onClick={() => onStage(f.path)}
                    title="Stage"
                    className="shrink-0 rounded p-1 text-muted opacity-0 hover:bg-surface hover:text-accent group-hover:opacity-100 focus:opacity-100"
                  >
                    <Plus size={14} />
                  </button>
                )}
                {showUnstage && (
                  <button
                    type="button"
                    onClick={() => onUnstage(f.path)}
                    title="Unstage"
                    className="shrink-0 rounded p-1 text-muted opacity-0 hover:bg-surface hover:text-accent group-hover:opacity-100 focus:opacity-100"
                  >
                    <Undo2 size={14} />
                  </button>
                )}
                {showDiscard && (
                  <button
                    type="button"
                    onClick={() => onDiscard(f.path)}
                    title="Descartar alterações"
                    className="shrink-0 rounded p-1 text-muted opacity-0 hover:bg-surface hover:text-red-600 group-hover:opacity-100 focus:opacity-100 dark:hover:text-red-400"
                  >
                    <Trash2 size={14} />
                  </button>
                )}
                {showRemove && (
                  <button
                    type="button"
                    onClick={() => onRemoveUntracked(f.path)}
                    title="Remover arquivo não rastreado"
                    className="shrink-0 rounded p-1 text-muted opacity-0 hover:bg-surface hover:text-red-600 group-hover:opacity-100 focus:opacity-100 dark:hover:text-red-400"
                  >
                    <Trash2 size={14} />
                  </button>
                )}
              </li>
            );
          })}
        </ul>
      )}
    </section>
  );
}
