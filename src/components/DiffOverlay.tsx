import { Minimize2 } from "lucide-react";
import { useCallback, useEffect, useRef, useState } from "react";

import {
  DiffDetailBody,
  type DetailTab,
  type WorktreeView,
} from "@/components/DiffDetailBody";
import { DiffViewer } from "@/components/DiffViewer";
import { useDialogA11y } from "@/hooks/useDialogA11y";
import { getCommitFileDiff, getFileDiff } from "@/lib/api";
import type { DiffHunk } from "@/lib/diff-hunks";
import type { BlameLineDto, BlameSourceDto, CommitDto } from "@/types";
interface DiffOverlayProps {
  open: boolean;
  onClose: () => void;
  filePath: string | null;
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
  workingTreeFile?: boolean;
  worktreeView?: WorktreeView;
  onWorktreeViewChange?: (view: WorktreeView) => void;
  hunks?: DiffHunk[];
  onDiscardHunk?: (patch: string) => void;
  writeDisabled?: boolean;
  onSaveWorktreeFile?: (content: string) => Promise<void>;
  fileReloadKey?: string | null;
  blameShowAuthoredAt?: boolean;
  branchName?: string | null;
}

function isNavigableBlameCommit(commitId: string): boolean {
  const hex = commitId.replace(/[^0-9a-f]/gi, "");
  return hex.length > 0 && !/^0+$/.test(hex);
}

interface PeekCommit {
  commitId: string;
  shortId: string;
}

interface CachedDiff {
  diff: string | null;
  loading: boolean;
  error: string | null;
}

