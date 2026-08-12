import { KindBadge } from "@/components/FileKindBadge";
import type { CommitFileContext } from "@/components/statusPanelTypes";
import type { FileChangeDto } from "@/types";

/** Arquivos tocados pelo commit selecionado. */
export function CommitFileList({
  files,
  selectedPath,
  onSelect,
  onContextMenu,
}: {
  files: FileChangeDto[];
  selectedPath: string | null;
  onSelect: (path: string) => void;
  onContextMenu?: (ctx: CommitFileContext) => void;
}) {
  if (files.length === 0) {
    return (
      <p className="px-2 py-4 text-center text-xs text-muted">
        Nenhum arquivo alterado neste commit
      </p>
    );
  }
  return (
    <ul className="space-y-0.5" onContextMenu={(e) => e.preventDefault()}>
      {files.map((f) => {
        const isSelected = selectedPath === f.path;
        return (
          <li key={`${f.kind}-${f.path}`}>
            <button
              type="button"
              onClick={() => onSelect(f.path)}
              onContextMenu={(e) => {
                e.preventDefault();
                e.stopPropagation();
                onSelect(f.path);
                onContextMenu?.({
                  path: f.path,
                  kind: f.kind,
                  clientX: e.clientX,
                  clientY: e.clientY,
                });
              }}
              className={`flex w-full items-start gap-2 rounded-md px-2 py-1 text-left focus:outline-none focus-visible:ring-2 focus-visible:ring-accent/30 ${
                isSelected ? "bg-surface ring-1 ring-border" : "hover:bg-surface/60"
              }`}
              title={f.path}
            >
              <KindBadge kind={f.kind} />
              <span className="min-w-0 flex-1 break-all font-mono text-xs text-text">
                {f.path}
              </span>
            </button>
          </li>
        );
      })}
    </ul>
  );
}
