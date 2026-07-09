import { Maximize2, Plus, Trash2, Undo2 } from "lucide-react";import { useEffect, useMemo, useState } from "react";

import { BlamePanel } from "@/components/BlamePanel";
import { ConflictOverlay } from "@/components/ConflictOverlay";
import { ConflictResolver } from "@/components/ConflictResolver";
import { DiffOverlay } from "@/components/DiffOverlay";
import { DiffViewer } from "@/components/DiffViewer";
import { extractHunks } from "@/lib/diff-hunks";
import type { BlameLineDto, BlameSourceDto, CommitDto } from "@/types";

type DetailTab = "diff" | "blame";

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
  /** RF-20 — arquivo em conflito: mostra resolvedor 3-vias. */
  conflicted?: boolean;
  conflictOperationKind?: "revert" | "merge" | "cherryPick" | null;
  writeDisabled?: boolean;
  onResolveConflictSide?: (side: "ours" | "theirs") => void;
  onResolveConflictContent?: (content: string) => void;
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
  detailTab,
  onDetailTabChange,
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
  detailTab: DetailTab;
  onDetailTabChange: (tab: DetailTab) => void;
}) {
  if (showBlame) {
    return (
      <div className="flex h-full min-h-0 flex-col">
        <div className="flex shrink-0 gap-0.5 border-b border-border px-2 py-1">
          {(["diff", "blame"] as const).map((tab) => (
            <button
              key={tab}
              type="button"
              onClick={() => onDetailTabChange(tab)}
              className={`rounded px-2.5 py-0.5 text-[11px] font-medium ${
                detailTab === tab
                  ? "bg-accent text-white"
                  : "text-muted hover:bg-surface hover:text-text"
              }`}
            >
              {tab === "diff" ? "Diff" : "Blame"}
            </button>
          ))}
        </div>
        <div className="min-h-0 flex-1 overflow-hidden">
          {detailTab === "diff" ? (
            <DiffViewer
              diff={diff}
              loading={loading}
              onLineClick={filePath ? onLineClick : undefined}
              selectedLine={blameFocusLine}
            />
          ) : (
            <BlamePanel
              path={filePath}
              source={blameSource}
              onSourceChange={onBlameSourceChange}
              lines={blameLines}
              focusLine={blameFocusLine}
              loading={blameLoading}
              error={blameError}
              showSourcePicker={!commit}
              embedded
            />
          )}
        </div>
      </div>
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
  conflicted,
  conflictOperationKind,
  writeDisabled,
  onResolveConflictSide,
  onResolveConflictContent,
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
  const [conflictExpanded, setConflictExpanded] = useState(false);
  const [detailTab, setDetailTab] = useState<DetailTab>("diff");  const showBlame = Boolean(filePath) && !conflicted;
  const hasDiffContent = Boolean(diff || loading || filePath);
  const hunks = useMemo(
    () => (diff && showDiscardFile && !conflicted ? extractHunks(diff) : []),
    [diff, showDiscardFile, conflicted],
  );

  useEffect(() => {
    setDiffExpanded(false);
    setDetailTab("diff");
  }, [filePath, commit?.id]);

  useEffect(() => {
    if (conflicted && filePath) {
      setConflictExpanded(true);
    } else {
      setConflictExpanded(false);
    }
  }, [conflicted, filePath]);

  if (
    conflicted &&
    filePath &&
    onResolveConflictSide &&
    onResolveConflictContent
  ) {
    return (
      <>
        <div className="relative flex h-full min-h-0 flex-col overflow-hidden">
          {conflictExpanded ? (
            <div className="flex min-h-0 flex-1 flex-col items-center justify-center gap-3 bg-surface/50 p-6 text-center">
              <p className="text-xs text-muted">
                Conflito em <span className="font-mono text-text">{filePath}</span>
              </p>
              <p className="text-[11px] text-muted">
                Resolução aberta em tela cheia — alterações ficam com mais espaço
                no painel de cima.
              </p>
              <button
                type="button"
                onClick={() => setConflictExpanded(false)}
                className="rounded-lg border border-border px-3 py-1.5 text-xs text-text hover:bg-surface"
              >
                Restaurar no painel
              </button>
            </div>
          ) : (
            <>
              <div className="flex shrink-0 justify-end border-b border-border px-2 py-1">
                <button
                  type="button"
                  onClick={() => setConflictExpanded(true)}
                  className="flex items-center gap-1 rounded px-2 py-0.5 text-[10px] text-muted hover:bg-surface hover:text-text"
                  aria-label="Destacar conflito em tela cheia"
                >
                  <Maximize2 size={12} />
                  Destacar conflito
                </button>
              </div>
              <div className="min-h-0 flex-1">
                <ConflictResolver
                  path={filePath}
                  operationKind={conflictOperationKind}
                  writeDisabled={writeDisabled}
                  onResolveSide={onResolveConflictSide}
                  onResolveContent={onResolveConflictContent}
                />
              </div>
            </>
          )}
        </div>
        <ConflictOverlay
          open={conflictExpanded}
          onClose={() => setConflictExpanded(false)}
          path={filePath}
          operationKind={conflictOperationKind}
          writeDisabled={writeDisabled}
          onResolveSide={onResolveConflictSide}
          onResolveContent={onResolveConflictContent}
        />
      </>
    );
  }

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
            detailTab={detailTab}
            onDetailTabChange={setDetailTab}
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
