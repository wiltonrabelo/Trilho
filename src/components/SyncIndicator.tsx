import { KeyRound, RefreshCw, Upload } from "lucide-react";
import type { SyncInfoDto, CredentialStatusDto } from "@/types";

interface SyncIndicatorProps {
  sync: SyncInfoDto | null;
  credential: CredentialStatusDto | null;
  onFetch: () => void;
  onPush?: () => void;
  onPull?: () => void;
  loading?: boolean;
  pushLoading?: boolean;
  error?: string | null;
}

function isAuthError(error: string | null | undefined): boolean {
  if (!error) return false;
  const lower = error.toLowerCase();
  return (
    lower.includes("autentica") ||
    lower.includes("credential") ||
    lower.includes("gcm") ||
    lower.includes("conectar")
  );
}

export function SyncIndicator({
  sync,
  credential,
  onFetch,
  onPush,
  onPull,
  loading,
  pushLoading,
  error,
}: SyncIndicatorProps) {
  const lastSync = sync?.lastFetchAt
    ? new Date(sync.lastFetchAt).toLocaleString("pt-BR")
    : null;
  const authError = isAuthError(error);
  const showCredentialHint =
    credential?.hint && sync?.upstream && !credential.gcmAvailable;
  const showPull = Boolean(sync?.upstream && sync.behind > 0 && onPull);
  const showPush = Boolean(sync?.upstream && sync.ahead > 0 && onPush);

  return (
    <div className="flex max-w-xs flex-col gap-1 text-xs">
      <div className="flex flex-wrap gap-1">
        <button
          type="button"
          onClick={onFetch}
          disabled={loading || pushLoading}
          className="flex items-center gap-1.5 rounded border border-border px-2 py-1 hover:bg-surface disabled:opacity-50"
          title="Sincronizar (fetch)"
        >
          <RefreshCw
            size={14}
            className={loading ? "animate-spin" : ""}
          />
          Fetch
        </button>
        {showPull && (
          <button
            type="button"
            onClick={onPull}
            disabled={loading || pushLoading}
            className="flex items-center gap-1 rounded border border-border px-2 py-1 hover:bg-surface disabled:opacity-50"
            title="Atualizar com pull --ff-only"
          >
            Pull ↓{sync!.behind}
          </button>
        )}
        {showPush && (
          <button
            type="button"
            onClick={onPush}
            disabled={loading || pushLoading}
            className="flex items-center gap-1 rounded border border-accent/50 bg-accent/10 px-2 py-1 text-accent hover:bg-accent/20 disabled:opacity-50"
            title="Enviar commits (push)"
          >
            <Upload size={14} className={pushLoading ? "animate-pulse" : ""} />
            Push ↑{sync!.ahead}
          </button>
        )}
        {authError && (
          <button
            type="button"
            onClick={onFetch}
            disabled={loading || pushLoading}
            className="flex items-center gap-1 rounded border border-accent/50 bg-accent/10 px-2 py-1 text-accent hover:bg-accent/20 disabled:opacity-50"
            title="Reautenticar via Git Credential Manager"
          >
            <KeyRound size={14} />
            Conectar
          </button>
        )}
      </div>
      {showCredentialHint && (
        <span className="text-amber-600 dark:text-amber-400">
          {credential!.hint}
        </span>
      )}
      {sync?.upstream && (
        <span className="text-muted">
          {sync.upstream}
          {sync.ahead > 0 || sync.behind > 0
            ? ` · ↑${sync.ahead} ↓${sync.behind}`
            : ""}
        </span>
      )}
      <span className="text-muted">
        {lastSync
          ? `Baseado na última sync: ${lastSync}`
          : "Ainda não sincronizado — status local"}
      </span>
      {error && (
        <span
          className={
            authError ? "text-amber-600 dark:text-amber-400" : "text-red-500"
          }
        >
          {error}
        </span>
      )}
    </div>
  );
}
