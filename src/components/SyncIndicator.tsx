import { AlertTriangle, KeyRound, RefreshCw, Upload } from "lucide-react";
import { isAuthError, syncSummary } from "@/lib/syncNotices";
import type { SyncInfoDto, CredentialStatusDto } from "@/types";

interface SyncIndicatorProps {
  sync: SyncInfoDto | null;
  credential: CredentialStatusDto | null;
  branch?: string | null;
  remoteUrl?: string | null;
  sshUsername?: string | null;
  upstreamConfigured?: boolean;
  isShallow?: boolean;
  writeDisabled?: boolean;
  onFetch: () => void;
  onPublish?: () => void;
  onPush?: () => void;
  onPushForce?: () => void;
  onPull?: () => void;
  onUnshallow?: () => void;
  onConnect?: () => void;
  loading?: boolean;
  pushLoading?: boolean;
  error?: string | null;
}

const BOTAO =
  "flex shrink-0 items-center gap-1.5 rounded border border-border px-2 py-1 hover:bg-surface disabled:opacity-50";
const BOTAO_DESTAQUE =
  "flex shrink-0 items-center gap-1 rounded border border-accent/50 bg-accent/10 px-2 py-1 text-accent hover:bg-accent/20 disabled:opacity-50";

/**
 * Ações de sincronização — uma linha só, sem quebra. Os avisos que antes
 * viviam aqui embaixo saíram para `SyncNoticeBar`, porque empilhados dentro do
 * cabeçalho eles espremiam tudo em telas de 1024px.
 */
export function SyncIndicator({
  sync,
  credential,
  branch,
  remoteUrl,
  sshUsername,
  upstreamConfigured = false,
  isShallow = false,
  writeDisabled,
  onFetch,
  onPublish,
  onPush,
  onPushForce,
  onPull,
  onUnshallow,
  onConnect,
  loading,
  pushLoading,
  error,
}: SyncIndicatorProps) {
  const authError = isAuthError(error);
  const showConnect = Boolean(onConnect);
  const needsPublish =
    Boolean(branch) && !writeDisabled && !upstreamConfigured;
  const showPull = Boolean(sync?.upstream && sync.behind > 0 && onPull);
  const showPush = Boolean(sync?.upstream && sync.ahead > 0 && onPush);
  const showPushForce = Boolean(
    sync?.upstream && sync.behind > 0 && onPushForce && !writeDisabled,
  );
  const busy = loading || pushLoading;
  const resumo = syncSummary({ sync, credential, remoteUrl, sshUsername }).join(
    "\n",
  );

  return (
    <div
      className="flex items-center gap-1.5 text-xs"
      role="region"
      aria-label="Sincronização com remoto"
    >
      <button
        type="button"
        onClick={onFetch}
        disabled={busy}
        aria-label="Sincronizar com o remoto (fetch)"
        className={BOTAO}
        title={`Sincronizar (fetch)\n${resumo}`}
      >
        <RefreshCw size={14} className={loading ? "animate-spin" : ""} />
        <span className="hidden lg:inline">Fetch</span>
      </button>
      {needsPublish && onPublish && (
        <button
          type="button"
          onClick={onPublish}
          disabled={busy}
          aria-label="Publicar branch no remoto"
          className={BOTAO_DESTAQUE}
          title="Vincular a branch ao GitHub e enviar"
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
          aria-label={`Puxar ${sync!.behind} commit(s) do remoto`}
          className={BOTAO}
          title="Atualizar com pull --ff-only"
        >
          Pull ↓{sync!.behind}
        </button>
      )}
      {isShallow && onUnshallow && (
        <button
          type="button"
          onClick={onUnshallow}
          disabled={busy}
          aria-label="Completar histórico do clone raso"
          className="flex shrink-0 items-center gap-1 rounded border border-amber-500/50 bg-amber-500/10 px-2 py-1 text-amber-700 hover:bg-amber-500/20 disabled:opacity-50 dark:text-amber-300"
          title="git fetch --unshallow — baixa todo o histórico"
        >
          <RefreshCw size={14} />
          <span className="hidden xl:inline">Completar histórico</span>
        </button>
      )}
      {showPush && (
        <button
          type="button"
          onClick={onPush}
          disabled={busy}
          aria-label={`Enviar ${sync!.ahead} commit(s) ao remoto`}
          className={BOTAO_DESTAQUE}
          title="Enviar commits (push)"
        >
          <Upload size={14} className={pushLoading ? "animate-pulse" : ""} />
          Push ↑{sync!.ahead}
        </button>
      )}
      {showPushForce && (
        <button
          type="button"
          onClick={onPushForce}
          disabled={busy}
          aria-label={`Push forçado — remoto ${sync!.behind} commit(s) à frente`}
          className="flex shrink-0 items-center gap-1 rounded border border-red-500/50 bg-red-500/10 px-2 py-1 text-red-700 hover:bg-red-500/20 disabled:opacity-50 dark:text-red-300"
          title="git push --force-with-lease — reescreve histórico remoto"
        >
          <AlertTriangle size={14} />
          <span className="hidden xl:inline">Force push</span>
        </button>
      )}
      {showConnect && onConnect && (
        <button
          type="button"
          onClick={onConnect}
          disabled={loading}
          aria-label="Conectar conta GitHub"
          className={BOTAO_DESTAQUE}
          title="Assistente de conexão GitHub"
        >
          <KeyRound size={14} />
          <span className="hidden lg:inline">
            {credential?.githubConnected ? "Conta" : "Conectar"}
          </span>
        </button>
      )}
      {authError && !onConnect && (
        <button
          type="button"
          onClick={onFetch}
          disabled={loading}
          aria-label="Conectar ou reautenticar no GitHub"
          className={BOTAO_DESTAQUE}
          title="Reautenticar via Git Credential Manager"
        >
          <KeyRound size={14} />
          <span className="hidden lg:inline">Conectar</span>
        </button>
      )}
    </div>
  );
}
