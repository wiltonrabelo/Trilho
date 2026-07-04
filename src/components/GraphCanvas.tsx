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
import { WorkingCopyRow } from "./WorkingCopyRow";

export const GRAPH_ROW_HEIGHT = 68;
/** Linha densa do grafo completo (estilo Git Graph do VS Code). */
export const COMPACT_ROW_HEIGHT = 26;
const LANE_STEP = 26;
const COMPACT_LANE_STEP = 11;
const GUTTER_PAD = 10;
const LIST_PAD = 8;
const WORKING_COPY_ROW = 0;

export interface TrailDivergence {
  mergeBaseId: string;
  baseName: string;
}

interface GraphCanvasProps {
  commits: CommitDto[];
  selectedId: string | null;
  headId: string | null;
  linear?: boolean;
  trails?: TrailKindDto[] | null;
  divergence?: TrailDivergence | null;
  compact?: boolean;
  showWorkingCopy?: boolean;
  workingCopySelected?: boolean;
  changeCount?: number;
  stagedCount?: number;
  onSelectWorkingCopy?: () => void;
  onSelect: (commit: CommitDto) => void;
}

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
  showWorkingCopy = false,
  workingCopySelected = false,
  changeCount = 0,
  stagedCount = 0,
  onSelectWorkingCopy,
  onSelect,
}: GraphCanvasProps) {
  const metrics: GraphMetrics = compact
    ? { rowHeight: COMPACT_ROW_HEIGHT, laneStep: COMPACT_LANE_STEP, nodeRadius: 5 }
    : { rowHeight: GRAPH_ROW_HEIGHT, laneStep: LANE_STEP, nodeRadius: 5 };
  const rowOffset = showWorkingCopy ? 1 : 0;
  const dual = linear && trails != null && trails.length === commits.length;
  const layout = useMemo(() => {
    if (dual) return layoutDual(commits, trails as TrailKindDto[]);
    return linear ? layoutLinear(commits) : layoutLanes(commits);
  }, [commits, linear, trails, dual]);
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
          rowOffset={rowOffset}
          selectedId={selectedId}
          headId={headId}
          gutterWidth={gutterWidth}
          divergenceRow={divergenceRow}
          dual={dual}
          metrics={metrics}
          showWorkingCopy={showWorkingCopy}
          workingCopySelected={workingCopySelected}
        />
      )}
      <ol
        className="relative z-[1] m-0 select-none list-none p-2"
        style={{ paddingLeft: gutterWidth }}
      >
        {showWorkingCopy && (
          <WorkingCopyRow
            changeCount={changeCount}
            stagedCount={stagedCount}
            selected={workingCopySelected}
            onSelect={() => onSelectWorkingCopy?.()}
            rowHeight={metrics.rowHeight}
            compact={compact}
          />
        )}
        {commits.map((commit, row) => {
          const node = nodeById.get(commit.id);
          return (
            <CommitRow
              key={commit.id}
              commit={commit}
              selected={!workingCopySelected && selectedId === commit.id}
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
  rowOffset,
  selectedId,
  headId,
  gutterWidth,
  divergenceRow = -1,
  dual = false,
  metrics,
  showWorkingCopy = false,
  workingCopySelected = false,
}: {
  layout: GraphLayout;
  rowCount: number;
  rowOffset: number;
  selectedId: string | null;
  headId: string | null;
  gutterWidth: number;
  divergenceRow?: number;
  dual?: boolean;
  metrics: GraphMetrics;
  showWorkingCopy?: boolean;
  workingCopySelected?: boolean;
}) {
  const laneX = (lane: number) =>
    GUTTER_PAD + lane * metrics.laneStep + metrics.laneStep / 2;
  const rowY = (visualRow: number) =>
    LIST_PAD + visualRow * metrics.rowHeight + metrics.rowHeight / 2;
  const toVisualRow = (layoutRow: number) => layoutRow + rowOffset;
  const r = metrics.nodeRadius;
  const nodeGap = r + 4;
  const nodeColor = (row: number, laneColor: string): string => {
    if (row === divergenceRow) return DIVERGENCE_COLOR;
    if (dual || divergenceRow < 0 || row < divergenceRow) return laneColor;
    return BASE_TRAIL_COLOR;
  };
  const totalRows = rowCount + rowOffset;
  const height =
    Math.max(totalRows * metrics.rowHeight, metrics.rowHeight) + LIST_PAD * 2;
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
        d: laneRailPath(
          x,
          rowY(toVisualRow(row)) + nodeGap,
          rowY(toVisualRow(row + 1)) - nodeGap,
        ),
      });
    }
  }

  const headNode = layout.nodes[0];
  const wcConnector =
    showWorkingCopy && headNode
      ? {
          x: laneX(headNode.lane),
          y1: rowY(WORKING_COPY_ROW) + r + 2,
          y2: rowY(toVisualRow(0)) - r - 2,
        }
      : null;

  return (
    <svg
      className="pointer-events-none absolute left-0 top-0 z-0"
      width={gutterWidth}
      height={height}
      aria-hidden
      role="presentation"
    >
      {wcConnector && (
        <path
          d={laneRailPath(wcConnector.x, wcConnector.y1, wcConnector.y2)}
          fill="none"
          stroke={headNode!.laneColor}
          strokeWidth={2}
          strokeLinecap="round"
          strokeOpacity={0.45}
          strokeDasharray="3 3"
        />
      )}

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
        const y1 = rowY(toVisualRow(edge.fromRow));
        const y2 = rowY(toVisualRow(edge.toRow));
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

      {showWorkingCopy && headNode && (
        <g>
          {workingCopySelected && (
            <circle
              cx={laneX(headNode.lane)}
              cy={rowY(WORKING_COPY_ROW)}
              r={r + 5}
              fill="rgb(var(--accent))"
              fillOpacity={0.12}
            />
          )}
          <circle
            cx={laneX(headNode.lane)}
            cy={rowY(WORKING_COPY_ROW)}
            r={workingCopySelected ? r + 1 : r}
            fill="rgb(var(--surface))"
            stroke="rgb(var(--accent))"
            strokeWidth={workingCopySelected ? 2 : 1.5}
            strokeDasharray="3 2"
          />
        </g>
      )}

      {layout.nodes.map((node) => {
        const row = rowOf(node.commitId);
        const selected = !workingCopySelected && selectedId === node.commitId;
        const isHead = headId === node.commitId;
        const cx = laneX(node.lane);
        const cy = rowY(toVisualRow(row));
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
