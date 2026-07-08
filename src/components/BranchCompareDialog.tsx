import { ArrowLeftRight, Columns2, GitCompare, Rows3, X } from "lucide-react";
import { useCallback, useEffect, useMemo, useRef, useState } from "react";

import { DiffViewer, type DiffLayout } from "@/components/DiffViewer";
import { useDialogA11y } from "@/hooks/useDialogA11y";
import {
  getBranchFileDiff,
  listBranchDiffFiles,
  listOrderedCompareRefs,
} from "@/lib/api";
import type {
  BranchDiffFileDto,
  BranchDiffModeDto,
  RemoteBranchRefDto,
} from "@/types";

interface BranchCompareDialogProps {
  open: boolean;
  localBranches: string[];
  remoteBranches: RemoteBranchRefDto[];
  currentBranch?: string | null;
  onClose: () => void;
}

function kindLabel(kind: BranchDiffFileDto["kind"]): string {
  switch (kind) {
    case "added":
      return "A";
    case "deleted":
      return "D";
    case "renamed":
      return "R";
    default:
      return "M";
  }
}

function kindClass(kind: BranchDiffFileDto["kind"]): string {
  switch (kind) {
    case "added":
      return "text-emerald-600 dark:text-emerald-400";
    case "deleted":
      return "text-red-600 dark:text-red-400";
    case "renamed":
      return "text-amber-600 dark:text-amber-400";
    default:
      return "text-muted";
  }
}

