import { useCallback, useEffect, useRef, useState } from "react";

interface ResizableRowsProps {
  top: React.ReactNode;
  bottom: React.ReactNode;
  /** Chave de persistência da altura no localStorage. */
  storageKey: string;
  /** Altura inicial (px) do painel de cima. */
  defaultTop?: number;
  minTop?: number;
  minBottom?: number;
  /** Teto opcional (px) do painel superior — útil em cherry-pick/revert. */
  topMaxHeight?: number;
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

/** Divide dois painéis empilhados com uma alça arrastável (redimensiona a
 *  altura). Espelha a lógica de `ResizableColumns`, mas no eixo vertical. */
export function ResizableRows({
  top,
  bottom,
  storageKey,
  defaultTop = 240,
  minTop = 120,
  minBottom = 120,
  topMaxHeight,
}: ResizableRowsProps) {
  const containerRef = useRef<HTMLDivElement>(null);
  const [topHeight, setTopHeight] = useState(() =>
    loadHeight(storageKey, defaultTop),
  );
  const dragging = useRef(false);
  const lastY = useRef(0);

  const clampTop = useCallback(
    (value: number, containerH: number) => {
      let max = Math.max(minTop, containerH - minBottom - 6);
      if (topMaxHeight !== undefined) {
        max = Math.min(max, topMaxHeight);
      }
      return Math.min(Math.max(value, minTop), max);
    },
    [minTop, minBottom, topMaxHeight],
  );

  useEffect(() => {
    if (topMaxHeight === undefined) return;
    const containerH = containerRef.current?.clientHeight ?? 0;
    if (containerH <= 0) return;
    setTopHeight((prev) => clampTop(prev, containerH));
  }, [topMaxHeight, clampTop]);

  useEffect(() => {
    localStorage.setItem(storageKey, String(topHeight));
  }, [storageKey, topHeight]);

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
      setTopHeight((prev) => {
        const containerH = containerRef.current?.clientHeight ?? 0;
        return clampTop(prev + delta, containerH);
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
  }, [minTop, minBottom, clampTop]);

  return (
    <div ref={containerRef} className="flex min-h-0 flex-1 flex-col overflow-hidden">
      <div
        className="min-h-0 shrink-0 overflow-hidden"
        style={{ height: topHeight }}
      >
        {top}
      </div>
      <div
        role="separator"
        aria-orientation="horizontal"
        aria-label="Redimensionar painel"
        onMouseDown={onMouseDown}
        className="group relative z-10 h-1 shrink-0 cursor-row-resize bg-border/60 hover:bg-accent/50 active:bg-accent"
      >
        <div className="absolute inset-x-0 -top-1 -bottom-1" />
      </div>
      <div className="min-h-0 flex-1 overflow-hidden">{bottom}</div>
    </div>
  );
}
