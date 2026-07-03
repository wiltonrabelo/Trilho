/** Uma linha renderizável no diff lado a lado. */
export interface DiffSideLine {
  text: string;
  kind: "context" | "add" | "remove" | "empty";
  lineNo?: number;
}

export interface DiffRow {
  left: DiffSideLine;
  right: DiffSideLine;
}

export interface ParsedDiffFile {
  oldPath: string;
  newPath: string;
  rows: DiffRow[];
}

export interface ParsedDiff {
  files: ParsedDiffFile[];
  /** Texto bruto quando o parser não reconhece formato unified. */
  rawFallback: string | null;
}

const HUNK_HEADER = /^@@ -(\d+)(?:,\d+)? \+(\d+)(?:,\d+)? @@/;

/**
 * Converte saída unified de `git diff` / `git show` em linhas lado a lado.
 */
export function parseUnifiedDiff(text: string): ParsedDiff {
  const trimmed = text.trim();
  if (!trimmed) {
    return { files: [], rawFallback: "(sem alterações)" };
  }

  if (!trimmed.includes("@@") && !trimmed.startsWith("diff --git")) {
    return { files: [], rawFallback: trimmed };
  }

  const chunks = splitDiffFiles(trimmed);
  const files: ParsedDiffFile[] = [];

  for (const chunk of chunks) {
    const parsed = parseDiffChunk(chunk);
    if (parsed) files.push(parsed);
  }

  if (files.length === 0) {
    return { files: [], rawFallback: trimmed };
  }

  return { files, rawFallback: null };
}

function splitDiffFiles(text: string): string[] {
  const parts = text.split(/^diff --git /m).filter(Boolean);
  if (parts.length === 0) return [text];
  return parts.map((p, i) => (i === 0 && text.startsWith("diff --git") ? `diff --git ${p}` : `diff --git ${p}`));
}

function parseDiffChunk(chunk: string): ParsedDiffFile | null {
  const lines = chunk.split("\n");
  let oldPath = "";
  let newPath = "";
  const rows: DiffRow[] = [];
  let leftNo = 0;
  let rightNo = 0;

  for (const line of lines) {
    if (line.startsWith("diff --git ")) {
      const m = line.match(/^diff --git a\/(.+?) b\/(.+)$/);
      if (m) {
        oldPath = m[1];
        newPath = m[2];
      }
      continue;
    }
    if (line.startsWith("--- ")) {
      if (!oldPath) oldPath = line.slice(4).replace(/^a\//, "");
      continue;
    }
    if (line.startsWith("+++ ")) {
      if (!newPath) newPath = line.slice(4).replace(/^b\//, "");
      continue;
    }
    if (line.startsWith("@@")) {
      const m = line.match(HUNK_HEADER);
      if (m) {
        leftNo = parseInt(m[1], 10);
        rightNo = parseInt(m[2], 10);
      }
      continue;
    }
    if (line.startsWith("\\ No newline")) continue;
    if (line.startsWith("index ") || line.startsWith("new file") || line.startsWith("deleted file")) {
      continue;
    }

    const prefix = line[0] ?? " ";
    const content = line.slice(1);

    if (prefix === " ") {
      rows.push({
        left: { text: content, kind: "context", lineNo: leftNo },
        right: { text: content, kind: "context", lineNo: rightNo },
      });
      leftNo += 1;
      rightNo += 1;
    } else if (prefix === "-") {
      rows.push({
        left: { text: content, kind: "remove", lineNo: leftNo },
        right: { text: "", kind: "empty" },
      });
      leftNo += 1;
    } else if (prefix === "+") {
      rows.push({
        left: { text: "", kind: "empty" },
        right: { text: content, kind: "add", lineNo: rightNo },
      });
      rightNo += 1;
    }
  }

  if (rows.length === 0 && !oldPath && !newPath) return null;
  return {
    oldPath: oldPath || newPath,
    newPath: newPath || oldPath,
    rows,
  };
}
