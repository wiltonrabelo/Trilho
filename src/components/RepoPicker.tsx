import { Download, FolderOpen, History, X } from "lucide-react";

import { open } from "@tauri-apps/plugin-dialog";

import { runningInTauri } from "@/lib/api";

interface RepoPickerProps {
  recentRepos: string[];
  onOpen: (path: string) => void;
  onRemoveRecent?: (path: string) => void;
  onClone?: () => void;
  loading?: boolean;
  currentPath?: string | null;
}

function pathsMatch(a: string, b: string) {
  return a.replace(/\\/g, "/").toLowerCase() === b.replace(/\\/g, "/").toLowerCase();
}

export function RepoPicker({
  recentRepos,
  onOpen,
  onRemoveRecent,
  onClone,
  loading,
  currentPath,
}: RepoPickerProps) {
  async function pickFolder() {
    if (!runningInTauri()) {
      onOpen("C:\\Projetos\\Trilho");
      return;
    }
    const selected = await open({
      directory: true,
      multiple: false,
      title: "Abrir repositório Git",
    });
    if (typeof selected === "string") {
      onOpen(selected);
    }
  }

  return (
    <div className="flex shrink-0 flex-col gap-3 p-3">
      <button
        type="button"
        onClick={pickFolder}
        disabled={loading}
        aria-label="Abrir pasta de repositório Git"
        className="flex items-center justify-center gap-2 rounded-lg bg-accent px-3 py-2 text-sm font-medium text-white hover:opacity-90 disabled:opacity-50"
      >
        <FolderOpen size={16} />
        Abrir repositório
      </button>

      {onClone && (
        <button
          type="button"
          onClick={onClone}
          disabled={loading}
          aria-label="Clonar repositório remoto"
          className="flex items-center justify-center gap-2 rounded-lg border border-border px-3 py-2 text-sm font-medium text-text hover:bg-surface disabled:opacity-50"
        >
          <Download size={16} />
          Clonar repositório
        </button>
      )}

      {recentRepos.length > 0 && (
        <div>
          <div className="mb-2 flex items-center gap-1.5 text-xs font-medium text-muted">
            <History size={14} />
            Recentes
          </div>
          <ul className="space-y-1">
            {recentRepos.map((path) => {
              const name = path.split(/[/\\]/).pop() ?? path;
              const active = !!currentPath && pathsMatch(path, currentPath);
              return (
                <li key={path} className="group flex items-center gap-0.5">
                  <button
                    type="button"
                    disabled={loading}
                    onClick={() => onOpen(path)}
                    aria-label={`Abrir repositório ${path}`}
                    className={`min-w-0 flex-1 truncate rounded px-2 py-1.5 text-left text-xs ${
                      active
                        ? "bg-accent/15 font-medium text-accent"
                        : "hover:bg-surface"
                    }`}
                    title={path}
                  >
                    {name}
                    {active ? " ✓" : ""}
                  </button>
                  {onRemoveRecent && (
                    <button
                      type="button"
                      disabled={loading}
                      onClick={() => onRemoveRecent(path)}
                      aria-label={`Remover ${name} dos recentes e fechar se estiver aberto`}
                      title="Remover dos recentes"
                      className="shrink-0 rounded p-1 text-muted hover:bg-surface hover:text-red-500 disabled:opacity-50"
                    >
                      <X size={12} aria-hidden />
                    </button>
                  )}
                </li>
              );
            })}
          </ul>
        </div>
      )}
    </div>
  );
}
