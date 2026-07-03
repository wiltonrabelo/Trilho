import { describe, expect, it } from "vitest";
import { layoutLinear } from "@/lib/graph/layout-linear";
import { layoutLanes } from "@/lib/graph/layout-lanes";
import type { CommitDto } from "@/types";

const LINEAR: CommitDto[] = [
  {
    id: "aaa",
    shortId: "aaa",
    summary: "top",
    authorName: "A",
    authoredAt: "2026-07-03T10:00:00-03:00",
    isLocalOnly: false,
    parentIds: ["bbb"],
  },
  {
    id: "bbb",
    shortId: "bbb",
    summary: "middle",
    authorName: "A",
    authoredAt: "2026-07-02T10:00:00-03:00",
    isLocalOnly: false,
    parentIds: ["ccc"],
  },
  {
    id: "ccc",
    shortId: "ccc",
    summary: "root",
    authorName: "A",
    authoredAt: "2026-07-01T10:00:00-03:00",
    isLocalOnly: false,
    parentIds: [],
  },
];

const MERGE: CommitDto[] = [
  {
    id: "merge",
    shortId: "merge",
    summary: "merge feature",
    authorName: "A",
    authoredAt: "2026-07-04T10:00:00-03:00",
    isLocalOnly: false,
    parentIds: ["main", "feat"],
  },
  {
    id: "main",
    shortId: "main",
    summary: "main tip",
    authorName: "A",
    authoredAt: "2026-07-03T10:00:00-03:00",
    isLocalOnly: false,
    parentIds: ["base"],
  },
  {
    id: "feat",
    shortId: "feat",
    summary: "feature tip",
    authorName: "B",
    authoredAt: "2026-07-03T09:00:00-03:00",
    isLocalOnly: false,
    parentIds: ["base"],
  },
  {
    id: "base",
    shortId: "base",
    summary: "base",
    authorName: "A",
    authoredAt: "2026-07-01T10:00:00-03:00",
    isLocalOnly: false,
    parentIds: [],
  },
];

describe("layoutLinear", () => {
  it("atribui lane 0 a todos os commits", () => {
    const layout = layoutLinear(LINEAR);
    expect(layout.laneCount).toBe(1);
    expect(layout.mode).toBe("linear");
    expect(layout.nodes.every((n) => n.lane === 0)).toBe(true);
  });

  it("cria edges entre commits adjacentes na lista", () => {
    const layout = layoutLinear(LINEAR);
    expect(layout.edges).toHaveLength(2);
    expect(layout.edges[0]).toMatchObject({
      fromCommitId: "aaa",
      toCommitId: "bbb",
      fromRow: 0,
      toRow: 1,
    });
  });
});

describe("layoutLanes", () => {
  it("degrada para linear em histórico sem ramificações", () => {
    const layout = layoutLanes(LINEAR);
    expect(layout.mode).toBe("linear");
    expect(layout.laneCount).toBe(1);
  });

  it("abre lanes paralelas em merge", () => {
    const layout = layoutLanes(MERGE);
    expect(layout.mode).toBe("lanes");
    expect(layout.laneCount).toBeGreaterThanOrEqual(2);

    const mergeNode = layout.nodes.find((n) => n.commitId === "merge");
    const featNode = layout.nodes.find((n) => n.commitId === "feat");
    expect(mergeNode?.isMerge).toBe(true);
    expect(featNode?.lane).toBeGreaterThan(0);

    const parentEdges = layout.edges.filter((e) => e.toCommitId === "base");
    expect(parentEdges.length).toBeGreaterThanOrEqual(2);
  });

  it("liga filho ao pai com coordenadas de linha", () => {
    const layout = layoutLanes(MERGE);
    const toMain = layout.edges.find(
      (e) => e.fromCommitId === "merge" && e.toCommitId === "main",
    );
    expect(toMain).toMatchObject({ fromRow: 0, toRow: 1 });
  });
});
