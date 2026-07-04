/** Paths SVG para o grafo — curvas suaves estilo GitKraken / mockup. */

/**
 * Aresta filho → pai, majoritariamente vertical com uma dobra curta (estilo
 * Git Graph do VS Code): evita longas diagonais que atravessam várias linhas.
 *
 * `bendAtTop` decide onde acontece a transição de lane:
 *  - `true` (merges / 2º+ pai): sai já no topo da lane do filho e desce reto na
 *    lane do pai — a lane do pai está livre abaixo.
 *  - `false` (first-parent / branch-off): desce reto na lane do filho e só
 *    cruza para a lane do pai perto do fim — a lane do filho está livre acima.
 *
 * y2 deve ser maior que y1 (pai abaixo na lista).
 */
export function smoothEdgePath(
  x1: number,
  y1: number,
  x2: number,
  y2: number,
  bendAtTop = true,
): string {
  if (Math.abs(x1 - x2) < 0.5) {
    return `M ${x1} ${y1} L ${x2} ${y2}`;
  }

  const dy = y2 - y1;
  if (dy <= 0) {
    return `M ${x1} ${y1} L ${x2} ${y2}`;
  }

  const bend = Math.min(18, dy / 2);
  if (dy <= bend * 2 + 1) {
    const mid = (y1 + y2) / 2;
    return `M ${x1} ${y1} C ${x1} ${mid}, ${x2} ${mid}, ${x2} ${y2}`;
  }

  if (bendAtTop) {
    const joinY = y1 + bend * 2;
    return `M ${x1} ${y1} C ${x1} ${y1 + bend}, ${x2} ${y1 + bend}, ${x2} ${joinY} L ${x2} ${y2}`;
  }

  const splitY = y2 - bend * 2;
  return `M ${x1} ${y1} L ${x1} ${splitY} C ${x1} ${y2 - bend}, ${x2} ${y2 - bend}, ${x2} ${y2}`;
}

/** Trilho vertical contínuo na mesma lane entre duas linhas adjacentes. */
export function laneRailPath(
  x: number,
  y1: number,
  y2: number,
): string {
  return `M ${x} ${y1} L ${x} ${y2}`;
}

/**
 * Aresta em cotovelo (trilha dupla): desce vertical na lane do filho e só
 * dobra para a lane do pai nos últimos pixels — muitas conexões cruzadas
 * empilhadas continuam legíveis (as curvas em S longas viram uma "faixa").
 */
export function elbowEdgePath(
  x1: number,
  y1: number,
  x2: number,
  y2: number,
): string {
  if (Math.abs(x1 - x2) < 0.5 || y2 - y1 <= 28) {
    return smoothEdgePath(x1, y1, x2, y2);
  }
  const elbowY = y2 - 24;
  return [
    `M ${x1} ${y1}`,
    `L ${x1} ${elbowY}`,
    `C ${x1} ${y2 - 6}, ${x2} ${y2 - 18}, ${x2} ${y2}`,
  ].join(" ");
}
