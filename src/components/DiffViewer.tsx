import { useMemo } from "react";
import { parseUnifiedDiff, type DiffRow } from "@/lib/diff-parse";
import type { DiffHunk } from "@/lib/diff-hunks";

export type DiffLayout = "sideBySide" | "unified";

interface DiffViewerProps {
  diff: string | null;
  loading?: boolean;
  layout?: DiffLayout;
  onLineClick?: (lineNo: number) => void;
  selectedLine?: number | null;
  /** Trechos para reverter (RF-18) — força layout unified com ações por hunk. */
  hunks?: DiffHunk[];
  onDiscardHunk?: (patch: string) => void;
}

function lineClass(kind: string): string {
  switch (kind) {
    case "add":
      return "bg-emerald-500/15";
    case "remove":
      return "bg-red-500/15";
    default:
      return "";
  }
}

function flattenUnifiedRows(rows: DiffRow[]): {
  oldLine: number | undefined;
  newLine: number | undefined;
  text: string;
  kind: string;
  prefix: string;
}[] {
  const out: {
    oldLine: number | undefined;
    newLine: number | undefined;
    text: string;
    kind: string;
    prefix: string;
  }[] = [];
  for (const row of rows) {
    if (row.left.kind === "remove") {
      out.push({
        oldLine: row.left.lineNo,
        newLine: undefined,
        text: row.left.text,
        kind: "remove",
        prefix: "-",
      });
    }
    if (row.right.kind === "add") {
      out.push({
        oldLine: undefined,
        newLine: row.right.lineNo,
        text: row.right.text,
        kind: "add",
        prefix: "+",
      });
    }
    if (row.left.kind === "context" && row.right.kind === "context") {
      out.push({
        oldLine: row.left.lineNo,
        newLine: row.right.lineNo,
        text: row.left.text,
        kind: "context",
        prefix: " ",
      });
    }
  }
  return out;
}

const HUNK_HEADER_RE = /^@@ -(\d+)(?:,\d+)? \+(\d+)(?:,\d+)? @@/;

type HunkViewRow =
  | { type: "header"; text: string }
  | {
      type: "line";
      prefix: string;
      kind: string;
      text: string;
      oldLine: number | null;
      newLine: number | null;
    };

function buildHunkViewRows(patch: string): HunkViewRow[] {
  const lines = patch.split("\n");
  const start = lines.findIndex((l) => l.startsWith("@@"));
  const body = start >= 0 ? lines.slice(start) : lines;

  const rows: HunkViewRow[] = [];
  let oldLine = 0;
  let newLine = 0;

  for (const line of body) {
    if (line.startsWith("@@")) {
      const m = line.match(HUNK_HEADER_RE);
      if (m) {
        oldLine = Number.parseInt(m[1], 10);
        newLine = Number.parseInt(m[2], 10);
      }
      rows.push({ type: "header", text: line });
      continue;
    }
    if (line.startsWith("\\ No newline") || line.startsWith("diff --git") ||
        line.startsWith("--- ") || line.startsWith("+++ ") ||
        line.startsWith("index ")) {
      continue;
    }
    if (!line && rows.length === 0) continue;

    const prefix = line[0] ?? " ";
    const text = line.slice(1);
    if (prefix === "-") {
      rows.push({
        type: "line",
        prefix,
        kind: "remove",
        text,
        oldLine,
        newLine: null,
      });
      oldLine += 1;
    } else if (prefix === "+") {
      rows.push({
        type: "line",
        prefix,
        kind: "add",
        text,
        oldLine: null,
        newLine,
      });
      newLine += 1;
    } else {
      rows.push({
        type: "line",
        prefix: " ",
        kind: "context",
        text: prefix === " " ? text : line,
        oldLine,
        newLine,
      });
      oldLine += 1;
      newLine += 1;
    }
  }
  return rows;
}

function HunkBlock({
  hunk,
  onDiscardHunk,
}: {
  hunk: DiffHunk;
  onDiscardHunk?: (patch: string) => void;
}) {
  const rows = useMemo(() => buildHunkViewRows(hunk.patch), [hunk.patch]);

  return (
    <div className="border-b border-border">
      <div className="sticky top-0 z-[1] flex items-center justify-between gap-2 border-b border-border bg-surface/95 px-3 py-1.5 backdrop-blur-sm">
        <span className="min-w-0 truncate font-mono text-[10px] text-muted">
          {hunk.header}
        </span>
        {onDiscardHunk && (
          <button
            type="button"
            onClick={() => onDiscardHunk(hunk.patch)}
            className="shrink-0 rounded border border-red-500/40 px-2 py-0.5 text-[10px] font-medium text-red-600 hover:bg-red-500/10 dark:text-red-400"
            title="Desfaz só este trecho (git apply --reverse)"
          >
            Reverter trecho
          </button>
        )}
      </div>
      <table className="w-full border-collapse font-mono text-xs">
        <tbody>
          {rows.map((row, i) => {
            if (row.type === "header") {
              return (
                <tr key={`h${i}`} className="bg-muted/10">
                  <td colSpan={4} className="px-2 py-0.5 text-[10px] text-muted">
                    {row.text}
                  </td>
                </tr>
              );
            }
            return (
              <tr key={`l${i}`} className="hover:bg-surface/50">
                <td className="w-10 select-none border-r border-border px-1 py-0.5 text-right tabular-nums text-muted">
                  {row.oldLine ?? ""}
                </td>
                <td className="w-10 select-none border-r border-border px-1 py-0.5 text-right tabular-nums text-muted">
                  {row.newLine ?? ""}
                </td>
                <td className="w-6 select-none border-r border-border px-1 py-0.5 text-center text-muted">
                  {row.prefix}
                </td>
                <td
                  className={`px-2 py-0.5 whitespace-pre-wrap break-all ${lineClass(row.kind)}`}
                >
                  {row.text || "\u00a0"}
                </td>
              </tr>
            );
          })}
        </tbody>
      </table>
    </div>
  );
}

