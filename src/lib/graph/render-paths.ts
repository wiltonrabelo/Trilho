/** Paths SVG para o grafo — curvas suaves estilo GitKraken / mockup. */

/**
 * Aresta filho → pai com curva em S (evita cantos retos de 90°).
 * y2 deve ser maior que y1 (pai abaixo na lista).
 */
export function smoothEdgePath(
  x1: number,
  y1: number,
  x2: number,
  y2: number,
): string {
  if (Math.abs(x1 - x2) < 0.5) {
    return `M ${x1} ${y1} L ${x2} ${y2}`;
  }

  const dy = y2 - y1;
  if (dy <= 0) {
    return `M ${x1} ${y1} L ${x2} ${y2}`;
  }

  const stem = Math.min(22, dy * 0.28);
  const bendY = y1 + stem;
  const joinY = y2 - stem;

  if (joinY <= bendY + 4) {
    const mid = (y1 + y2) / 2;
    return `M ${x1} ${y1} C ${x1} ${mid}, ${x2} ${mid}, ${x2} ${y2}`;
  }

  const midBand = (bendY + joinY) / 2;
  return [
    `M ${x1} ${y1}`,
    `L ${x1} ${bendY}`,
    `C ${x1} ${midBand}, ${x2} ${midBand}, ${x2} ${joinY}`,
    `L ${x2} ${y2}`,
  ].join(" ");
}

/** Trilho vertical contínuo na mesma lane entre duas linhas adjacentes. */
export function laneRailPath(
  x: number,
  y1: number,
  y2: number,
): string {
  return `M ${x} ${y1} L ${x} ${y2}`;
}
