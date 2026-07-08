import { useEffect, useMemo, useRef, useState } from "react";

import { formatStagedFileListText } from "@/lib/staged-file-list";
import type { FileChangeDto } from "@/types";

interface AmendSeed {
  summary: string;
  body: string;
}

interface CommitFormProps {
  canAmend: boolean;
  stagedCount: number;
  stagedFiles?: FileChangeDto[];
  /** Explica por que amend não aparece (commit já enviado, etc.). */
  amendUnavailableReason?: string | null;
  /** Pré-preenche amend com a mensagem do HEAD. */
  amendSeed?: AmendSeed | null;
  /** Incrementa para abrir amend a partir de «Editar mensagem». */
  amendIntent?: number;
  /** Só durante preview/execução de commit — não bloqueia por stage/unstage. */
  busy?: boolean;
  onCommit: (summary: string, body: string, amend: boolean) => void;
}

export function CommitForm({
  canAmend,
  stagedCount,
  stagedFiles = [],
  amendUnavailableReason,
  amendSeed,
  amendIntent = 0,
  busy,
  onCommit,
}: CommitFormProps) {
  const [summary, setSummary] = useState("");
  const [body, setBody] = useState("");
  const [amend, setAmend] = useState(false);
  /** Última listagem injetada automaticamente — se o corpo ainda for igual, atualiza ao mudar o stage. */
  const lastAutoBody = useRef("");

  const fileListText = useMemo(
    () => formatStagedFileListText(stagedFiles),
    [stagedFiles],
  );

  useEffect(() => {
    if (amend) return;
    setBody((current) => {
      const prev = lastAutoBody.current;
      if (current !== "" && current !== prev) return current;
      lastAutoBody.current = fileListText;
      return fileListText;
    });
  }, [amend, fileListText]);

  function applyAmendSeed() {
    if (!amendSeed) return;
    setSummary(amendSeed.summary);
    setBody(amendSeed.body);
  }

  useEffect(() => {
    if (amendIntent > 0 && canAmend && amendSeed) {
      setAmend(true);
      applyAmendSeed();
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps -- só reage ao intent explícito
  }, [amendIntent]);

  function toggleAmend(checked: boolean) {
    setAmend(checked);
    if (checked && amendSeed) {
      applyAmendSeed();
      return;
    }
    if (!checked) {
      lastAutoBody.current = fileListText;
      setBody(fileListText);
    }
  }

  function onBodyChange(value: string) {
    setBody(value);
  }

  function submit() {
    if (!summary.trim()) return;
    if (!amend && stagedCount === 0) return;
    onCommit(summary.trim(), body.trim(), amend);
    if (!amend) {
      setSummary("");
      lastAutoBody.current = fileListText;
      setBody(fileListText);
    }
    setAmend(false);
  }

  const canSubmit =
    Boolean(summary.trim()) && (amend || stagedCount > 0);

  return (
    <div className="border-t border-border bg-surface px-3 py-2">
      <div className="mb-1.5 text-[11px] font-semibold uppercase tracking-wide text-muted">
        Commit
      </div>
      <input
        type="text"
        value={summary}
        onChange={(e) => setSummary(e.target.value)}
        placeholder="Resumo do commit"
        disabled={busy}
        aria-label="Resumo do commit"
        className="mb-2 w-full rounded border border-border bg-bg px-2 py-1.5 text-xs placeholder:text-muted focus:outline-none focus:ring-1 focus:ring-accent/40 disabled:opacity-50"
      />
      <textarea
        value={body}
        onChange={(e) => onBodyChange(e.target.value)}
        placeholder="Descrição (opcional)"
        rows={4}
        disabled={busy}
        aria-label="Descrição do commit (opcional)"
        className="mb-2 min-h-[5rem] max-h-[min(320px,45vh)] w-full resize-y rounded border border-border bg-bg px-2 py-1.5 font-mono text-xs placeholder:text-muted focus:outline-none focus:ring-1 focus:ring-accent/40 disabled:opacity-50 disabled:resize-none"
      />
      <div className="flex flex-col gap-1.5">
        <div className="flex items-center justify-between gap-2">
          {canAmend ? (
            <label className="flex items-center gap-1.5 text-xs text-muted">
              <input
                type="checkbox"
                checked={amend}
                onChange={(e) => toggleAmend(e.target.checked)}
                disabled={busy}
                className="rounded border-border"
              />
              Amend (último commit local)
            </label>
          ) : (
            <span />
          )}
          <button
            type="button"
            onClick={submit}
            disabled={busy || !canSubmit}
            className="rounded-lg bg-accent px-3 py-1 text-xs font-medium text-white hover:opacity-90 disabled:opacity-50"
          >
            {amend ? "Amend" : "Commit"}
          </button>
        </div>
        {!amend && stagedCount === 0 && (
          <p className="text-[11px] leading-snug text-muted">
            Nenhum arquivo em stage — adicione alterações antes de commitar.
            {canAmend
              ? " Ou marque Amend para alterar só a mensagem do último commit."
              : ""}
          </p>
        )}
        {!canAmend && amendUnavailableReason && (
          <p className="text-[11px] leading-snug text-muted">
            {amendUnavailableReason}
          </p>
        )}
      </div>
    </div>
  );
}
