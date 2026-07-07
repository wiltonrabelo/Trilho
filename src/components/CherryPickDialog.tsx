import { GitBranchPlus } from "lucide-react";
import { useEffect, useMemo, useRef, useState } from "react";

import { useDialogA11y } from "@/hooks/useDialogA11y";
import type { CommitDto } from "@/types";

interface CherryPickDialogProps {
  open: boolean;
  /** Commit clicado — pré-selecionado. */
  primaryCommit: CommitDto | null;
  /** Candidatos (visão exclusiva de branch); vazio = só o commit primário. */
  candidates: CommitDto[];
  loading?: boolean;
  error?: string | null;
  onCancel: () => void;
  onContinue: (commitIds: string[], recordOrigin: boolean) => void;
}

/** Ordem para o Git: do mais antigo ao mais recente (lista vem do mais novo). */
function orderOldestFirst(selected: Set<string>, commits: CommitDto[]): string[] {
  return commits.filter((c) => selected.has(c.id)).reverse().map((c) => c.id);
}

function isMergeCommit(c: CommitDto): boolean {
  return c.parentIds.length > 1;
}

export function CherryPickDialog({
  open: isOpen,
  primaryCommit,
  candidates,
  loading,
  error,
  onCancel,
  onContinue,
}: CherryPickDialogProps) {
  const panelRef = useRef<HTMLDivElement>(null);
  const multi = candidates.length > 1;
  const orderedCandidates = candidates;

  const [selected, setSelected] = useState<Set<string>>(new Set());
  const [recordOrigin, setRecordOrigin] = useState(false);

  useDialogA11y(isOpen, onCancel, panelRef);

  useEffect(() => {
    if (isOpen && primaryCommit) {
      const initial = isMergeCommit(primaryCommit)
        ? new Set<string>()
        : new Set([primaryCommit.id]);
      setSelected(initial);
      setRecordOrigin(false);
    }
  }, [isOpen, primaryCommit?.id]);

  const hasSelectedMerge = useMemo(
    () => orderedCandidates.some((c) => selected.has(c.id) && isMergeCommit(c)),
    [orderedCandidates, selected],
  );

  const pickableCount = useMemo(
    () => orderedCandidates.filter((c) => !isMergeCommit(c)).length,
    [orderedCandidates],
  );

  const selectedCount = selected.size;
  const orderedIds = useMemo(
    () => orderOldestFirst(selected, orderedCandidates),
    [selected, orderedCandidates],
  );

  if (!isOpen || !primaryCommit) return null;

  function toggle(id: string) {
    const commit = orderedCandidates.find((c) => c.id === id);
    if (commit && isMergeCommit(commit)) return;
    setSelected((prev) => {
      const next = new Set(prev);
      if (next.has(id)) {
        if (next.size > 1) next.delete(id);
      } else {
        next.add(id);
      }
      return next;
    });
  }

  return (
    <div
      className="fixed inset-0 z-50 flex items-center justify-center bg-black/50 p-4"
      role="presentation"
      onMouseDown={(e) => {
        if (e.target === e.currentTarget) onCancel();
      }}
    >
      <div
        ref={panelRef}
        role="dialog"
        aria-modal="true"
        aria-labelledby="cherry-pick-dialog-title"
        className="flex max-h-[90vh] w-full max-w-md flex-col rounded-xl border border-border bg-surface shadow-lg"
      >
        <div className="flex items-center gap-2 border-b border-border px-4 py-3">
          <GitBranchPlus size={18} className="text-accent" />
          <h2 id="cherry-pick-dialog-title" className="text-sm font-semibold text-text">
            Cherry-pick
          </h2>
        </div>

        <div className="min-h-0 flex-1 space-y-3 overflow-y-auto px-4 py-3 text-sm">
          <p className="text-xs text-muted">
            Aplica as alterações na branch em checkout
            {multi
              ? ` — ${selectedCount} commit(s) selecionado(s), do mais antigo ao mais recente.`
              : "."}
          </p>

          {multi ? (
            <>
              {pickableCount < orderedCandidates.length ? (
                <p className="text-[10px] leading-snug text-amber-800 dark:text-amber-200">
                  Commits de merge não podem ser cherry-picked nesta versão — selecione só
                  commits normais (sem «merge»).
                </p>
              ) : null}
              {pickableCount > 1 ? (
                <button
                  type="button"
                  onClick={() =>
                    setSelected(
                      new Set(
                        orderedCandidates
                          .filter((c) => !isMergeCommit(c))
                          .map((c) => c.id),
                      ),
                    )
                  }
                  className="text-[10px] text-accent hover:underline"
                >
                  Selecionar todos os commits normais ({pickableCount})
                </button>
              ) : null}
              <ul className="max-h-48 space-y-1 overflow-y-auto rounded-md border border-border p-2">
                {orderedCandidates.map((c) => {
                  const merge = isMergeCommit(c);
                  const checked = selected.has(c.id);
                  return (
                    <li key={c.id}>
                      <label
                        className={`flex items-start gap-2 rounded px-1 py-1 ${
                          merge
                            ? "cursor-not-allowed opacity-50"
                            : "cursor-pointer hover:bg-bg"
                        }`}
                      >
                        <input
                          type="checkbox"
                          checked={checked}
                          disabled={merge}
                          onChange={() => toggle(c.id)}
                          className="mt-0.5"
                        />
                        <span className="min-w-0 flex-1">
                          <span className="block truncate text-xs text-text">
                            {c.summary}
                            {merge ? (
                              <span className="ml-1 text-[10px] text-muted">(merge)</span>
                            ) : null}
                          </span>
                          <span className="font-mono text-[10px] text-muted">
                            {c.shortId}
                          </span>
                        </span>
                      </label>
                    </li>
                  );
                })}
              </ul>
            </>
          ) : isMergeCommit(primaryCommit) ? (
            <p className="rounded-md border border-amber-500/40 bg-amber-500/10 px-3 py-2 text-xs text-amber-900 dark:text-amber-100">
              Este commit é um merge — cherry-pick exige escolher qual lado manter
              (git cherry-pick -m), operação fora do MVP. Escolha um commit normal na
              trilha.
            </p>
          ) : (
            <div className="rounded-md border border-border px-3 py-2 text-xs">
              <p className="font-medium text-text">{primaryCommit.summary}</p>
              <p className="font-mono text-[10px] text-muted">
                {primaryCommit.shortId}
              </p>
            </div>
          )}

          <label className="flex cursor-pointer items-start gap-2 rounded-md border border-border px-3 py-2 text-xs text-text">
            <input
              type="checkbox"
              checked={recordOrigin}
              onChange={(e) => setRecordOrigin(e.target.checked)}
              className="mt-0.5"
            />
            <span>
              Registrar origem na mensagem{" "}
              <span className="text-muted">(git cherry-pick -x)</span>
            </span>
          </label>

          {error && (
            <p className="text-xs text-red-600 dark:text-red-400">{error}</p>
          )}
        </div>

        <div className="flex justify-end gap-2 border-t border-border px-4 py-3">
          <button
            type="button"
            onClick={onCancel}
            className="rounded-lg border border-border px-3 py-1.5 text-xs text-muted hover:bg-bg"
          >
            Cancelar
          </button>
          <button
            type="button"
            disabled={
              loading || orderedIds.length === 0 || hasSelectedMerge
            }
            onClick={() => onContinue(orderedIds, recordOrigin)}
            className="rounded-lg bg-accent px-3 py-1.5 text-xs font-medium text-white hover:opacity-90 disabled:opacity-50"
          >
            {loading ? "Carregando…" : "Continuar"}
          </button>
        </div>
      </div>
    </div>
  );
}
