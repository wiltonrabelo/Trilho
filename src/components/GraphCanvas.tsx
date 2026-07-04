import { useMemo } from "react";
import type { CommitDto } from "@/types";
import type { TrailKindDto } from "@/types";
import {
  elbowEdgePath,
  layoutDual,
  layoutLanes,
  layoutLinear,
  laneRailPath,
  smoothEdgePath,
  type GraphLayout,
} from "@/lib/graph";
import { CommitRow } from "./CommitRow";

export const GRAPH_ROW_HEIGHT = 68;
/** Linha densa do grafo completo (estilo Git Graph do VS Code). */
export const COMPACT_ROW_HEIGHT = 26;
const LANE_STEP = 26;
// Lanes estreitas e compactadas à esquerda, como o grafo do VS Code.
const COMPACT_LANE_STEP = 11;
const GUTTER_PAD = 10;
/** Padding vertical da lista (`p-2`) — o SVG precisa do mesmo offset,
 *  senão os nós desalinham das linhas. */
const LIST_PAD = 8;

/** Divergência com a branch de origem (RF-02): separa visualmente os commits
 *  da branch atual (acima do merge-base) dos commits herdados da base. */
export interface TrailDivergence {
  mergeBaseId: string;
  baseName: string;
}

interface GraphCanvasProps {
  commits: CommitDto[];
  selectedId: string | null;
  headId: string | null;
  /** Trilha da branch atual (first-parent): lane única, linha reta. */
  linear?: boolean;
  /** Linha de cada commit na trilha dupla (paralelo a `commits`). */
  trails?: TrailKindDto[] | null;
  divergence?: TrailDivergence | null;
  /** Linhas densas (grafo completo). */
  compact?: boolean;
  onSelect: (commit: CommitDto) => void;
}

/** Métricas do desenho — variam entre visão confortável e compacta. */
interface GraphMetrics {
  rowHeight: number;
  laneStep: number;
  nodeRadius: number;
}

const BASE_TRAIL_COLOR = "#7d859088";
const DIVERGENCE_COLOR = "#D29922";

export function GraphCanvas({
  commits,
  selectedId,
  headId,
  linear = false,
  trails = null,
  divergence = null,
  compact = false,
  onSelect,
}: GraphCanvasProps) {
  // Nó no MESMO tamanho nas duas visões (pedido do stakeholder); o compacto
  // muda só densidade das linhas e largura das lanes.
  const metrics: GraphMetrics = compact
    ? { rowHeight: COMPACT_ROW_HEIGHT, laneStep: COMPACT_LANE_STEP, nodeRadius: 5 }
    : { rowHeight: GRAPH_ROW_HEIGHT, laneStep: LANE_STEP, nodeRadius: 5 };
  const dual = linear && trails != null && trails.length === commits.length;
  const layout = useMemo(() => {
    if (dual) return layoutDual(commits, trails as TrailKindDto[]);
    return linear ? layoutLinear(commits) : layoutLanes(commits);
  }, [commits, linear, trails, dual]);
  // Linha do ponto de divergência na Trilha (-1 = fora da página carregada).
  const divergenceRow = useMemo(
    () =>
      linear && divergence
        ? commits.findIndex((c) => c.id === divergence.mergeBaseId)
        : -1,
    [linear, divergence, commits],
  );
  const nodeById = useMemo(
    () => new Map(layout.nodes.map((n) => [n.commitId, n])),
    [layout.nodes],
  );
  const useRail = layout.nodes.length > 0;

  const gutterWidth =
    GUTTER_PAD * 2 + Math.max(layout.laneCount, 1) * metrics.laneStep;

  return (
    <div className="relative min-h-0 flex-1 overflow-auto">
      {useRail && (
        <LaneOverlay
          layout={layout}
          rowCount={commits.length}
          selectedId={selectedId}
          headId={headId}
          gutterWidth={gutterWidth}
          divergenceRow={divergenceRow}
          dual={dual}
          metrics={metrics}
        />
      )}
      <ol
        className="relative z-[1] m-0 select-none list-none p-2"
        style={{ paddingLeft: gutterWidth }}
      >
        {commits.map((commit, row) => {
          const node = nodeById.get(commit.id);
          return (
            <CommitRow
              key={commit.id}
              commit={commit}
              selected={selectedId === commit.id}
              isHead={headId === commit.id}
              onSelect={onSelect}
              showSpineBelow={false}
              showDot={false}
              isMerge={node?.isMerge ?? commit.parentIds.length > 1}
              rowHeight={metrics.rowHeight}
              compact={compact}
              divergenceBase={
                row === divergenceRow ? divergence?.baseName : undefined
              }
              onBaseTrail={
                dual
                  ? trails?.[row] === "shared" && row !== divergenceRow
                  : divergenceRow >= 0 && row > divergenceRow
              }
            />
          );
        })}
      </ol>
    </div>
  );
}

