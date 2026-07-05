import { Plus, Undo2 } from "lucide-react";
import type { CommitDto } from "@/types";
import { BlamePanel } from "@/components/BlamePanel";
import { DiffViewer } from "@/components/DiffViewer";
import { ResizableRows } from "@/components/ResizableRows";
import type { BlameLineDto, BlameSourceDto } from "@/types";

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
  canUncommit?: boolean;
  canEditMessage?: boolean;
  /** Por que não dá para editar a mensagem deste commit. */
  messageEditHint?: string | null;
  onRevert?: () => void;
  onUncommit?: () => void;
  onEditMessage?: () => void;
  /** Working tree: arquivo selecionado pode ir p/ stage ou sair do stage. */
  workingTreeFile?: boolean;
  showStageFile?: boolean;
  showUnstageFile?: boolean;
  onStageFile?: () => void;
  onUnstageFile?: () => void;
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
  canUncommit,
  canEditMessage,
  messageEditHint,
  onRevert,
  onUncommit,
  onEditMessage,
  workingTreeFile,
  showStageFile,
  showUnstageFile,
  onStageFile,
  onUnstageFile,
}: DetailPanelProps) {
  const showBlame = Boolean(filePath);

  if (!commit && !diff && !loading && !showBlame) {
    return (
      <div className="flex h-full items-center justify-center p-6 text-sm text-muted">
        Selecione um commit ou arquivo para ver detalhes
      </div>
    );
  }

  return (
    <div className="flex h-full flex-col overflow-hidden">
      {commit && (
        <div className="border-b border-border px-4 py-3">
          <h2 className="text-sm font-semibold">{commit.summary}</h2>
          <p className="mt-1 text-xs text-muted">
            <span className="font-mono">{commit.id}</span>
            {" · "}
            {commit.authorName}
            {" · "}
            {new Date(commit.authoredAt).toLocaleString("pt-BR")}
          </p>
          {commit.body && (
            <p className="mt-2 whitespace-pre-wrap text-xs text-text">
              {commit.body}
            </p>
          )}
          {(onRevert ||
            (canUncommit && onUncommit) ||
            (canEditMessage && onEditMessage)) && (
            <div className="mt-2 flex flex-wrap gap-2">
              {canEditMessage && onEditMessage && (
                <button
                  type="button"
                  onClick={onEditMessage}
                  className="rounded border border-accent/40 bg-accent/10 px-2 py-0.5 text-[10px] text-accent hover:bg-accent/20"
                >
                  Editar mensagem
                </button>
              )}
              {onRevert && (
                <button
                  type="button"
                  onClick={onRevert}
                  className="rounded border border-border px-2 py-0.5 text-[10px] text-muted hover:bg-surface hover:text-text"
                >
                  Reverter commit
                </button>
              )}
              {canUncommit && onUncommit && (
                <button
                  type="button"
                  onClick={onUncommit}
                  className="rounded border border-border px-2 py-0.5 text-[10px] text-muted hover:bg-surface hover:text-text"
                >
                  Uncommit (soft)
                </button>
              )}
            </div>
          )}
          {messageEditHint && (
            <p className="mt-2 text-[10px] leading-snug text-muted">
              {messageEditHint}
            </p>
          )}
        </div>
      )}
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
          </div>
        </div>
      )}
      {showBlame ? (
        <ResizableRows
          storageKey="trilho.rows.detail.v1"
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
      ) : (
        <div className="min-h-0 flex-1">
          <DiffViewer
            diff={diff}
            loading={loading}
            onLineClick={filePath ? onLineClick : undefined}
            selectedLine={blameFocusLine}
          />
        </div>
      )}
    </div>
  );
}
