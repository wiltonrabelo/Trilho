import { Minimize2 } from "lucide-react";
import { useEffect, useRef, useState } from "react";

import { BlamePanel } from "@/components/BlamePanel";
import { DiffViewer } from "@/components/DiffViewer";
import { useDialogA11y } from "@/hooks/useDialogA11y";
import type { BlameLineDto, BlameSourceDto, CommitDto } from "@/types";

type DetailTab = "diff" | "blame";

interface DiffOverlayProps {
  open: boolean;
  onClose: () => void;
  filePath: string | null;
  diff: string | null;
  loading?: boolean;
  commit: CommitDto | null;
  blameSource: BlameSourceDto;
  onBlameSourceChange: (source: BlameSourceDto) => void;
  blameLines: BlameLineDto[];
  blameFocusLine: number | null;
  blameLoading?: boolean;
  blameError?: string | null;
  onLineClick?: (lineNo: number) => void;
}

export function DiffOverlay({
  open,
  onClose,
  filePath,
  diff,
  loading,
  commit,
  blameSource,
  onBlameSourceChange,
  blameLines,
  blameFocusLine,
  blameLoading,
  blameError,
  onLineClick,
}: DiffOverlayProps) {
  const panelRef = useRef<HTMLDivElement>(null);
  const [detailTab, setDetailTab] = useState<DetailTab>("diff");
  useDialogA11y(open, onClose, panelRef);

  useEffect(() => {
    if (open) setDetailTab("diff");
  }, [open, filePath]);

  if (!open) return null;

  const showBlame = Boolean(filePath);
  const title = filePath ?? "Diff";

  return (
    <div
      className="fixed inset-0 z-50 flex flex-col bg-bg"
      role="dialog"
      aria-modal="true"
      aria-labelledby="diff-overlay-title"
    >
      <div ref={panelRef} className="flex h-full min-h-0 flex-col">
        <div className="flex shrink-0 items-center justify-between gap-3 border-b border-border bg-surface px-4 py-2">
          <h2
            id="diff-overlay-title"
            className="min-w-0 truncate font-mono text-xs font-medium"
            title={title}
          >
            {title}
          </h2>
          <div className="flex shrink-0 items-center">
            <button
              type="button"
              onClick={onClose}
              className="flex items-center gap-1.5 rounded-lg border border-border px-2.5 py-1 text-xs text-muted hover:bg-bg hover:text-text"
            >
              <Minimize2 size={14} />
              Restaurar
            </button>
          </div>
        </div>
        <div className="min-h-0 flex-1">
          {showBlame ? (
            <div className="flex h-full min-h-0 flex-col">
              <div className="flex shrink-0 gap-0.5 border-b border-border px-2 py-1">
                {(["diff", "blame"] as const).map((tab) => (
                  <button
                    key={tab}
                    type="button"
                    onClick={() => setDetailTab(tab)}
                    className={`rounded px-2.5 py-0.5 text-[11px] font-medium ${
                      detailTab === tab
                        ? "bg-accent text-white"
                        : "text-muted hover:bg-surface hover:text-text"
                    }`}
                  >
                    {tab === "diff" ? "Diff" : "Blame"}
                  </button>
                ))}
              </div>
              <div className="min-h-0 flex-1 overflow-hidden">
                {detailTab === "diff" ? (
                  <DiffViewer
                    diff={diff}
                    loading={loading}
                    onLineClick={filePath ? onLineClick : undefined}
                    selectedLine={blameFocusLine}
                  />
                ) : (
                  <BlamePanel
                    path={filePath}
                    source={blameSource}
                    onSourceChange={onBlameSourceChange}
                    lines={blameLines}
                    focusLine={blameFocusLine}
                    loading={blameLoading}
                    error={blameError}
                    showSourcePicker={!commit}
                    embedded
                  />
                )}
              </div>
            </div>
          ) : (
            <DiffViewer
              diff={diff}
              loading={loading}
              onLineClick={filePath ? onLineClick : undefined}
              selectedLine={blameFocusLine}
            />
          )}
        </div>
      </div>
    </div>
  );
}
