import type { CommitDto } from "@/types";

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
      <ol className="flex-1 overflow-auto p-2">
        {commits.map((c, i) => (
          <li key={c.id} className="relative flex">
            {i < commits.length - 1 && (
              <span
                className="absolute left-[11px] top-6 bottom-0 w-px bg-border"
                aria-hidden
              />
            )}
            <button
              type="button"
              onClick={() => onSelect(c)}
              className={`mb-1 flex w-full items-start gap-2 rounded-lg px-2 py-2 text-left text-sm transition-colors ${
                selectedId === c.id
                  ? "bg-accent/15 ring-1 ring-accent/40"
                  : "hover:bg-surface"
              }`}
            >
              <span
                className={`mt-1.5 h-2.5 w-2.5 shrink-0 rounded-full ${
                  selectedId === c.id ? "bg-accent" : "bg-border"
                }`}
              />
              <div className="min-w-0 flex-1">
                <div className="flex items-center gap-2">
                  <p className="truncate font-medium">{c.summary}</p>
                  {c.isLocalOnly && (
                    <span className="shrink-0 rounded bg-accent/15 px-1 py-0.5 text-[10px] font-semibold uppercase text-accent">
                      local
                    </span>
                  )}
                </div>
                <p className="mt-0.5 text-xs text-muted">
                  <span className="font-mono">{c.shortId}</span>
                  {" · "}
                  {c.authorName}
                  {" · "}
                  {new Date(c.authoredAt).toLocaleString("pt-BR")}
                </p>
              </div>
            </button>
          </li>
        ))}
      </ol>
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
