import { Pencil } from "lucide-react";
import { useEffect, useRef, useState } from "react";

import { useDialogA11y } from "@/hooks/useDialogA11y";

interface RewordDialogProps {
  open: boolean;
  shortId: string;
  initialSummary: string;
  initialBody: string;
  /** Commit já enviado — exige push forçado após reword. */
  requiresForcePush?: boolean;
  loading?: boolean;
  error?: string | null;
  onCancel: () => void;
  onContinue: (summary: string, body: string, forcePush: boolean) => void;
}

export function RewordDialog({
  open: isOpen,
  shortId,
  initialSummary,
  initialBody,
  requiresForcePush = false,
  loading,
  error,
  onCancel,
  onContinue,
}: RewordDialogProps) {
  const [summary, setSummary] = useState(initialSummary);
  const [body, setBody] = useState(initialBody);
  const [forcePush, setForcePush] = useState(requiresForcePush);
  const panelRef = useRef<HTMLDivElement>(null);

  useDialogA11y(isOpen, onCancel, panelRef);

  useEffect(() => {
    if (isOpen) {
      setSummary(initialSummary);
      setBody(initialBody);
      setForcePush(requiresForcePush);
    }
  }, [isOpen, initialSummary, initialBody, requiresForcePush]);

  if (!isOpen) return null;

  const canContinue =
    summary.trim().length > 0 && (!requiresForcePush || forcePush);

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
        aria-labelledby="reword-dialog-title"
        className="w-full max-w-md rounded-xl border border-border bg-surface p-4 shadow-lg"
      >
        <div className="mb-3 flex items-center gap-2">
          <Pencil size={18} className="text-accent" />
          <h2 id="reword-dialog-title" className="text-sm font-semibold text-text">
            Editar mensagem do commit {shortId}
          </h2>
        </div>

        <p className="mb-3 text-xs text-muted">
          Reescreve a mensagem e reaplica os commits seguintes — todos receberão novos SHAs.
          {requiresForcePush
            ? " Como este commit já está no remoto, o histórico reescrito será enviado com push forçado (--force-with-lease)."
            : " Só vale para commits ainda não enviados ao remoto."}
        </p>

        {requiresForcePush && (
          <label className="mb-3 flex cursor-pointer items-start gap-2 rounded-md border border-amber-500/40 bg-amber-500/10 px-3 py-2 text-xs text-amber-800 dark:text-amber-200">
            <input
              type="checkbox"
              checked={forcePush}
              onChange={(e) => setForcePush(e.target.checked)}
              className="mt-0.5"
            />
            <span>
              Enviar histórico reescrito ao remoto com{" "}
              <span className="font-mono">push --force-with-lease</span> (obrigatório)
            </span>
          </label>
        )}

        <label className="mb-3 block text-xs text-muted">
          Resumo
          <input
            type="text"
            value={summary}
            onChange={(e) => setSummary(e.target.value)}
            className="mt-1 w-full rounded-lg border border-border bg-bg px-2 py-1.5 text-sm text-text"
            autoFocus
          />
        </label>

        <label className="mb-3 block text-xs text-muted">
          Corpo (opcional)
          <textarea
            value={body}
            onChange={(e) => setBody(e.target.value)}
            rows={4}
            className="mt-1 w-full resize-y rounded-lg border border-border bg-bg px-2 py-1.5 text-sm text-text"
          />
        </label>

        {error ? (
          <p className="mb-3 rounded-md border border-red-500/40 bg-red-500/10 px-3 py-2 text-xs text-red-600 dark:text-red-400">
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
            disabled={loading || !canContinue}
            onClick={() => onContinue(summary.trim(), body.trim(), forcePush)}
            className="rounded-lg bg-accent px-3 py-1.5 text-xs font-medium text-white hover:opacity-90 disabled:opacity-50"
          >
            {loading ? "Abrindo preview…" : "Continuar"}
          </button>
        </div>
      </div>
    </div>
  );
}
