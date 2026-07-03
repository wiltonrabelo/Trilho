import type { FileChangeDto } from "@/types";

interface StatusPanelProps {
  staged: FileChangeDto[];
  unstaged: FileChangeDto[];
  untracked: FileChangeDto[];
  selectedPath: string | null;
  onSelectFile: (path: string, staged: boolean) => void;
}

function FileList({
  title,
  files,
  staged,
  selectedPath,
  onSelect,
}: {
  title: string;
  files: FileChangeDto[];
  staged: boolean;
  selectedPath: string | null;
  onSelect: (path: string, staged: boolean) => void;
}) {
  if (files.length === 0) return null;
  return (
    <div className="mb-3">
      <div className="mb-1 text-[10px] font-semibold uppercase tracking-wide text-muted">
        {title}
      </div>
      <ul className="space-y-0.5">
        {files.map((f) => (
          <li key={`${staged}-${f.path}`}>
            <button
              type="button"
              onClick={() => onSelect(f.path, staged)}
              className={`w-full rounded px-2 py-1 text-left font-mono text-xs break-all ${
                selectedPath === f.path
                  ? "bg-accent/15 text-accent"
                  : "hover:bg-surface"
              }`}
              title={f.path}
            >
              {f.path}
            </button>
          </li>
        ))}
      </ul>
    </div>
  );
}

export function StatusPanel({
  staged,
  unstaged,
  untracked,
  selectedPath,
  onSelectFile,
}: StatusPanelProps) {
  const total = staged.length + unstaged.length + untracked.length;
  return (
    <div className="flex h-full flex-col">
      <div className="border-b border-border px-3 py-2 text-xs font-medium text-muted">
        Alterações {total > 0 ? `(${total})` : ""}
      </div>
      <div className="flex-1 overflow-auto p-2">
        {total === 0 ? (
          <p className="px-2 py-4 text-center text-xs text-muted">
            Working tree limpa
          </p>
        ) : (
          <>
            <FileList
              title="Staged"
              files={staged}
              staged
              selectedPath={selectedPath}
              onSelect={onSelectFile}
            />
            <FileList
              title="Unstaged"
              files={unstaged}
              staged={false}
              selectedPath={selectedPath}
              onSelect={onSelectFile}
            />
            <FileList
              title="Untracked"
              files={untracked}
              staged={false}
              selectedPath={selectedPath}
              onSelect={onSelectFile}
            />
          </>
        )}
      </div>
    </div>
  );
}
