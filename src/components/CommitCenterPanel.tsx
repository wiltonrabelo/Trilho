import { useState } from "react";

import { AssistantChat } from "@/components/AssistantChat";
import { CommitSummaryPanel } from "@/components/CommitSummaryPanel";
import type {
  AssistantUiContextDto,
  CommitDto,
  WriteRequestDto,
} from "@/types";

type CenterTab = "detalhes" | "assistente";

interface CommitCenterPanelProps {
  commit: CommitDto | null;
  canUncommit?: boolean;
  canEditMessage?: boolean;
  messageEditHint?: string | null;
  onRevert?: () => void;
  revertBlockedReason?: string | null;
  revertInfoHint?: string | null;
  onReset?: () => void;
  resetHint?: string | null;
  onCherryPick?: () => void;
  cherryPickHint?: string | null;
  onUncommit?: () => void;
  onEditMessage?: () => void;
  onCreateTag?: () => void;
  onProposeWrite: (req: WriteRequestDto) => void;
  writeDisabled?: boolean;
  uiContext?: AssistantUiContextDto | null;
}

export function CommitCenterPanel(props: CommitCenterPanelProps) {
  const [tab, setTab] = useState<CenterTab>("detalhes");
  const {
    onProposeWrite,
    writeDisabled,
    uiContext,
    ...summaryProps
  } = props;

  return (
    <div className="flex h-full min-h-0 flex-col border-t border-border bg-surface/50">
      <div className="flex shrink-0 gap-0.5 border-b border-border px-2 py-1">
        {(["detalhes", "assistente"] as const).map((t) => (
          <button
            key={t}
            type="button"
            onClick={() => setTab(t)}
            className={`rounded px-2.5 py-0.5 text-[11px] font-medium ${
              tab === t
                ? "bg-accent text-white"
                : "text-muted hover:bg-surface hover:text-text"
            }`}
          >
            {t === "detalhes" ? "Detalhes" : "Assistente"}
          </button>
        ))}
      </div>
      <div className="min-h-0 flex-1 overflow-hidden">
        {tab === "detalhes" ? (
          <div className="h-full overflow-auto">
            <CommitSummaryPanel {...summaryProps} />
          </div>
        ) : (
          <AssistantChat
            onProposeWrite={onProposeWrite}
            writeDisabled={writeDisabled}
            uiContext={uiContext}
          />
        )}
      </div>
    </div>
  );
}
