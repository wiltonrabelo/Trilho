import { Maximize2, Plus, Trash2, Undo2 } from "lucide-react";
import { useEffect, useMemo, useState } from "react";

import { BlamePanel } from "@/components/BlamePanel";
import { DiffOverlay } from "@/components/DiffOverlay";
import { DiffViewer } from "@/components/DiffViewer";
import { ResizableRows } from "@/components/ResizableRows";
import { extractHunks } from "@/lib/diff-hunks";
import type { BlameLineDto, BlameSourceDto, CommitDto } from "@/types";

interface DetailPanelProps {
  commit: CommitDto | null;
  filePath: string | null;
  diff: string | null;
  loading?: boolean;
  blameSource: BlameSourceDto;
  onBlameSourceChange: (source: BlameSourceDto) => void;
  blameLines: BlameLineDto[];
  blameFocusLine: number | null;
  blameLoading?: boolean;
  blameError?: string | null;
  onLineClick?: (lineNo: number) => void;
  workingTreeFile?: boolean;
  showStageFile?: boolean;
  showUnstageFile?: boolean;
  showDiscardFile?: boolean;
  showRemoveUntracked?: boolean;
  onStageFile?: () => void;
  onUnstageFile?: () => void;
  onDiscardFile?: () => void;
  onRemoveUntracked?: () => void;
  onDiscardHunk?: (patch: string) => void;
}

function DiffDetailBody({
  filePath,
  showBlame,
  diff,
  loading,
  commit,
  blameSource,
  onBlameSourceChange,
  blameLines,
  blameFocusLine,
  blameLoading,
  blameError,
  onLineClick,
  rowsStorageKey,
}: {
  filePath: string | null;
  showBlame: boolean;
  diff: string | null;
  loading?: boolean;
  commit: CommitDto | null;
  blameSource: BlameSourceDto;
  onBlameSourceChange: (source: BlameSourceDto) => void;
  blameLines: BlameLineDto[];
  blameFocusLine: number | null;
  blameLoading?: boolean;
  blameError?: string | null;
  onLineClick?: (lineNo: number) => void;
  rowsStorageKey: string;
}) {
  if (showBlame) {
    return (
      <ResizableRows
        storageKey={rowsStorageKey}
        defaultTop={220}
        minTop={100}
        minBottom={120}
        top={
          <DiffViewer
            diff={diff}
            loading={loading}
            onLineClick={filePath ? onLineClick : undefined}
            selectedLine={blameFocusLine}
          />
        }
        bottom={
          <BlamePanel
            path={filePath}
            source={blameSource}
            onSourceChange={onBlameSourceChange}
            lines={blameLines}
            focusLine={blameFocusLine}
            loading={blameLoading}
            error={blameError}
            showSourcePicker={!commit}
          />
        }
      />
    );
  }

  return (
    <div className="min-h-0 flex-1">
      <DiffViewer
        diff={diff}
        loading={loading}
        onLineClick={filePath ? onLineClick : undefined}
        selectedLine={blameFocusLine}
      />
    </div>
  );
}

