import type { BranchOriginDto } from "@/types";

const CONFIDENCE_LABEL: Record<BranchOriginDto["confidence"], string> = {
  high: "Alta",
  medium: "Média",
  low: "Baixa",
  indeterminate: "Indeterminada",
};

const CONFIDENCE_CLASS: Record<BranchOriginDto["confidence"], string> = {
  high: "text-emerald-500",
  medium: "text-accent",
  low: "text-amber-500",
  indeterminate: "text-muted",
};

interface BranchOriginBadgeProps {
  origin: BranchOriginDto | null;
  loading?: boolean;
}

export function BranchOriginBadge({ origin, loading }: BranchOriginBadgeProps) {
  if (loading) {
    return <span className="text-xs text-muted">origem…</span>;
  }
  if (!origin) return null;

  return (
    <span
      className="ml-2 inline-flex items-center gap-1 rounded-md border border-border px-2 py-0.5 text-[10px]"
      title={origin.explanation}
    >
      <span className="text-muted">origem</span>
      {origin.candidate ? (
        <>
          <span className="font-medium">{origin.candidate}</span>
          <span className={CONFIDENCE_CLASS[origin.confidence]}>
            ({CONFIDENCE_LABEL[origin.confidence]})
          </span>
        </>
      ) : (
        <span className={CONFIDENCE_CLASS[origin.confidence]}>
          {CONFIDENCE_LABEL[origin.confidence]}
        </span>
      )}
    </span>
  );
}
