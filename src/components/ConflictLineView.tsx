import {
  conflictLineClass,
  highlightAgainstBase,
  splitConflictLines,
  type ConflictLineKind,
} from "@/lib/conflict-highlight";

interface ConflictLineViewProps {
  label: string;
  content: string;
  /** Com base disponível, destaca linhas alteradas (estilo diff). */
  base?: string;
  side?: "ours" | "theirs";
  /** Sem diff contra base — numeração + hover apenas. */
  plain?: boolean;
}

/** Comparação lado a lado (estilo DiffViewer) — atual vs entrando. */
export function ConflictSideBySideView({
  ours,
  theirs,
  base,
}: {
  ours: string;
  theirs: string;
  base: string;
}) {
  const oursRows = highlightAgainstBase(base, ours);
  const theirsRows = highlightAgainstBase(base, theirs);
  const lineCount = Math.max(oursRows.length, theirsRows.length, 1);

  return (
    <div className="overflow-hidden rounded border border-amber-500/40 bg-bg ring-1 ring-amber-500/20">
      <table className="w-full border-collapse font-mono text-xs">
        <thead>
          <tr className="border-b border-border bg-surface text-[10px] uppercase tracking-wide text-muted">
            <th className="w-10 border-r border-border px-2 py-1.5 text-right font-medium">
              #
            </th>
            <th className="w-[calc(50%-2.5rem)] border-r border-border px-3 py-1.5 text-left font-medium">
              Atual (ours)
            </th>
            <th className="w-10 border-r border-border px-2 py-1.5 text-right font-medium">
              #
            </th>
            <th className="px-3 py-1.5 text-left font-medium">Entrando (theirs)</th>
          </tr>
        </thead>
        <tbody>
          {Array.from({ length: lineCount }, (_, i) => {
            const left = oursRows[i];
            const right = theirsRows[i];
            return (
              <tr key={i} className="border-b border-border/50">
                <td className="w-10 select-none border-r border-border px-2 py-0.5 text-right text-muted">
                  {left?.lineNo ?? i + 1}
                </td>
                <td
                  className={`w-[calc(50%-2.5rem)] border-r border-border px-2 py-0.5 whitespace-pre-wrap break-all ${
                    left ? conflictLineClass(left.kind, "ours") : ""
                  }`}
                >
                  {left?.text || "\u00a0"}
                </td>
                <td className="w-10 select-none border-r border-border px-2 py-0.5 text-right text-muted">
                  {right?.lineNo ?? i + 1}
                </td>
                <td
                  className={`px-2 py-0.5 whitespace-pre-wrap break-all ${
                    right ? conflictLineClass(right.kind, "theirs") : ""
                  }`}
                >
                  {right?.text || "\u00a0"}
                </td>
              </tr>
            );
          })}
        </tbody>
      </table>
    </div>
  );
}

function plainRows(content: string): { lineNo: number; text: string; kind: ConflictLineKind }[] {
  return splitConflictLines(content).map((text, i) => ({
    lineNo: i + 1,
    text,
    kind: "context" as const,
  }));
}

export function ConflictLineView({
  label,
  content,
  base,
  side = "ours",
  plain = false,
}: ConflictLineViewProps) {
  const rows =
    plain || !base
      ? plainRows(content)
      : highlightAgainstBase(base, content);

  if (rows.length === 0) {
    return (
      <div className="overflow-hidden rounded border border-border bg-bg">
        <div className="border-b border-border bg-surface px-3 py-1.5 font-mono text-xs text-muted">
          {label}
        </div>
        <p className="px-3 py-2 text-xs text-muted">—</p>
      </div>
    );
  }

  return (
    <div className="overflow-hidden rounded border border-border bg-bg">
      <div className="border-b border-border bg-surface px-3 py-1.5 font-mono text-xs text-muted">
        {label}
      </div>
      <table className="w-full border-collapse font-mono text-xs">
        <tbody>
          {rows.map((row) => (
            <tr
              key={row.lineNo}
              className={
                plain || !base
                  ? "hover:bg-surface/50"
                  : conflictLineClass(row.kind, side)
              }
            >
              <td className="w-10 select-none border-r border-border px-2 py-0.5 text-right text-muted">
                {row.lineNo}
              </td>
              <td className="px-2 py-0.5 whitespace-pre-wrap break-all">
                {row.text || "\u00a0"}
              </td>
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  );
}
