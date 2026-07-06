import { Copy, ExternalLink, KeyRound, Terminal, X } from "lucide-react";
import { useEffect, useRef, useState } from "react";

import { useDialogA11y } from "@/hooks/useDialogA11y";
import type { CredentialStatusDto, SshTestResultDto } from "@/types";
type ConnectMode = "gcm" | "pat" | "ssh";

interface ConnectDialogProps {
  open: boolean;
  credential: CredentialStatusDto | null;
  remoteUrl?: string | null;
  loading?: boolean;
  error?: string | null;
  sshTest?: SshTestResultDto | null;
  copyHint?: string | null;  onCancel: () => void;
  onGcmLogin: () => void;
  onSavePat: (pat: string) => void;
  onConfigureGcm: () => void;
  onTestSsh: () => void;
  onCopyPublicKey: (name: string) => void;
}

export function ConnectDialog({
  open: isOpen,
  credential,
  remoteUrl,
  loading,
  error,
  sshTest,
  copyHint,  onCancel,
  onGcmLogin,
  onSavePat,
  onConfigureGcm,
  onTestSsh,
  onCopyPublicKey,
}: ConnectDialogProps) {
  const [mode, setMode] = useState<ConnectMode>("gcm");
  const [pat, setPat] = useState("");
  const panelRef = useRef<HTMLDivElement>(null);

  useDialogA11y(isOpen, onCancel, panelRef);

  const usesSshRemote =
    remoteUrl?.startsWith("git@") || remoteUrl?.startsWith("ssh://");
  const usesHttpsRemote =
    remoteUrl?.startsWith("https://") || remoteUrl?.startsWith("http://");

  useEffect(() => {
    if (!isOpen) return;
    if (usesSshRemote) setMode("ssh");
    else if (usesHttpsRemote) setMode("gcm");
  }, [isOpen, usesSshRemote, usesHttpsRemote]);

  if (!isOpen) return null;

  const httpsConnected = credential?.githubConnected;
  const httpsUsername = credential?.githubUsername;
  const sshKeys = credential?.sshKeys ?? [];
  function submitPat() {
    const trimmed = pat.trim();
    if (!trimmed) return;
    onSavePat(trimmed);
    setPat("");
  }

  return (
    <div
      className="fixed inset-0 z-50 flex items-center justify-center bg-black/50 p-4"
      role="dialog"
      aria-modal="true"
      aria-labelledby="connect-dialog-title"
    >
      <div
        ref={panelRef}
        className="w-full max-w-md rounded-xl border border-border bg-surface shadow-lg"
      >
        <div className="flex items-center justify-between border-b border-border px-4 py-3">
          <h2 id="connect-dialog-title" className="text-sm font-semibold">
            Conectar ao GitHub
          </h2>
          <button
            type="button"
            onClick={onCancel}
            disabled={loading}
            className="rounded p-1 text-muted hover:bg-bg hover:text-text disabled:opacity-50"
            aria-label="Fechar"
          >
            <X size={16} />
          </button>
        </div>

        <div className="space-y-3 px-4 py-3 text-sm">
          {usesSshRemote ? (
            <>
              {sshTest?.success ? (
                <p className="rounded-md border border-emerald-500/40 bg-emerald-500/10 px-2 py-1.5 text-xs text-emerald-700 dark:text-emerald-300">
                  SSH conectado
                  {sshTest.username ? ` como @${sshTest.username}` : ""} — chave aceita
                  pelo GitHub para este PC.
                </p>
              ) : (
                <p className="rounded-md border border-amber-500/40 bg-amber-500/10 px-2 py-1.5 text-xs text-amber-700 dark:text-amber-300">
                  Este repositório usa SSH (<code className="font-mono">git@</code>). Use a aba
                  SSH e «Testar SSH com GitHub» — a conta depende da chave pública cadastrada
                  no GitHub.
                </p>
              )}
              {httpsConnected && httpsUsername && httpsUsername !== "git" && (
                <p className="text-[11px] text-muted">
                  Há também credencial HTTPS (@{httpsUsername}) no GCM — não é usada neste
                  remoto SSH.
                </p>
              )}
            </>
          ) : usesHttpsRemote ? (
            httpsConnected ? (
              <p className="rounded-md border border-emerald-500/40 bg-emerald-500/10 px-2 py-1.5 text-xs text-emerald-700 dark:text-emerald-300">
                HTTPS conectado
                {httpsUsername && httpsUsername !== "git"
                  ? ` como @${httpsUsername}`
                  : ""}{" "}
                — credencial no Windows Credential Manager.
              </p>
            ) : (
              <p className="text-xs text-muted">
                Este repositório usa HTTPS — autentique com Login GCM ou token (PAT).
              </p>
            )
          ) : (
            <p className="text-xs text-muted">
              Autentique para fetch, push e clone. HTTPS usa GCM/PAT; SSH usa chaves em{" "}
              <code className="font-mono">~/.ssh</code>.
            </p>
          )}

          {credential?.hint && !usesSshRemote && (            <div className="space-y-2 rounded-md border border-amber-500/40 bg-amber-500/10 px-2 py-1.5 text-xs text-amber-700 dark:text-amber-300">
              <p>{credential.hint}</p>
              <button
                type="button"
                onClick={onConfigureGcm}
                disabled={loading}
                className="rounded border border-amber-500/50 px-2 py-1 hover:bg-amber-500/20 disabled:opacity-50"
              >
                Configurar GCM
              </button>
            </div>
          )}

          <div className="flex flex-wrap gap-1 border-b border-border pb-2 text-xs">
            {(["gcm", "pat", "ssh"] as const).map((tab) => (
              <button
                key={tab}
                type="button"
                onClick={() => setMode(tab)}
                className={`rounded px-2 py-1 ${
                  mode === tab ? "bg-accent/15 text-accent" : "text-muted hover:bg-bg"
                }`}
              >
                {tab === "gcm" ? "Login GCM" : tab === "pat" ? "Token (PAT)" : "SSH"}
              </button>
            ))}
          </div>

          {mode === "gcm" && (
            <div className="space-y-2 text-xs">
              <p className="text-muted">
                Abre o assistente do Git Credential Manager (navegador ou device code).
              </p>
              <button
                type="button"
                onClick={onGcmLogin}
                disabled={loading || !credential?.gcmAvailable}
                className="flex w-full items-center justify-center gap-1.5 rounded-lg bg-accent px-3 py-2 text-xs font-medium text-white hover:opacity-90 disabled:opacity-50"
              >
                <KeyRound size={14} />
                {loading ? "Aguardando login…" : "Abrir login do GitHub"}
              </button>
            </div>
          )}

          {mode === "pat" && (
            <div className="space-y-2 text-xs">
              <p className="text-muted">
                Cole um{" "}
                <a
                  href="https://github.com/settings/tokens"
                  target="_blank"
                  rel="noreferrer"
                  className="inline-flex items-center gap-0.5 text-accent hover:underline"
                >
                  token de acesso pessoal
                  <ExternalLink size={12} />
                </a>{" "}
                com escopo <code className="font-mono">repo</code>.
              </p>
              <label className="block text-muted">
                Token
                <input
                  type="password"
                  value={pat}
                  onChange={(e) => setPat(e.target.value)}
                  placeholder="ghp_…"
                  disabled={loading}
                  autoComplete="off"
                  className="mt-1 w-full rounded border border-border bg-bg px-2 py-1.5 font-mono text-xs text-text placeholder:text-muted focus:outline-none focus:ring-1 focus:ring-accent/40 disabled:opacity-50"
                />
              </label>
              <button
                type="button"
                onClick={submitPat}
                disabled={loading || !pat.trim() || !credential?.gcmAvailable}
                className="w-full rounded-lg border border-accent/50 bg-accent/10 px-3 py-2 text-xs font-medium text-accent hover:bg-accent/20 disabled:opacity-50"
              >
                {loading ? "Salvando…" : "Salvar token"}
              </button>
            </div>
          )}

          {mode === "ssh" && (
            <div className="space-y-2 text-xs">
              {usesHttpsRemote && (
                <p className="rounded-md border border-border bg-bg/50 px-2 py-1.5 text-muted">
                  Este repositório usa <strong>HTTPS</strong> — fetch e push usam o GCM/PAT, não
                  SSH. O teste abaixo só verifica se <em>este PC</em> tem chave aceita pelo
                  GitHub.
                </p>
              )}
              <p className="text-muted">
                Para URLs <code className="font-mono">git@github.com:…</code>, o Git usa chaves
                SSH. Adicione a chave pública em{" "}
                <a
                  href="https://github.com/settings/keys"
                  target="_blank"
                  rel="noreferrer"
                  className="inline-flex items-center gap-0.5 text-accent hover:underline"
                >
                  GitHub → SSH keys
                  <ExternalLink size={12} />
                </a>
                .
              </p>
              {sshKeys.length === 0 ? (
                <p className="text-amber-600 dark:text-amber-400">
                  Nenhuma chave encontrada em ~/.ssh. Gere uma com{" "}
                  <code className="font-mono">ssh-keygen -t ed25519</code> no terminal.
                </p>
              ) : (
                <ul className="space-y-1 rounded border border-border bg-bg/50 p-2">
                  {sshKeys.map((key) => (
                    <li
                      key={key.name}
                      className="flex items-center justify-between gap-2 font-mono text-[11px]"
                    >
                      <span>
                        {key.name}
                        {!key.hasPublic && (
                          <span className="ml-1 text-amber-600">(sem .pub)</span>
                        )}
                      </span>
                      {key.hasPublic && (
                        <button
                          type="button"
                          onClick={() => onCopyPublicKey(key.name)}
                          disabled={loading}
                          className="flex items-center gap-1 rounded border border-border px-1.5 py-0.5 text-muted hover:bg-surface disabled:opacity-50"
                          title="Copiar chave pública"
                        >
                          <Copy size={12} />
                          Copiar
                        </button>
                      )}
                    </li>
                  ))}
                </ul>
              )}
              <button
                type="button"
                onClick={onTestSsh}
                disabled={loading}
                className="flex w-full items-center justify-center gap-1.5 rounded-lg border border-accent/50 bg-accent/10 px-3 py-2 text-xs font-medium text-accent hover:bg-accent/20 disabled:opacity-50"
              >
                <Terminal size={14} />
                {loading ? "Testando…" : "Testar SSH com GitHub"}
              </button>
              {sshTest && (
                <p
                  className={
                    sshTest.success
                      ? "rounded-md border border-emerald-500/40 bg-emerald-500/10 px-2 py-1.5 text-emerald-700 dark:text-emerald-300"
                      : "rounded-md border border-red-500/40 bg-red-500/10 px-2 py-1.5 text-red-500"
                  }
                >
                  {sshTest.message}
                  {usesHttpsRemote && sshTest.success && (
                    <>
                      {" "}
                      Este repo não usa SSH — a conta ativa aqui continua sendo a do HTTPS
                      {httpsUsername && httpsUsername !== "git"
                        ? ` (@${httpsUsername})`
                        : ""}
                      .
                    </>
                  )}
                  {usesHttpsRemote && !sshTest.success && (
                    <> Falha no teste SSH — não afeta o HTTPS já configurado neste repositório.</>
                  )}
                </p>
              )}
            </div>
          )}
          {copyHint && (
            <p className="rounded-md border border-emerald-500/40 bg-emerald-500/10 px-2 py-1.5 text-xs text-emerald-700 dark:text-emerald-300">
              {copyHint}
            </p>
          )}

          {error && !sshTest && (
            <p className="rounded-md border border-red-500/40 bg-red-500/10 px-2 py-1.5 text-xs text-red-500">
              {error}
            </p>
          )}
        </div>

        <div className="flex justify-end border-t border-border px-4 py-3">
          <button
            type="button"
            onClick={onCancel}
            disabled={loading}
            className="rounded-lg border border-border px-3 py-1.5 text-xs text-muted hover:bg-bg disabled:opacity-50"
          >
            Fechar
          </button>
        </div>
      </div>
    </div>
  );
}
