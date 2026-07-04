import { useMemo } from "react";
import { parseUnifiedDiff } from "@/lib/diff-parse";

interface DiffViewerProps {
  diff: string | null;
  loading?: boolean;
  onLineClick?: (lineNo: number) => void;
  selectedLine?: number | null;
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

export function DiffViewer({
  diff,
  loading,
  onLineClick,
  selectedLine,
}: DiffViewerProps) {
  const parsed = useMemo(() => (diff ? parseUnifiedDiff(diff) : null), [diff]);

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
