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
        </div>
      )}
      {filePath && !commit && (
        <div className="border-b border-border px-4 py-2 text-xs font-medium truncate">
          {filePath}
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
