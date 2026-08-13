import type { CredentialStatusDto, SyncInfoDto } from "@/types";

export type SyncNoticeSeverity = "warning" | "error";

export interface SyncNotice {
  id: string;
  severity: SyncNoticeSeverity;
  message: string;
}

export interface SyncNoticeInput {
  credential: CredentialStatusDto | null;
  /** Repositório tem remoto configurado (muda o texto de «publicar»). */
  hasRemote: boolean;
  isShallow: boolean;
  needsPublish: boolean;
  error?: string | null;
}

export function isAuthError(error: string | null | undefined): boolean {
  if (!error) return false;
  const lower = error.toLowerCase();
  return (
    lower.includes("autentica") ||
    lower.includes("credential") ||
    lower.includes("gcm") ||
    lower.includes("conectar")
  );
}

/**
 * Avisos que pedem ação do usuário. Ficam fora da barra de topo: enfiados lá
 * eles empurravam os botões e esmagavam o cabeçalho em telas menores.
 */
export function syncNotices({
  credential,
  hasRemote,
  isShallow,
  needsPublish,
  error,
}: SyncNoticeInput): SyncNotice[] {
  const notices: SyncNotice[] = [];

  if (isShallow) {
    notices.push({
      id: "shallow",
      severity: "warning",
      message:
        "Clone raso — só parte do histórico está local. Use «Completar histórico» para baixar o restante.",
    });
  }

  if (needsPublish) {
    notices.push({
      id: "publish",
      severity: "warning",
      message: hasRemote
        ? "Branch sem rastreamento remoto — faça Fetch ou use Publicar para vincular ao GitHub."
        : "Repositório só local — use Publicar para conectar ao GitHub e enviar a branch.",
    });
  }

  if (credential?.hint && !credential.gcmAvailable) {
    notices.push({
      id: "credential",
      severity: "warning",
      message:
        "Conta Git ainda não configurada — use «Conectar» para abrir o assistente (GCM ou token).",
    });
  }

  if (error) {
    notices.push({
      id: "error",
      severity: isAuthError(error) ? "warning" : "error",
      message: error,
    });
  }

  return notices;
}

export interface SyncSummaryInput {
  sync: SyncInfoDto | null;
  credential: CredentialStatusDto | null;
  remoteUrl?: string | null;
  sshUsername?: string | null;
}

/**
 * Informação de rotina (upstream, última sync, conta) — vira tooltip do botão
 * de Fetch em vez de ocupar linhas fixas do cabeçalho.
 */
export function syncSummary({
  sync,
  credential,
  remoteUrl,
  sshUsername,
}: SyncSummaryInput): string[] {
  const lines: string[] = [];

  if (sync?.upstream) {
    const divergencia =
      sync.ahead > 0 || sync.behind > 0 ? ` · ↑${sync.ahead} ↓${sync.behind}` : "";
    lines.push(`${sync.upstream}${divergencia}`);
  }

  lines.push(
    sync?.lastFetchAt
      ? `Baseado na última sync: ${new Date(sync.lastFetchAt).toLocaleString("pt-BR")}`
      : "Ainda não sincronizado — status local",
  );

  const usaSsh = remoteUrl?.startsWith("git@") || remoteUrl?.startsWith("ssh://");
  const usaHttps =
    remoteUrl?.startsWith("https://") || remoteUrl?.startsWith("http://");
  const usuarioHttps =
    credential?.githubConnected &&
    credential.githubUsername &&
    credential.githubUsername !== "git"
      ? credential.githubUsername
      : null;

  if (usaSsh && sshUsername) {
    lines.push(`GitHub SSH: @${sshUsername}`);
  } else if (usaHttps && usuarioHttps) {
    lines.push(`GitHub HTTPS: @${usuarioHttps}`);
  } else if (!usaSsh && !usaHttps && credential?.githubConnected && credential.githubUsername) {
    lines.push(`GitHub: @${credential.githubUsername}`);
  }

  return lines;
}
