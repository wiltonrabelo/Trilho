import type { CommitDto } from "@/types";
import { layoutLinear } from "./layout-linear";
import { laneColor } from "./lane-colors";
import type { GraphEdge, GraphLayout, GraphNode } from "./types";

const MAX_LANES = 12;

/**
 * Layout estilo gitgraph: atribui colunas (lanes) por first-parent;
 * merges abrem lanes adicionais para os outros pais.
 * Commits em ordem newest-first (topológica reversa).
 */
export function layoutLanes(commits: CommitDto[]): GraphLayout {
  if (commits.length === 0) {
    return { nodes: [], edges: [], laneCount: 0, mode: "lanes" };
  }

  const idSet = new Set(commits.map((c) => c.id));
  const rowIndex = new Map(commits.map((c, i) => [c.id, i]));

  /** Próximo commit esperado em cada coluna. */
  const lanes: (string | null)[] = [];
  const nodes: GraphNode[] = [];

  for (const commit of commits) {
    let lane = lanes.indexOf(commit.id);
    if (lane === -1) {
      lane = lanes.indexOf(null);
      if (lane === -1) {
        lane = lanes.length;
        lanes.push(null);
      }
    } else {
      lanes[lane] = null;
    }

    const isMerge = commit.parentIds.length > 1;
    nodes.push({
      commitId: commit.id,
      lane,
      laneColor: laneColor(lane),
      isMerge,
    });

    const parents = commit.parentIds.filter((p) => idSet.has(p));
    if (parents.length > 0) {
      lanes[lane] = parents[0];
    }
    for (let i = 1; i < parents.length; i++) {
      let extraLane = lanes.indexOf(null);
      if (extraLane === -1) {
        extraLane = lanes.length;
        lanes.push(null);
      }
      lanes[extraLane] = parents[i];
    }
  }

  const rawLaneCount = Math.max(1, ...nodes.map((n) => n.lane + 1));
  // Histórico muito ramificado (muitas lanes concorrentes): em vez de colapsar
  // tudo numa única coluna — o que fazia o grafo "voltar para a trilha da
  // branch" ao carregar mais páginas —, comprimimos as lanes excedentes na
  // última coluna. O grafo continua multi-trilha e estável entre páginas.
  if (rawLaneCount > MAX_LANES) {
    const cap = MAX_LANES - 1;
    for (const n of nodes) {
      if (n.lane > cap) {
        n.lane = cap;
        n.laneColor = laneColor(cap);
      }
    }
  }
  const laneCount = Math.min(rawLaneCount, MAX_LANES);

  const laneById = new Map(nodes.map((n) => [n.commitId, n.lane]));
  const edges: GraphEdge[] = [];

  for (const commit of commits) {
    const fromLane = laneById.get(commit.id)!;
    const fromRow = rowIndex.get(commit.id)!;
    for (const parentId of commit.parentIds) {
      if (!idSet.has(parentId)) continue;
      const toLane = laneById.get(parentId)!;
      const toRow = rowIndex.get(parentId)!;
      if (toRow <= fromRow) continue;
      edges.push({
        fromCommitId: commit.id,
        toCommitId: parentId,
        fromLane,
        toLane,
        fromRow,
        toRow,
        firstParent: parentId === commit.parentIds[0],
      });
    }
  }

  const hasBranching = laneCount > 1 || nodes.some((n) => n.isMerge);
  if (!hasBranching && laneCount === 1) {
    return layoutLinear(commits);
  }

  return { nodes, edges, laneCount, mode: "lanes" };
}
