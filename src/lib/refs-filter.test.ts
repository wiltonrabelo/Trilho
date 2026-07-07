import { describe, expect, it } from "vitest";

import {
  filterBranches,
  filterRemoteBranches,
  groupByRemote,
} from "./refs-filter";

describe("refs-filter", () => {
  it("filtra branches locais por substring", () => {
    const branches = ["main", "main_teste_3", "feature/foo"];
    expect(filterBranches(branches, "main")).toEqual(["main", "main_teste_3"]);
    expect(filterBranches(branches, "FOO")).toEqual(["feature/foo"]);
  });

  it("filtra remotos por nome completo ou remoto", () => {
    const refs = [
      { remote: "origin", branch: "main" },
      { remote: "upstream", branch: "develop" },
    ];
    expect(filterRemoteBranches(refs, "origin")).toEqual([
      { remote: "origin", branch: "main" },
    ]);
    expect(filterRemoteBranches(refs, "develop")).toEqual([
      { remote: "upstream", branch: "develop" },
    ]);
  });

  it("agrupa remotos por remoto e ordena", () => {
    const refs = [
      { remote: "origin", branch: "zeta" },
      { remote: "upstream", branch: "main" },
      { remote: "origin", branch: "alpha" },
    ];
    expect(groupByRemote(refs)).toEqual([
      ["origin", [
        { remote: "origin", branch: "alpha" },
        { remote: "origin", branch: "zeta" },
      ]],
      ["upstream", [{ remote: "upstream", branch: "main" }]],
    ]);
  });
});