export function DetailPanel({
  commit,
  filePath,
  diff,
  loading,
  blameSource,
  onBlameSourceChange,
  blameLines,
  blameFocusLine,
  blameLoading,
  blameError,
  onLineClick,
  workingTreeFile,
  showStageFile,
  showUnstageFile,
  showDiscardFile,
  showRemoveUntracked,
  onStageFile,
  onUnstageFile,
  onDiscardFile,
  onRemoveUntracked,
  onDiscardHunk,
}: DetailPanelProps) {
  const [diffExpanded, setDiffExpanded] = useState(false);
  const showBlame = Boolean(filePath);
  const hasDiffContent = Boolean(diff || loading || filePath);
  const hunks = useMemo(
    () => (diff && showDiscardFile ? extractHunks(diff) : []),
    [diff, showDiscardFile],
  );

  useEffect(() => {
    setDiffExpanded(false);
  }, [filePath, commit?.id]);

  if (!commit && !diff && !loading && !showBlame) {
    return (
      <div className="flex h-full items-center justify-center p-6 text-sm text-muted">
        Selecione um commit ou arquivo para ver detalhes
      </div>
    );
  }

  return (
    <div className="relative flex h-full flex-col overflow-hidden">
      {filePath && workingTreeFile && (
        <div className="flex items-center justify-between gap-2 border-b border-border px-4 py-2">
          <span className="min-w-0 truncate text-xs font-medium">{filePath}</span>
          <div className="flex shrink-0 gap-1.5">
            {showStageFile && onStageFile && (
              <button
                type="button"
                onClick={onStageFile}
                className="flex items-center gap-1 rounded border border-accent/40 bg-accent/10 px-2 py-0.5 text-[10px] text-accent hover:bg-accent/20"
              >
                <Plus size={12} />
                Stage
              </button>
            )}
            {showUnstageFile && onUnstageFile && (
              <button
                type="button"
                onClick={onUnstageFile}
                className="flex items-center gap-1 rounded border border-border px-2 py-0.5 text-[10px] text-muted hover:bg-surface hover:text-text"
              >
                <Undo2 size={12} />
                Unstage
              </button>
            )}
            {showDiscardFile && onDiscardFile && (
              <button
                type="button"
                onClick={onDiscardFile}
                className="flex items-center gap-1 rounded border border-red-500/40 px-2 py-0.5 text-[10px] text-red-600 hover:bg-red-500/10 dark:text-red-400"
              >
                <Trash2 size={12} />
                Descartar arquivo
              </button>
            )}
            {showRemoveUntracked && onRemoveUntracked && (
              <button
                type="button"
                onClick={onRemoveUntracked}
                className="flex items-center gap-1 rounded border border-red-500/40 px-2 py-0.5 text-[10px] text-red-600 hover:bg-red-500/10 dark:text-red-400"
              >
                <Trash2 size={12} />
                Remover
              </button>
            )}
          </div>
        </div>
      )}

      {hunks.length > 0 && onDiscardHunk && (
        <div className="flex flex-wrap gap-2 border-b border-border px-4 py-2">
          <span className="text-[10px] text-muted">Descartar trecho:</span>
          {hunks.map((hunk) => (
            <button
              key={hunk.index}
              type="button"
              onClick={() => onDiscardHunk(hunk.patch)}
              title={hunk.header}
              className="rounded border border-red-500/30 px-2 py-0.5 font-mono text-[10px] text-red-600 hover:bg-red-500/10 dark:text-red-400"
            >
              #{hunk.index + 1}
            </button>
          ))}
        </div>
      )}

      {hasDiffContent && !diffExpanded && (
        <div className="flex shrink-0 justify-end border-b border-border px-2 py-1">
          <button
            type="button"
            onClick={() => setDiffExpanded(true)}
            className="flex items-center gap-1 rounded px-2 py-0.5 text-[10px] text-muted hover:bg-surface hover:text-text"
            aria-label="Destacar diff em tela ampliada"
          >
            <Maximize2 size={12} />
            Destacar diff
          </button>
        </div>
      )}

      {diffExpanded ? (
        <div className="flex min-h-0 flex-1 flex-col items-center justify-center gap-2 bg-surface/50 p-6 text-center">
          <p className="text-xs text-muted">Diff em tela ampliada</p>
          {filePath && (
            <p className="max-w-full truncate font-mono text-[10px] text-muted">
              {filePath}
            </p>
          )}
          <button
            type="button"
            onClick={() => setDiffExpanded(false)}
            className="rounded-lg border border-border px-3 py-1.5 text-xs text-text hover:bg-surface"
          >
            Restaurar no painel
          </button>
        </div>
      ) : (
        <div className="min-h-0 flex-1">
          <DiffDetailBody
            filePath={filePath}
            showBlame={showBlame}
            diff={diff}
            loading={loading}
            commit={commit}
            blameSource={blameSource}
            onBlameSourceChange={onBlameSourceChange}
            blameLines={blameLines}
            blameFocusLine={blameFocusLine}
            blameLoading={blameLoading}
            blameError={blameError}
            onLineClick={onLineClick}
            rowsStorageKey="trilho.rows.detail.v1"
          />
        </div>
      )}

      <DiffOverlay
        open={diffExpanded}
        onClose={() => setDiffExpanded(false)}
        filePath={filePath}
        diff={diff}
        loading={loading}
        commit={commit}
        blameSource={blameSource}
        onBlameSourceChange={onBlameSourceChange}
        blameLines={blameLines}
        blameFocusLine={blameFocusLine}
        blameLoading={blameLoading}
        blameError={blameError}
        onLineClick={onLineClick}
      />
    </div>
  );
}
