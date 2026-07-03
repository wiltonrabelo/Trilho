import { FolderOpen, History } from "lucide-react";
import { open } from "@tauri-apps/plugin-dialog";
import { runningInTauri } from "@/lib/api";

interface RepoPickerProps {
  recentRepos: string[];
  onOpen: (path: string) => void;
  loading?: boolean;
}

export function RepoPicker({ recentRepos, onOpen, loading }: RepoPickerProps) {
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
    <div className="flex h-full flex-col gap-3 p-3">
      <button
        type="button"
        onClick={pickFolder}
        disabled={loading}
        className="flex items-center justify-center gap-2 rounded-lg bg-accent px-3 py-2 text-sm font-medium text-white hover:opacity-90 disabled:opacity-50"
      >
        <FolderOpen size={16} />
        Abrir repositório
      </button>

      {recentRepos.length > 0 && (
        <div>
          <div className="mb-2 flex items-center gap-1.5 text-xs font-medium text-muted">
            <History size={14} />
            Recentes
          </div>
          <ul className="space-y-1">
            {recentRepos.map((path) => (
              <li key={path}>
                <button
                  type="button"
                  disabled={loading}
                  onClick={() => onOpen(path)}
                  className="w-full truncate rounded px-2 py-1.5 text-left text-xs hover:bg-surface"
                  title={path}
                >
                  {path.split(/[/\\]/).pop() ?? path}
                </button>
              </li>
            ))}
          </ul>
        </div>
      )}
    </div>
  );
}
