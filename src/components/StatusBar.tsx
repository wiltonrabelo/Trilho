import { GitBranch } from "lucide-react";

import type { SyncInfoDto } from "@/types";

interface StatusBarProps {
  branch: string | null;
  isDetached?: boolean;
  repoPath: string;
  sync: SyncInfoDto | null;
  changeCount: number;
  upstreamConfigured?: boolean;
}

export function StatusBar({
  branch,
  isDetached,
  repoPath,
  sync,
  changeCount,
  upstreamConfigured,
}: StatusBarProps) {
  const syncLabel =
    upstreamConfigured && sync
      ? sync.ahead > 0 || sync.behind > 0
        ? `${sync.ahead}↑ ${sync.behind}↓`
        : "em dia"
      : upstreamConfigured
        ? "—"
        : "sem upstream";

  return (
    <footer
      className="flex shrink-0 items-center justify-between gap-4 border-t border-border bg-surface px-3 py-1 text-xs text-muted"
      aria-label="Barra de status"
    >
      <div className="flex min-w-0 flex-wrap items-center gap-x-3 gap-y-0.5">
        <span className="inline-flex items-center gap-1">
          <GitBranch size={12} className="shrink-0 opacity-70" />
          {isDetached ? (
            <span className="text-amber-600 dark:text-amber-400">detached HEAD</span>
          ) : (
            <span className="font-medium text-text">{branch ?? "—"}</span>
          )}
        </span>
        {upstreamConfigured && (
          <span title="Ahead / behind em relação ao remoto">{syncLabel}</span>
        )}
        <span>
          {changeCount === 0
            ? "Working tree limpa"
            : `${changeCount} alteração${changeCount === 1 ? "" : "ões"}`}
        </span>
      </div>
      <span
        className="min-w-0 truncate font-mono text-[11px]"
        title={repoPath}
      >
        {repoPath}
      </span>
    </footer>
  );
}
