import { describe, expect, it } from "vitest";

import { extractHunks } from "./diff-hunks";

describe("extractHunks", () => {
  it("extrai um hunk com cabeçalhos de arquivo", () => {
    const diff = `--- a/foo.txt
+++ b/foo.txt
@@ -1,3 +1,4 @@
 line1
-old
+new
 line3`;
    const hunks = extractHunks(diff);
    expect(hunks).toHaveLength(1);
    expect(hunks[0].header).toContain("@@");
    expect(hunks[0].patch).toContain("--- a/foo.txt");
    expect(hunks[0].patch).toContain("-old");
  });
});
