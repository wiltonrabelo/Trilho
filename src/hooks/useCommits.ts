import { useCallback, useEffect, useLayoutEffect, useRef, useState } from "react";

import {
  getCommitDiff,
  getCommitFileDiff,
  getDualTrail,
  listBranchExclusiveCommits,
  listCommitFiles,
  listCommits,
} from "@/lib/api";
import type { CommitDto, FileChangeDto, RepoInfo, TrailKindDto } from "@/types";

const PAGE_SIZE = 100;
const DUAL_TRAIL_LIMIT = 300;
const EXCLUSIVE_LIMIT = 300;

/** Visão do grafo: "trail" = trilha da branch atual + linha da base (RF-02),
 *  legível em repositórios com muitos merges; "graph" = grafo completo. */
export type GraphView = "trail" | "graph";

export function useCommits(repo: RepoInfo | null, baseBranch: string | null) {
  const [view, setView] = useState<GraphView>("graph");
  const [commits, setCommits] = useState<CommitDto[]>([]);
  const [trails, setTrails] = useState<TrailKindDto[] | null>(null);
  const [hasMore, setHasMore] = useState(false);
  const [loading, setLoading] = useState(false);
  const [focusedBranch, setFocusedBranch] = useState<string | null>(null);
  const [checkoutHeadCommit, setCheckoutHeadCommit] =
    useState<CommitDto | null>(null);
  const [selectedCommit, setSelectedCommit] = useState<CommitDto | null>(null);
  const [commitDiff, setCommitDiff] = useState<string | null>(null);
  const [diffLoading, setDiffLoading] = useState(false);
  const [commitFiles, setCommitFiles] = useState<FileChangeDto[]>([]);
  const [selectedCommitFile, setSelectedCommitFile] = useState<string | null>(
    null,
  );
  const [commitFileDiff, setCommitFileDiff] = useState<string | null>(null);

  const repoRef = useRef(repo);
  repoRef.current = repo;
  const viewRef = useRef(view);
  viewRef.current = view;
  const baseBranchRef = useRef(baseBranch);
  baseBranchRef.current = baseBranch;
  const focusedBranchRef = useRef(focusedBranch);
  focusedBranchRef.current = focusedBranch;

  const focusBranch = useCallback(
    (branch: string) => {
      if (!repo?.branch || branch === repo.branch) {
        setFocusedBranch(null);
        return;
      }
      setFocusedBranch((prev) => (prev === branch ? null : branch));
    },
    [repo?.branch],
  );

  const clearFocusedBranch = useCallback(() => {
    setFocusedBranch(null);
  }, []);

  const refresh = useCallback(async () => {
    const current = repoRef.current;
    if (!current) return;
    const reqPath = current.path;
    const stillCurrent = () => repoRef.current?.path === reqPath;

    const viewMode = viewRef.current;
    const base = baseBranchRef.current;
    const focused = focusedBranchRef.current;

    setLoading(true);
    try {
      const headListPromise = listCommits(1, null, false);

      if (focused) {
        const [list, headList] = await Promise.all([
          listBranchExclusiveCommits(focused, EXCLUSIVE_LIMIT),
          headListPromise,
        ]);
        if (!stillCurrent()) return;
        setCommits(list);
        setTrails(null);
        setHasMore(list.length >= EXCLUSIVE_LIMIT);
        setCheckoutHeadCommit(headList[0] ?? null);
        return;
      }

      if (viewMode === "trail") {
        // Garantia: trilha first-parent sempre aparece; dual trail enriquece depois.
        const [list, headList] = await Promise.all([
          listCommits(PAGE_SIZE, null, true),
          headListPromise,
        ]);
        if (!stillCurrent()) return;
        setCommits(list);
        setTrails(null);
        setHasMore(list.length >= PAGE_SIZE);
        setCheckoutHeadCommit(headList[0] ?? null);

        if (base) {
          try {
            const entries = await getDualTrail(base, DUAL_TRAIL_LIMIT);
            if (!stillCurrent()) return;
            if (entries.length > 0) {
              setCommits(entries.map((e) => e.commit));
              setTrails(entries.map((e) => e.trail));
              setHasMore(false);
            }
          } catch {
            /* mantém first-parent já carregado */
          }
        }
        return;
      }

      const [list, headList] = await Promise.all([
        listCommits(PAGE_SIZE, null, false),
        headListPromise,
      ]);
      if (!stillCurrent()) return;
      setCommits(list);
      setTrails(null);
      setHasMore(list.length >= PAGE_SIZE);
      setCheckoutHeadCommit(headList[0] ?? null);
    } catch {
      if (!stillCurrent()) return;
      setCommits([]);
      setTrails(null);
      setHasMore(false);
    } finally {
      if (stillCurrent()) setLoading(false);
    }
  }, []);

  useLayoutEffect(() => {
    setCommits([]);
    setTrails(null);
    setHasMore(false);
    setSelectedCommit(null);
    setCommitDiff(null);
    setCommitFiles([]);
    setSelectedCommitFile(null);
    setCommitFileDiff(null);
    setFocusedBranch(null);
    setCheckoutHeadCommit(null);
  }, [repo?.path]);

  useEffect(() => {
    if (view === "trail" && !baseBranch && !focusedBranch) {
      setTrails(null);
    }
  }, [view, baseBranch, focusedBranch]);

  useEffect(() => {
    if (!repo?.path) return;
    void refresh();
  }, [repo?.path, repo?.branch, view, baseBranch, focusedBranch, refresh]);

  async function loadMore() {
    if (!repo || commits.length === 0) return;
    const reqPath = repo.path;
    const after = commits[commits.length - 1]!.id;
    setLoading(true);
    try {
      const more = focusedBranch
        ? await listBranchExclusiveCommits(
            focusedBranch,
            EXCLUSIVE_LIMIT,
            after,
          )
        : await listCommits(PAGE_SIZE, after, view === "trail");
      if (repoRef.current?.path !== reqPath) return;
      setCommits((prev) => [...prev, ...more]);
      setHasMore(
        more.length >= (focusedBranch ? EXCLUSIVE_LIMIT : PAGE_SIZE),
      );
    } finally {
      setLoading(false);
    }
  }

  const selectCommit = useCallback(async (commit: CommitDto) => {
    const reqPath = repo?.path ?? null;
    setSelectedCommit(commit);
    setSelectedCommitFile(null);
    setCommitFileDiff(null);
    setCommitDiff(null);
    setCommitFiles([]);
    setDiffLoading(true);
    try {
      const [d, files] = await Promise.all([
        getCommitDiff(commit.id),
        listCommitFiles(commit.id),
      ]);
      if (reqPath && repoRef.current?.path !== reqPath) return;
      setCommitDiff(d || "(sem alterações)");
      setCommitFiles(files);
    } catch (e) {
      if (reqPath && repoRef.current?.path !== reqPath) return;
      setCommitDiff(`Erro: ${e}`);
    } finally {
      setDiffLoading(false);
    }
  }, [repo?.path]);

  const selectedCommitRef = useRef<CommitDto | null>(null);
  selectedCommitRef.current = selectedCommit;

  const selectCommitFile = useCallback(async (path: string) => {
    const commitId = selectedCommitRef.current?.id;
    if (!commitId) return;
    setSelectedCommitFile(path);
    setCommitFileDiff(null);
    setDiffLoading(true);
    try {
      const d = await getCommitFileDiff(commitId, path);
      setCommitFileDiff(d || "(sem alterações)");
    } catch (e) {
      setCommitFileDiff(`Erro: ${e}`);
    } finally {
      setDiffLoading(false);
    }
  }, []);

  const clearSelection = useCallback(() => {
    setSelectedCommit(null);
    setCommitDiff(null);
    setCommitFiles([]);
    setSelectedCommitFile(null);
    setCommitFileDiff(null);
  }, []);

  const selectCommitBySha = useCallback(
    async (sha: string) => {
      if (!repo) return;
      const reqPath = repo.path;
      const matches = (c: CommitDto) =>
        c.id === sha ||
        c.id.startsWith(sha) ||
        sha.startsWith(c.id) ||
        c.shortId === sha.slice(0, 7);

      const found = commits.find(matches);
      if (found) {
        await selectCommit(found);
        return;
      }

      const stub: CommitDto = {
        id: sha,
        shortId: sha.slice(0, 7),
        summary: `Commit ${sha.slice(0, 7)}`,
        body: null,
        authorName: "",
        authoredAt: new Date().toISOString(),
        isLocalOnly: false,
        parentIds: [],
        refs: [],
      };
      if (repoRef.current?.path !== reqPath) return;
      await selectCommit(stub);
    },
    [repo, commits, selectCommit],
  );

  return {
    view,
    setView,
    commits,
    trails,
    hasMore,
    loading,
    focusedBranch,
    focusBranch,
    clearFocusedBranch,
    checkoutHeadCommit,
    selectedCommit,
    commitDiff,
    commitFiles,
    selectedCommitFile,
    commitFileDiff,
    diffLoading,
    refresh,
    loadMore,
    selectCommit,
    selectCommitBySha,
    selectCommitFile,
    clearSelection,
  };
}
