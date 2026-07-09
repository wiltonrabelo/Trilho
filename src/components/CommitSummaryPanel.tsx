import type { CommitDto } from "@/types";

interface CommitSummaryPanelProps {
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
}

export function CommitSummaryPanel({
  commit,
  canUncommit,
  canEditMessage,
  messageEditHint,
  onRevert,
  revertBlockedReason,
  revertInfoHint,
  onReset,
  resetHint,
  onCherryPick,
  cherryPickHint,
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
    onReset ||
    onCherryPick ||
    onCreateTag ||
    (canUncommit && onUncommit) ||
    (canEditMessage && onEditMessage);

  return (
    <div className="flex h-full min-h-0 flex-col overflow-auto border-t border-border bg-surface/50 px-4 py-3">
      {revertBlockedReason && (
        <p className="mb-2 rounded-md border border-amber-500/40 bg-amber-500/10 px-3 py-2 text-[11px] leading-snug text-amber-800 dark:text-amber-200">
          {revertBlockedReason}
        </p>
      )}
      {showActions && (
        <div className="mb-2 flex flex-wrap gap-2">
          {onRevert && (
            <button type="button" onClick={onRevert} className="btn-toolbar">
              Reverter commit
            </button>
          )}
          {onReset && (
            <button type="button" onClick={onReset} className="btn-toolbar">
              Resetar para aqui…
            </button>
          )}
          {onCherryPick && (
            <button type="button" onClick={onCherryPick} className="btn-toolbar">
              Cherry-pick
            </button>
          )}
          {onCreateTag && (
            <button type="button" onClick={onCreateTag} className="btn-toolbar">
              Criar tag…
            </button>
          )}
          {canEditMessage && onEditMessage && (
            <button
              type="button"
              onClick={onEditMessage}
              className="btn-toolbar-primary"
            >
              Editar mensagem
            </button>
          )}
          {canUncommit && onUncommit && (
            <button type="button" onClick={onUncommit} className="btn-toolbar">
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
      {cherryPickHint && (
        <p className="mt-2 text-[10px] leading-snug text-muted">{cherryPickHint}</p>
      )}
      {revertInfoHint && (
        <p className="mt-2 text-[10px] leading-snug text-muted">{revertInfoHint}</p>
      )}
      {resetHint && (
        <p className="mt-2 text-[10px] leading-snug text-muted">{resetHint}</p>
      )}
    </div>
  );
}
