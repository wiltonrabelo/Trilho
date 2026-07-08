import type { FileChangeDto, FileChangeKind } from "@/types";

export type StagedFileSymbol = "+" | "-" | "~";

/** Sinal exibido na prévia do commit (backlog «Lista de arquivos no commit»). */
export function stagedFileSymbol(kind: FileChangeKind): StagedFileSymbol {
  switch (kind) {
    case "added":
      return "+";
    case "deleted":
      return "-";
    case "modified":
    case "renamed":
    case "conflicted":
      return "~";
    default:
      return "~";
  }
}

export interface StagedFileLine {
  symbol: StagedFileSymbol;
  path: string;
}

/** Lista ordenada alfabeticamente dos arquivos em staging. */
export function buildStagedFileLines(staged: FileChangeDto[]): StagedFileLine[] {
  return [...staged]
    .sort((a, b) => a.path.localeCompare(b.path, "pt-BR"))
    .map((file) => ({
      symbol: stagedFileSymbol(file.kind),
      path: file.path,
    }));
}

/** Texto plano para a descrição do commit (cabeçalho + listagem). */
export function formatStagedFileListText(staged: FileChangeDto[]): string {
  const lines = buildStagedFileLines(staged);
  if (lines.length === 0) return "";
  const list = lines.map(({ symbol, path }) => `${symbol} ${path}`).join("\n");
  return `Arquivos do commit:\n\n${list}`;
}
