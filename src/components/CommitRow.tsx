import type { CommitDto } from "@/types";

interface CommitRowProps {
  commit: CommitDto;
  selected: boolean;
  isHead: boolean;
  onSelect: (commit: CommitDto) => void;
  showSpineBelow: boolean;
  showDot?: boolean;
  dotColor?: string;
  isMerge?: boolean;
  rowHeight?: number;
}

function formatRelativeTime(iso: string): string {
  const date = new Date(iso);
  const diffMs = Date.now() - date.getTime();
  const mins = Math.floor(diffMs / 60_000);
  if (mins < 1) return "agora";
  if (mins < 60) return `há ${mins} min`;
  const hours = Math.floor(mins / 60);
  if (hours < 24) return `há ${hours} h`;
  const days = Math.floor(hours / 24);
  if (days < 30) return `há ${days} dia${days > 1 ? "s" : ""}`;
  return date.toLocaleDateString("pt-BR");
}

export function CommitRow({
  commit,
  selected,
  isHead,
  onSelect,
  showSpineBelow,
  showDot = true,
  dotColor,
  isMerge,
  rowHeight = 56,
}: CommitRowProps) {
  const absTime = new Date(commit.authoredAt).toLocaleString("pt-BR");
  const relTime = formatRelativeTime(commit.authoredAt);

  return (
    <li style={{ minHeight: rowHeight }} className="relative flex items-center">
      {showSpineBelow && (
        <span
          className="absolute left-[11px] top-6 bottom-0 w-px bg-border"
          aria-hidden
        />
      )}
      <button
        type="button"
        onClick={() => onSelect(commit)}
        title={absTime}
        aria-selected={selected}
        className={`flex w-full items-center gap-2 rounded-lg px-2 py-1.5 text-left text-sm transition-colors focus:outline-none focus-visible:ring-2 focus-visible:ring-accent/30 ${
          selected
            ? "bg-surface ring-1 ring-border"
            : "hover:bg-surface/60"
        }`}
      >
        {showDot && (
          <span
            className={`h-2.5 w-2.5 shrink-0 rounded-full ${
              !dotColor && !selected ? "bg-border" : ""
            }`}
            style={
              dotColor || selected
                ? { backgroundColor: dotColor ?? "rgb(var(--accent))" }
                : undefined
            }
          />
        )}
        <div className="min-w-0 flex-1">
          <div className="flex items-center gap-1.5">
            <p className="truncate font-medium leading-snug text-text">
              {commit.summary}
            </p>
            {isHead && (
              <span className="shrink-0 rounded px-1 py-0.5 text-[10px] font-semibold uppercase tracking-wide text-accent">
                HEAD
              </span>
            )}
            {isMerge && (
              <span className="shrink-0 rounded bg-muted/25 px-1 py-0.5 text-[10px] font-semibold uppercase tracking-wide text-muted">
                merge
              </span>
            )}
            {commit.isLocalOnly && (
              <span className="shrink-0 rounded bg-amber-500/15 px-1 py-0.5 text-[10px] font-semibold uppercase tracking-wide text-amber-600 dark:text-amber-400">
                local
              </span>
            )}
          </div>
          <p className="mt-0.5 truncate text-xs text-muted">
            <span className="font-mono text-[11px]">{commit.shortId}</span>
            <span className="mx-1.5 opacity-40">·</span>
            {commit.authorName}
            <span className="mx-1.5 opacity-40">·</span>
            <span title={absTime}>{relTime}</span>
          </p>
        </div>
      </button>
    </li>
  );
}
