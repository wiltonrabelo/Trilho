import { describe, expect, it } from "vitest";

import type { WriteRequestDto } from "@/types";

import { writesMatch } from "./writeRequestMatch";

describe("writesMatch", () => {
  it("kinds diferentes nunca casam", () => {
    expect(
      writesMatch({ kind: "stage", path: "a.txt" }, { kind: "unstage", path: "a.txt" })
    ).toBe(false);
  });

  it("operações de arquivo único casam pelo path", () => {
    expect(
      writesMatch({ kind: "stage", path: "a.txt" }, { kind: "stage", path: "a.txt" })
    ).toBe(true);
    expect(
      writesMatch({ kind: "stage", path: "a.txt" }, { kind: "stage", path: "b.txt" })
    ).toBe(false);
  });

  it("operações em lote comparam a lista na ordem", () => {
    const a: WriteRequestDto = { kind: "stageMany", paths: ["a", "b"] };
    expect(writesMatch(a, { kind: "stageMany", paths: ["a", "b"] })).toBe(true);
    expect(writesMatch(a, { kind: "stageMany", paths: ["b", "a"] })).toBe(false);
    expect(writesMatch(a, { kind: "stageMany", paths: ["a"] })).toBe(false);
  });

  it("commit casa pelo resumo", () => {
    expect(
      writesMatch(
        { kind: "commit", summary: "fix: x", body: "" },
        { kind: "commit", summary: "fix: x", body: "outra" }
      )
    ).toBe(true);
  });

  it("revert casa pelo commit", () => {
    expect(
      writesMatch(
        { kind: "revert", commitId: "abc" },
        { kind: "revert", commitId: "abc" }
      )
    ).toBe(true);
    expect(
      writesMatch(
        { kind: "revert", commitId: "abc" },
        { kind: "revert", commitId: "def" }
      )
    ).toBe(false);
  });

  it("kinds sem regra própria caem na comparação estrutural", () => {
    expect(writesMatch({ kind: "stageAll" }, { kind: "stageAll" })).toBe(true);
  });
});
