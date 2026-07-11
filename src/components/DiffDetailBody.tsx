import { useEffect, useMemo, useState } from "react";

import { BlamePanel, BlameSourcePicker } from "@/components/BlamePanel";
import { DiffViewer } from "@/components/DiffViewer";
import { WorktreeFileEditor } from "@/components/WorktreeFileEditor";
import {
  EMPTY_BLAME_FILTERS,
  filterAndSortBlameLines,
  hasActiveBlameFilters,
  uniqueBlameAuthors,
  type BlameFilters,
} from "@/lib/blame-filters";
import type { DiffHunk } from "@/lib/diff-hunks";
import type { BlameLineDto, BlameSourceDto, CommitDto } from "@/types";

export type DetailTab = "diff" | "blame";
export type WorktreeView = "changes" | "file";

interface DiffDetailBodyProps {
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
  hunks?: DiffHunk[];
  onDiscardHunk?: (patch: string) => void;
  worktreeView?: WorktreeView;
  writeDisabled?: boolean;
  onSaveWorktreeFile?: (content: string) => Promise<void>;
  fileReloadKey?: string | null;
  blameShowAuthoredAt?: boolean;
  onBlameCommitClick?: (commitId: string) => void;
  /** Legenda opcional na aba Diff (ex.: commit do blame no overlay). */
  diffSubtitle?: string;
  /** Picker Commit|WT|Staging visível nas abas Diff e Blame (overlay). */
  showBlameSourcePicker?: boolean;
  /** Branch em checkout — contexto do blame. */
  branchName?: string | null;
}

function BlameFiltersBar({
  filters,
  authors,
  totalLines,
  filteredCount,
  onChange,
  onClear,
}: {
  filters: BlameFilters;
  authors: string[];
  totalLines: number;
  filteredCount: number;
  onChange: (next: BlameFilters) => void;
  onClear: () => void;
}) {
  return (
    <div className="flex shrink-0 flex-wrap items-center gap-2 border-b border-border px-3 py-1.5 text-[11px]">
      <label className="flex items-center gap-1 text-muted">
        Autor
        <select
          value={filters.author ?? ""}
          onChange={(e) =>
            onChange({
              ...filters,
              author: e.target.value || null,
            })
          }
          className="max-w-[10rem] rounded border border-border bg-bg px-1.5 py-0.5 text-text"
        >
          <option value="">Todos</option>
          {authors.map((author) => (
            <option key={author} value={author}>
              {author}
            </option>
          ))}
        </select>
      </label>
      <label className="flex items-center gap-1 text-muted">
        De
        <input
          type="date"
          value={filters.dateFrom}
          onChange={(e) =>
            onChange({ ...filters, dateFrom: e.target.value })
          }
          className="rounded border border-border bg-bg px-1.5 py-0.5 text-text"
        />
      </label>
      <label className="flex items-center gap-1 text-muted">
        Até
        <input
          type="date"
          value={filters.dateTo}
          onChange={(e) =>
            onChange({ ...filters, dateTo: e.target.value })
          }
          className="rounded border border-border bg-bg px-1.5 py-0.5 text-text"
        />
      </label>
      {hasActiveBlameFilters(filters) && (
        <button
          type="button"
          onClick={onClear}
          className="rounded border border-border px-2 py-0.5 text-muted hover:bg-surface hover:text-text"
        >
          Limpar
        </button>
      )}
      <span className="ml-auto text-[10px] text-muted">
        {filteredCount} de {totalLines} · mais recente primeiro
      </span>
    </div>
  );
}

export function DiffDetailBody({
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
  hunks,
  onDiscardHunk,
  worktreeView,
  writeDisabled,
  onSaveWorktreeFile,
  fileReloadKey,
  blameShowAuthoredAt,
  onBlameCommitClick,
  diffSubtitle,
  showBlameSourcePicker,
  branchName,
}: DiffDetailBodyProps) {
  const [blameFilters, setBlameFilters] = useState<BlameFilters>(
    EMPTY_BLAME_FILTERS,
  );

  useEffect(() => {
    setBlameFilters(EMPTY_BLAME_FILTERS);
  }, [filePath]);

  const blameAuthors = useMemo(
    () => uniqueBlameAuthors(blameLines),
    [blameLines],
  );
  const displayedBlameLines = useMemo(
    () => filterAndSortBlameLines(blameLines, blameFilters),
    [blameLines, blameFilters],
  );

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
        {showBlameSourcePicker && (
          <div className="flex shrink-0 items-center justify-end border-b border-border px-3 py-1.5">
            <BlameSourcePicker
              source={blameSource}
              onSourceChange={onBlameSourceChange}
            />
          </div>
        )}
        {detailTab === "blame" && !blameLoading && blameLines.length > 0 && (
          <BlameFiltersBar
            filters={blameFilters}
            authors={blameAuthors}
            totalLines={blameLines.length}
            filteredCount={displayedBlameLines.length}
            onChange={setBlameFilters}
            onClear={() => setBlameFilters(EMPTY_BLAME_FILTERS)}
          />
        )}
        {diffSubtitle && detailTab === "diff" && (
          <p className="shrink-0 border-b border-border px-3 py-1 text-[10px] text-muted">
            Diff do{" "}
            <span className="font-mono text-accent">{diffSubtitle}</span>
          </p>
        )}
        <div className="min-h-0 flex-1 overflow-hidden">
          {detailTab === "diff" ? (
            worktreeView === "file" && filePath ? (
              <WorktreeFileEditor
                path={filePath}
                writeDisabled={writeDisabled}
                onSave={onSaveWorktreeFile}
                reloadKey={fileReloadKey}
              />
            ) : (
              <DiffViewer
                diff={diff}
                loading={loading}
                layout={hunks?.length ? "unified" : "sideBySide"}
                hunks={hunks}
                onDiscardHunk={onDiscardHunk}
                onLineClick={filePath ? onLineClick : undefined}
                selectedLine={blameFocusLine}
              />
            )
          ) : (
            <BlamePanel
              path={filePath}
              source={blameSource}
              onSourceChange={onBlameSourceChange}
              lines={displayedBlameLines}
              focusLine={blameFocusLine}
              loading={blameLoading}
              error={blameError}
              showSourcePicker={!commit && !showBlameSourcePicker}
              embedded
              showAuthoredAt={blameShowAuthoredAt}
              branchName={branchName}
              onCommitClick={blameShowAuthoredAt ? onBlameCommitClick : undefined}
              emptyHint={
                blameLines.length > 0 && displayedBlameLines.length === 0
                  ? "Nenhuma linha corresponde aos filtros."
                  : null
              }
            />
          )}
        </div>
      </div>
    );
  }

  return (
    <div className="min-h-0 flex-1">
      {worktreeView === "file" && filePath ? (
        <WorktreeFileEditor
          path={filePath}
          writeDisabled={writeDisabled}
          onSave={onSaveWorktreeFile}
          reloadKey={fileReloadKey}
        />
      ) : (
        <DiffViewer
          diff={diff}
          loading={loading}
          layout={hunks?.length ? "unified" : "sideBySide"}
          hunks={hunks}
          onDiscardHunk={onDiscardHunk}
          onLineClick={filePath ? onLineClick : undefined}
          selectedLine={blameFocusLine}
        />
      )}
    </div>
  );
}
