import type { FileChangeDto, FileChangeKind } from "@/types";

interface StatusPanelProps {
  staged: FileChangeDto[];
  unstaged: FileChangeDto[];
  untracked: FileChangeDto[];
  selectedPath: string | null;
  selectedStaged: boolean | null;
  onSelectFile: (path: string, staged: boolean) => void;
}

const KIND_BADGE: Record<
  FileChangeKind,
  { label: string; className: string }
> = {
  modified: {
    label: "M",
    className:
      "bg-amber-500/15 text-amber-700 dark:text-amber-400",
  },
  added: {
    label: "A",
    className: "bg-emerald-500/15 text-emerald-700 dark:text-emerald-400",
  },
  deleted: {
    label: "D",
    className: "bg-red-500/15 text-red-600 dark:text-red-400",
  },
  renamed: {
    label: "R",
    className: "bg-violet-500/15 text-violet-700 dark:text-violet-300",
  },
  untracked: {
    label: "U",
    className: "bg-muted/25 text-muted",
  },
};

function KindBadge({ kind }: { kind: FileChangeKind }) {
  const b = KIND_BADGE[kind];
  return (
    <span
      className={`inline-flex h-4 w-4 shrink-0 items-center justify-center rounded text-[10px] font-bold ${b.className}`}
    >
      {b.label}
    </span>
  );
}

function FileList({
  title,
  files,
  staged,
  selectedPath,
  selectedStaged,
  onSelect,
}: {
  title: string;
  files: FileChangeDto[];
  staged: boolean;
  selectedPath: string | null;
  selectedStaged: boolean | null;
  onSelect: (path: string, staged: boolean) => void;
}) {
  return (
    <section className="mb-4 border-b border-border/60 pb-3 last:mb-0 last:border-0">
      <div className="mb-1.5 flex items-center justify-between px-1">
        <span className="text-[10px] font-semibold uppercase tracking-wide text-muted">
          {title}
        </span>
        <span className="text-[10px] tabular-nums text-muted">{files.length}</span>
      </div>
      {files.length === 0 ? (
        <p className="px-2 py-1 text-xs text-muted/70">—</p>
      ) : (
        <ul className="space-y-0.5">
          {files.map((f) => {
            const isSelected =
              selectedPath === f.path && selectedStaged === staged;
            return (
              <li key={`${staged}-${f.kind}-${f.path}`}>
                <button
                  type="button"
                  onClick={() => onSelect(f.path, staged)}
                  className={`flex w-full items-start gap-2 rounded-md px-2 py-1 text-left focus:outline-none focus-visible:ring-2 focus-visible:ring-accent/30 ${
                    isSelected
                      ? "bg-surface ring-1 ring-border"
                      : "hover:bg-surface/60"
                  }`}
                  title={f.path}
                >
                  <KindBadge kind={f.kind} />
                  <span className="min-w-0 flex-1 break-all font-mono text-xs text-text">
                    {f.path}
                  </span>
                </button>
              </li>
            );
          })}
        </ul>
      )}
    </section>
  );
}

export function StatusPanel({
  staged,
  unstaged,
  untracked,
  selectedPath,
  selectedStaged,
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
              selectedStaged={selectedStaged}
              onSelect={onSelectFile}
            />
            <FileList
              title="Unstaged"
              files={unstaged}
              staged={false}
              selectedPath={selectedPath}
              selectedStaged={selectedStaged}
              onSelect={onSelectFile}
            />
            <FileList
              title="Untracked"
              files={untracked}
              staged={false}
              selectedPath={selectedPath}
              selectedStaged={selectedStaged}
              onSelect={onSelectFile}
            />
          </>
        )}
      </div>
    </div>
  );
}
