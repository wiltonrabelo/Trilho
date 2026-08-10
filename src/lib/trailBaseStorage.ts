export const STORAGE_PREFIX = "trilho.trailBase.";

export function storageKey(repoPath: string): string {
  return `${STORAGE_PREFIX}${repoPath}`;
}

export function loadStoredTrailBase(repoPath: string | null): string | null {
  if (!repoPath || typeof localStorage === "undefined") return null;
  try {
    return localStorage.getItem(storageKey(repoPath));
  } catch {
    return null;
  }
}

export function saveStoredTrailBase(
  repoPath: string,
  base: string | null,
): void {
  if (typeof localStorage === "undefined") return;
  try {
    if (base) localStorage.setItem(storageKey(repoPath), base);
    else localStorage.removeItem(storageKey(repoPath));
  } catch {
    /* ignore */
  }
}
