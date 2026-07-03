import type { CommitDto } from "@/types";
import { DiffViewer } from "@/components/DiffViewer";

interface DetailPanelProps {
  commit: CommitDto | null;
  diff: string | null;
  loading?: boolean;
}

export function DetailPanel({ commit, diff, loading }: DetailPanelProps) {
  if (!commit && !diff && !loading) {
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
      <div className="min-h-0 flex-1">
        <DiffViewer diff={diff} loading={loading} />
      </div>
    </div>
  );
}
