import type { RemoteBranchRefDto, StashEntryDto, TagEntryDto } from "@/types";

export function filterBranches(branches: string[], query: string): string[] {
  const q = query.trim().toLowerCase();
  if (!q) return branches;
  return branches.filter((b) => b.toLowerCase().includes(q));
}

export function filterRemoteBranches(
  refs: RemoteBranchRefDto[],
  query: string,
): RemoteBranchRefDto[] {
  const q = query.trim().toLowerCase();
  if (!q) return refs;
  return refs.filter(
    (r) =>
      r.branch.toLowerCase().includes(q) ||
      r.remote.toLowerCase().includes(q) ||
      `${r.remote}/${r.branch}`.toLowerCase().includes(q),
  );
}

export function filterStashes(
  stashes: StashEntryDto[],
  query: string,
): StashEntryDto[] {
  const q = query.trim().toLowerCase();
  if (!q) return stashes;
  return stashes.filter(
    (s) =>
      s.message.toLowerCase().includes(q) ||
      s.reference.toLowerCase().includes(q) ||
      String(s.index).includes(q),
  );
}

export function filterTags(tags: TagEntryDto[], query: string): TagEntryDto[] {
  const q = query.trim().toLowerCase();
  if (!q) return tags;
  return tags.filter(
    (t) =>
      t.name.toLowerCase().includes(q) ||
      t.shortId.toLowerCase().includes(q) ||
      t.commitId.toLowerCase().includes(q),
  );
}

export function groupByRemote(
  refs: RemoteBranchRefDto[],
): [string, RemoteBranchRefDto[]][] {
  const map = new Map<string, RemoteBranchRefDto[]>();
  for (const ref of refs) {
    const list = map.get(ref.remote) ?? [];
    list.push(ref);
    map.set(ref.remote, list);
  }
  return [...map.entries()]
    .map(([remote, items]) => [
      remote,
      [...items].sort((a, b) => a.branch.localeCompare(b.branch)),
    ] as [string, RemoteBranchRefDto[]])
    .sort(([a], [b]) => a.localeCompare(b));
}
