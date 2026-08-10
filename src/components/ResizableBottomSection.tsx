import { useCallback, useEffect, useRef, useState } from "react";

interface ResizableBottomSectionProps {
  children: React.ReactNode;
  /** Persistência da altura no localStorage. */
  storageKey: string;
  /** Altura inicial (px). */
  defaultHeight?: number;
  minHeight?: number;
  /** Fração máxima da altura do container pai (0–1). */
  maxHeightRatio?: number;
  /** Rótulo acessível da alça. */
  ariaLabel?: string;
}

function loadHeight(key: string, fallback: number): number {
  try {
    const raw = localStorage.getItem(key);
    if (raw) {
      const n = Number(raw);
      if (Number.isFinite(n) && n > 0) return n;
    }
  } catch {
    /* ignore */
  }
  return fallback;
}

/**
 * Painel inferior com altura controlável pela alça no topo
 * (arrastar para cima aumenta; para baixo reduz).
 */
export function ResizableBottomSection({
  children,
  storageKey,
  defaultHeight = 160,
  minHeight = 96,
  maxHeightRatio = 0.55,
  ariaLabel = "Redimensionar painel de commit",
}: ResizableBottomSectionProps) {
  const containerRef = useRef<HTMLDivElement>(null);
  const [height, setHeight] = useState(() =>
    loadHeight(storageKey, defaultHeight),
  );
  const dragging = useRef(false);
  const lastY = useRef(0);

  const clampHeight = useCallback(
    (value: number, parentH: number) => {
      const max = Math.max(
        minHeight,
        Math.floor(parentH * maxHeightRatio),
      );
      return Math.min(Math.max(value, minHeight), max);
    },
    [minHeight, maxHeightRatio],
  );

  useEffect(() => {
    const parent = containerRef.current?.parentElement;
    if (!parent) return;
    const ro = new ResizeObserver(() => {
      const parentH = parent.clientHeight;
      if (parentH <= 0) return;
      setHeight((prev) => clampHeight(prev, parentH));
    });
    ro.observe(parent);
    return () => ro.disconnect();
  }, [clampHeight]);

  useEffect(() => {
    try {
      localStorage.setItem(storageKey, String(height));
    } catch {
      /* ignore */
    }
  }, [storageKey, height]);

  const onMouseDown = useCallback((e: React.MouseEvent) => {
    e.preventDefault();
    dragging.current = true;
    lastY.current = e.clientY;
    document.body.style.cursor = "row-resize";
    document.body.style.userSelect = "none";
  }, []);

  useEffect(() => {
    function onMove(e: MouseEvent) {
      if (!dragging.current) return;
      const delta = e.clientY - lastY.current;
      lastY.current = e.clientY;
      // Arrastar para cima (delta negativo) aumenta a altura do painel inferior.
      setHeight((prev) => {
        const parentH = containerRef.current?.parentElement?.clientHeight ?? 0;
        return clampHeight(prev - delta, parentH);
      });
    }
    function onUp() {
      if (!dragging.current) return;
      dragging.current = false;
      document.body.style.cursor = "";
      document.body.style.userSelect = "";
    }
    window.addEventListener("mousemove", onMove);
    window.addEventListener("mouseup", onUp);
    return () => {
      window.removeEventListener("mousemove", onMove);
      window.removeEventListener("mouseup", onUp);
    };
  }, [clampHeight]);

  return (
    <div ref={containerRef} className="flex shrink-0 flex-col">
      <div
        role="separator"
        aria-orientation="horizontal"
        aria-label={ariaLabel}
        aria-valuenow={Math.round(height)}
        onMouseDown={onMouseDown}
        title="Arraste para mostrar mais alterações ou mais espaço de commit"
        className="group relative z-10 h-1.5 shrink-0 cursor-row-resize border-t border-border bg-border/40 hover:bg-accent/50 active:bg-accent"
      >
        <div className="absolute inset-x-0 -top-1.5 -bottom-1.5" />
      </div>
      <div
        className="min-h-0 overflow-y-auto bg-surface"
        style={{ height }}
      >
        {children}
      </div>
    </div>
  );
}
