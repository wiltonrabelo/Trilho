/** Hunk unified extraído de um diff Git (para `git apply --reverse`). */
export interface DiffHunk {
  index: number;
  header: string;
  patch: string;
}

/**
 * Extrai hunks de um diff unified (`git diff` / `git diff --cached`).
 * Hunks grandes com alterações distantes são divididos em trechos revertíveis.
 */
export function extractHunks(rawDiff: string): DiffHunk[] {
  const text = rawDiff.trim();
  if (!text.includes("@@")) return [];

  const lines = text.split("\n");
  const hunks: DiffHunk[] = [];
  let hunkStart = -1;
  let header = "";

  const pushHunk = (start: number, end: number) => {
    const bodyLines = lines.slice(start, end);
    const fileHeaders = fileHeadersBefore(lines, start);
    for (const segment of splitHunkBody(header, bodyLines, fileHeaders)) {
      hunks.push(segment);
    }
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

  return hunks.map((hunk, index) => ({ ...hunk, index }));
}

function splitHunkBody(
  header: string,
  bodyLines: string[],
  fileHeaders: string | null,
): DiffHunk[] {
  const hunkHeader = bodyLines[0]?.startsWith("@@") ? bodyLines[0] : header;
  const contentLines = bodyLines[0]?.startsWith("@@") ? bodyLines.slice(1) : bodyLines;
  const parsed = parseHunkHeader(hunkHeader);
  if (!parsed || contentLines.length === 0) {
    return [buildHunk(0, hunkHeader, contentLines, fileHeaders)];
  }

  const ranges = findSegmentRanges(contentLines);
  if (ranges.length <= 1) {
    return [buildHunk(0, hunkHeader, contentLines, fileHeaders)];
  }

  return ranges.map((range, index) => {
    const segmentLines = contentLines.slice(range[0], range[1] + 1);
    const segmentHeader = buildSegmentHeader(
      contentLines,
      range,
      parsed.oldStart,
      parsed.newStart,
    );
    return buildHunk(index, segmentHeader, segmentLines, fileHeaders);
  });
}

function buildHunk(
  index: number,
  header: string,
  bodyLines: string[],
  fileHeaders: string | null,
): DiffHunk {
  const body = [header, ...bodyLines].join("\n");
  const patch = fileHeaders ? `${fileHeaders}\n${body}\n` : `${body}\n`;
  return { index, header, patch };
}

function parseHunkHeader(header: string): {
  oldStart: number;
  newStart: number;
} | null {
  const match = header.match(/^@@ -(\d+)(?:,\d+)? \+(\d+)(?:,\d+)? @@/);
  if (!match) return null;
  return {
    oldStart: Number.parseInt(match[1], 10),
    newStart: Number.parseInt(match[2], 10),
  };
}

function isChangeLine(line: string): boolean {
  return line.startsWith("-") || line.startsWith("+");
}

/** Separa alterações distantes dentro do mesmo hunk Git. */
function findSegmentRanges(bodyLines: string[]): [number, number][] {
  const changeIndices = bodyLines
    .map((line, index) => (isChangeLine(line) ? index : -1))
    .filter((index) => index >= 0);

  if (changeIndices.length === 0) {
    return [[0, bodyLines.length - 1]];
  }

  const groups: { first: number; last: number }[] = [];
  let groupFirst = changeIndices[0];
  let groupLast = changeIndices[0];

  for (let i = 1; i < changeIndices.length; i++) {
    const idx = changeIndices[i];
    const between = bodyLines.slice(groupLast + 1, idx);
    const separated =
      between.length > 0 &&
      between.every((line) => !isChangeLine(line));

    if (separated) {
      groups.push({ first: groupFirst, last: groupLast });
      groupFirst = idx;
      groupLast = idx;
    } else {
      groupLast = idx;
    }
  }
  groups.push({ first: groupFirst, last: groupLast });

  if (groups.length <= 1) {
    return [[0, bodyLines.length - 1]];
  }

  const ranges: [number, number][] = [];
  for (let g = 0; g < groups.length; g++) {
    const start =
      g === 0
        ? 0
        : (() => {
            const gapStart = groups[g - 1].last + 1;
            const gapEnd = groups[g].first - 1;
            const split = gapStart + Math.floor((gapEnd - gapStart + 1) / 2);
            return split;
          })();

    const end =
      g === groups.length - 1
        ? bodyLines.length - 1
        : (() => {
            const gapStart = groups[g].last + 1;
            const gapEnd = groups[g + 1].first - 1;
            const split = gapStart + Math.floor((gapEnd - gapStart + 1) / 2);
            return split - 1;
          })();

    ranges.push([start, end]);
  }

  return ranges;
}

function lineNumbersAt(
  bodyLines: string[],
  index: number,
  oldStart: number,
  newStart: number,
): { oldLine: number; newLine: number } {
  let oldLine = oldStart;
  let newLine = newStart;

  for (let i = 0; i < index; i++) {
    const prefix = bodyLines[i][0] ?? " ";
    if (prefix === " ") {
      oldLine += 1;
      newLine += 1;
    } else if (prefix === "-") {
      oldLine += 1;
    } else if (prefix === "+") {
      newLine += 1;
    }
  }

  return { oldLine, newLine };
}

function countOldLines(lines: string[]): number {
  return lines.filter((line) => {
    const prefix = line[0] ?? " ";
    return prefix === " " || prefix === "-";
  }).length;
}

function countNewLines(lines: string[]): number {
  return lines.filter((line) => {
    const prefix = line[0] ?? " ";
    return prefix === " " || prefix === "+";
  }).length;
}

function buildSegmentHeader(
  bodyLines: string[],
  range: [number, number],
  oldStart: number,
  newStart: number,
): string {
  const segmentLines = bodyLines.slice(range[0], range[1] + 1);
  const atStart = lineNumbersAt(bodyLines, range[0], oldStart, newStart);
  const oldCount = countOldLines(segmentLines);
  const newCount = countNewLines(segmentLines);
  return `@@ -${atStart.oldLine},${oldCount} +${atStart.newLine},${newCount} @@`;
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
