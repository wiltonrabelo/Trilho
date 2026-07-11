import { useEffect, useMemo, useState } from "react";

import { listLocalBranches } from "@/lib/api";

const STORAGE_PREFIX = "trilho.trailBase.";

interface TrailBaseSelectorProps {
  repoPath: string | null;
  currentBranch: string | null;
  /** Base sugerida (origem inferida). */
  suggestedBase: string | null;
  value: string | null;
  onChange: (base: string | null) => void;
  disabled?: boolean;
}

function storageKey(repoPath: string): string {
  return `${STORAGE_PREFIX}${repoPath}`;
}

export function loadStoredTrailBase(repoPath: string | null): string | null {
  if (!repoPath || typeof localStorage === "undefined") return null;
  try {
    return localStorage.getItem(storageKey(repoPath));
  } catch {
    return null;
  }
}

export function TrailBaseSelector({
  repoPath,
  currentBranch,
  suggestedBase,
  value,
  onChange,
  disabled,
}: TrailBaseSelectorProps) {
  const [branches, setBranches] = useState<string[]>([]);

  useEffect(() => {
    if (!repoPath) {
      setBranches([]);
      return;
    }
    let cancelled = false;
    void listLocalBranches()
      .then((list) => {
        if (!cancelled) setBranches(list);
      })
      .catch(() => {
        if (!cancelled) setBranches([]);
      });
    return () => {
      cancelled = true;
    };
  }, [repoPath]);

  const options = useMemo(() => {
    const set = new Set(branches);
    if (suggestedBase) set.add(suggestedBase);
    if (value) set.add(value);
    return [...set]
      .filter((b) => b && b !== currentBranch)
      .sort((a, b) => a.localeCompare(b));
  }, [branches, suggestedBase, value, currentBranch]);

  if (disabled || options.length === 0) return null;

  return (
    <label className="flex items-center gap-1.5 text-[11px] text-muted">
      <span className="shrink-0">Comparar com</span>
      <select
        className="max-w-[10rem] rounded border border-border bg-surface px-1.5 py-0.5 text-[11px] text-text"
        value={value ?? ""}
        onChange={(e) => {
          const next = e.target.value || null;
          onChange(next);
          if (repoPath) {
            try {
              if (next) localStorage.setItem(storageKey(repoPath), next);
              else localStorage.removeItem(storageKey(repoPath));
            } catch {
              /* ignore */
            }
          }
        }}
        title="Branch base da trilha comparada (segunda lane)"
      >
        <option value="">
          {suggestedBase ? `Auto (${suggestedBase})` : "—"}
        </option>
        {options.map((b) => (
          <option key={b} value={b}>
            {b}
            {b === suggestedBase ? " · sugerida" : ""}
          </option>
        ))}
      </select>
    </label>
  );
}
