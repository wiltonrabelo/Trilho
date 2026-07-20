import { Maximize2 } from "lucide-react";
import { useEffect, useMemo, useState } from "react";

import { ConflictOverlay } from "@/components/ConflictOverlay";
import { ConflictResolver } from "@/components/ConflictResolver";
import { DiffDetailBody, type DetailTab, type WorktreeView } from "@/components/DiffDetailBody";
import { DiffOverlay } from "@/components/DiffOverlay";
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
  /** RF-20 — arquivo em conflito: mostra resolvedor 3-vias. */
  conflicted?: boolean;
  conflictOperationKind?: "revert" | "merge" | "cherryPick" | null;
  writeDisabled?: boolean;
  onResolveConflictSide?: (side: "ours" | "theirs") => void;
  onResolveConflictContent?: (content: string) => void;
  onDiscardHunk?: (patch: string) => void;
  /** Arquivo alterado no working tree — abas Alterações / Arquivo. */
  canRevertHunks?: boolean;
  onSaveWorktreeFile?: (content: string) => Promise<void>;
  fileReloadKey?: string | null;
  branchName?: string | null;
  /** Incrementar para abrir a aba Blame (menu de contexto do arquivo). */
  openBlameRequest?: number;
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
  onDiscardHunk,
  canRevertHunks,
  onSaveWorktreeFile,
  fileReloadKey,
  branchName,
  openBlameRequest = 0,
}: DetailPanelProps) {
  const [diffExpanded, setDiffExpanded] = useState(false);
  const [conflictExpanded, setConflictExpanded] = useState(false);
  const [detailTab, setDetailTab] = useState<DetailTab>("diff");
  const [worktreeView, setWorktreeView] = useState<WorktreeView>("changes");
  const showBlame = Boolean(filePath) && !conflicted;
  const hasDiffContent = Boolean(diff || loading || filePath);
  const hunks = useMemo(
    () =>
      diff && canRevertHunks && onDiscardHunk && !conflicted
        ? extractHunks(diff)
        : [],
    [diff, canRevertHunks, onDiscardHunk, conflicted],
  );

  useEffect(() => {
    setDiffExpanded(false);
    setDetailTab("diff");
    setWorktreeView("changes");
  }, [filePath]);

  useEffect(() => {
    setDetailTab("diff");
    setWorktreeView("changes");
  }, [commit?.id]);

  useEffect(() => {
    if (openBlameRequest > 0 && filePath && !conflicted) {
      setDetailTab("blame");
      setDiffExpanded(false);
    }
  }, [openBlameRequest, filePath, conflicted]);

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
        <div className="flex flex-col gap-2 border-b border-border px-4 py-2">
          <p
            className="font-mono text-xs font-medium leading-snug break-all text-text"
            title={filePath}
          >
            {filePath}
          </p>
          <div className="flex flex-wrap items-center gap-2">
            <div
              className="inline-flex rounded-md border border-border p-0.5"
              role="group"
              aria-label="Visão do arquivo"
            >
              <button
                type="button"
                onClick={() => setWorktreeView("changes")}
                className={`rounded px-2 py-0.5 text-[10px] font-medium ${
                  worktreeView === "changes"
                    ? "bg-accent text-white"
                    : "text-muted hover:bg-surface"
                }`}
              >
                Alterações
              </button>
              <button
                type="button"
                onClick={() => setWorktreeView("file")}
                className={`rounded px-2 py-0.5 text-[10px] font-medium ${
                  worktreeView === "file"
                    ? "bg-accent text-white"
                    : "text-muted hover:bg-surface"
                }`}
              >
                Arquivo
              </button>
            </div>
          </div>
        </div>
      )}

      {hunks.length > 0 && worktreeView === "changes" && (
        <p className="border-b border-border px-4 py-1.5 text-[10px] text-muted">
          Cada trecho abaixo pode ser revertido individualmente — use{" "}
          <span className="font-medium text-text">Reverter trecho</span> no cabeçalho
          do bloco.
        </p>
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
            hunks={hunks}
            onDiscardHunk={onDiscardHunk}
            worktreeView={workingTreeFile ? worktreeView : undefined}
            writeDisabled={writeDisabled}
            onSaveWorktreeFile={workingTreeFile ? onSaveWorktreeFile : undefined}
            fileReloadKey={fileReloadKey}
            branchName={branchName}
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
        workingTreeFile={workingTreeFile}
        worktreeView={worktreeView}
        onWorktreeViewChange={workingTreeFile ? setWorktreeView : undefined}
        hunks={hunks}
        onDiscardHunk={onDiscardHunk}
        writeDisabled={writeDisabled}
        onSaveWorktreeFile={workingTreeFile ? onSaveWorktreeFile : undefined}
        fileReloadKey={fileReloadKey}
        branchName={branchName}
      />
    </div>
  );
}
