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

  it("divide hunk único quando há alterações distantes no mesmo bloco", () => {
    const diff = `--- a/Anotacoes.txt
+++ b/Anotacoes.txt
@@ -1,10 +1,10 @@
 linha 1
-linha 2
+linha 2_2
 linha 3
 linha 4
 linha 5
 linha 6
 linha 7
 linha 8
-linha 9
+linha 9_9
 linha 10`;
    const hunks = extractHunks(diff);
    expect(hunks).toHaveLength(2);
    expect(hunks[0].patch).toContain("-linha 2");
    expect(hunks[0].patch).not.toContain("-linha 9");
    expect(hunks[1].patch).toContain("-linha 9");
    expect(hunks[1].patch).not.toContain("-linha 2");
    expect(hunks[0].header).toMatch(/@@ -\d+,\d+ \+\d+,\d+ @@/);
    expect(hunks[1].header).toMatch(/@@ -\d+,\d+ \+\d+,\d+ @@/);
  });
});
