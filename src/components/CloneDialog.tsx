import { Download, FolderOpen } from "lucide-react";
import { useEffect, useRef, useState } from "react";

import { open } from "@tauri-apps/plugin-dialog";

import { useDialogA11y } from "@/hooks/useDialogA11y";
import {
  listCloneRemoteBranches,
  repoNameFromUrl,
  runningInTauri,
} from "@/lib/api";

import type { CloneFormValues } from "@/types";

interface CloneDialogProps {
  open: boolean;
  loading?: boolean;
  error?: string | null;
  onCancel: () => void;
  onContinue: (values: CloneFormValues) => void;
}

const HOST_TEMPLATES = {
  github: "https://github.com/usuario/repositorio.git",
  gitlab: "https://gitlab.com/usuario/repositorio.git",
} as const;

function looksLikeRemoteUrl(url: string): boolean {
  const u = url.trim();
  return u.startsWith("https://") || u.startsWith("git@");
}

/** Modelo dos atalhos — ainda não é URL real; não dispara ls-remote. */
function isHostTemplateUrl(url: string): boolean {
  const u = url.trim().toLowerCase();
  return (
    u === HOST_TEMPLATES.github.toLowerCase() ||
    u === HOST_TEMPLATES.gitlab.toLowerCase() ||
    u.includes("/usuario/repositorio")
  );
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
  const [branch, setBranch] = useState<string | null>(null);
  const [shallow, setShallow] = useState(false);
  const [depth, setDepth] = useState("1");
  const [branches, setBranches] = useState<string[]>([]);
  const [branchesLoading, setBranchesLoading] = useState(false);
  const [branchesError, setBranchesError] = useState<string | null>(null);
  const folderTouched = useRef(false);
  const panelRef = useRef<HTMLDivElement>(null);

  useDialogA11y(isOpen, onCancel, panelRef);

  useEffect(() => {
    if (!isOpen) {
      setUrl("");
      setParentDir("");
      setFolderName("");
      setBranch(null);
      setShallow(false);
      setDepth("1");
      setBranches([]);
      setBranchesError(null);
      folderTouched.current = false;
    }
  }, [isOpen]);

  useEffect(() => {
    if (!folderTouched.current && url.trim()) {
      setFolderName(repoNameFromUrl(url));
    }
  }, [url]);

  useEffect(() => {
    if (
      !isOpen ||
      !runningInTauri() ||
      !looksLikeRemoteUrl(url) ||
      isHostTemplateUrl(url)
    ) {
      setBranches([]);
      setBranchesError(null);
      return;
    }

    const timer = window.setTimeout(() => {
      setBranchesLoading(true);
      setBranchesError(null);
      void listCloneRemoteBranches(url.trim())
        .then((list) => {
          setBranches(list);
          setBranch((current) =>
            current && list.includes(current) ? current : null,
          );
        })
        .catch((e) => {
          setBranches([]);
          setBranchesError(e instanceof Error ? e.message : String(e));
        })
        .finally(() => setBranchesLoading(false));
    }, 600);

    return () => window.clearTimeout(timer);
  }, [isOpen, url]);

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

  function applyHostTemplate(template: string) {
    setUrl(template);
    folderTouched.current = false;
  }

  function submit() {
    const u = url.trim();
    const p = parentDir.trim();
    const f = folderName.trim();
    if (!u || !p || !f) return;

    let depthValue: number | null = null;
    if (shallow) {
      const parsed = Number.parseInt(depth, 10);
      if (!Number.isFinite(parsed) || parsed < 1) return;
      depthValue = parsed;
    }

    onContinue({
      url: u,
      parentDir: p,
      folderName: f,
      branch,
      depth: depthValue,
    });
  }

  const depthInvalid =
    shallow &&
    (!Number.isFinite(Number.parseInt(depth, 10)) ||
      Number.parseInt(depth, 10) < 1);

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

        <div className="max-h-[70vh] space-y-3 overflow-y-auto px-4 py-3 text-sm">
          <p className="text-xs text-muted">
            Baixe um repositório remoto e abra-o no Trilho. Na primeira vez pode
            abrir o login do GitHub (GCM).
          </p>

          <div>
            <div className="mb-1.5 flex flex-wrap gap-2">
              <button
                type="button"
                onClick={() => applyHostTemplate(HOST_TEMPLATES.github)}
                disabled={loading}
                title="Preenche o modelo de URL do GitHub"
                className="rounded border border-border px-2 py-1 text-[11px] text-muted hover:bg-bg hover:text-text disabled:opacity-50"
              >
                GitHub
              </button>
              <button
                type="button"
                onClick={() => applyHostTemplate(HOST_TEMPLATES.gitlab)}
                disabled={loading}
                title="Preenche o modelo de URL do GitLab"
                className="rounded border border-border px-2 py-1 text-[11px] text-muted hover:bg-bg hover:text-text disabled:opacity-50"
              >
                GitLab
              </button>
            </div>
            <p className="text-[11px] text-muted">
              Atalhos só preenchem o modelo — edite usuário e repositório na URL.
            </p>
          </div>

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
            Branch inicial
            <select
              value={branch ?? ""}
              onChange={(e) =>
                setBranch(e.target.value ? e.target.value : null)
              }
              disabled={loading || branchesLoading || !looksLikeRemoteUrl(url)}
              className="mt-1 w-full rounded border border-border bg-bg px-2 py-1.5 text-xs text-text focus:outline-none focus:ring-1 focus:ring-accent/40 disabled:opacity-50"
            >
              <option value="">
                {branchesLoading
                  ? "Carregando branches…"
                  : "Padrão do remoto"}
              </option>
              {branches.map((b) => (
                <option key={b} value={b}>
                  {b}
                </option>
              ))}
            </select>
            {branchesError && !isHostTemplateUrl(url) && (
              <span className="mt-1 block text-[11px] text-amber-600">
                Não foi possível listar branches — será usada a branch padrão.
              </span>
            )}
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

          <details className="rounded border border-border px-2 py-2 text-xs">
            <summary className="cursor-pointer font-medium text-muted">
              Avançado
            </summary>
            <div className="mt-2 space-y-2">
              <label className="flex items-center gap-2 text-muted">
                <input
                  type="checkbox"
                  checked={shallow}
                  onChange={(e) => setShallow(e.target.checked)}
                  disabled={loading}
                  className="rounded border-border"
                />
                Clone raso (shallow)
              </label>
              {shallow && (
                <label className="block text-muted">
                  Profundidade
                  <input
                    type="number"
                    min={1}
                    value={depth}
                    onChange={(e) => setDepth(e.target.value)}
                    disabled={loading}
                    className="mt-1 w-full rounded border border-border bg-bg px-2 py-1.5 text-xs text-text focus:outline-none focus:ring-1 focus:ring-accent/40 disabled:opacity-50"
                  />
                </label>
              )}
            </div>
          </details>

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
            disabled={
              loading ||
              !url.trim() ||
              !parentDir.trim() ||
              !folderName.trim() ||
              depthInvalid
            }
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