function LaneOverlay({
  layout,
  rowCount,
  selectedId,
  headId,
  gutterWidth,
  divergenceRow = -1,
  dual = false,
  metrics,
}: {
  layout: GraphLayout;
  rowCount: number;
  selectedId: string | null;
  headId: string | null;
  gutterWidth: number;
  divergenceRow?: number;
  dual?: boolean;
  metrics: GraphMetrics;
}) {
  const laneX = (lane: number) =>
    GUTTER_PAD + lane * metrics.laneStep + metrics.laneStep / 2;
  const rowY = (row: number) =>
    LIST_PAD + row * metrics.rowHeight + metrics.rowHeight / 2;
  const r = metrics.nodeRadius;
  const nodeGap = r + 4;
  // Ponto de divergência = âmbar. Na trilha simples (sem lane da base), os
  // commits abaixo dele ficam apagados; na dupla, cada lane mantém sua cor
  // (o layout já esmaece o trilho comum).
  const nodeColor = (row: number, laneColor: string): string => {
    if (row === divergenceRow) return DIVERGENCE_COLOR;
    if (dual || divergenceRow < 0 || row < divergenceRow) return laneColor;
    return BASE_TRAIL_COLOR;
  };
  const height =
    Math.max(rowCount * metrics.rowHeight, metrics.rowHeight) + LIST_PAD * 2;
  const nodeById = new Map(layout.nodes.map((n) => [n.commitId, n]));
  const rowOf = (id: string) =>
    layout.nodes.findIndex((n) => n.commitId === id);

  const railSegments: { d: string; color: string; key: string }[] = [];
  for (let row = 0; row < rowCount - 1; row++) {
    const upper = layout.nodes[row];
    const lower = layout.nodes[row + 1];
    if (!upper || !lower) continue;
    if (upper.lane === lower.lane) {
      const x = laneX(upper.lane);
      railSegments.push({
        key: `rail-${row}`,
        color: nodeColor(row + 1, upper.laneColor),
        d: laneRailPath(x, rowY(row) + nodeGap, rowY(row + 1) - nodeGap),
      });
    }
  }

  return (
    <svg
      className="pointer-events-none absolute left-0 top-0 z-0"
      width={gutterWidth}
      height={height}
      aria-hidden
      role="presentation"
    >
      {railSegments.map((seg) => (
        <path
          key={seg.key}
          d={seg.d}
          fill="none"
          stroke={seg.color}
          strokeWidth={2}
          strokeLinecap="round"
          strokeOpacity={0.55}
        />
      ))}

      {layout.edges.map((edge, i) => {
        const x1 = laneX(edge.fromLane);
        const x2 = laneX(edge.toLane);
        const y1 = rowY(edge.fromRow);
        const y2 = rowY(edge.toRow);
        const childNode = nodeById.get(edge.fromCommitId);
        const crossing = edge.fromLane !== edge.toLane;
        return (
          <path
            key={`${edge.fromCommitId}-${edge.toCommitId}-${i}`}
            d={
              dual
                ? elbowEdgePath(x1, y1 + r + 1, x2, y2 - r - 1)
                : smoothEdgePath(
                    x1,
                    y1 + r + 1,
                    x2,
                    y2 - r - 1,
                    !edge.firstParent,
                  )
            }
            fill="none"
            stroke={childNode?.laneColor ?? "rgb(var(--border))"}
            strokeWidth={dual && crossing ? 1.75 : 2}
            strokeLinecap="round"
            strokeLinejoin="round"
            strokeOpacity={dual && crossing ? 0.55 : 0.9}
          />
        );
      })}

      {layout.nodes.map((node) => {
        const row = rowOf(node.commitId);
        const selected = selectedId === node.commitId;
        const isHead = headId === node.commitId;
        const cx = laneX(node.lane);
        const cy = rowY(row);
        return (
          <g key={node.commitId}>
            {selected && (
              <circle
                cx={cx}
                cy={cy}
                r={r + 5}
                fill="rgb(var(--accent))"
                fillOpacity={0.12}
              />
            )}
            {isHead && (
              <circle
                cx={cx}
                cy={cy}
                r={r + 4}
                fill="none"
                stroke="rgb(var(--accent))"
                strokeWidth={1.75}
                strokeDasharray="4 3"
                strokeOpacity={0.9}
              />
            )}
            <circle
              cx={cx}
              cy={cy}
              r={selected || row === divergenceRow ? r + 1 : r}
              fill={nodeColor(row, node.laneColor)}
              stroke={selected ? "rgb(var(--text))" : "rgb(var(--surface))"}
              strokeWidth={selected ? 2 : 1.5}
            />
          </g>
        );
      })}
    </svg>
  );
}

