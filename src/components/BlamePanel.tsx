import { useCallback, useEffect, useMemo, useRef, useState } from "react";

import type { BlameLineDto, BlameSourceDto } from "@/types";

const SOURCE_LABEL: Record<BlameSourceDto, string> = {
  commit: "Commit",
  workingTree: "Working tree",
  staging: "Staging",
};

export function BlameSourcePicker({
  source,
  onSourceChange,
}: {
  source: BlameSourceDto;
  onSourceChange: (source: BlameSourceDto) => void;
}) {
  return (
    <div className="flex gap-1">
      {(Object.keys(SOURCE_LABEL) as BlameSourceDto[]).map((key) => (
        <button
          key={key}
          type="button"
          onClick={() => onSourceChange(key)}
          className={`rounded px-2 py-0.5 text-[10px] ${
            source === key
              ? "bg-accent/20 text-accent"
              : "text-muted hover:bg-surface"
          }`}
        >
          {SOURCE_LABEL[key]}
        </button>
      ))}
    </div>
  );
}

const BLAME_COL_STORAGE = "trilho-blame-col-widths-v1";

type BlameColKey = "line" | "commit" | "author" | "date";

const DEFAULT_COL_WIDTHS: Record<BlameColKey, number> = {
  line: 40,
  commit: 68,
  author: 132,
  date: 128,
};

const MIN_COL_WIDTHS: Record<BlameColKey, number> = {
  line: 32,
  commit: 52,
  author: 72,
  date: 96,
};

interface BlamePanelProps {
  path: string | null;
  source: BlameSourceDto;
  onSourceChange: (source: BlameSourceDto) => void;
  lines: BlameLineDto[];
  focusLine: number | null;
  loading?: boolean;
  error?: string | null;
  showSourcePicker?: boolean;
  /** Sem borda superior quando embutido em abas do DetailPanel. */
  embedded?: boolean;
  /** Modo destacado (overlay) — exibe data/hora e colunas redimensionáveis. */
  showAuthoredAt?: boolean;
  /** Branch em checkout (contexto do blame). */
  branchName?: string | null;
  /** Mensagem quando não há linhas (ex.: filtros sem resultado). */
  emptyHint?: string | null;
  /** Só no modo destacado — abre o diff do commit no mesmo arquivo. */
  onCommitClick?: (commitId: string) => void;
}

function formatAuthoredAt(iso: string): string {
  const date = new Date(iso);
  if (Number.isNaN(date.getTime())) return iso;
  return date.toLocaleString("pt-BR", {
    day: "2-digit",
    month: "2-digit",
    year: "numeric",
    hour: "2-digit",
    minute: "2-digit",
  });
}

function loadColWidths(): Record<BlameColKey, number> {
  try {
    const raw = localStorage.getItem(BLAME_COL_STORAGE);
    if (!raw) return { ...DEFAULT_COL_WIDTHS };
    const parsed = JSON.parse(raw) as Partial<Record<BlameColKey, number>>;
    return {
      line: clampCol("line", parsed.line ?? DEFAULT_COL_WIDTHS.line),
      commit: clampCol("commit", parsed.commit ?? DEFAULT_COL_WIDTHS.commit),
      author: clampCol("author", parsed.author ?? DEFAULT_COL_WIDTHS.author),
      date: clampCol("date", parsed.date ?? DEFAULT_COL_WIDTHS.date),
    };
  } catch {
    return { ...DEFAULT_COL_WIDTHS };
  }
}

function clampCol(key: BlameColKey, value: number): number {
  return Math.max(MIN_COL_WIDTHS[key], Math.round(value));
}

function gridTemplate(
  widths: Record<BlameColKey, number>,
  showAuthoredAt: boolean,
): string {
  const parts = [
    `${widths.line}px`,
    `${widths.commit}px`,
    `${widths.author}px`,
  ];
  if (showAuthoredAt) parts.push(`${widths.date}px`);
  parts.push("minmax(8rem, 1fr)");
  return parts.join(" ");
}

function isNavigableCommit(commitId: string): boolean {
  const hex = commitId.replace(/[^0-9a-f]/gi, "");
  return hex.length > 0 && !/^0+$/.test(hex);
}

