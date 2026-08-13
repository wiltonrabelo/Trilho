import { describe, expect, it } from "vitest";

import { normalizeRepoPath, sameRepoPath } from "./repoPath";

describe("normalizeRepoPath", () => {
  it("unifica separador, barra final e caixa", () => {
    expect(normalizeRepoPath("C:\\Projetos\\Trilho\\")).toBe(
      "c:\\projetos\\trilho",
    );
    expect(normalizeRepoPath("  C:/Projetos/Trilho  ")).toBe(
      "c:\\projetos\\trilho",
    );
  });

  it("colapsa separadores repetidos", () => {
    expect(normalizeRepoPath("C:\\\\Projetos//Trilho")).toBe(
      "c:\\projetos\\trilho",
    );
  });

  it("preserva o prefixo UNC e a raiz Unix", () => {
    expect(normalizeRepoPath("\\\\servidor\\share\\repo")).toBe(
      "\\\\servidor\\share\\repo",
    );
    expect(normalizeRepoPath("/home/u/repo")).toBe("\\home\\u\\repo");
  });

  it("ignora caixa só em ASCII, como o Rust", () => {
    // `to_ascii_lowercase` do backend não mexe em acentuadas: se o frontend
    // baixasse a caixa delas, os dois lados discordariam.
    expect(normalizeRepoPath("C:\\Projetos\\AÇÕES")).toBe("c:\\projetos\\aÇÕes");
  });
});

describe("sameRepoPath", () => {
  it("aceita as mesmas variações que o backend", () => {
    expect(sameRepoPath("C:\\Projetos\\Trilho", "c:/projetos/trilho")).toBe(
      true,
    );
    expect(sameRepoPath("C:\\Projetos\\Trilho\\", "C:\\Projetos\\Trilho")).toBe(
      true,
    );
    expect(sameRepoPath(" C:\\Projetos\\Trilho ", "C:\\Projetos\\Trilho")).toBe(
      true,
    );
    expect(sameRepoPath("C:\\Projetos\\\\Trilho", "C:/Projetos/Trilho")).toBe(
      true,
    );
  });

  it("distingue repositórios diferentes", () => {
    expect(sameRepoPath("C:\\Projetos\\Trilho", "C:\\Projetos\\Outro")).toBe(
      false,
    );
    expect(sameRepoPath("\\\\servidor\\share", "\\servidor\\share")).toBe(false);
  });
});
