import { useMemo } from "react";
import type { CommitDto } from "@/types";
import {
  layoutLanes,
  laneRailPath,
  smoothEdgePath,
  type GraphLayout,
} from "@/lib/graph";
import { CommitRow } from "./CommitRow";

export const GRAPH_ROW_HEIGHT = 68;
const LANE_STEP = 26;
const GUTTER_PAD = 14;

interface GraphCanvasProps {
  commits: CommitDto[];
  selectedId: string | null;
  headId: string | null;
  onSelect: (commit: CommitDto) => void;
}

export function GraphCanvas({
  commits,
  selectedId,
  headId,
  onSelect,
}: GraphCanvasProps) {
  const layout = useMemo(() => layoutLanes(commits), [commits]);
  const nodeById = useMemo(
    () => new Map(layout.nodes.map((n) => [n.commitId, n])),
    [layout.nodes],
  );
  const useRail = layout.nodes.length > 0;

  const gutterWidth =
    GUTTER_PAD * 2 + Math.max(layout.laneCount, 1) * LANE_STEP;

  return (
    <div className="relative min-h-0 flex-1 overflow-auto">
      {useRail && (
        <LaneOverlay
          layout={layout}
          rowCount={commits.length}
          selectedId={selectedId}
          headId={headId}
          gutterWidth={gutterWidth}
        />
      )}
      <ol
        className="relative z-[1] m-0 select-none list-none p-2"
        style={{ paddingLeft: gutterWidth }}
      >
        {commits.map((commit) => {
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
              isMerge={node?.isMerge}
              rowHeight={GRAPH_ROW_HEIGHT}
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
}: {
  layout: GraphLayout;
  rowCount: number;
  selectedId: string | null;
  headId: string | null;
  gutterWidth: number;
}) {
  const height = Math.max(rowCount * GRAPH_ROW_HEIGHT, GRAPH_ROW_HEIGHT);
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
        color: upper.laneColor,
        d: laneRailPath(x, rowY(row) + 9, rowY(row + 1) - 9),
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
          strokeWidth={3}
          strokeLinecap="round"
          strokeOpacity={0.35}
        />
      ))}

      {layout.edges.map((edge, i) => {
        const x1 = laneX(edge.fromLane);
        const x2 = laneX(edge.toLane);
        const y1 = rowY(edge.fromRow);
        const y2 = rowY(edge.toRow);
        const childNode = nodeById.get(edge.fromCommitId);
        return (
          <path
            key={`${edge.fromCommitId}-${edge.toCommitId}-${i}`}
            d={smoothEdgePath(x1, y1 + 6, x2, y2 - 6)}
            fill="none"
            stroke={childNode?.laneColor ?? "rgb(var(--border))"}
            strokeWidth={2.5}
            strokeLinecap="round"
            strokeLinejoin="round"
            strokeOpacity={0.9}
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
                r={10}
                fill="rgb(var(--accent))"
                fillOpacity={0.12}
              />
            )}
            {isHead && (
              <circle
                cx={cx}
                cy={cy}
                r={9}
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
              r={selected ? 6 : 5}
              fill={node.laneColor}
              stroke={selected ? "rgb(var(--text))" : "rgb(var(--surface))"}
              strokeWidth={selected ? 2 : 1.5}
            />
          </g>
        );
      })}
    </svg>
  );
}

function laneX(lane: number): number {
  return GUTTER_PAD + lane * LANE_STEP + LANE_STEP / 2;
}

function rowY(row: number): number {
  return row * GRAPH_ROW_HEIGHT + GRAPH_ROW_HEIGHT / 2;
}
