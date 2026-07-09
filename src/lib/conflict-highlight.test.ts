import { describe, expect, it } from "vitest";

import { highlightAgainstBase, splitConflictLines } from "@/lib/conflict-highlight";

describe("conflict-highlight", () => {
  it("normaliza CRLF", () => {
    expect(splitConflictLines("a\r\nb")).toEqual(["a", "b"]);
  });

  it("destaca linhas diferentes da base", () => {
    const rows = highlightAgainstBase("linha 1\nlinha 2", "linha 1\nlinha 3");
    expect(rows[0]?.kind).toBe("context");
    expect(rows[1]?.kind).toBe("changed");
  });
});
