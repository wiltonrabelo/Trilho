import { describe, expect, it } from "vitest";

import {
  buildStagedFileLines,
  formatStagedFileListText,
  stagedFileSymbol,
} from "@/lib/staged-file-list";
import type { FileChangeDto } from "@/types";

describe("stagedFileSymbol", () => {
  it("mapeia tipos para + - ~", () => {
    expect(stagedFileSymbol("added")).toBe("+");
    expect(stagedFileSymbol("deleted")).toBe("-");
    expect(stagedFileSymbol("modified")).toBe("~");
    expect(stagedFileSymbol("renamed")).toBe("~");
  });
});

describe("buildStagedFileLines", () => {
  it("ordena alfabeticamente e formata renomeação como ~", () => {
    const staged: FileChangeDto[] = [
      { path: "z.ts", kind: "modified", staged: true },
      { path: "a.ts", kind: "added", staged: true },
      { path: "old.ts → new.ts", kind: "renamed", staged: true },
      { path: "gone.ts", kind: "deleted", staged: true },
    ];
    expect(buildStagedFileLines(staged)).toEqual([
      { symbol: "+", path: "a.ts" },
      { symbol: "-", path: "gone.ts" },
      { symbol: "~", path: "old.ts → new.ts" },
      { symbol: "~", path: "z.ts" },
    ]);
  });
});

describe("formatStagedFileListText", () => {
  it("gera uma linha por arquivo", () => {
    const staged: FileChangeDto[] = [
      { path: "src/App.tsx", kind: "modified", staged: true },
      { path: "src/novo.ts", kind: "added", staged: true },
    ];
    expect(formatStagedFileListText(staged)).toBe(
      "Arquivos do commit:\n\n~ src/App.tsx\n+ src/novo.ts",
    );
  });

  it("retorna vazio sem arquivos staged", () => {
    expect(formatStagedFileListText([])).toBe("");
  });
});
