import type { CommitDto } from "@/types";

interface CommitSummaryPanelProps {
  commit: CommitDto | null;
  canUncommit?: boolean;
  canEditMessage?: boolean;
  messageEditHint?: string | null;
  onRevert?: () => void;
  onUncommit?: () => void;
  onEditMessage?: () => void;
  onCreateTag?: () => void;
}

export function CommitSummaryPanel({
  commit,
  canUncommit,
  canEditMessage,
  messageEditHint,
  onRevert,
  onUncommit,
  onEditMessage,
  onCreateTag,
}: CommitSummaryPanelProps) {
  if (!commit) {
    return (
      <div className="flex h-full items-center justify-center px-4 text-xs text-muted">
        Selecione um commit para ver a mensagem
      </div>
    );
  }

  const showActions =
    onRevert ||
    onCreateTag ||
    (canUncommit && onUncommit) ||
    (canEditMessage && onEditMessage);

  return (
    <div className="flex h-full min-h-0 flex-col overflow-auto border-t border-border px-4 py-3">
      {showActions && (
        <div className="mb-2 flex flex-wrap gap-2">
          {onRevert && (
            <button
              type="button"
              onClick={onRevert}
              className="rounded border border-border px-2 py-0.5 text-[10px] text-muted hover:bg-surface hover:text-text"
            >
              Reverter commit
            </button>
          )}
          {onCreateTag && (
            <button
              type="button"
              onClick={onCreateTag}
              className="rounded border border-border px-2 py-0.5 text-[10px] text-muted hover:bg-surface hover:text-text"
            >
              Criar tag…
            </button>
          )}
          {canEditMessage && onEditMessage && (
            <button
              type="button"
              onClick={onEditMessage}
              className="rounded border border-accent/40 bg-accent/10 px-2 py-0.5 text-[10px] text-accent hover:bg-accent/20"
            >
              Editar mensagem
            </button>
          )}
          {canUncommit && onUncommit && (
            <button
              type="button"
              onClick={onUncommit}
              className="rounded border border-border px-2 py-0.5 text-[10px] text-muted hover:bg-surface hover:text-text"
            >
              Uncommit (soft)
            </button>
          )}
        </div>
      )}
      <h2 className="text-sm font-semibold">{commit.summary}</h2>
      <p className="mt-1 text-xs text-muted">
        <span className="font-mono">{commit.id}</span>
        {" · "}
        {commit.authorName}
        {" · "}
        {new Date(commit.authoredAt).toLocaleString("pt-BR")}
      </p>
      {commit.body && (
        <p className="mt-2 whitespace-pre-wrap text-xs text-text">{commit.body}</p>
      )}
      {messageEditHint && (
        <p className="mt-2 text-[10px] leading-snug text-muted">{messageEditHint}</p>
      )}
    </div>
  );
}