function emptyCachedDiff(): CachedDiff {
  return { diff: null, loading: false, error: null };
}
export function DiffOverlay({
  open,
  onClose,
  filePath,
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
  workingTreeFile,
  worktreeView = "changes",
  onWorktreeViewChange,
  hunks,
  onDiscardHunk,
  writeDisabled,
  onSaveWorktreeFile,
  fileReloadKey,
  blameShowAuthoredAt = true,
  branchName,
}: DiffOverlayProps) {
  const panelRef = useRef<HTMLDivElement>(null);
  const [detailTab, setDetailTab] = useState<DetailTab>("diff");
  const [peekCommit, setPeekCommit] = useState<PeekCommit | null>(null);
  const [commitDiffCache, setCommitDiffCache] = useState<CachedDiff>(
    emptyCachedDiff,
  );
  const [stagingDiffCache, setStagingDiffCache] = useState<CachedDiff>(
    emptyCachedDiff,
  );
  const [overlaySource, setOverlaySource] =
    useState<BlameSourceDto>(blameSource);
  useDialogA11y(open, onClose, panelRef);

  const loadCommitDiff = useCallback(
    async (commitId: string) => {
      if (!filePath) return;
      setCommitDiffCache({ diff: null, loading: true, error: null });
      try {
        const d = await getCommitFileDiff(commitId, filePath);
        setCommitDiffCache({
          diff: d || "(sem alterações)",
          loading: false,
          error: null,
        });
      } catch (e) {
        setCommitDiffCache({
          diff: null,
          loading: false,
          error: String(e),
        });
      }
    },
    [filePath],
  );

  const loadStagingDiff = useCallback(async () => {
    if (!filePath) return;
    setStagingDiffCache({ diff: null, loading: true, error: null });
    try {
      const d = await getFileDiff(filePath, true);
      setStagingDiffCache({
        diff: d || "(sem alterações)",
        loading: false,
        error: null,
      });
    } catch (e) {
      setStagingDiffCache({
        diff: null,
        loading: false,
        error: String(e),
      });
    }
  }, [filePath]);

  useEffect(() => {
    if (open) {
      setDetailTab("diff");
      setPeekCommit(null);
      setCommitDiffCache(emptyCachedDiff());
      setStagingDiffCache(emptyCachedDiff());
      setOverlaySource(blameSource);
    }
  }, [open, filePath, blameSource]);

  const handleOverlaySourceChange = useCallback(
    (source: BlameSourceDto) => {
      setOverlaySource(source);
      if (source === "workingTree") {
        onBlameSourceChange("workingTree");
        return;
      }
      if (source === "staging") {
        onBlameSourceChange("staging");
        if (!stagingDiffCache.diff && !stagingDiffCache.loading) {
          void loadStagingDiff();
        }
        return;
      }
      if (source === "commit" && peekCommit) {
        if (!commitDiffCache.diff && !commitDiffCache.loading) {
          void loadCommitDiff(peekCommit.commitId);
        }
      }
    },
    [
      onBlameSourceChange,
      peekCommit,
      commitDiffCache.diff,
      commitDiffCache.loading,
      stagingDiffCache.diff,
      stagingDiffCache.loading,
      loadCommitDiff,
      loadStagingDiff,
    ],
  );

  const handleBlameCommitClick = useCallback(
    async (commitId: string) => {
      if (!filePath || !isNavigableBlameCommit(commitId)) return;
      const shortId = commitId.slice(0, 7);
      setPeekCommit({ commitId, shortId });
      setOverlaySource("commit");
      setDetailTab("diff");
      await loadCommitDiff(commitId);
    },
    [filePath, loadCommitDiff],
  );
  if (!open) return null;

  const showBlame = Boolean(filePath);
  const title = filePath ?? "Diff";
  const showWorktreeTabs = Boolean(workingTreeFile && filePath && onWorktreeViewChange);
  const isCommitView = overlaySource === "commit";
  const isStagingView = overlaySource === "staging";
  const isWorkingTreeView = overlaySource === "workingTree";

  const showHunkHint =
    showWorktreeTabs &&
    worktreeView === "changes" &&
    isWorkingTreeView &&
    (hunks?.length ?? 0) > 0;

  const overlayDiff = isCommitView
    ? peekCommit
      ? commitDiffCache.error
        ? `Erro: ${commitDiffCache.error}`
        : commitDiffCache.diff
      : "Selecione um commit na aba Blame para ver o diff deste arquivo."
    : isStagingView
      ? stagingDiffCache.error
        ? `Erro: ${stagingDiffCache.error}`
        : stagingDiffCache.diff
      : diff;
  const overlayLoading = isCommitView
    ? Boolean(peekCommit && commitDiffCache.loading)
    : isStagingView
      ? stagingDiffCache.loading
      : loading;
  const overlayHunks = isWorkingTreeView ? hunks : [];
  const overlayWorktreeView: WorktreeView | undefined =
    isCommitView || isStagingView
      ? "changes"
      : workingTreeFile
        ? worktreeView
        : undefined;

  return (
    <div
      className="fixed inset-0 z-50 flex flex-col bg-bg"
      role="dialog"
      aria-modal="true"
      aria-labelledby="diff-overlay-title"
    >
      <div ref={panelRef} className="flex h-full min-h-0 flex-col">
        <div className="flex shrink-0 flex-col gap-2 border-b border-border bg-surface px-4 py-2">
          <div className="flex items-start justify-between gap-3">
            <h2
              id="diff-overlay-title"
              className="min-w-0 flex-1 font-mono text-xs font-medium leading-snug break-all"
              title={title}
            >
              {title}
            </h2>
            <button
              type="button"
              onClick={onClose}
              className="flex shrink-0 items-center gap-1.5 rounded-lg border border-border px-2.5 py-1 text-xs text-muted hover:bg-bg hover:text-text"
            >
              <Minimize2 size={14} />
              Restaurar
            </button>
          </div>
          {showWorktreeTabs && (
            <div
              className="inline-flex w-fit rounded-md border border-border p-0.5"
              role="group"
              aria-label="Visão do arquivo"
            >
              <button
                type="button"
                onClick={() => onWorktreeViewChange!("changes")}
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
                onClick={() => onWorktreeViewChange!("file")}
                className={`rounded px-2 py-0.5 text-[10px] font-medium ${
                  worktreeView === "file"
                    ? "bg-accent text-white"
                    : "text-muted hover:bg-surface"
                }`}
              >
                Arquivo
              </button>
            </div>
          )}
        </div>
        {showHunkHint && (
          <p className="shrink-0 border-b border-border px-4 py-1.5 text-[10px] text-muted">
            Cada trecho abaixo pode ser revertido individualmente — use{" "}
            <span className="font-medium text-text">Reverter trecho</span> no cabeçalho
            do bloco.
          </p>
        )}
        <div className="min-h-0 flex-1">
          {showBlame ? (
            <DiffDetailBody
              filePath={filePath}
              showBlame
              diff={overlayDiff}
              loading={overlayLoading}
              commit={commit}
              blameSource={overlaySource}
              onBlameSourceChange={handleOverlaySourceChange}
              blameLines={blameLines}
              blameFocusLine={blameFocusLine}
              blameLoading={blameLoading}
              blameError={blameError}
              onLineClick={onLineClick}
              detailTab={detailTab}
              onDetailTabChange={setDetailTab}
              hunks={overlayHunks}
              onDiscardHunk={isWorkingTreeView ? onDiscardHunk : undefined}
              worktreeView={overlayWorktreeView}
              writeDisabled={writeDisabled}
              onSaveWorktreeFile={onSaveWorktreeFile}
              fileReloadKey={fileReloadKey}
              blameShowAuthoredAt={blameShowAuthoredAt}
              onBlameCommitClick={handleBlameCommitClick}
              diffSubtitle={
                isCommitView && peekCommit
                  ? `commit ${peekCommit.shortId}`
                  : isStagingView
                    ? "staging"
                    : undefined
              }
              showBlameSourcePicker
              branchName={branchName}
            />
          ) : (
            <DiffViewer
              diff={diff}
              loading={loading}
              onLineClick={filePath ? onLineClick : undefined}
              selectedLine={blameFocusLine}
            />
          )}
        </div>
      </div>
    </div>
  );
}
