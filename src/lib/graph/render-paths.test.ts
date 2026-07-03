import { describe, expect, it } from "vitest";
import { smoothEdgePath } from "@/lib/graph/render-paths";

describe("smoothEdgePath", () => {
  it("gera linha reta na mesma lane", () => {
    expect(smoothEdgePath(20, 10, 20, 50)).toBe("M 20 10 L 20 50");
  });

  it("gera curva entre lanes (sem segmentos ortogonais puros)", () => {
    const d = smoothEdgePath(20, 10, 46, 50);
    expect(d).toContain("C");
    expect(d).not.toMatch(/L 20 \d+ L 46/);
  });
});
