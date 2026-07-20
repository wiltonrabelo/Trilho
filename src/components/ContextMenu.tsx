import { useEffect, useLayoutEffect, useRef } from "react";

export interface ContextMenuItem {
  id: string;
  label: string;
  disabled?: boolean;
  primary?: boolean;
  separatorBefore?: boolean;
  onSelect: () => void;
}

interface ContextMenuProps {
  x: number;
  y: number;
  title?: string;
  emptyLabel?: string;
  ariaLabel: string;
  items: ContextMenuItem[];
  onClose: () => void;
}

/** Menu flutuante genérico (commit, arquivo, etc.). */
export function ContextMenu({
  x,
  y,
  title,
  emptyLabel = "Nenhuma ação disponível.",
  ariaLabel,
  items,
  onClose,
}: ContextMenuProps) {
  const panelRef = useRef<HTMLDivElement>(null);

  useLayoutEffect(() => {
    const el = panelRef.current;
    if (!el) return;
    const pad = 8;
    const rect = el.getBoundingClientRect();
    let left = x;
    let top = y;
    if (left + rect.width > window.innerWidth - pad) {
      left = Math.max(pad, window.innerWidth - rect.width - pad);
    }
    if (top + rect.height > window.innerHeight - pad) {
      top = Math.max(pad, window.innerHeight - rect.height - pad);
    }
    el.style.left = `${left}px`;
    el.style.top = `${top}px`;
  }, [x, y, items.length, title]);

  useEffect(() => {
    function onKey(e: KeyboardEvent) {
      if (e.key === "Escape") {
        e.preventDefault();
        onClose();
      }
    }
    function onPointerDown(e: MouseEvent) {
      if (panelRef.current?.contains(e.target as Node)) return;
      onClose();
    }
    function onScroll() {
      onClose();
    }
    document.addEventListener("keydown", onKey);
    document.addEventListener("mousedown", onPointerDown);
    window.addEventListener("scroll", onScroll, true);
    return () => {
      document.removeEventListener("keydown", onKey);
      document.removeEventListener("mousedown", onPointerDown);
      window.removeEventListener("scroll", onScroll, true);
    };
  }, [onClose]);

  return (
    <div
      ref={panelRef}
      role="menu"
      aria-label={ariaLabel}
      className="fixed z-[100] min-w-[12rem] max-w-[20rem] rounded-lg border border-border bg-surface py-1 shadow-lg"
      style={{ left: x, top: y }}
    >
      {title && (
        <p className="truncate border-b border-border px-3 py-1.5 font-mono text-[10px] text-muted">
          {title}
        </p>
      )}
      {items.length === 0 ? (
        <p className="px-3 py-2 text-[11px] text-muted">{emptyLabel}</p>
      ) : (
        <ul className="py-0.5">
          {items.map((item) => (
            <li key={item.id} role="none">
              {item.separatorBefore && (
                <div className="my-1 border-t border-border" role="separator" />
              )}
              <button
                type="button"
                role="menuitem"
                disabled={item.disabled}
                className={`flex w-full px-3 py-1.5 text-left text-xs transition-colors disabled:cursor-not-allowed disabled:opacity-40 ${
                  item.primary
                    ? "font-medium text-accent hover:bg-accent/10"
                    : "text-text hover:bg-bg"
                }`}
                onClick={() => {
                  if (item.disabled) return;
                  item.onSelect();
                  onClose();
                }}
              >
                {item.label}
              </button>
            </li>
          ))}
        </ul>
      )}
    </div>
  );
}
