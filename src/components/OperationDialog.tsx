import { Copy, X } from "lucide-react";
import { useRef, useState } from "react";

import { useDialogA11y } from "@/hooks/useDialogA11y";
import type { OperationPreviewDto } from "@/types";

interface OperationDialogProps {
  preview: OperationPreviewDto | null;
  loading?: boolean;
  onConfirm: () => void;
  onCancel: () => void;
  title?: string;
  /** Linha de progresso (ex.: git clone --progress). */
  progressLine?: string | null;
}

export function OperationDialog({
  preview,
  loading,
  onConfirm,
  onCancel,
  title = "Confirmar operação",
  progressLine,
}: OperationDialogProps) {
  const [copied, setCopied] = useState(false);
  const panelRef = useRef<HTMLDivElement>(null);

  useDialogA11y(Boolean(preview), onCancel, panelRef);

  if (!preview) return null;

  const blocked = Boolean(preview.blocked);

  async function copyCommands() {
    await navigator.clipboard.writeText(preview!.commands.join("\n"));
    setCopied(true);
    setTimeout(() => setCopied(false), 1500);
  }

  return (
    <div
      className="fixed inset-0 z-50 flex items-center justify-center bg-black/50 p-4"
      role="dialog"
      aria-modal="true"
      aria-labelledby="op-dialog-title"
    >
      <div
        ref={panelRef}
        className="max-h-[90vh] w-full max-w-lg overflow-auto rounded-xl border border-border bg-surface shadow-lg"
      >
        <div className="flex items-center justify-between border-b border-border px-4 py-3">
          <h2 id="op-dialog-title" className="text-sm font-semibold">
            {title}
          </h2>
          <button
            type="button"
            onClick={onCancel}
            className="rounded p-1 text-muted hover:bg-bg hover:text-text"
            aria-label="Fechar"
          >
            <X size={16} />
          </button>
        </div>

        <div className="space-y-3 px-4 py-3 text-sm">
          <p className="text-xs text-muted break-all">{preview.repoPath}</p>

          {preview.description && (
            <p className="text-text">{preview.description}</p>
          )}

          {progressLine && (
            <p className="truncate font-mono text-[10px] text-muted" title={progressLine}>
              {progressLine}
            </p>
          )}

          {blocked ? (
            <p className="rounded-md border border-amber-500/40 bg-amber-500/10 px-3 py-2 text-xs text-amber-700 dark:text-amber-300">
              {preview.blocked}
            </p>
          ) : (
            <div className="rounded-md border border-border bg-bg p-3">
              <div className="mb-2 flex items-center justify-between">
                <span className="text-[10px] font-semibold uppercase tracking-wide text-muted">
                  Comando Git
                </span>
                <button
                  type="button"
                  onClick={() => void copyCommands()}
                  className="flex items-center gap-1 text-[10px] text-accent hover:underline"
                >
                  <Copy size={12} />
                  {copied ? "Copiado" : "Copiar"}
                </button>
              </div>
              <pre className="overflow-x-auto whitespace-pre-wrap break-all font-mono text-xs text-text">
                {preview.commands.join("\n")}
              </pre>
            </div>
          )}
        </div>

        <div className="flex justify-end gap-2 border-t border-border px-4 py-3">
          <button
            type="button"
            onClick={onCancel}
            disabled={loading}
            className="rounded-lg border border-border px-3 py-1.5 text-xs text-muted hover:bg-bg disabled:opacity-50"
          >
            Cancelar
          </button>
          <button
            type="button"
            onClick={onConfirm}
            disabled={loading || blocked}
            className="rounded-lg bg-accent px-3 py-1.5 text-xs font-medium text-white hover:opacity-90 disabled:opacity-50"
          >
            {loading ? "Executando…" : "Confirmar"}
          </button>
        </div>
      </div>
    </div>
  );
}
