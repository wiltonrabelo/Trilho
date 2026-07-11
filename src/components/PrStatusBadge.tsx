import { ChevronDown, GitPullRequest } from "lucide-react";
import { useEffect, useRef, useState } from "react";

import type { BranchPrStatusDto, PrSummaryDto } from "@/types";

interface PrStatusBadgeProps {
  status: BranchPrStatusDto | null;
  loading?: boolean;
}

function PrChip({
  label,
  url,
  className,
}: {
  label: string;
  url: string;
  className: string;
}) {
  return (
    <a
      href={url}
      target="_blank"
      rel="noopener noreferrer"
      className={`inline-flex items-center gap-1 rounded-md border px-2 py-0.5 text-[10px] font-medium hover:opacity-90 ${className}`}
      title={label}
    >
      <GitPullRequest size={11} />
      {label}
    </a>
  );
}

const OPEN_CLS =
  "border-emerald-500/40 bg-emerald-500/10 text-emerald-700 dark:text-emerald-400";
const MERGED_CLS =
  "border-violet-500/40 bg-violet-500/10 text-violet-700 dark:text-violet-300";
const CLOSED_CLS = "border-border bg-muted/20 text-muted";

type Chip = { key: string; label: string; url: string; className: string };

function truncatePrLabel(number: number, title: string): string {
  const base = `PR #${number}`;
  const t = title.trim();
  if (!t) return base;
  const max = 52;
  const room = max - base.length - 3;
  if (room <= 0 || t.length <= room) {
    return `${base} — ${t}`;
  }
  return `${base} — ${t.slice(0, room)}…`;
}

function buildChips(status: BranchPrStatusDto): Chip[] {
  const chips: Chip[] = [];
  for (const pr of status.open) {
    chips.push({
      key: `open-${pr.number}`,
      label: truncatePrLabel(pr.number, pr.title),
      url: pr.url,
      className: OPEN_CLS,
    });
  }
  for (const pr of status.merged) {
    chips.push({
      key: `merged-${pr.number}`,
      label: `PR mergeado #${pr.number}`,
      url: pr.url,
      className: MERGED_CLS,
    });
  }
  for (const pr of status.closed) {
    chips.push({
      key: `closed-${pr.number}`,
      label: `PR fechado #${pr.number}`,
      url: pr.url,
      className: CLOSED_CLS,
    });
  }
  return chips;
}

function menuLabel(status: BranchPrStatusDto): string {
  const parts: string[] = [];
  if (status.open.length) parts.push(`${status.open.length} aberto(s)`);
  if (status.merged.length) parts.push(`${status.merged.length} mergeado(s)`);
  if (status.closed.length) parts.push(`${status.closed.length} fechado(s)`);
  return parts.join(" · ") || "PRs";
}

function MenuSection({
  title,
  items,
  className,
  onPick,
}: {
  title: string;
  items: PrSummaryDto[];
  className: string;
  onPick: () => void;
}) {
  if (items.length === 0) return null;
  return (
    <div className="border-b border-border py-1 last:border-b-0">
      <p className="px-2 py-0.5 text-[9px] font-semibold uppercase tracking-wide text-muted">
        {title}
      </p>
      {items.map((pr) => (
        <a
          key={pr.number}
          href={pr.url}
          target="_blank"
          rel="noopener noreferrer"
          onClick={onPick}
          className={`flex items-start gap-1.5 px-2 py-1 text-[11px] hover:bg-bg ${className}`}
        >
          <GitPullRequest size={12} className="mt-0.5 shrink-0" />
          <span className="min-w-0">
            <span className="font-medium">#{pr.number}</span>{" "}
            <span className="text-muted line-clamp-2">{pr.title}</span>
          </span>
        </a>
      ))}
    </div>
  );
}

/** RF-12 r2 — com muitos PRs, menu em vez de chips inline. */
export function PrStatusBadge({ status, loading }: PrStatusBadgeProps) {
  const [open, setOpen] = useState(false);
  const rootRef = useRef<HTMLSpanElement>(null);

  useEffect(() => {
    if (!open) return;
    function onDoc(e: MouseEvent) {
      if (!rootRef.current?.contains(e.target as Node)) setOpen(false);
    }
    document.addEventListener("mousedown", onDoc);
    return () => document.removeEventListener("mousedown", onDoc);
  }, [open]);

  if (loading) {
    return <span className="text-[10px] text-muted">PR…</span>;
  }
  if (!status?.visible) return null;

  if (
    status.notice &&
    status.open.length === 0 &&
    status.merged.length === 0 &&
    status.closed.length === 0
  ) {
    return (
      <span
        className="text-[10px] text-amber-700 dark:text-amber-300"
        title={status.notice}
      >
        PR indisponível — {status.notice}
      </span>
    );
  }

  const chips = buildChips(status);
  if (chips.length === 0) return null;

  // Poucos PRs: chips inline; muitos: menu.
  if (chips.length <= 2) {
    return (
      <span className="inline-flex flex-wrap items-center gap-1">
        {chips.map((chip) => (
          <PrChip
            key={chip.key}
            label={chip.label}
            url={chip.url}
            className={chip.className}
          />
        ))}
      </span>
    );
  }

  return (
    <span ref={rootRef} className="relative inline-flex">
      <button
        type="button"
        onClick={() => setOpen((v) => !v)}
        className="inline-flex items-center gap-1 rounded-md border border-border bg-surface px-2 py-0.5 text-[10px] font-medium text-text hover:bg-bg"
        aria-expanded={open}
        aria-haspopup="menu"
        title={menuLabel(status)}
      >
        <GitPullRequest size={11} />
        {chips.length} PRs
        <ChevronDown size={10} />
      </button>
      {open && (
        <div
          role="menu"
          className="absolute left-0 top-full z-50 mt-1 max-h-64 w-64 overflow-auto rounded-lg border border-border bg-surface shadow-lg"
        >
          <MenuSection
            title="Abertos"
            items={status.open}
            className="text-emerald-700 dark:text-emerald-400"
            onPick={() => setOpen(false)}
          />
          <MenuSection
            title="Mergeados"
            items={status.merged}
            className="text-violet-700 dark:text-violet-300"
            onPick={() => setOpen(false)}
          />
          <MenuSection
            title="Fechados"
            items={status.closed}
            className="text-muted"
            onPick={() => setOpen(false)}
          />
        </div>
      )}
    </span>
  );
}
