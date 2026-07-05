import { Plus, Undo2 } from "lucide-react";
import {
  countChecked,
  fileCheckKey,
  pathsFromChecked,
  type FileCheckSection,
} from "@/lib/fileCheck";
import type { CommitDto, FileChangeDto, FileChangeKind } from "@/types";

interface StatusPanelProps {
  staged: FileChangeDto[];
  unstaged: FileChangeDto[];
  untracked: FileChangeDto[];
  selectedPath: string | null;
  selectedStaged: boolean | null;
  checkedPaths: ReadonlySet<string>;
  onSelectFile: (
    path: string,
    staged: boolean,
    meta?: { ctrlKey?: boolean; shiftKey?: boolean },
  ) => void;
  onToggleCheck: (path: string, section: FileCheckSection) => void;
  commit: CommitDto | null;
  commitFiles: FileChangeDto[];
  selectedCommitFile: string | null;
  onSelectCommitFile: (path: string) => void;
  onStage?: (path: string) => void;
  onStageMany?: (paths: string[]) => void;
  onStageAll?: () => void;
  onUnstage?: (path: string) => void;
  onUnstageMany?: (paths: string[]) => void;
  onUnstageAll?: () => void;
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
  checkSection,
  selectedPath,
  selectedStaged,
  checkedPaths,
  onSelect,
  onToggleCheck,
  onStage,
  onUnstage,
}: {
  title: string;
  files: FileChangeDto[];
  staged: boolean;
  checkSection: FileCheckSection;
  selectedPath: string | null;
  selectedStaged: boolean | null;
  checkedPaths: ReadonlySet<string>;
  onSelect: (
    path: string,
    staged: boolean,
    meta?: { ctrlKey?: boolean; shiftKey?: boolean },
  ) => void;
  onToggleCheck: (path: string, section: FileCheckSection) => void;
  onStage?: (path: string) => void;
  onUnstage?: (path: string) => void;
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
            const isChecked = checkedPaths.has(
              fileCheckKey(checkSection, f.path),
            );
            const showStage = !staged && onStage;
            const showUnstage = staged && onUnstage;
            return (
              <li
                key={`${staged}-${f.kind}-${f.path}`}
                className="group flex items-center gap-0.5"
              >
                <input
                  type="checkbox"
                  checked={isChecked}
                  onChange={() => onToggleCheck(f.path, checkSection)}
                  title="Selecionar para stage/unstage em lote"
                  className="ml-1 shrink-0 rounded border-border"
                />
                <button
                  type="button"
                  onClick={(e) =>
                    onSelect(f.path, staged, {
                      ctrlKey: e.ctrlKey || e.metaKey,
                      shiftKey: e.shiftKey,
                    })
                  }
                  className={`flex min-w-0 flex-1 items-start gap-2 rounded-md px-2 py-1 text-left focus:outline-none focus-visible:ring-2 focus-visible:ring-accent/30 ${
                    isSelected || isChecked
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
                {showStage && (
                  <button
                    type="button"
                    onClick={() => onStage(f.path)}
                    title="Stage"
                    className="shrink-0 rounded p-1 text-muted opacity-0 hover:bg-surface hover:text-accent group-hover:opacity-100 focus:opacity-100"
                  >
                    <Plus size={14} />
                  </button>
                )}
                {showUnstage && (
                  <button
                    type="button"
                    onClick={() => onUnstage(f.path)}
                    title="Unstage"
                    className="shrink-0 rounded p-1 text-muted opacity-0 hover:bg-surface hover:text-accent group-hover:opacity-100 focus:opacity-100"
                  >
                    <Undo2 size={14} />
                  </button>
                )}
              </li>
            );
          })}
        </ul>
      )}
    </section>
  );
}

function CommitFileList({
  files,
  selectedPath,
  onSelect,
}: {
  files: FileChangeDto[];
  selectedPath: string | null;
  onSelect: (path: string) => void;
}) {
  if (files.length === 0) {
    return (
      <p className="px-2 py-4 text-center text-xs text-muted">
        Nenhum arquivo alterado neste commit
      </p>
    );
  }
  return (
    <ul className="space-y-0.5">
      {files.map((f) => {
        const isSelected = selectedPath === f.path;
        return (
          <li key={`${f.kind}-${f.path}`}>
            <button
              type="button"
              onClick={() => onSelect(f.path)}
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
  );
}

export function StatusPanel({
  staged,
  unstaged,
  untracked,
  selectedPath,
  selectedStaged,
  checkedPaths,
  onSelectFile,
  onToggleCheck,
  commit,
  commitFiles,
  selectedCommitFile,
  onSelectCommitFile,
  onStage,
  onStageMany,
  onStageAll,
  onUnstage,
  onUnstageMany,
  onUnstageAll,
}: StatusPanelProps) {
  if (commit) {
    return (
      <div className="flex h-full flex-col">
        <div className="shrink-0 border-b border-border px-3 py-2 text-xs font-medium text-muted">
          Arquivos do commit{" "}
          <span className="font-mono text-[10px]">{commit.shortId}</span>
          {commitFiles.length > 0 ? ` (${commitFiles.length})` : ""}
        </div>
        <div className="min-h-0 flex-1 overflow-auto p-2">
          <CommitFileList
            files={commitFiles}
            selectedPath={selectedCommitFile}
            onSelect={onSelectCommitFile}
          />
        </div>
      </div>
    );
  }

  const total = staged.length + unstaged.length + untracked.length;
  const stageablePaths = pathsFromChecked(checkedPaths, "working").filter(
    (p) =>
      unstaged.some((f) => f.path === p) || untracked.some((f) => f.path === p),
  );
  const unstagedPaths = pathsFromChecked(checkedPaths, "staged").filter((p) =>
    staged.some((f) => f.path === p),
  );
  const checkedWorkingCount = countChecked(checkedPaths, "working");
  const checkedStagedCount = countChecked(checkedPaths, "staged");

  const canStageAll =
    (unstaged.length > 0 || untracked.length > 0) && Boolean(onStageAll);
  const canStageMany = stageablePaths.length > 0 && Boolean(onStageMany);
  const canUnstageMany = unstagedPaths.length > 0 && Boolean(onUnstageMany);

  // `selectedPath != null` (e não Boolean(...)) para o TypeScript estreitar
  // o tipo de string | null nos usos abaixo.
  const canStageSelected =
    selectedPath != null &&
    Boolean(onStage) &&
    (unstaged.some((f) => f.path === selectedPath) ||
      untracked.some((f) => f.path === selectedPath)) &&
    !stageablePaths.includes(selectedPath) &&
    !checkedPaths.has(fileCheckKey("working", selectedPath));

  const canUnstageSelected =
    selectedPath != null &&
    Boolean(onUnstage) &&
    staged.some((f) => f.path === selectedPath) &&
    !unstagedPaths.includes(selectedPath) &&
    !checkedPaths.has(fileCheckKey("staged", selectedPath));

  return (
    <div className="flex h-full flex-col">
      <div className="shrink-0 border-b border-border px-3 py-2 text-xs font-medium text-muted">
        <div className="flex items-center justify-between gap-2">
          <span>Alterações {total > 0 ? `(${total})` : ""}</span>
          <div className="flex shrink-0 flex-wrap justify-end gap-2">
            {canStageMany && (
              <button
                type="button"
                onClick={() => onStageMany!(stageablePaths)}
                className="text-[10px] text-accent hover:underline"
              >
                Stage selecionados ({stageablePaths.length})
              </button>
            )}
            {canUnstageMany && (
              <button
                type="button"
                onClick={() => onUnstageMany!(unstagedPaths)}
                className="text-[10px] text-accent hover:underline"
              >
                Unstage selecionados ({unstagedPaths.length})
              </button>
            )}
            {canStageAll && (
              <button
                type="button"
                onClick={onStageAll}
                className="text-[10px] text-accent hover:underline"
              >
                Stage tudo
              </button>
            )}
            {staged.length > 0 && onUnstageAll && (
              <button
                type="button"
                onClick={onUnstageAll}
                className="text-[10px] text-accent hover:underline"
              >
                Unstage tudo
              </button>
            )}
          </div>
        </div>
        {(canStageSelected || canUnstageSelected) && (
          <div className="mt-2 flex flex-wrap items-center gap-2">
            <span className="min-w-0 truncate font-mono text-[10px] text-text">
              {selectedPath}
            </span>
            {canStageSelected && (
              <button
                type="button"
                onClick={() => onStage!(selectedPath!)}
                className="flex shrink-0 items-center gap-1 rounded border border-accent/40 bg-accent/10 px-2 py-0.5 text-[10px] text-accent hover:bg-accent/20"
              >
                <Plus size={12} />
                Stage
              </button>
            )}
            {canUnstageSelected && (
              <button
                type="button"
                onClick={() => onUnstage!(selectedPath!)}
                className="flex shrink-0 items-center gap-1 rounded border border-border px-2 py-0.5 text-[10px] text-muted hover:bg-surface hover:text-text"
              >
                <Undo2 size={12} />
                Unstage
              </button>
            )}
          </div>
        )}
        {(checkedWorkingCount > 0 || checkedStagedCount > 0) && (
          <p className="mt-1.5 text-[10px] text-muted">
            {checkedWorkingCount > 0 &&
              `${checkedWorkingCount} unstaged/untracked`}
            {checkedWorkingCount > 0 && checkedStagedCount > 0 && " · "}
            {checkedStagedCount > 0 && `${checkedStagedCount} staged`}
            {" · Ctrl+clique alterna"}
          </p>
        )}
      </div>
      <div className="min-h-0 flex-1 overflow-auto p-2">
        {total === 0 ? (
          <p className="px-2 py-1 text-center text-xs text-muted/70">
            Working tree limpa
          </p>
        ) : (
          <>
            <FileList
              title="Staged"
              files={staged}
              staged
              checkSection="staged"
              selectedPath={selectedPath}
              selectedStaged={selectedStaged}
              checkedPaths={checkedPaths}
              onSelect={onSelectFile}
              onToggleCheck={onToggleCheck}
              onUnstage={onUnstage}
            />
            <FileList
              title="Unstaged"
              files={unstaged}
              staged={false}
              checkSection="working"
              selectedPath={selectedPath}
              selectedStaged={selectedStaged}
              checkedPaths={checkedPaths}
              onSelect={onSelectFile}
              onToggleCheck={onToggleCheck}
              onStage={onStage}
            />
            <FileList
              title="Untracked"
              files={untracked}
              staged={false}
              checkSection="working"
              selectedPath={selectedPath}
              selectedStaged={selectedStaged}
              checkedPaths={checkedPaths}
              onSelect={onSelectFile}
              onToggleCheck={onToggleCheck}
              onStage={onStage}
            />
          </>
        )}
      </div>
    </div>
  );
}
