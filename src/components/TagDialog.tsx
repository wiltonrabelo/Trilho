import { Tag } from "lucide-react";
import { useEffect, useRef, useState } from "react";

import { useDialogA11y } from "@/hooks/useDialogA11y";

interface TagDialogProps {
  open: boolean;
  commitShortId: string;
  hasRemote?: boolean;
  loading?: boolean;
  error?: string | null;
  onCancel: () => void;
  onContinue: (values: {
    name: string;
    annotated: boolean;
    message: string;
    pushToRemote: boolean;
  }) => void;
}

export function TagDialog({
  open: isOpen,
  commitShortId,
  hasRemote = false,
  loading,
  error,
  onCancel,
  onContinue,
}: TagDialogProps) {
  const [name, setName] = useState("");
  const [annotated, setAnnotated] = useState(true);
  const [message, setMessage] = useState("");
  const [pushToRemote, setPushToRemote] = useState(false);
  const panelRef = useRef<HTMLDivElement>(null);

  useDialogA11y(isOpen, onCancel, panelRef);

  useEffect(() => {
    if (!isOpen) {
      setName("");
      setAnnotated(true);
      setMessage("");
      setPushToRemote(false);
    }
  }, [isOpen]);

  if (!isOpen) return null;

  const trimmedName = name.trim();
  const trimmedMessage = message.trim();
  const canContinue =
    trimmedName.length > 0 && (!annotated || trimmedMessage.length > 0);

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
        aria-labelledby="tag-dialog-title"
        className="w-full max-w-md rounded-xl border border-border bg-surface p-4 shadow-lg"
      >
        <div className="mb-3 flex items-center gap-2">
          <Tag size={18} className="text-accent" />
          <h2 id="tag-dialog-title" className="text-sm font-semibold text-text">
            Criar tag
          </h2>
        </div>

        <p className="mb-3 text-xs text-muted">
          Commit <span className="font-mono">{commitShortId}</span>
        </p>

        <label className="mb-3 block text-xs text-muted">
          Nome da tag
          <input
            type="text"
            value={name}
            onChange={(e) => setName(e.target.value)}
            placeholder="Ex.: v1.0.0"
            disabled={loading}
            className="mt-1 w-full rounded-md border border-border bg-bg px-2 py-1.5 text-sm text-text placeholder:text-muted focus:border-accent focus:outline-none disabled:opacity-50"
          />
        </label>

        <fieldset className="mb-3">
          <legend className="mb-1 text-xs text-muted">Tipo</legend>
          <label className="mr-4 inline-flex cursor-pointer items-center gap-1.5 text-xs text-text">
            <input
              type="radio"
              name="tag-type"
              checked={annotated}
              onChange={() => setAnnotated(true)}
              disabled={loading}
            />
            Anotada
          </label>
          <label className="inline-flex cursor-pointer items-center gap-1.5 text-xs text-text">
            <input
              type="radio"
              name="tag-type"
              checked={!annotated}
              onChange={() => setAnnotated(false)}
              disabled={loading}
            />
            Leve
          </label>
        </fieldset>

        {annotated ? (
          <label className="mb-3 block text-xs text-muted">
            Mensagem
            <textarea
              value={message}
              onChange={(e) => setMessage(e.target.value)}
              placeholder="Ex.: Release 1.0.0"
              rows={3}
              disabled={loading}
              className="mt-1 w-full resize-none rounded-md border border-border bg-bg px-2 py-1.5 text-sm text-text placeholder:text-muted focus:border-accent focus:outline-none disabled:opacity-50"
            />
          </label>
        ) : null}

        <label className="mb-4 flex cursor-pointer items-start gap-2 text-xs text-text">
          <input
            type="checkbox"
            checked={pushToRemote}
            onChange={(e) => setPushToRemote(e.target.checked)}
            disabled={loading || !hasRemote}
            className="mt-0.5"
          />
          <span>
            Enviar ao remoto
            {!hasRemote ? (
              <span className="block text-[10px] text-muted">
                Nenhum remoto configurado neste repositório.
              </span>
            ) : (
              <span className="block text-[10px] text-muted">
                Equivale a <code className="font-mono">git push origin &lt;tag&gt;</code>
              </span>
            )}
          </span>
        </label>

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
            onClick={() =>
              onContinue({
                name: trimmedName,
                annotated,
                message: trimmedMessage,
                pushToRemote,
              })
            }
            disabled={loading || !canContinue}
            className="rounded-lg bg-accent px-3 py-1.5 text-xs font-medium text-white hover:opacity-90 disabled:opacity-50"
          >
            Continuar…
          </button>
        </div>
      </div>
    </div>
  );
}