function BlameResizableGrid({
  lines,
  focusLine,
  showAuthoredAt,
  onCommitClick,
}: {
  lines: BlameLineDto[];
  focusLine: number | null;
  showAuthoredAt: boolean;
  onCommitClick?: (commitId: string) => void;
}) {
  const [colWidths, setColWidths] = useState(loadColWidths);
  const resizeRef = useRef<{
    key: BlameColKey;
    startX: number;
    startWidth: number;
  } | null>(null);

  const template = useMemo(
    () => gridTemplate(colWidths, showAuthoredAt),
    [colWidths, showAuthoredAt],
  );

  const persistWidths = useCallback((next: Record<BlameColKey, number>) => {
    try {
      localStorage.setItem(BLAME_COL_STORAGE, JSON.stringify(next));
    } catch {
      /* quota / private mode */
    }
  }, []);

  useEffect(() => {
    const onMove = (event: PointerEvent) => {
      const active = resizeRef.current;
      if (!active) return;
      const delta = event.clientX - active.startX;
      setColWidths((prev) => {
        const next = {
          ...prev,
          [active.key]: clampCol(
            active.key,
            active.startWidth + delta,
          ),
        };
        return next;
      });
    };

    const onUp = () => {
      if (!resizeRef.current) return;
      resizeRef.current = null;
      document.body.style.cursor = "";
      document.body.style.userSelect = "";
      setColWidths((prev) => {
        persistWidths(prev);
        return prev;
      });
    };

    window.addEventListener("pointermove", onMove);
    window.addEventListener("pointerup", onUp);
    return () => {
      window.removeEventListener("pointermove", onMove);
      window.removeEventListener("pointerup", onUp);
    };
  }, [persistWidths]);

  const startResize = (key: BlameColKey, event: React.PointerEvent) => {
    event.preventDefault();
    resizeRef.current = {
      key,
      startX: event.clientX,
      startWidth: colWidths[key],
    };
    document.body.style.cursor = "col-resize";
    document.body.style.userSelect = "none";
  };

  const headerCells: { key: BlameColKey | "content"; label: string }[] = [
    { key: "line", label: "Linha" },
    { key: "commit", label: "Commit" },
    { key: "author", label: "Autor" },
  ];
  if (showAuthoredAt) headerCells.push({ key: "date", label: "Data" });
  headerCells.push({ key: "content", label: "Conteúdo" });

  return (
    <div className="min-h-0 flex-1 overflow-auto font-mono text-xs">
      <div className="min-w-full">
        <div
          className="sticky top-0 z-[1] grid border-b border-border bg-surface/95 text-[10px] font-medium text-muted backdrop-blur-sm"
          style={{ gridTemplateColumns: template }}
        >
          {headerCells.map((cell) => (
            <div
              key={cell.key}
              className="relative truncate border-r border-border px-2 py-1 last:border-r-0"
            >
              {cell.label}
              {cell.key !== "content" && (
                <div
                  role="separator"
                  aria-orientation="vertical"
                  aria-label={`Redimensionar coluna ${cell.label}`}
                  className="absolute top-0 right-0 z-[2] h-full w-1.5 cursor-col-resize touch-none hover:bg-accent/40 active:bg-accent/60"
                  onPointerDown={(event) =>
                    startResize(cell.key as BlameColKey, event)
                  }
                />
              )}
            </div>
          ))}
        </div>

        {lines.map((row) => (
          <div
            key={`${row.line}-${row.commitId}`}
            className={`grid border-b border-border/40 ${
              focusLine === row.line
                ? "bg-accent/10"
                : "hover:bg-surface/50"
            }`}
            style={{ gridTemplateColumns: template }}
          >
            <div className="truncate border-r border-border px-2 py-0.5 text-right text-muted">
              {row.line}
            </div>
            <div className="truncate border-r border-border px-2 py-0.5 text-accent">
              {onCommitClick && isNavigableCommit(row.commitId) ? (
                <button
                  type="button"
                  onClick={() => onCommitClick(row.commitId)}
                  className="max-w-full truncate text-left text-accent hover:underline"
                  title="Ver diff deste commit"
                >
                  {row.shortId}
                </button>
              ) : (
                row.shortId
              )}
            </div>
            <div className="truncate border-r border-border px-2 py-0.5 text-muted">
              {row.author}
            </div>
            {showAuthoredAt && (
              <div className="truncate border-r border-border px-2 py-0.5 text-[11px] tabular-nums text-muted">
                {formatAuthoredAt(row.authoredAt)}
              </div>
            )}
            <div className="min-w-0 px-2 py-0.5 whitespace-pre-wrap break-all">
              {row.content || row.summary}
            </div>
          </div>
        ))}
      </div>
    </div>
  );
}

export function BlamePanel({
  path,
  source,
  onSourceChange,
  lines,
  focusLine,
  loading,
  error,
  showSourcePicker = true,
  embedded = false,
  showAuthoredAt = false,
  branchName,
  emptyHint,
  onCommitClick,
}: BlamePanelProps) {
  if (!path && lines.length === 0 && !loading) {
    return null;
  }

  return (
    <div
      className={`flex h-full flex-col ${embedded ? "" : "border-t border-border"}`}
    >
      <div className="flex items-center justify-between gap-2 border-b border-border px-3 py-2">
        <span className="truncate text-xs font-medium">
          Blame {path ? `· ${path}` : ""}
          {branchName ? (
            <span className="ml-1 font-normal text-muted">· {branchName}</span>
          ) : null}
        </span>
        {showSourcePicker && (
          <BlameSourcePicker source={source} onSourceChange={onSourceChange} />
        )}
      </div>

      {loading && (
        <div className="p-3 text-xs text-muted">Carregando blame…</div>
      )}
      {error && (
        <div className="p-3 text-xs text-red-500">{error}</div>
      )}
      {!loading && !error && lines.length === 0 && path && (
        <div className="p-3 text-xs text-muted">
          {emptyHint ?? "Arquivo vazio nesta versão — sem linhas para blame."}
        </div>
      )}
      {!loading && !error && lines.length > 0 && showAuthoredAt && (
        <BlameResizableGrid
          lines={lines}
          focusLine={focusLine}
          showAuthoredAt
          onCommitClick={onCommitClick}
        />
      )}
      {!loading && !error && lines.length > 0 && !showAuthoredAt && (
        <div className="min-h-0 flex-1 overflow-auto font-mono text-xs">
          <table className="w-max min-w-full border-collapse">
            <tbody>
              {lines.map((row) => (
                <tr
                  key={`${row.line}-${row.commitId}`}
                  className={
                    focusLine === row.line
                      ? "bg-accent/10"
                      : "hover:bg-surface/50"
                  }
                >
                  <td className="w-8 select-none border-r border-border px-2 py-0.5 text-right text-muted">
                    {row.line}
                  </td>
                  <td className="w-14 select-none border-r border-border px-2 py-0.5 text-accent">
                    {row.shortId}
                  </td>
                  <td className="border-r border-border px-2 py-0.5 text-muted">
                    {row.author}
                  </td>
                  <td className="px-2 py-0.5 whitespace-pre-wrap break-all">
                    {row.content || row.summary}
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      )}
    </div>
  );
}
