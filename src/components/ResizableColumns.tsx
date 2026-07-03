import { useCallback, useEffect, useRef, useState } from "react";

const STORAGE_KEY = "trilho.layout.v1";

interface LayoutSizes {
  left: number;
  right: number;
}

interface ResizableColumnsProps {
  left: React.ReactNode;
  center: React.ReactNode;
  right: React.ReactNode;
  defaultLeft?: number;
  defaultRight?: number;
  minLeft?: number;
  minRight?: number;
  minCenter?: number;
}

function loadSizes(fallback: LayoutSizes): LayoutSizes {
  try {
    const raw = localStorage.getItem(STORAGE_KEY);
    if (!raw) return fallback;
    const parsed = JSON.parse(raw) as LayoutSizes;
    if (
      typeof parsed.left === "number" &&
      typeof parsed.right === "number"
    ) {
      return parsed;
    }
  } catch {
    /* ignore */
  }
  return fallback;
}

function ResizeHandle({
  onDrag,
}: {
  onDrag: (deltaX: number) => void;
}) {
  const dragging = useRef(false);
  const lastX = useRef(0);

  const onMouseDown = useCallback((e: React.MouseEvent) => {
    e.preventDefault();
    dragging.current = true;
    lastX.current = e.clientX;
    document.body.style.cursor = "col-resize";
    document.body.style.userSelect = "none";
  }, []);

  useEffect(() => {
    function onMouseMove(e: MouseEvent) {
      if (!dragging.current) return;
      const delta = e.clientX - lastX.current;
      lastX.current = e.clientX;
      onDrag(delta);
    }

    function onMouseUp() {
      if (!dragging.current) return;
      dragging.current = false;
      document.body.style.cursor = "";
      document.body.style.userSelect = "";
    }

    window.addEventListener("mousemove", onMouseMove);
    window.addEventListener("mouseup", onMouseUp);
    return () => {
      window.removeEventListener("mousemove", onMouseMove);
      window.removeEventListener("mouseup", onMouseUp);
    };
  }, [onDrag]);

  return (
    <div
      role="separator"
      aria-orientation="vertical"
      aria-label="Redimensionar coluna"
      onMouseDown={onMouseDown}
      className="group relative z-10 w-1 shrink-0 cursor-col-resize bg-border/60 hover:bg-accent/50 active:bg-accent"
    >
      <div className="absolute inset-y-0 -left-1 -right-1" />
    </div>
  );
}

export function ResizableColumns({
  left,
  center,
  right,
  defaultLeft = 200,
  defaultRight = 320,
  minLeft = 160,
  minRight = 220,
  minCenter = 240,
}: ResizableColumnsProps) {
  const containerRef = useRef<HTMLDivElement>(null);
  const [sizes, setSizes] = useState<LayoutSizes>(() =>
    loadSizes({ left: defaultLeft, right: defaultRight }),
  );

  useEffect(() => {
    localStorage.setItem(STORAGE_KEY, JSON.stringify(sizes));
  }, [sizes]);

  const clampLeft = useCallback(
    (nextLeft: number, containerWidth: number) => {
      const maxLeft =
        containerWidth - sizes.right - minCenter - 8; /* 2 handles */
      return Math.min(Math.max(nextLeft, minLeft), Math.max(minLeft, maxLeft));
    },
    [sizes.right, minLeft, minCenter],
  );

  const clampRight = useCallback(
    (nextRight: number, containerWidth: number) => {
      const maxRight =
        containerWidth - sizes.left - minCenter - 8;
      return Math.min(
        Math.max(nextRight, minRight),
        Math.max(minRight, maxRight),
      );
    },
    [sizes.left, minRight, minCenter],
  );

  const onDragLeft = useCallback(
    (deltaX: number) => {
      const width = containerRef.current?.clientWidth ?? 0;
      if (!width) return;
      setSizes((prev) => ({
        ...prev,
        left: clampLeft(prev.left + deltaX, width),
      }));
    },
    [clampLeft],
  );

  const onDragRight = useCallback(
    (deltaX: number) => {
      const width = containerRef.current?.clientWidth ?? 0;
      if (!width) return;
      setSizes((prev) => ({
        ...prev,
        right: clampRight(prev.right - deltaX, width),
      }));
    },
    [clampRight],
  );

  return (
    <div ref={containerRef} className="flex min-h-0 flex-1 overflow-hidden">
      <aside
        className="flex h-full shrink-0 flex-col overflow-hidden bg-bg"
        style={{ width: sizes.left }}
      >
        {left}
      </aside>

      <ResizeHandle onDrag={onDragLeft} />

      <section className="flex min-w-0 flex-1 flex-col overflow-hidden">
        {center}
      </section>

      <ResizeHandle onDrag={onDragRight} />

      <section
        className="flex h-full shrink-0 flex-col overflow-hidden"
        style={{ width: sizes.right }}
      >
        {right}
      </section>
    </div>
  );
}
