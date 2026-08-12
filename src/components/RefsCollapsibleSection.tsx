import { ChevronDown, ChevronRight } from "lucide-react";

/** Seção recolhível do painel de refs, com ícone e contador. */
export function CollapsibleSection({
  title,
  icon,
  count,
  open,
  onToggle,
  children,
}: {
  title: string;
  icon: React.ReactNode;
  count: number;
  open: boolean;
  onToggle: () => void;
  children: React.ReactNode;
}) {
  return (
    <section className="flex min-h-0 flex-col">
      <button
        type="button"
        onClick={onToggle}
        aria-expanded={open}
        className="flex shrink-0 items-center gap-1.5 rounded-md px-1 py-1 text-[11px] font-medium uppercase tracking-wide text-muted hover:bg-surface hover:text-text"
      >
        {open ? <ChevronDown size={12} /> : <ChevronRight size={12} />}
        {icon}
        {title}
        <span className="ml-auto text-[10px] font-normal normal-case tracking-normal">
          {count}
        </span>
      </button>
      {open ? <div className="py-0.5">{children}</div> : null}
    </section>
  );
}
