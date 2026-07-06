import type { GraphView } from "@/hooks/useCommits";
import type { CommitDto, TrailKindDto } from "@/types";
import { GraphCanvas, type TrailDivergence } from "./GraphCanvas";

interface CommitGraphProps {
  commits: CommitDto[];
  selectedId: string | null;
  view: GraphView;
  onViewChange: (view: GraphView) => void;
  trails?: TrailKindDto[] | null;
  divergence?: TrailDivergence | null;
  workingCopySelected?: boolean;
  changeCount?: number;
  stagedCount?: number;
  onSelectWorkingCopy?: () => void;
  onSelect: (commit: CommitDto) => void;
  onLoadMore?: () => void;
  hasMore?: boolean;
  loading?: boolean;
}

const VIEWS: { value: GraphView; label: string; hint: string }[] = [
  {
    value: "trail",
    label: "Trilha da branch",
    hint: "Só a linha da branch atual (--first-parent); merges colapsados",
  },
  {
    value: "graph",
    label: "Grafo completo",
    hint: "Todas as linhas de desenvolvimento com lanes",
  },
];

export function CommitGraph({
  commits,
  selectedId,
  view,
  onViewChange,
  trails,
  divergence,
  workingCopySelected,
  changeCount,
  stagedCount,
  onSelectWorkingCopy,
  onSelect,
  onLoadMore,
  hasMore,
  loading,
}: CommitGraphProps) {
  return (
    <div className="flex h-full flex-col">
      <div className="flex items-center justify-between border-b border-border px-3 py-1.5">
        <span className="text-xs font-medium text-muted">Trilha de commits</span>
        <div className="inline-flex items-center gap-0.5 rounded-md border border-border p-0.5" role="group" aria-label="Visão do grafo">
          {VIEWS.map(({ value, label, hint }) => (
            <button
              key={value}
              type="button"
              title={hint}
              aria-label={label}
              aria-pressed={view === value}
              onClick={() => onViewChange(value)}
              className={`rounded px-2 py-0.5 text-[11px] font-medium transition-colors ${
                view === value
                  ? "bg-accent text-white"
                  : "text-muted hover:bg-surface hover:text-text"
              }`}
            >
              {label}
            </button>
          ))}
        </div>
      </div>
      <GraphCanvas
        commits={commits}
        selectedId={selectedId}
        headId={commits[0]?.id ?? null}
        linear={view === "trail"}
        trails={view === "trail" ? trails : null}
        divergence={divergence}
        compact={view === "graph"}
        showWorkingCopy
        workingCopySelected={workingCopySelected}
        changeCount={changeCount}
        stagedCount={stagedCount}
        onSelectWorkingCopy={onSelectWorkingCopy}
        onSelect={onSelect}
      />
      {hasMore && (
        <div className="border-t border-border p-2">
          <button
            type="button"
            onClick={onLoadMore}
            disabled={loading}
            aria-busy={loading}
            aria-label="Carregar mais commits"
            className="w-full rounded py-1.5 text-xs text-accent hover:bg-surface disabled:opacity-50"
          >
            {loading ? "Carregando…" : "Carregar mais"}
          </button>
        </div>
      )}
    </div>
  );
}
