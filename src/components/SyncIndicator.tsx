import { KeyRound, RefreshCw } from "lucide-react";
import type { SyncInfoDto } from "@/types";

interface SyncIndicatorProps {
  sync: SyncInfoDto | null;
  onFetch: () => void;
  loading?: boolean;
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
  onFetch,
  loading,
  error,
}: SyncIndicatorProps) {
  const lastSync = sync?.lastFetchAt
    ? new Date(sync.lastFetchAt).toLocaleString("pt-BR")
    : null;
  const authError = isAuthError(error);

  return (
    <div className="flex max-w-xs flex-col gap-1 text-xs">
      <div className="flex gap-1">
        <button
          type="button"
          onClick={onFetch}
          disabled={loading}
          className="flex items-center gap-1.5 rounded border border-border px-2 py-1 hover:bg-surface disabled:opacity-50"
          title="Sincronizar (fetch)"
        >
          <RefreshCw size={14} className={loading ? "animate-spin" : ""} />
          Sincronizar
        </button>
        {authError && (
          <button
            type="button"
            onClick={onFetch}
            disabled={loading}
            className="flex items-center gap-1 rounded border border-accent/50 bg-accent/10 px-2 py-1 text-accent hover:bg-accent/20 disabled:opacity-50"
            title="Abrir Git Credential Manager"
          >
            <KeyRound size={14} />
            Conectar
          </button>
        )}
      </div>
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
        <span className={authError ? "text-amber-600 dark:text-amber-400" : "text-red-500"}>
          {error}
        </span>
      )}
    </div>
  );
}
