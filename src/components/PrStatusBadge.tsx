import { GitPullRequest } from "lucide-react";

import type { BranchPrStatusDto } from "@/types";

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

export function PrStatusBadge({ status, loading }: PrStatusBadgeProps) {
  if (loading) {
    return <span className="text-[10px] text-muted">PR…</span>;
  }
  if (!status?.visible) return null;

  if (status.notice && status.open.length === 0 && status.merged.length === 0 && status.closed.length === 0) {
    return (
      <span className="text-[10px] text-muted" title={status.notice}>
        PR indisponível
      </span>
    );
  }

  const chips: { key: string; label: string; url: string; className: string }[] = [];

  for (const pr of status.open) {
    chips.push({
      key: `open-${pr.number}`,
      label: `PR aberto #${pr.number}`,
      url: pr.url,
      className: "border-emerald-500/40 bg-emerald-500/10 text-emerald-700 dark:text-emerald-400",
    });
  }
  for (const pr of status.merged) {
    chips.push({
      key: `merged-${pr.number}`,
      label: `PR mergeado #${pr.number}`,
      url: pr.url,
      className: "border-violet-500/40 bg-violet-500/10 text-violet-700 dark:text-violet-300",
    });
  }
  for (const pr of status.closed) {
    chips.push({
      key: `closed-${pr.number}`,
      label: `PR fechado #${pr.number}`,
      url: pr.url,
      className: "border-border bg-muted/20 text-muted",
    });
  }

  if (chips.length === 0) return null;

  return (
    <span className="inline-flex flex-wrap items-center gap-1">
      {chips.map((chip) => (
        <PrChip key={chip.key} label={chip.label} url={chip.url} className={chip.className} />
      ))}
    </span>
  );
}
