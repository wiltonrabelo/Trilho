import type { BlameLineDto } from "@/types";

export interface BlameFilters {
  author: string | null;
  dateFrom: string;
  dateTo: string;
}

export const EMPTY_BLAME_FILTERS: BlameFilters = {
  author: null,
  dateFrom: "",
  dateTo: "",
};

function startOfDay(isoDate: string): number {
  const [y, m, d] = isoDate.split("-").map(Number);
  if (!y || !m || !d) return 0;
  return new Date(y, m - 1, d, 0, 0, 0, 0).getTime();
}

function endOfDay(isoDate: string): number {
  const [y, m, d] = isoDate.split("-").map(Number);
  if (!y || !m || !d) return Number.MAX_SAFE_INTEGER;
  return new Date(y, m - 1, d, 23, 59, 59, 999).getTime();
}

export function compareBlameByNewestFirst(
  a: BlameLineDto,
  b: BlameLineDto,
): number {
  const ta = new Date(a.authoredAt).getTime();
  const tb = new Date(b.authoredAt).getTime();
  if (tb !== ta) return tb - ta;
  return a.line - b.line;
}

export function filterAndSortBlameLines(
  lines: BlameLineDto[],
  filters: BlameFilters,
): BlameLineDto[] {
  let result = lines;

  if (filters.author) {
    const author = filters.author.toLowerCase();
    result = result.filter((line) => line.author.toLowerCase() === author);
  }

  if (filters.dateFrom) {
    const from = startOfDay(filters.dateFrom);
    result = result.filter(
      (line) => new Date(line.authoredAt).getTime() >= from,
    );
  }

  if (filters.dateTo) {
    const to = endOfDay(filters.dateTo);
    result = result.filter((line) => new Date(line.authoredAt).getTime() <= to);
  }

  return [...result].sort(compareBlameByNewestFirst);
}

export function uniqueBlameAuthors(lines: BlameLineDto[]): string[] {
  return [...new Set(lines.map((line) => line.author))].sort((a, b) =>
    a.localeCompare(b, "pt-BR"),
  );
}

export function hasActiveBlameFilters(filters: BlameFilters): boolean {
  return Boolean(filters.author || filters.dateFrom || filters.dateTo);
}
