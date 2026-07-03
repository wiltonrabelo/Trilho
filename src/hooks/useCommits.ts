import { useCallback, useEffect, useState } from "react";

import { getCommitDiff, listCommits } from "@/lib/api";
import type { CommitDto, RepoInfo } from "@/types";

const PAGE_SIZE = 100;

export function useCommits(repo: RepoInfo | null) {
  const [commits, setCommits] = useState<CommitDto[]>([]);
  const [commitSkip, setCommitSkip] = useState(0);
  const [hasMore, setHasMore] = useState(false);
  const [loading, setLoading] = useState(false);
  const [selectedCommit, setSelectedCommit] = useState<CommitDto | null>(null);
  const [commitDiff, setCommitDiff] = useState<string | null>(null);
  const [diffLoading, setDiffLoading] = useState(false);

  const refresh = useCallback(async () => {
    if (!repo) return;
    const list = await listCommits(PAGE_SIZE, 0);
    setCommits(list);
    setCommitSkip(0);
    setHasMore(list.length >= PAGE_SIZE);
  }, [repo]);

  useEffect(() => {
    if (!repo) {
      setCommits([]);
      setCommitSkip(0);
      setHasMore(false);
      setSelectedCommit(null);
      setCommitDiff(null);
      return;
    }
    void refresh();
  }, [repo, refresh]);

  async function loadMore() {
    const nextSkip = commitSkip + PAGE_SIZE;
    setLoading(true);
    try {
      const more = await listCommits(PAGE_SIZE, nextSkip);
      setCommits((prev) => [...prev, ...more]);
      setCommitSkip(nextSkip);
      setHasMore(more.length >= PAGE_SIZE);
    } finally {
      setLoading(false);
    }
  }

  async function selectCommit(commit: CommitDto) {
    setSelectedCommit(commit);
    setCommitDiff(null);
    setDiffLoading(true);
    try {
      const d = await getCommitDiff(commit.id);
      setCommitDiff(d || "(sem alterações)");
    } catch (e) {
      setCommitDiff(`Erro: ${e}`);
    } finally {
      setDiffLoading(false);
    }
  }

  function clearSelection() {
    setSelectedCommit(null);
    setCommitDiff(null);
  }

  return {
    commits,
    hasMore,
    loading,
    selectedCommit,
    commitDiff,
    diffLoading,
    refresh,
    loadMore,
    selectCommit,
    clearSelection,
  };
}
