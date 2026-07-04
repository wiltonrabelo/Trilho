import { useState } from "react";

interface CommitFormProps {
  canAmend: boolean;
  /** Só durante preview/execução de commit — não bloqueia por stage/unstage. */
  busy?: boolean;
  onCommit: (summary: string, body: string, amend: boolean) => void;
}

export function CommitForm({ canAmend, busy, onCommit }: CommitFormProps) {
  const [summary, setSummary] = useState("");
  const [body, setBody] = useState("");
  const [amend, setAmend] = useState(false);

  function submit() {
    if (!summary.trim()) return;
    onCommit(summary.trim(), body.trim(), amend);
    if (!amend) {
      setSummary("");
      setBody("");
    }
    setAmend(false);
  }

  return (
    <div className="border-t border-border bg-surface px-3 py-2">
      <div className="mb-1.5 text-[10px] font-semibold uppercase tracking-wide text-muted">
        Commit
      </div>
      <input
        type="text"
        value={summary}
        onChange={(e) => setSummary(e.target.value)}
        placeholder="Resumo do commit"
        disabled={busy}
        className="mb-2 w-full rounded border border-border bg-bg px-2 py-1.5 text-xs placeholder:text-muted focus:outline-none focus:ring-1 focus:ring-accent/40 disabled:opacity-50"
      />
      <textarea
        value={body}
        onChange={(e) => setBody(e.target.value)}
        placeholder="Descrição (opcional)"
        rows={4}
        disabled={busy}
        className="mb-2 min-h-[5rem] max-h-[min(320px,45vh)] w-full resize-y rounded border border-border bg-bg px-2 py-1.5 text-xs placeholder:text-muted focus:outline-none focus:ring-1 focus:ring-accent/40 disabled:opacity-50 disabled:resize-none"
      />
      <div className="flex items-center justify-between gap-2">
        {canAmend ? (
          <label className="flex items-center gap-1.5 text-[10px] text-muted">
            <input
              type="checkbox"
              checked={amend}
              onChange={(e) => setAmend(e.target.checked)}
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
          disabled={busy || !summary.trim()}
          className="rounded-lg bg-accent px-3 py-1 text-xs font-medium text-white hover:opacity-90 disabled:opacity-50"
        >
          {amend ? "Amend" : "Commit"}
        </button>
      </div>
    </div>
  );
}
