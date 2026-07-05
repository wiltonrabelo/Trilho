import { Upload, X } from "lucide-react";
import { useState } from "react";

interface PublishDialogProps {
  open: boolean;
  branch?: string | null;
  loading?: boolean;
  error?: string | null;
  onCancel: () => void;
  onContinue: (remoteUrl: string) => void;
}

export function PublishDialog({
  open,
  branch,
  loading,
  error,
  onCancel,
  onContinue,
}: PublishDialogProps) {
  const [url, setUrl] = useState("");

  if (!open) return null;

  const b = branch ?? "master";

  function submit() {
    const trimmed = url.trim();
    if (!trimmed) return;
    onContinue(trimmed);
  }

  return (
    <div
      className="fixed inset-0 z-50 flex items-center justify-center bg-black/50 p-4"
      role="dialog"
      aria-modal="true"
      aria-labelledby="publish-dialog-title"
    >
      <div className="w-full max-w-md rounded-xl border border-border bg-surface shadow-lg">
        <div className="flex items-center justify-between border-b border-border px-4 py-3">
          <h2 id="publish-dialog-title" className="text-sm font-semibold">
            Publicar branch
          </h2>
          <button
            type="button"
            onClick={onCancel}
            disabled={loading}
            className="rounded p-1 text-muted hover:bg-bg hover:text-text disabled:opacity-50"
            aria-label="Fechar"
          >
            <X size={16} />
          </button>
        </div>

        <div className="space-y-3 px-4 py-3 text-sm">
          <p className="text-text">
            O Trilho conecta este repositório ao remoto e publica a branch{" "}
            <strong>{b}</strong> pela primeira vez.
          </p>
          <p className="text-xs text-muted">
            Crie um repositório <strong>vazio</strong> no GitHub (sem README) e
            cole a URL abaixo. Na confirmação, o Trilho executa a conexão e o
            envio — na primeira vez pode abrir o login do GitHub.
          </p>
          <label className="block text-xs text-muted">
            URL do repositório remoto
            <input
              type="url"
              value={url}
              onChange={(e) => setUrl(e.target.value)}
              placeholder="https://github.com/usuario/repositorio.git"
              disabled={loading}
              className="mt-1 w-full rounded border border-border bg-bg px-2 py-1.5 text-xs text-text placeholder:text-muted focus:outline-none focus:ring-1 focus:ring-accent/40 disabled:opacity-50"
              autoFocus
            />
          </label>
          {error && (
            <p className="rounded-md border border-red-500/40 bg-red-500/10 px-2 py-1.5 text-xs text-red-500">
              {error}
            </p>
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
            onClick={submit}
            disabled={loading || !url.trim()}
            className="flex items-center gap-1.5 rounded-lg bg-accent px-3 py-1.5 text-xs font-medium text-white hover:opacity-90 disabled:opacity-50"
          >
            <Upload size={14} />
            {loading ? "Preparando…" : "Continuar"}
          </button>
        </div>
      </div>
    </div>
  );
}
