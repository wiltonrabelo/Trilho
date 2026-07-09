export type ConflictLineKind = "context" | "changed";

/** Normaliza quebras de linha (RF-20 / EOL). */
export function splitConflictLines(text: string): string[] {
  if (!text) return [];
  return text.replace(/\r\n/g, "\n").replace(/\r/g, "\n").split("\n");
}

/** Destaca linhas que divergem do ancestral comum (estilo diff). */
export function highlightAgainstBase(
  base: string,
  side: string,
): { lineNo: number; text: string; kind: ConflictLineKind }[] {
  const baseLines = splitConflictLines(base);
  const sideLines = splitConflictLines(side);
  return sideLines.map((text, i) => {
    const bl = baseLines[i];
    const kind: ConflictLineKind =
      bl !== undefined && bl === text ? "context" : "changed";
    return { lineNo: i + 1, text, kind };
  });
}

export function conflictLineClass(kind: ConflictLineKind, side: "ours" | "theirs"): string {
  if (kind === "context") return "hover:bg-surface/50";
  return side === "ours"
    ? "bg-red-500/25 border-l-2 border-l-red-500 hover:bg-red-500/30 dark:bg-red-500/15"
    : "bg-emerald-500/25 border-l-2 border-l-emerald-500 hover:bg-emerald-500/30 dark:bg-emerald-500/15";
}
