import { Archive } from "lucide-react";
import { useEffect, useRef, useState } from "react";

import { useDialogA11y } from "@/hooks/useDialogA11y";

export interface StashDialogCounts {
  staged: number;
  unstaged: number;
  untracked: number;
}

interface StashDialogProps {
  open: boolean;
  counts: StashDialogCounts;
  loading?: boolean;
  error?: string | null;
  onCancel: () => void;
  onContinue: (message: string, includeUntracked: boolean) => void;
}

export function StashDialog({
  open: isOpen,
  counts,
  loading,
  error,
  onCancel,
  onContinue,
}: StashDialogProps) {
  const [message, setMessage] = useState("");
  const [includeUntracked, setIncludeUntracked] = useState(false);
  const panelRef = useRef<HTMLDivElement>(null);

  useDialogA11y(isOpen, onCancel, panelRef);

  useEffect(() => {
    if (!isOpen) {
      setMessage("");
      setIncludeUntracked(false);
    }
  }, [isOpen]);

  if (!isOpen) return null;

  const tracked = counts.staged + counts.unstaged;
  const stashable = tracked + (includeUntracked ? counts.untracked : 0);

  return (
    <div
      className="fixed inset-0 z-50 flex items-center justify-center bg-black/50 p-4"
      role="presentation"
      onMouseDown={(e) => {
        if (e.target === e.currentTarget) onCancel();
      }}
    >
      <div
        ref={panelRef}
        role="dialog"
        aria-modal="true"
        aria-labelledby="stash-dialog-title"
        className="w-full max-w-md rounded-xl border border-border bg-surface p-4 shadow-lg"
      >
        <div className="mb-3 flex items-center gap-2">
          <Archive size={18} className="text-accent" />
          <h2 id="stash-dialog-title" className="text-sm font-semibold text-text">
            Guardar alterações (stash)
          </h2>
        </div>

        <p className="mb-3 text-xs text-muted">
          {tracked > 0
            ? `${tracked} arquivo(s) rastreado(s) (${counts.staged} em stage, ${counts.unstaged} fora).`
            : "Nenhuma alteração rastreada."}
          {counts.untracked > 0
            ? ` ${counts.untracked} não rastreado(s) — marque a opção abaixo para incluir.`
            : ""}
        </p>

        <label className="mb-3 block text-xs text-muted">
          Mensagem (opcional)
          <input
            type="text"
            value={message}
            onChange={(e) => setMessage(e.target.value)}
            placeholder="Ex.: WIP antes de trocar de branch"
            disabled={loading}
            className="mt-1 w-full rounded-md border border-border bg-bg px-2 py-1.5 text-sm text-text placeholder:text-muted focus:border-accent focus:outline-none disabled:opacity-50"
          />
        </label>

        <label className="mb-4 flex cursor-pointer items-start gap-2 text-xs text-text">
          <input
            type="checkbox"
            checked={includeUntracked}
            onChange={(e) => setIncludeUntracked(e.target.checked)}
            disabled={loading || counts.untracked === 0}
            className="mt-0.5"
          />
          <span>
            Incluir não rastreados ({counts.untracked})
            {counts.untracked > 0 ? (
              <span className="block text-[10px] text-muted">
                Equivale a <code className="font-mono">git stash push -u</code>
              </span>
            ) : null}
          </span>
        </label>

        {stashable === 0 ? (
          <p className="mb-3 text-xs text-amber-600 dark:text-amber-400">
            Nada será guardado — inclua não rastreados ou altere arquivos antes.
          </p>
        ) : null}

        {error ? (
          <p className="mb-3 text-xs text-red-600 dark:text-red-400" role="alert">
            {error}
          </p>
        ) : null}

        <div className="flex justify-end gap-2">
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
            onClick={() => onContinue(message.trim(), includeUntracked)}
            disabled={loading || stashable === 0}
            className="rounded-lg bg-accent px-3 py-1.5 text-xs font-medium text-white hover:opacity-90 disabled:opacity-50"
          >
            Continuar…
          </button>
        </div>
      </div>
    </div>
  );
}
