import type { CommitDto } from "@/types";
import { laneColor } from "./lane-colors";
import type { GraphEdge, GraphLayout, GraphNode } from "./types";

/**
 * Layout linear (M1): uma lane única; edges conectam commits adjacentes na lista.
 * A ordem da lista é a do revwalk (TIME), não topo-sort.
 */
export function layoutLinear(commits: CommitDto[]): GraphLayout {
  const nodes: GraphNode[] = commits.map((c) => ({
    commitId: c.id,
    lane: 0,
    laneColor: laneColor(0),
  }));

  const edges: GraphEdge[] = [];
  for (let i = 0; i < commits.length - 1; i++) {
    edges.push({
      fromCommitId: commits[i].id,
      toCommitId: commits[i + 1].id,
      fromLane: 0,
      toLane: 0,
      fromRow: i,
      toRow: i + 1,
    });
  }

  return { nodes, edges, laneCount: 1, mode: "linear" };
}
