/** Chave de seleção em lote — staged e working tree são independentes (MM). */
export type FileCheckSection = "staged" | "working";

export function fileCheckKey(section: FileCheckSection, path: string): string {
  return `${section}:${path}`;
}

export function pathsFromChecked(
  checked: ReadonlySet<string>,
  section: FileCheckSection,
): string[] {
  const prefix = `${section}:`;
  return [...checked]
    .filter((k) => k.startsWith(prefix))
    .map((k) => k.slice(prefix.length));
}

export function countChecked(
  checked: ReadonlySet<string>,
  section: FileCheckSection,
): number {
  const prefix = `${section}:`;
  let n = 0;
  for (const k of checked) {
    if (k.startsWith(prefix)) n++;
  }
  return n;
}