export function DiffViewer({
  diff,
  loading,
  layout = "sideBySide",
  onLineClick,
  selectedLine,
  hunks,
  onDiscardHunk,
}: DiffViewerProps) {
  const parsed = useMemo(() => (diff ? parseUnifiedDiff(diff) : null), [diff]);
  const useHunkView = Boolean(hunks?.length && onDiscardHunk);

  if (loading) {
    return (
      <div className="flex h-full items-center justify-center text-sm text-muted">
        Carregando diff…
      </div>
    );
  }

  if (!parsed) {
    return (
      <div className="flex h-full items-center justify-center text-sm text-muted">
        Sem diff
      </div>
    );
  }

  if (parsed.rawFallback) {
    return (
      <pre className="h-full overflow-auto p-4 font-mono text-xs whitespace-pre-wrap">
        {parsed.rawFallback}
      </pre>
    );
  }

  if (useHunkView && hunks) {
    return (
      <div className="h-full overflow-auto">
        {hunks.map((hunk) => (
          <HunkBlock
            key={hunk.index}
            hunk={hunk}
            onDiscardHunk={onDiscardHunk}
          />
        ))}
      </div>
    );
  }

  if (layout === "unified") {
    return (
      <div className="h-full overflow-auto">
        {parsed.files.map((file) => (
          <div
            key={`${file.oldPath}-${file.newPath}`}
            className="border-b border-border"
          >
            <div className="sticky top-0 border-b border-border bg-surface px-3 py-1.5 font-mono text-xs text-muted">
              {file.oldPath !== file.newPath
                ? `${file.oldPath} → ${file.newPath}`
                : file.newPath}
            </div>
            <table className="w-full border-collapse font-mono text-xs">
              <tbody>
                {flattenUnifiedRows(file.rows).map((row, i) => {
                  const clickLine = row.newLine ?? row.oldLine;
                  const isSelected =
                    selectedLine != null && clickLine === selectedLine;
                  return (
                    <tr
                      key={i}
                      className={`hover:bg-surface/50 ${isSelected ? "ring-1 ring-inset ring-accent/40" : ""} ${onLineClick && clickLine ? "cursor-pointer" : ""}`}
                      onClick={() => {
                        if (onLineClick && clickLine) onLineClick(clickLine);
                      }}
                    >
                      <td className="w-10 select-none border-r border-border px-1 py-0.5 text-right tabular-nums text-muted">
                        {row.oldLine ?? ""}
                      </td>
                      <td className="w-10 select-none border-r border-border px-1 py-0.5 text-right tabular-nums text-muted">
                        {row.newLine ?? ""}
                      </td>
                      <td className="w-6 select-none border-r border-border px-1 py-0.5 text-center text-muted">
                        {row.prefix}
                      </td>
                      <td
                        className={`px-2 py-0.5 whitespace-pre-wrap break-all ${lineClass(row.kind)}`}
                      >
                        {row.text || "\u00a0"}
                      </td>
                    </tr>
                  );
                })}
              </tbody>
            </table>
          </div>
        ))}
      </div>
    );
  }

  return (
    <div className="h-full overflow-auto">
      {parsed.files.map((file) => (
        <div key={`${file.oldPath}-${file.newPath}`} className="border-b border-border">
          <div className="sticky top-0 border-b border-border bg-surface px-3 py-1.5 font-mono text-xs text-muted">
            {file.oldPath !== file.newPath
              ? `${file.oldPath} → ${file.newPath}`
              : file.newPath}
          </div>
          <table className="w-full border-collapse font-mono text-xs">
            <tbody>
              {file.rows.map((row, i) => {
                const rightLine = row.right.lineNo;
                const isSelected =
                  selectedLine != null && rightLine === selectedLine;
                return (
                  <tr
                    key={i}
                    className={`hover:bg-surface/50 ${isSelected ? "ring-1 ring-inset ring-accent/40" : ""} ${onLineClick && rightLine ? "cursor-pointer" : ""}`}
                    onClick={() => {
                      if (onLineClick && rightLine) onLineClick(rightLine);
                    }}
                  >
                    <td className="w-10 select-none border-r border-border px-2 py-0.5 text-right text-muted">
                      {row.left.lineNo ?? ""}
                    </td>
                    <td
                      className={`w-[calc(50%-2.5rem)] border-r border-border px-2 py-0.5 whitespace-pre-wrap break-all ${lineClass(row.left.kind)}`}
                    >
                      {row.left.text || "\u00a0"}
                    </td>
                    <td className="w-10 select-none border-r border-border px-2 py-0.5 text-right text-muted">
                      {row.right.lineNo ?? ""}
                    </td>
                    <td
                      className={`w-[calc(50%-2.5rem)] px-2 py-0.5 whitespace-pre-wrap break-all ${lineClass(row.right.kind)}`}
                    >
                      {row.right.text || "\u00a0"}
                    </td>
                  </tr>
                );
              })}
            </tbody>
          </table>
        </div>
      ))}
    </div>
  );
}
