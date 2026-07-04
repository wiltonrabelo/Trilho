import type { BlameLineDto, BlameSourceDto } from "@/types";

const SOURCE_LABEL: Record<BlameSourceDto, string> = {
  commit: "Commit",
  workingTree: "Working tree",
  staging: "Staging",
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
}: BlamePanelProps) {
  if (!path && lines.length === 0 && !loading) {
    return null;
  }

  return (
    <div className="flex h-full flex-col border-t border-border">
      <div className="flex items-center justify-between gap-2 border-b border-border px-3 py-2">
        <span className="truncate text-xs font-medium">
          Blame {path ? `· ${path}` : ""}
        </span>
        {showSourcePicker && (
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
        )}
      </div>

      {loading && (
        <div className="p-3 text-xs text-muted">Carregando blame…</div>
      )}
      {error && (
        <div className="p-3 text-xs text-red-500">{error}</div>
      )}
      {!loading && !error && (
        <div className="min-h-0 flex-1 overflow-auto font-mono text-xs">
          <table className="w-full border-collapse">
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
