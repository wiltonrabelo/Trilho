import { Plus, Trash2, Undo2 } from "lucide-react";
import {
  countChecked,
  fileCheckKey,
  pathsFromChecked,
  type FileCheckSection,
} from "@/lib/fileCheck";
import type { CommitDto, FileChangeDto, FileChangeKind, OperationInProgressDto } from "@/types";

interface StatusPanelProps {
  staged: FileChangeDto[];
  unstaged: FileChangeDto[];
  untracked: FileChangeDto[];
  operationInProgress?: OperationInProgressDto | null;
  onAbortOperation?: (kind: OperationInProgressDto["kind"]) => void;
  onContinueOperation?: (kind: OperationInProgressDto["kind"]) => void;
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
  onStash?: () => void;
  onDiscard?: (path: string) => void;
  onDiscardMany?: (paths: string[]) => void;
  onDiscardAll?: () => void;
  onRemoveUntracked?: (path: string) => void;
  onRemoveUntrackedMany?: (paths: string[]) => void;
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
  conflicted: {
    label: "!",
    className: "bg-orange-500/20 text-orange-700 dark:text-orange-300",
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
  onDiscard,
  onRemoveUntracked,
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
  onDiscard?: (path: string) => void;
  onRemoveUntracked?: (path: string) => void;
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
            const showStage = !staged && onStage && f.kind !== "conflicted";
            const showUnstage = staged && onUnstage && f.kind !== "conflicted";
            const showDiscard = !staged && onDiscard && f.kind !== "conflicted";
            const showRemove = !staged && onRemoveUntracked;
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
                {showDiscard && (
                  <button
                    type="button"
                    onClick={() => onDiscard(f.path)}
                    title="Descartar alterações"
                    className="shrink-0 rounded p-1 text-muted opacity-0 hover:bg-surface hover:text-red-600 group-hover:opacity-100 focus:opacity-100 dark:hover:text-red-400"
                  >
                    <Trash2 size={14} />
                  </button>
                )}
                {showRemove && (
                  <button
                    type="button"
                    onClick={() => onRemoveUntracked(f.path)}
                    title="Remover arquivo não rastreado"
                    className="shrink-0 rounded p-1 text-muted opacity-0 hover:bg-surface hover:text-red-600 group-hover:opacity-100 focus:opacity-100 dark:hover:text-red-400"
                  >
                    <Trash2 size={14} />
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
  operationInProgress,
  onAbortOperation,
  onContinueOperation,
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
  onStash,
  onDiscard,
  onDiscardMany,
  onDiscardAll,
  onRemoveUntracked,
  onRemoveUntrackedMany,
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
  const discardablePaths = pathsFromChecked(checkedPaths, "working").filter((p) =>
    unstaged.some((f) => f.path === p && f.kind !== "conflicted"),
  );
  const untrackedPaths = pathsFromChecked(checkedPaths, "working").filter((p) =>
    untracked.some((f) => f.path === p),
  );
  const checkedWorkingCount = countChecked(checkedPaths, "working");
  const checkedStagedCount = countChecked(checkedPaths, "staged");

  const hasDiscardableUnstaged = unstaged.some((f) => f.kind !== "conflicted");
  const canStageAll =
    (unstaged.length > 0 || untracked.length > 0) && Boolean(onStageAll);
  const canStageMany = stageablePaths.length > 0 && Boolean(onStageMany);
  const canUnstageMany = unstagedPaths.length > 0 && Boolean(onUnstageMany);
  const canDiscardMany = discardablePaths.length > 0 && Boolean(onDiscardMany);
  const canRemoveMany = untrackedPaths.length > 0 && Boolean(onRemoveUntrackedMany);

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

  const canDiscardSelected =
    selectedPath != null &&
    Boolean(onDiscard) &&
    unstaged.some((f) => f.path === selectedPath && f.kind !== "conflicted") &&
    !discardablePaths.includes(selectedPath) &&
    !checkedPaths.has(fileCheckKey("working", selectedPath));

  const canRemoveSelected =
    selectedPath != null &&
    Boolean(onRemoveUntracked) &&
    untracked.some((f) => f.path === selectedPath) &&
    !untrackedPaths.includes(selectedPath) &&
    !checkedPaths.has(fileCheckKey("working", selectedPath));

  return (
    <div className="flex h-full flex-col">
      <div className="shrink-0 border-b border-border px-3 py-2 text-xs font-medium text-muted">
        <span>Alterações {total > 0 ? `(${total})` : ""}</span>
        <div className="mt-1.5 flex flex-wrap gap-x-2 gap-y-1">
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
            {canDiscardMany && (
              <button
                type="button"
                onClick={() => onDiscardMany!(discardablePaths)}
                className="text-[10px] text-red-600 hover:underline dark:text-red-400"
              >
                Descartar selecionados ({discardablePaths.length})
              </button>
            )}
            {canRemoveMany && (
              <button
                type="button"
                onClick={() => onRemoveUntrackedMany!(untrackedPaths)}
                className="text-[10px] text-red-600 hover:underline dark:text-red-400"
              >
                Remover não rastreados ({untrackedPaths.length})
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
            {hasDiscardableUnstaged && !operationInProgress && onDiscardAll && (
              <button
                type="button"
                onClick={onDiscardAll}
                title="Descarta todas as alterações fora do stage"
                className="text-[10px] text-red-600 hover:underline dark:text-red-400"
              >
                Descartar tudo
              </button>
            )}
            {total > 0 && onStash && (
              <button
                type="button"
                onClick={onStash}
                className="text-[10px] text-accent hover:underline"
              >
                Guardar (stash)
              </button>
            )}
        </div>
        {(canStageSelected || canUnstageSelected || canDiscardSelected || canRemoveSelected) && (
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
            {canDiscardSelected && (
              <button
                type="button"
                onClick={() => onDiscard!(selectedPath!)}
                className="flex shrink-0 items-center gap-1 rounded border border-red-500/40 px-2 py-0.5 text-[10px] text-red-600 hover:bg-red-500/10 dark:text-red-400"
              >
                <Trash2 size={12} />
                Descartar
              </button>
            )}
            {canRemoveSelected && (
              <button
                type="button"
                onClick={() => onRemoveUntracked!(selectedPath!)}
                className="flex shrink-0 items-center gap-1 rounded border border-red-500/40 px-2 py-0.5 text-[10px] text-red-600 hover:bg-red-500/10 dark:text-red-400"
              >
                <Trash2 size={12} />
                Remover
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
        {operationInProgress && (
          <div className="mb-3 rounded border border-orange-500/40 bg-orange-500/10 px-3 py-2 text-xs text-orange-800 dark:text-orange-200">
            <p>{operationInProgress.message}</p>
            <div className="mt-2 flex flex-wrap gap-3">
              {operationInProgress.canContinue && onContinueOperation && (
                <button
                  type="button"
                  onClick={() => onContinueOperation(operationInProgress.kind)}
                  className="text-[10px] font-medium text-orange-900 underline hover:no-underline dark:text-orange-100"
                >
                  {operationInProgress.kind === "revert"
                    ? "Continuar revert"
                    : operationInProgress.kind === "merge"
                      ? "Continuar merge"
                      : "Continuar cherry-pick"}
                </button>
              )}
              {onAbortOperation && (
                <button
                  type="button"
                  onClick={() => onAbortOperation(operationInProgress.kind)}
                  className="text-[10px] font-medium text-orange-900 underline hover:no-underline dark:text-orange-100"
                >
                  {operationInProgress.kind === "revert"
                    ? "Abortar revert"
                    : operationInProgress.kind === "merge"
                      ? "Abortar merge"
                      : "Abortar cherry-pick"}
                </button>
              )}
            </div>
          </div>
        )}
        {total === 0 && !operationInProgress ? (
          <p className="px-2 py-1 text-center text-xs text-muted/70">
            Working tree limpa
          </p>
        ) : total === 0 ? null : (
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
              onDiscard={onDiscard}
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
              onRemoveUntracked={onRemoveUntracked}
            />
          </>
        )}
      </div>
    </div>
  );
}