export function BranchCompareDialog({
  open,
  localBranches,
  remoteBranches,
  currentBranch,
  onClose,
}: BranchCompareDialogProps) {
  const panelRef = useRef<HTMLDivElement>(null);
  useDialogA11y(open, onClose, panelRef);

  const baseRefs = useMemo(() => {
    const remotes = remoteBranches.map((r) => `${r.remote}/${r.branch}`);
    return [...localBranches, ...remotes];
  }, [localBranches, remoteBranches]);

  const [orderedRefs, setOrderedRefs] = useState<string[]>([]);
  const [left, setLeft] = useState("");
  const [right, setRight] = useState("");
  const [mode, setMode] = useState<BranchDiffModeDto>("mergeBase");
  const [layout, setLayout] = useState<DiffLayout>("sideBySide");
  const [files, setFiles] = useState<BranchDiffFileDto[]>([]);
  const [range, setRange] = useState("");
  const [selectedPath, setSelectedPath] = useState<string | null>(null);
  const [diff, setDiff] = useState<string | null>(null);
  const [loadingList, setLoadingList] = useState(false);
  const [loadingDiff, setLoadingDiff] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const allRefs = orderedRefs.length > 0 ? orderedRefs : baseRefs;

  useEffect(() => {
    if (!open) return;
    let cancelled = false;
    void listOrderedCompareRefs(baseRefs)
      .then((ordered) => {
        if (cancelled) return;
        setOrderedRefs(ordered.length > 0 ? ordered : baseRefs);
      })
      .catch(() => {
        if (!cancelled) setOrderedRefs(baseRefs);
      });
    return () => {
      cancelled = true;
    };
  }, [open, baseRefs]);

  useEffect(() => {
    if (!open || allRefs.length === 0) return;
    const leftInit =
      (currentBranch && allRefs.includes(currentBranch)
        ? currentBranch
        : allRefs[0]) ?? "";
    const rightInit = allRefs.find((r) => r !== leftInit) ?? allRefs[0] ?? "";
    setLeft(leftInit);
    setRight(rightInit);
    setMode("mergeBase");
    setLayout("sideBySide");
    setFiles([]);
    setRange("");
    setSelectedPath(null);
    setDiff(null);
    setError(null);
  }, [open, allRefs, currentBranch]);

  const loadList = useCallback(async () => {
    if (!left || !right || left === right) {
      setError("Escolha duas branches diferentes.");
      setFiles([]);
      return;
    }
    setLoadingList(true);
    setError(null);
    setSelectedPath(null);
    setDiff(null);
    try {
      const summary = await listBranchDiffFiles(left, right, mode);
      setFiles(summary.files);
      setRange(summary.range);
    } catch (e) {
      setFiles([]);
      setRange("");
      setError(e instanceof Error ? e.message : String(e));
    } finally {
      setLoadingList(false);
    }
  }, [left, right, mode]);

  useEffect(() => {
    if (!open || !left || !right) return;
    void loadList();
  }, [open, loadList, left, right]);

  useEffect(() => {
    if (!open || !selectedPath) {
      setDiff(null);
      return;
    }
    let cancelled = false;
    setLoadingDiff(true);
    void getBranchFileDiff(left, right, selectedPath, mode)
      .then((text) => {
        if (!cancelled) setDiff(text || "Sem alterações neste arquivo.");
      })
      .catch((e) => {
        if (!cancelled) {
          setDiff(null);
          setError(e instanceof Error ? e.message : String(e));
        }
      })
      .finally(() => {
        if (!cancelled) setLoadingDiff(false);
      });
    return () => {
      cancelled = true;
    };
  }, [open, selectedPath, left, right, mode]);

  if (!open) return null;

  const swap = () => {
    setLeft(right);
    setRight(left);
  };

  return (
    <div
      className="fixed inset-0 z-50 flex items-center justify-center bg-black/50 p-4"
      role="presentation"
      onMouseDown={(e) => {
        if (e.target === e.currentTarget) onClose();
      }}
    >
      <div
        ref={panelRef}
        role="dialog"
        aria-modal="true"
        aria-labelledby="branch-compare-title"
        className="flex h-[min(90vh,720px)] w-full max-w-5xl flex-col rounded-xl border border-border bg-surface shadow-lg"
      >
        <div className="flex shrink-0 items-center gap-2 border-b border-border px-4 py-3">
          <GitCompare size={18} className="text-accent" />
          <h2
            id="branch-compare-title"
            className="text-sm font-semibold text-text"
          >
            Comparar branches
          </h2>
          <button
            type="button"
            onClick={onClose}
            aria-label="Fechar"
            className="ml-auto rounded-md p-1 text-muted hover:bg-bg hover:text-text"
          >
            <X size={16} />
          </button>
        </div>

        <div className="flex shrink-0 flex-wrap items-end gap-2 border-b border-border px-4 py-3">
          <label className="flex min-w-[140px] flex-1 flex-col gap-1 text-[11px] text-muted">
            A (base)
            <select
              value={left}
              onChange={(e) => setLeft(e.target.value)}
              className="rounded-md border border-border bg-bg px-2 py-1.5 text-xs text-text"
            >
              {allRefs.map((r) => (
                <option key={`L-${r}`} value={r}>
                  {r}
                </option>
              ))}
            </select>
          </label>
          <button
            type="button"
            onClick={swap}
            title="Trocar A ↔ B"
            className="mb-0.5 rounded-md border border-border p-1.5 text-muted hover:bg-bg hover:text-text"
          >
            <ArrowLeftRight size={14} />
          </button>
          <label className="flex min-w-[140px] flex-1 flex-col gap-1 text-[11px] text-muted">
            B (destino)
            <select
              value={right}
              onChange={(e) => setRight(e.target.value)}
              className="rounded-md border border-border bg-bg px-2 py-1.5 text-xs text-text"
            >
              {allRefs.map((r) => (
                <option key={`R-${r}`} value={r}>
                  {r}
                </option>
              ))}
            </select>
          </label>
          <fieldset className="flex flex-col gap-1 text-[11px] text-muted">
            <legend className="sr-only">Modo de comparação</legend>
            <span>Comparação</span>
            <div className="flex gap-1">
              <button
                type="button"
                onClick={() => setMode("mergeBase")}
                className={`rounded-md border px-2 py-1.5 text-xs ${
                  mode === "mergeBase"
                    ? "border-accent/50 bg-accent/10 text-accent"
                    : "border-border text-muted hover:bg-bg"
                }`}
                title="O que B tem desde que divergiu de A (A...B)"
              >
                Merge-base
              </button>
              <button
                type="button"
                onClick={() => setMode("tips")}
                className={`rounded-md border px-2 py-1.5 text-xs ${
                  mode === "tips"
                    ? "border-accent/50 bg-accent/10 text-accent"
                    : "border-border text-muted hover:bg-bg"
                }`}
                title="Diferença direta entre as pontas (A..B)"
              >
                Pontas
              </button>
            </div>
          </fieldset>
          <fieldset className="flex flex-col gap-1 text-[11px] text-muted">
            <legend className="sr-only">Layout do diff</legend>
            <span>Layout</span>
            <div className="flex gap-1">
              <button
                type="button"
                onClick={() => setLayout("sideBySide")}
                className={`flex items-center gap-1 rounded-md border px-2 py-1.5 text-xs ${
                  layout === "sideBySide"
                    ? "border-accent/50 bg-accent/10 text-accent"
                    : "border-border text-muted hover:bg-bg"
                }`}
                title="Diff lado a lado"
              >
                <Columns2 size={12} />
                Lado a lado
              </button>
              <button
                type="button"
                onClick={() => setLayout("unified")}
                className={`flex items-center gap-1 rounded-md border px-2 py-1.5 text-xs ${
                  layout === "unified"
                    ? "border-accent/50 bg-accent/10 text-accent"
                    : "border-border text-muted hover:bg-bg"
                }`}
                title="Diff unificado"
              >
                <Rows3 size={12} />
                Unificado
              </button>
            </div>
          </fieldset>
        </div>

        <p className="shrink-0 px-4 py-1.5 text-[11px] text-muted">
          {mode === "mergeBase"
            ? "A...B — alterações em B desde o ancestral comum com A (padrão)."
            : "A..B — diferença direta entre as pontas das branches."}
          {range ? (
            <span className="ml-2 font-mono text-text/80">{range}</span>
          ) : null}
          {loadingList ? <span className="ml-2">Carregando…</span> : null}
        </p>

        {error ? (
          <p className="mx-4 mb-2 shrink-0 rounded-md border border-red-500/40 bg-red-500/10 px-3 py-2 text-xs text-red-600 dark:text-red-400">
            {error}
          </p>
        ) : null}

        <div className="flex min-h-0 flex-1">
          <div className="w-64 shrink-0 overflow-auto border-r border-border">
            {files.length === 0 && !loadingList ? (
              <p className="p-3 text-xs text-muted">Nenhum arquivo diferente.</p>
            ) : (
              <ul className="py-1">
                {files.map((f) => (
                  <li key={f.path}>
                    <button
                      type="button"
                      onClick={() => setSelectedPath(f.path)}
                      className={`flex w-full items-start gap-1.5 px-3 py-1.5 text-left text-xs hover:bg-bg ${
                        selectedPath === f.path ? "bg-accent/10" : ""
                      }`}
                    >
                      <span
                        className={`mt-0.5 w-3 shrink-0 font-mono font-semibold ${kindClass(f.kind)}`}
                      >
                        {kindLabel(f.kind)}
                      </span>
                      <span className="min-w-0 flex-1 break-all font-mono text-text">
                        {f.path}
                      </span>
                      <span className="shrink-0 font-mono text-[10px] text-muted">
                        +{f.additions}/−{f.deletions}
                      </span>
                    </button>
                  </li>
                ))}
              </ul>
            )}
          </div>
          <div className="min-w-0 flex-1">
            {selectedPath ? (
              <DiffViewer
                diff={diff}
                loading={loadingDiff}
                layout={layout}
              />
            ) : (
              <div className="flex h-full items-center justify-center text-sm text-muted">
                Selecione um arquivo
              </div>
            )}
          </div>
        </div>
      </div>
    </div>
  );
}
