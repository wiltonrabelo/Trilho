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
  /** Quando há PR aberto, a base do PR prevalece sobre a heurística. */
  prBaseBranch?: string | null;
  prNumber?: number | null;
}

export function BranchOriginBadge({
  origin,
  loading,
  prBaseBranch,
  prNumber,
}: BranchOriginBadgeProps) {
  if (loading) {
    return <span className="text-xs text-muted">origem…</span>;
  }
  if (!origin) return null;

  const candidate = prBaseBranch ?? origin.candidate;
  const confidence = prBaseBranch ? ("high" as const) : origin.confidence;
  const explanation = prBaseBranch
    ? `Base do PR aberto${prNumber ? ` #${prNumber}` : ""}: ${prBaseBranch}`
    : origin.explanation;

  return (
    <span
      className="ml-2 inline-flex items-center gap-1 rounded-md border border-border px-2 py-0.5 text-[10px]"
      title={explanation}
    >
      <span className="text-muted">origem</span>
      {candidate ? (
        <>
          <span className="font-medium">{candidate}</span>
          <span className={CONFIDENCE_CLASS[confidence]}>
            ({CONFIDENCE_LABEL[confidence]}
            {prBaseBranch ? " · PR" : ""})
          </span>
        </>
      ) : (
        <span className={CONFIDENCE_CLASS[confidence]}>
          {CONFIDENCE_LABEL[confidence]}
        </span>
      )}
    </span>
  );
}
