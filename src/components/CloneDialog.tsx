import { Download, FolderOpen } from "lucide-react";
import { useEffect, useRef, useState } from "react";

import { open } from "@tauri-apps/plugin-dialog";

import { useDialogA11y } from "@/hooks/useDialogA11y";
import { repoNameFromUrl, runningInTauri } from "@/lib/api";

interface CloneDialogProps {
  open: boolean;
  loading?: boolean;
  error?: string | null;
  onCancel: () => void;
  onContinue: (url: string, parentDir: string, folderName: string) => void;
}

export function CloneDialog({
  open: isOpen,
  loading,
  error,
  onCancel,
  onContinue,
}: CloneDialogProps) {
  const [url, setUrl] = useState("");
  const [parentDir, setParentDir] = useState("");
  const [folderName, setFolderName] = useState("");
  const folderTouched = useRef(false);
  const panelRef = useRef<HTMLDivElement>(null);

  useDialogA11y(isOpen, onCancel, panelRef);

  useEffect(() => {
    if (!isOpen) {
      setUrl("");
      setParentDir("");
      setFolderName("");
      folderTouched.current = false;
    }
  }, [isOpen]);

  useEffect(() => {
    if (!folderTouched.current && url.trim()) {
      setFolderName(repoNameFromUrl(url));
    }
  }, [url]);

  if (!isOpen) return null;

  async function pickParent() {
    if (!runningInTauri()) {
      setParentDir("C:\\Projetos");
      return;
    }
    const selected = await open({
      directory: true,
      multiple: false,
      title: "Pasta de destino do clone",
    });
    if (typeof selected === "string") {
      setParentDir(selected);
    }
  }

  function submit() {
    const u = url.trim();
    const p = parentDir.trim();
    const f = folderName.trim();
    if (!u || !p || !f) return;
    onContinue(u, p, f);
  }

  return (
    <div
      className="fixed inset-0 z-50 flex items-center justify-center bg-black/50 p-4"
      role="dialog"
      aria-modal="true"
      aria-labelledby="clone-dialog-title"
    >
      <div
        ref={panelRef}
        className="w-full max-w-md rounded-xl border border-border bg-surface shadow-lg"
      >
        <div className="border-b border-border px-4 py-3">
          <h2 id="clone-dialog-title" className="text-sm font-semibold">
            Clonar repositório
          </h2>
        </div>

        <div className="space-y-3 px-4 py-3 text-sm">
          <p className="text-xs text-muted">
            Baixe um repositório remoto e abra-o no Trilho. Na primeira vez pode
            abrir o login do GitHub (GCM).
          </p>
          <label className="block text-xs text-muted">
            URL do repositório
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
          <label className="block text-xs text-muted">
            Pasta de destino
            <div className="mt-1 flex gap-2">
              <input
                type="text"
                readOnly
                value={parentDir}
                placeholder="Escolha a pasta pai…"
                className="min-w-0 flex-1 truncate rounded border border-border bg-bg px-2 py-1.5 text-xs text-text placeholder:text-muted"
              />
              <button
                type="button"
                onClick={() => void pickParent()}
                disabled={loading}
                className="flex shrink-0 items-center gap-1 rounded border border-border px-2 py-1.5 text-xs text-muted hover:bg-bg hover:text-text disabled:opacity-50"
              >
                <FolderOpen size={14} />
                Escolher
              </button>
            </div>
          </label>
          <label className="block text-xs text-muted">
            Nome da pasta
            <input
              type="text"
              value={folderName}
              onChange={(e) => {
                folderTouched.current = true;
                setFolderName(e.target.value);
              }}
              disabled={loading}
              className="mt-1 w-full rounded border border-border bg-bg px-2 py-1.5 text-xs text-text placeholder:text-muted focus:outline-none focus:ring-1 focus:ring-accent/40 disabled:opacity-50"
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
            disabled={loading || !url.trim() || !parentDir.trim() || !folderName.trim()}
            className="flex items-center gap-1.5 rounded-lg bg-accent px-3 py-1.5 text-xs font-medium text-white hover:opacity-90 disabled:opacity-50"
          >
            <Download size={14} />
            {loading ? "Preparando…" : "Continuar"}
          </button>
        </div>
      </div>
    </div>
  );
}
