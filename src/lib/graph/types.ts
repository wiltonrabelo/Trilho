/** Modelo de layout do grafo (RF-01) — espelha PLANO §6 GraphNode. */

export interface GraphNode {
  commitId: string;
  lane: number;
  laneColor: string;
  isMerge?: boolean;
}

export interface GraphEdge {
  fromCommitId: string;
  toCommitId: string;
  fromLane: number;
  toLane: number;
  fromRow: number;
  toRow: number;
  /** Aresta para o primeiro pai (continuação da linha) vs. merge (2º+ pai). */
  firstParent?: boolean;
}

export interface GraphLayout {
  nodes: GraphNode[];
  edges: GraphEdge[];
  laneCount: number;
  /** Indica se o layout usa lanes reais ou fallback linear. */
  mode: "linear" | "lanes";
}

export { laneColor, LANE_COLORS } from "./lane-colors";
