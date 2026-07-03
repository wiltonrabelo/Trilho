/** Paleta de lanes (PLANO §8) — pastéis distintos do acento violeta. */
export const LANE_COLORS = [
  "#9b8afb",
  "#56d4a0",
  "#f0a84d",
  "#6ec8e8",
  "#f07178",
  "#c792ea",
  "#e2b340",
  "#82aaff",
] as const;

export function laneColor(lane: number): string {
  return LANE_COLORS[lane % LANE_COLORS.length];
}
