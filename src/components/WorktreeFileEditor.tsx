import { useCallback, useEffect, useState } from "react";

import { readWorktreeFile } from "@/lib/api";

interface WorktreeFileEditorProps {
  path: string;
  writeDisabled?: boolean;
  onSave?: (content: string) => Promise<void>;
  /** Muda quando o arquivo no disco foi alterado (ex.: após reverter trecho). */
  reloadKey?: string | null;
}

/** Editor do conteúdo atual no working tree — salva com Ctrl+S ou botão Salvar. */
export function WorktreeFileEditor({
  path,
  writeDisabled,
  onSave,
  reloadKey,
}: WorktreeFileEditorProps) {
  const [content, setContent] = useState("");
  const [savedContent, setSavedContent] = useState("");
  const [error, setError] = useState<string | null>(null);
  const [saveError, setSaveError] = useState<string | null>(null);
  const [loading, setLoading] = useState(true);
  const [saving, setSaving] = useState(false);

  const dirty = content !== savedContent;
  const canSave = Boolean(onSave) && dirty && !writeDisabled && !saving;

  useEffect(() => {
    let cancelled = false;
    setLoading(true);
    setError(null);
    setSaveError(null);
    void readWorktreeFile(path)
      .then((text) => {
        if (!cancelled) {
          setContent(text);
          setSavedContent(text);
        }
      })
      .catch((e) => {
        if (!cancelled) {
          setError(e instanceof Error ? e.message : String(e));
          setContent("");
          setSavedContent("");
        }
      })
      .finally(() => {
        if (!cancelled) setLoading(false);
      });
    return () => {
      cancelled = true;
    };
  }, [path, reloadKey]);

  const handleSave = useCallback(async () => {
    if (!onSave || !dirty || writeDisabled || saving) return;
    setSaving(true);
    setSaveError(null);
    try {
      await onSave(content);
      setSavedContent(content);
    } catch (e) {
      setSaveError(e instanceof Error ? e.message : String(e));
    } finally {
      setSaving(false);
    }
  }, [content, dirty, onSave, saving, writeDisabled]);

  if (loading) {
    return (
      <div className="flex h-full items-center justify-center text-sm text-muted">
        Carregando arquivo…
      </div>
    );
  }

  if (error) {
    return (
      <div className="flex h-full items-center justify-center p-4 text-sm text-red-600 dark:text-red-400">
        {error}
      </div>
    );
  }

  return (
    <div className="flex h-full min-h-0 flex-col">
      <div className="flex shrink-0 items-center justify-between gap-2 border-b border-border px-3 py-1.5">
        <span className="text-[10px] text-muted">
          {saveError
            ? saveError
            : saving
              ? "Salvando…"
              : dirty
                ? "Alterações não salvas"
                : "Salvo no working tree"}
        </span>
        <button
          type="button"
          onClick={() => void handleSave()}
          disabled={!canSave}
          className="rounded border border-accent/40 bg-accent/10 px-2 py-0.5 text-[10px] font-medium text-accent hover:bg-accent/20 disabled:cursor-not-allowed disabled:opacity-40"
          title="Salvar no working tree (Ctrl+S)"
        >
          Salvar
        </button>
      </div>
      <textarea
        readOnly={writeDisabled || !onSave}
        spellCheck={false}
        className="min-h-0 flex-1 w-full resize-none border-0 bg-transparent p-3 font-mono text-xs leading-relaxed text-text focus:outline-none disabled:cursor-not-allowed disabled:opacity-70"
        value={content}
        onChange={(event) => {
          setSaveError(null);
          setContent(event.target.value);
        }}
        onKeyDown={(event) => {
          if ((event.ctrlKey || event.metaKey) && event.key.toLowerCase() === "s") {
            event.preventDefault();
            void handleSave();
          }
        }}
        aria-label={`Conteúdo de ${path}`}
      />
    </div>
  );
}
