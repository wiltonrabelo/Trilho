/** Hunk unified extraído de um diff Git (para `git apply --reverse`). */
export interface DiffHunk {
  index: number;
  header: string;
  patch: string;
}

/**
 * Extrai hunks de um diff unified (`git diff` / `git diff --cached`).
 * Inclui cabeçalhos `---`/`+++` quando presentes no texto.
 */
export function extractHunks(rawDiff: string): DiffHunk[] {
  const text = rawDiff.trim();
  if (!text.includes("@@")) return [];

  const lines = text.split("\n");
  const hunks: DiffHunk[] = [];
  let hunkStart = -1;
  let header = "";
  let index = 0;

  const pushHunk = (start: number, end: number) => {
    const body = lines.slice(start, end).join("\n");
    const fileHeaders = fileHeadersBefore(lines, start);
    const patch = fileHeaders ? `${fileHeaders}\n${body}\n` : `${body}\n`;
    hunks.push({ index, header, patch });
    index += 1;
  };

  for (let i = 0; i < lines.length; i++) {
    if (lines[i].startsWith("@@")) {
      if (hunkStart >= 0) {
        pushHunk(hunkStart, i);
      }
      header = lines[i];
      hunkStart = i;
    }
  }
  if (hunkStart >= 0) {
    pushHunk(hunkStart, lines.length);
  }

  return hunks;
}

function fileHeadersBefore(lines: string[], hunkStart: number): string | null {
  const parts: string[] = [];
  for (let i = hunkStart - 1; i >= 0; i--) {
    const line = lines[i];
    if (line.startsWith("+++ ")) {
      parts.unshift(line);
    } else if (line.startsWith("--- ")) {
      parts.unshift(line);
      break;
    } else if (line.startsWith("diff --git ")) {
      break;
    }
  }
  return parts.length >= 2 ? parts.join("\n") : null;
}
