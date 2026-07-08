import { RotateCcw } from "lucide-react";
import { useEffect, useRef, useState } from "react";

import { useDialogA11y } from "@/hooks/useDialogA11y";
import { previewWriteOperation } from "@/lib/api";

export type ResetModeDto = "soft" | "mixed" | "hard";

interface ResetDialogProps {
  open: boolean;
  shortId: string;
  requiresForcePush?: boolean;
  loading?: boolean;
  error?: string | null;
  commitId: string | null;
  onCancel: () => void;
  onContinue: (mode: ResetModeDto, forcePush: boolean) => void;
}

const MODES: {
  id: ResetModeDto;
  title: string;
  detail: string;
}[] = [
  {
    id: "soft",
    title: "Soft",
    detail: "Mantém alterações no staging.",
  },
  {
    id: "mixed",
    title: "Mixed",
    detail: "Mantém alterações na working tree (fora do stage).",
  },
  {
    id: "hard",
    title: "Hard",
    detail: "Descarta staging e working tree (irreversível).",
  },
];

export function ResetDialog({
  open: isOpen,
  shortId,
  requiresForcePush: requiresForcePushProp = false,
  loading,
  error,
  commitId,
  onCancel,
  onContinue,
}: ResetDialogProps) {
  const [mode, setMode] = useState<ResetModeDto>("mixed");
  const [forcePush, setForcePush] = useState(false);
  const [requiresForcePush, setRequiresForcePush] = useState(requiresForcePushProp);
  const panelRef = useRef<HTMLDivElement>(null);

  useDialogA11y(isOpen, onCancel, panelRef);

  useEffect(() => {
    if (!isOpen) return;
    setMode("mixed");
    setForcePush(requiresForcePushProp);
    setRequiresForcePush(requiresForcePushProp);
  }, [isOpen, requiresForcePushProp]);

  useEffect(() => {
    if (!isOpen || !commitId) return;
    let cancelled = false;
    void previewWriteOperation({
      kind: "reset",
      commitId,
      mode: "mixed",
      forcePush: false,
    })
      .then((preview) => {
        if (cancelled) return;
        const needs =
          preview.description.includes("push forçado") ||
          preview.commands.some((c) => c.includes("force-with-lease"));
        setRequiresForcePush(needs);
        if (needs) setForcePush(false);
      })
      .catch(() => {
        /* preview falhou — mantém heurística do pai */
      });
    return () => {
      cancelled = true;
    };
  }, [isOpen, commitId]);

  if (!isOpen) return null;

  const canContinue = !requiresForcePush || forcePush;

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
        aria-labelledby="reset-dialog-title"
        className="w-full max-w-md rounded-xl border border-border bg-surface p-4 shadow-lg"
      >
        <div className="mb-3 flex items-center gap-2">
          <RotateCcw size={18} className="text-accent" />
          <h2 id="reset-dialog-title" className="text-sm font-semibold text-text">
            Resetar para {shortId}
          </h2>
        </div>

        <p className="mb-3 text-xs text-muted">
          Move o HEAD para este commit e remove os commits mais recentes da branch
          local. Para desfazer sem reescrever histórico, prefira{" "}
          <span className="font-medium">Reverter commit</span>.
        </p>

        <fieldset className="mb-3 space-y-2">
          <legend className="mb-1 text-xs font-medium text-muted">Modo</legend>
          {MODES.map((m) => (
            <label
              key={m.id}
              className={`flex cursor-pointer items-start gap-2 rounded-md border px-3 py-2 text-xs ${
                mode === m.id
                  ? "border-accent/50 bg-accent/10"
                  : "border-border hover:bg-bg"
              }`}
            >
              <input
                type="radio"
                name="reset-mode"
                checked={mode === m.id}
                onChange={() => setMode(m.id)}
                className="mt-0.5"
              />
              <span>
                <span className="font-medium text-text">{m.title}</span>
                <span className="block text-muted">{m.detail}</span>
              </span>
            </label>
          ))}
        </fieldset>

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
            onClick={() => onContinue(mode, forcePush)}
            className="rounded-lg bg-accent px-3 py-1.5 text-xs font-medium text-white hover:opacity-90 disabled:opacity-50"
          >
            {loading ? "Abrindo preview…" : "Continuar"}
          </button>
        </div>
      </div>
    </div>
  );
}
