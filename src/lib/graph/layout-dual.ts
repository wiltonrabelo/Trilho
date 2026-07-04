import type { CommitDto, TrailKindDto } from "@/types";
import { laneColor } from "./lane-colors";
import type { GraphEdge, GraphLayout, GraphNode } from "./types";

/** Trilho comum (antes da divergência) — esmaecido. Hex literal com alpha:
 *  var() em atributo SVG não é confiável em todos os WebViews. */
const SHARED_COLOR = "#7d859088";

/**
 * Layout da trilha dupla: lane 0 = branch atual, lane 1 = base (ex.:
 * development) e trilho comum. Edges seguem os pais reais presentes na
 * página, então a divergência e os merges de convergência aparecem como
 * curvas entre as duas lanes.
 */
export function layoutDual(
  commits: CommitDto[],
  trails: TrailKindDto[],
): GraphLayout {
  const rowById = new Map<string, number>();
  commits.forEach((c, i) => rowById.set(c.id, i));

  const laneOf = (trail: TrailKindDto): number => (trail === "current" ? 0 : 1);

  const nodes: GraphNode[] = commits.map((c, i) => {
    const trail = trails[i] ?? "current";
    return {
      commitId: c.id,
      lane: laneOf(trail),
      laneColor:
        trail === "current"
          ? laneColor(0)
          : trail === "base"
            ? laneColor(1)
            : SHARED_COLOR,
      isMerge: c.parentIds.length > 1,
    };
  });

  const edges: GraphEdge[] = [];
  commits.forEach((c, row) => {
    for (const parentId of c.parentIds) {
      const parentRow = rowById.get(parentId);
      if (parentRow === undefined) continue; // pai fora da página
      edges.push({
        fromCommitId: c.id,
        toCommitId: parentId,
        fromLane: nodes[row].lane,
        toLane: nodes[parentRow].lane,
        fromRow: row,
        toRow: parentRow,
        firstParent: parentId === c.parentIds[0],
      });
    }
  });

  return { nodes, edges, laneCount: 2, mode: "lanes" };
}
