import { ScrollText, X } from "lucide-react";
import { useEffect, useRef, useState } from "react";

import { useDialogA11y } from "@/hooks/useDialogA11y";
import { listAuditLog } from "@/lib/api";
import type { AuditEntryDto } from "@/types";

interface AuditLogDialogProps {
  open: boolean;
  onClose: () => void;
}

const ACTION_LABEL: Record<AuditEntryDto["action"], string> = {
  add: "Stage",
  commit: "Commit",
  push: "Push",
  pushForce: "Force push",
  reset: "Reset",
  revert: "Revert",
  cherryPick: "Cherry-pick",
  reword: "Reword",
};

export function AuditLogDialog({ open, onClose }: AuditLogDialogProps) {
  const panelRef = useRef<HTMLDivElement>(null);
  const [entries, setEntries] = useState<AuditEntryDto[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  useDialogA11y(open, onClose, panelRef);

  useEffect(() => {
    if (!open) return;
    let cancelled = false;
    setLoading(true);
    setError(null);
    void listAuditLog(7)
      .then((list) => {
        if (!cancelled) setEntries(list);
      })
      .catch((e) => {
        if (!cancelled) setError(e instanceof Error ? e.message : String(e));
      })
      .finally(() => {
        if (!cancelled) setLoading(false);
      });
    return () => {
      cancelled = true;
    };
  }, [open]);

  if (!open) return null;

  return (
    <div
      className="fixed inset-0 z-[100] flex items-center justify-center bg-black/50 p-4"
      role="dialog"
      aria-modal="true"
      aria-labelledby="audit-log-title"
    >
      <div
        ref={panelRef}
        className="flex max-h-[85vh] w-full max-w-2xl flex-col overflow-hidden rounded-xl border border-border bg-surface shadow-lg"
      >
        <div className="flex shrink-0 items-center justify-between border-b border-border px-4 py-3">
          <div className="flex items-center gap-2">
            <ScrollText size={16} className="text-accent" />
            <h2 id="audit-log-title" className="text-sm font-semibold">
              Histórico de ações
            </h2>
          </div>
          <button
            type="button"
            onClick={onClose}
            className="rounded p-1 text-muted hover:bg-bg hover:text-text"
            aria-label="Fechar"
          >
            <X size={16} />
          </button>
        </div>

        <p className="shrink-0 border-b border-border px-4 py-2 text-[11px] text-muted">
          Últimos 7 dias · log local do app (não vai para o repositório)
        </p>

        <div className="min-h-0 flex-1 overflow-auto px-4 py-3">
          {loading && (
            <p className="text-xs text-muted">Carregando…</p>
          )}
          {error && (
            <p className="rounded-md border border-red-500/40 bg-red-500/10 px-3 py-2 text-xs text-red-600 dark:text-red-400">
              {error}
            </p>
          )}
          {!loading && !error && entries.length === 0 && (
            <p className="text-xs text-muted">
              Nenhuma ação registrada ainda. Stage, commit, push, reset, revert,
              cherry-pick e reword aparecem aqui.
            </p>
          )}
          {!loading && entries.length > 0 && (
            <ul className="space-y-2">
              {entries.map((e, i) => (
                <li
                  key={`${e.timestamp}-${i}`}
                  className="rounded-lg border border-border bg-bg/60 px-3 py-2"
                >
                  <div className="flex flex-wrap items-baseline gap-x-2 gap-y-0.5">
                    <span
                      className={`text-[10px] font-semibold uppercase tracking-wide ${
                        e.result === "error"
                          ? "text-red-600 dark:text-red-400"
                          : "text-accent"
                      }`}
                    >
                      {ACTION_LABEL[e.action] ?? e.action}
                    </span>
                    <span className="text-[10px] text-muted">
                      {formatWhen(e.timestamp)}
                    </span>
                    {e.branch && (
                      <span className="font-mono text-[10px] text-muted">
                        {e.branch}
                      </span>
                    )}
                    {e.result === "error" && (
                      <span className="text-[10px] font-medium text-red-600 dark:text-red-400">
                        falhou
                      </span>
                    )}
                  </div>
                  <pre className="mt-1 overflow-x-auto whitespace-pre-wrap break-all font-mono text-[10px] text-text">
                    {e.command}
                  </pre>
                  {e.error && (
                    <p className="mt-1 text-[10px] text-red-600 dark:text-red-400">
                      {e.error}
                    </p>
                  )}
                  <p className="mt-1 truncate text-[10px] text-muted" title={e.repo}>
                    {e.repo}
                  </p>
                </li>
              ))}
            </ul>
          )}
        </div>

        <div className="flex shrink-0 justify-end border-t border-border px-4 py-3">
          <button
            type="button"
            onClick={onClose}
            className="rounded-lg border border-border px-3 py-1.5 text-xs text-muted hover:bg-bg"
          >
            Fechar
          </button>
        </div>
      </div>
    </div>
  );
}

function formatWhen(iso: string): string {
  try {
    return new Date(iso).toLocaleString("pt-BR");
  } catch {
    return iso;
  }
}
