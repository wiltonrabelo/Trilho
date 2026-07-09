import { Minimize2 } from "lucide-react";
import { useRef } from "react";

import { ConflictResolver } from "@/components/ConflictResolver";
import { useDialogA11y } from "@/hooks/useDialogA11y";

interface ConflictOverlayProps {
  open: boolean;
  onClose: () => void;
  path: string;
  operationKind?: "revert" | "merge" | "cherryPick" | null;
  writeDisabled?: boolean;
  onResolveSide: (side: "ours" | "theirs") => void;
  onResolveContent: (content: string) => void;
}

/** Tela cheia para resolver conflito — mesmo padrão do «Destacar diff». */
export function ConflictOverlay({
  open,
  onClose,
  path,
  operationKind,
  writeDisabled,
  onResolveSide,
  onResolveContent,
}: ConflictOverlayProps) {
  const panelRef = useRef<HTMLDivElement>(null);
  useDialogA11y(open, onClose, panelRef);

  if (!open) return null;

  return (
    <div
      className="fixed inset-0 z-50 flex flex-col bg-bg"
      role="dialog"
      aria-modal="true"
      aria-labelledby="conflict-overlay-title"
    >
      <div ref={panelRef} className="flex h-full min-h-0 flex-col">
        <div className="flex shrink-0 items-center justify-between gap-3 border-b border-border bg-surface px-4 py-2">
          <h2
            id="conflict-overlay-title"
            className="min-w-0 truncate font-mono text-xs font-medium"
            title={path}
          >
            Conflito · {path}
          </h2>
          <button
            type="button"
            onClick={onClose}
            className="flex shrink-0 items-center gap-1.5 rounded-lg border border-border px-2.5 py-1 text-xs text-muted hover:bg-bg hover:text-text"
          >
            <Minimize2 size={14} />
            Restaurar
          </button>
        </div>
        <div className="min-h-0 flex-1">
          <ConflictResolver
            path={path}
            expanded
            operationKind={operationKind}
            writeDisabled={writeDisabled}
            onResolveSide={onResolveSide}
            onResolveContent={onResolveContent}
          />
        </div>
      </div>
    </div>
  );
}
