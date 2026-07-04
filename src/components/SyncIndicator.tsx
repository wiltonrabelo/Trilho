import { KeyRound, RefreshCw, Upload } from "lucide-react";
import type { SyncInfoDto, CredentialStatusDto } from "@/types";

interface SyncIndicatorProps {
  sync: SyncInfoDto | null;
  credential: CredentialStatusDto | null;
  branch?: string | null;
  hasRemote?: boolean;
  upstreamConfigured?: boolean;
  writeDisabled?: boolean;
  onFetch: () => void;
  onPublish?: () => void;
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
  branch,
  hasRemote = false,
  upstreamConfigured = false,
  writeDisabled,
  onFetch,
  onPublish,
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
  const needsCredentialSetup = Boolean(
    credential?.hint && !credential.gcmAvailable,
  );
  const needsPublish =
    Boolean(branch) && !writeDisabled && !upstreamConfigured;
  const showPull = Boolean(sync?.upstream && sync.behind > 0 && onPull);
  const showPush = Boolean(sync?.upstream && sync.ahead > 0 && onPush);
  const busy = loading || pushLoading;

  return (
    <div className="flex max-w-md flex-col gap-1 text-xs">
      <div className="flex flex-wrap gap-1">
        <button
          type="button"
          onClick={onFetch}
          disabled={busy}
          className="flex items-center gap-1.5 rounded border border-border px-2 py-1 hover:bg-surface disabled:opacity-50"
          title="Sincronizar (fetch)"
        >
          <RefreshCw size={14} className={loading ? "animate-spin" : ""} />
          Fetch
        </button>
        {needsPublish && onPublish && (
          <button
            type="button"
            onClick={onPublish}
            disabled={busy}
            className="flex items-center gap-1 rounded border border-accent/50 bg-accent/10 px-2 py-1 text-accent hover:bg-accent/20 disabled:opacity-50"
            title="Publicar branch no remoto pela primeira vez"
          >
            <Upload size={14} />
            Publicar
          </button>
        )}
        {showPull && (
          <button
            type="button"
            onClick={onPull}
            disabled={busy}
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
            disabled={busy}
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
            disabled={busy}
            className="flex items-center gap-1 rounded border border-accent/50 bg-accent/10 px-2 py-1 text-accent hover:bg-accent/20 disabled:opacity-50"
            title="Reautenticar via Git Credential Manager"
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
      {needsPublish && (
        <span className="text-amber-600 dark:text-amber-400">
          {hasRemote
            ? "Branch ainda não publicada — use Publicar para enviar ao remoto e habilitar Push."
            : "Repositório só local — use Publicar para conectar ao GitHub e enviar a branch."}
        </span>
      )}
      {needsCredentialSetup && (
        <span className="text-amber-600 dark:text-amber-400">
          Conta Git ainda não configurada neste PC — na primeira publicação ou
          sync, o Trilho abrirá o login do GitHub (Git Credential Manager).
        </span>
      )}
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
