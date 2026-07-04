interface WorkingCopyRowProps {
  changeCount: number;
  stagedCount: number;
  selected: boolean;
  onSelect: () => void;
  rowHeight: number;
  compact?: boolean;
}

/** Nó virtual no topo da trilha — mudanças ainda não commitadas (estilo SourceTree). */
export function WorkingCopyRow({
  changeCount,
  stagedCount,
  selected,
  onSelect,
  rowHeight,
  compact = false,
}: WorkingCopyRowProps) {
  const subtitle =
    changeCount === 0
      ? "Working tree limpa"
      : stagedCount > 0
        ? `${changeCount} arquivos · ${stagedCount} staged`
        : `${changeCount} arquivos`;

  if (compact) {
    return (
      <li
        style={{ height: rowHeight }}
        className="relative flex items-center overflow-hidden"
      >
        <button
          type="button"
          onClick={onSelect}
          title="Alterações locais — staged, unstaged e untracked"
          aria-selected={selected}
          className={`flex w-full items-center gap-2 rounded px-2 py-0.5 text-left text-xs transition-colors focus:outline-none focus-visible:ring-2 focus-visible:ring-accent/30 ${
            selected ? "bg-surface ring-1 ring-border" : "hover:bg-surface/60"
          }`}
        >
          <span className="min-w-0 flex-1 truncate">
            <span className="font-medium text-text">Alterações locais</span>
            <span className="ml-2 text-muted">{subtitle}</span>
          </span>
          {changeCount > 0 && (
            <span className="shrink-0 rounded-full border border-accent/40 bg-accent/10 px-1.5 py-0.5 text-[10px] tabular-nums text-accent">
              {changeCount}
            </span>
          )}
        </button>
      </li>
    );
  }

  return (
    <li style={{ height: rowHeight }} className="relative flex items-center overflow-hidden">
      <button
        type="button"
        onClick={onSelect}
        title="Alterações locais — staged, unstaged e untracked"
        aria-selected={selected}
        className={`flex w-full items-center gap-2 rounded-lg px-2 py-1.5 text-left text-sm transition-colors focus:outline-none focus-visible:ring-2 focus-visible:ring-accent/30 ${
          selected ? "bg-surface ring-1 ring-border" : "hover:bg-surface/60"
        }`}
      >
        <div className="min-w-0 flex-1">
          <p className="truncate font-medium leading-snug text-text">
            Alterações locais
          </p>
          <p className="mt-0.5 truncate text-xs text-muted">{subtitle}</p>
        </div>
        {changeCount > 0 && (
          <span className="shrink-0 rounded-full border border-accent/40 bg-accent/10 px-2 py-0.5 text-[10px] tabular-nums text-accent">
            {changeCount}
          </span>
        )}
      </button>
    </li>
  );
}
