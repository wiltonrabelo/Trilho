import { useEffect, useMemo, useState } from "react";
import { Check, Columns2, GitMerge } from "lucide-react";

import { ConflictLineView, ConflictSideBySideView } from "@/components/ConflictLineView";
import { getConflictFile } from "@/lib/api";
import type { ConflictFileViewDto, ConflictRegionDto } from "@/types";

type Choice = "ours" | "theirs" | "both" | "bothTheirsFirst" | "custom";

interface ConflictResolverProps {
  path: string;
  /** Tela cheia — layout lado a lado estilo diff. */
  expanded?: boolean;
  operationKind?: "revert" | "merge" | "cherryPick" | null;
  writeDisabled?: boolean;
  onResolveSide: (side: "ours" | "theirs") => void;
  onResolveContent: (content: string) => void;
}

function regionPreview(region: ConflictRegionDto, choice: Choice, custom: string): string {
  switch (choice) {
    case "ours":
      return region.ours;
    case "theirs":
      return region.theirs;
    case "both":
      return region.ours + region.theirs;
    case "bothTheirsFirst":
      return region.theirs + region.ours;
    case "custom":
      return custom;
  }
}

export function ConflictResolver({
  path,
  expanded = false,
  operationKind,
  writeDisabled,
  onResolveSide,
  onResolveContent,
}: ConflictResolverProps) {
  const [view, setView] = useState<ConflictFileViewDto | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [choices, setChoices] = useState<Choice[]>([]);
  const [customs, setCustoms] = useState<string[]>([]);
  const [active, setActive] = useState(0);

  useEffect(() => {
    let cancelled = false;
    setLoading(true);
    setError(null);
    void getConflictFile(path)
      .then((v) => {
        if (cancelled) return;
        setView(v);
        const n = v.regions.filter((r) => r.kind === "conflict").length;
        setChoices(Array.from({ length: n }, () => "ours"));
        setCustoms(
          Array.from({ length: n }, (_, i) => {
            const conflicts = v.regions.filter((r) => r.kind === "conflict");
            return conflicts[i]?.ours ?? "";
          }),
        );
        setActive(0);
      })
      .catch((e) => {
        if (!cancelled) setError(String(e));
      })
      .finally(() => {
        if (!cancelled) setLoading(false);
      });
    return () => {
      cancelled = true;
    };
  }, [path]);

  const conflictRegions = useMemo(
    () => view?.regions.filter((r) => r.kind === "conflict") ?? [],
    [view],
  );

  const preview = useMemo(() => {
    if (!view) return "";
    let choiceI = 0;
    let out = "";
    for (const region of view.regions) {
      if (region.kind !== "conflict") {
        out += region.text;
        continue;
      }
      out += regionPreview(
        region,
        choices[choiceI] ?? "ours",
        customs[choiceI] ?? "",
      );
      choiceI += 1;
    }
    return out;
  }, [view, choices, customs]);

  const baseContent = view?.base.available ? view.base.content : "";

  function buildPreviewForAllBlocks(choice: Choice): string {
    if (!view) return "";
    let choiceI = 0;
    let out = "";
    for (const region of view.regions) {
      if (region.kind !== "conflict") {
        out += region.text;
        continue;
      }
      out += regionPreview(region, choice, customs[choiceI] ?? "");
      choiceI += 1;
    }
    return out;
  }

  function acceptBothFile() {
    onResolveContent(buildPreviewForAllBlocks("both"));
  }

  if (loading) {
    return (
      <div className="flex h-full items-center justify-center text-sm text-muted">
        Carregando conflito…
      </div>
    );
  }

  if (error || !view) {
    return (
      <div className="flex h-full items-center justify-center p-4 text-sm text-red-500">
        {error ?? "Não foi possível carregar o conflito."}
      </div>
    );
  }

  const current = conflictRegions[active];

  return (
    <div className="flex h-full min-h-0 flex-col overflow-hidden">
      <div className="flex shrink-0 flex-wrap items-center gap-2 border-b border-border px-3 py-2">
        <GitMerge size={14} className="text-amber-600" />
        <span className="text-xs font-medium">
          {view.conflictCount > 0
            ? `${view.conflictCount} bloco(s) em conflito`
            : "Conflito sem marcadores"}
        </span>
        <div className="ml-auto flex flex-wrap gap-1.5">
          <button
            type="button"
            disabled={writeDisabled}
            onClick={() => onResolveSide("ours")}
            className="rounded border border-border px-2 py-0.5 text-[10px] text-text hover:bg-surface disabled:opacity-50"
            title="Aceita o lado atual (HEAD) no arquivo inteiro"
          >
            Aceitar atual (arquivo)
          </button>
          <button
            type="button"
            disabled={writeDisabled}
            onClick={() => onResolveSide("theirs")}
            className="rounded border border-border px-2 py-0.5 text-[10px] text-text hover:bg-surface disabled:opacity-50"
            title="Aceita o lado entrando no arquivo inteiro"
          >
            Aceitar entrando (arquivo)
          </button>
          <button
            type="button"
            disabled={writeDisabled || view.conflictCount === 0}
            onClick={acceptBothFile}
            className="rounded border border-border px-2 py-0.5 text-[10px] text-text hover:bg-surface disabled:opacity-50"
            title="Mantém atual e entrando em cada bloco (atual → entrando) e marca o arquivo como resolvido"
          >
            Aceitar ambos (arquivo)
          </button>
          <button
            type="button"
            disabled={writeDisabled || preview.includes("<<<<<<<")}
            onClick={() => onResolveContent(preview)}
            className="flex items-center gap-1 rounded border border-accent/40 bg-accent/10 px-2 py-0.5 text-[10px] text-accent hover:bg-accent/20 disabled:opacity-50"
          >
            <Check size={12} />
            Marcar resolvido
          </button>
        </div>
      </div>
      {operationKind === "revert" && (
        <p className="shrink-0 border-b border-amber-500/30 bg-amber-500/10 px-3 py-1.5 text-[10px] leading-snug text-amber-800 dark:text-amber-200">
          Em um revert, «Aceitar entrando» aplica o desfazimento; «Aceitar atual» mantém o arquivo
          como está (o revert não altera nada).
        </p>
      )}

      {conflictRegions.length > 0 && (
        <div className="flex shrink-0 items-center gap-2 border-b border-border px-3 py-1.5">
          <Columns2 size={12} className="text-muted" />
          <span className="text-[10px] text-muted">Bloco</span>
          {conflictRegions.map((_, i) => (
            <button
              key={i}
              type="button"
              onClick={() => setActive(i)}
              className={`rounded px-2 py-0.5 text-[10px] font-medium ${
                active === i
                  ? "bg-accent text-white"
                  : "text-muted hover:bg-surface hover:text-text"
              }`}
            >
              {i + 1}
            </button>
          ))}
        </div>
      )}

      {/* Corpo rolável — comparação, resultado e base no mesmo scroll */}
      <div className="min-h-0 flex-1 overflow-y-auto overflow-x-hidden">
        {current && (
          <div className="space-y-3 border-b border-border px-3 py-3">
            <div className="flex flex-wrap gap-1.5">
              {(
                [
                  ["ours", "Atual"],
                  ["theirs", "Entrando"],
                  ["both", "Ambos (atual→entrando)"],
                  ["bothTheirsFirst", "Ambos (entrando→atual)"],
                  ["custom", "Editar"],
                ] as const
              ).map(([value, label]) => (
                <button
                  key={value}
                  type="button"
                  onClick={() =>
                    setChoices((prev) => {
                      const next = [...prev];
                      next[active] = value;
                      return next;
                    })
                  }
                  className={`rounded border px-2 py-0.5 text-[10px] ${
                    choices[active] === value
                      ? "border-accent bg-accent/15 text-accent"
                      : "border-border text-muted hover:bg-surface"
                  }`}
                >
                  {label}
                </button>
              ))}
            </div>

            <div
              className={
                expanded
                  ? "space-y-3"
                  : "grid grid-cols-1 gap-3 lg:grid-cols-2"
              }
            >
              {expanded ? (
                <ConflictSideBySideView
                  ours={current.ours}
                  theirs={current.theirs}
                  base={baseContent}
                />
              ) : (
                <>
                  <ConflictLineView
                    label="Atual (ours)"
                    content={current.ours}
                    base={baseContent}
                    side="ours"
                  />
                  <ConflictLineView
                    label="Entrando (theirs)"
                    content={current.theirs}
                    base={baseContent}
                    side="theirs"
                  />
                </>
              )}
            </div>

            {choices[active] === "custom" && (
              <textarea
                value={customs[active] ?? ""}
                onChange={(e) =>
                  setCustoms((prev) => {
                    const next = [...prev];
                    next[active] = e.target.value;
                    return next;
                  })
                }
                className="h-28 w-full resize-y rounded border border-border bg-bg px-2 py-1.5 font-mono text-xs text-text"
                spellCheck={false}
              />
            )}
          </div>
        )}

        <div className="space-y-2 px-3 py-3">
          <ConflictLineView
            label="Resultado"
            content={preview || "(vazio)"}
            plain
          />
          {view.base.available && (
            <details>
              <summary className="cursor-pointer text-[10px] text-muted">
                Ver ancestral comum (base)
              </summary>
              <div className="mt-2">
                <ConflictLineView
                  label="Base (ancestral comum)"
                  content={view.base.content}
                  plain
                />
              </div>
            </details>
          )}
        </div>
      </div>
    </div>
  );
}
