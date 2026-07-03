import { describe, expect, it } from "vitest";
import { parseUnifiedDiff } from "@/lib/diff-parse";

const SAMPLE = `diff --git a/src/foo.ts b/src/foo.ts
index abc..def 100644
--- a/src/foo.ts
+++ b/src/foo.ts
@@ -1,4 +1,4 @@
 context
-removed line
+added line
 keep
`;

describe("parseUnifiedDiff", () => {
  it("parseia hunks unified em linhas lado a lado", () => {
    const result = parseUnifiedDiff(SAMPLE);
    expect(result.files).toHaveLength(1);
    expect(result.files[0].newPath).toBe("src/foo.ts");
    expect(result.files[0].rows).toHaveLength(4);
    expect(result.files[0].rows[1].left.kind).toBe("remove");
    expect(result.files[0].rows[1].right.kind).toBe("empty");
    expect(result.files[0].rows[2].left.kind).toBe("empty");
    expect(result.files[0].rows[2].right.kind).toBe("add");
  });

  it("retorna fallback para texto não-diff", () => {
    const result = parseUnifiedDiff("commit message only");
    expect(result.files).toHaveLength(0);
    expect(result.rawFallback).toBe("commit message only");
  });
});
