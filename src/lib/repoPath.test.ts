import { describe, expect, it } from "vitest";

import { normalizeRepoPath, sameRepoPath } from "./repoPath";

describe("normalizeRepoPath", () => {
  it("unifica separador, barra final e caixa", () => {
    expect(normalizeRepoPath("C:\\Projetos\\Trilho\\")).toBe("c:/projetos/trilho");
    expect(normalizeRepoPath("  C:/Projetos/Trilho  ")).toBe("c:/projetos/trilho");
  });

  it("colapsa separadores repetidos", () => {
    expect(normalizeRepoPath("C:\\\\Projetos//Trilho")).toBe("c:/projetos/trilho");
  });
});

describe("sameRepoPath", () => {
  it("aceita as mesmas variações que o backend", () => {
    expect(sameRepoPath("C:\\Projetos\\Trilho", "c:/projetos/trilho")).toBe(true);
    expect(sameRepoPath("C:\\Projetos\\Trilho\\", "C:\\Projetos\\Trilho")).toBe(true);
    expect(sameRepoPath(" C:\\Projetos\\Trilho ", "C:\\Projetos\\Trilho")).toBe(true);
  });

  it("distingue repositórios diferentes", () => {
    expect(sameRepoPath("C:\\Projetos\\Trilho", "C:\\Projetos\\Outro")).toBe(false);
  });
});
