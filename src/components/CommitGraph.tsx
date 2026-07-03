import type { CommitDto } from "@/types";
import { GraphCanvas } from "./GraphCanvas";

interface CommitGraphProps {
  commits: CommitDto[];
  selectedId: string | null;
  onSelect: (commit: CommitDto) => void;
  onLoadMore?: () => void;
  hasMore?: boolean;
  loading?: boolean;
}

export function CommitGraph({
  commits,
  selectedId,
  onSelect,
  onLoadMore,
  hasMore,
  loading,
}: CommitGraphProps) {
  return (
    <div className="flex h-full flex-col">
      <div className="border-b border-border px-3 py-2 text-xs font-medium text-muted">
        Trilha de commits
      </div>
      <GraphCanvas
        commits={commits}
        selectedId={selectedId}
        headId={commits[0]?.id ?? null}
        onSelect={onSelect}
      />
      {hasMore && (
        <div className="border-t border-border p-2">
          <button
            type="button"
            onClick={onLoadMore}
            disabled={loading}
            className="w-full rounded py-1.5 text-xs text-accent hover:bg-surface disabled:opacity-50"
          >
            Carregar mais
          </button>
        </div>
      )}
    </div>
  );
}
