import { useCallback, useEffect, useRef, useState } from "react";

import {
  closeRepo,
  getFileDiff,
  getRecentRepos,
  getRepoStatus,
  openRepo,
  removeRecentRepo,
} from "@/lib/api";
import type { RepoInfo, RepoStatusDto } from "@/types";

function reconcileStagedFlag(
  path: string,
  preferStaged: boolean,
  status: RepoStatusDto,
): boolean | null {
  const inStaged = status.staged.some((f) => f.path === path);
  const inUnstaged = status.unstaged.some((f) => f.path === path);
  const inUntracked = status.untracked.some((f) => f.path === path);
  if (!inStaged && !inUnstaged && !inUntracked) return null;
  if (preferStaged && inStaged) return true;
  if (!preferStaged && (inUnstaged || inUntracked)) return false;
  if (inStaged) return true;
  if (inUnstaged || inUntracked) return false;
  return null;
}

function sameRepoPath(a: string, b: string): boolean {
  const norm = (p: string) =>
    p.replace(/\\/g, "/").replace(/\/$/, "").toLowerCase();
  return norm(a) === norm(b);
}

export function useRepo() {
  const [repo, setRepo] = useState<RepoInfo | null>(null);
  const [recentRepos, setRecentRepos] = useState<string[]>([]);
  const [status, setStatus] = useState<RepoStatusDto | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [selectedFile, setSelectedFile] = useState<{
    path: string;
    staged: boolean;
  } | null>(null);
  const [fileDiff, setFileDiff] = useState<string | null>(null);
  const [fileLoading, setFileLoading] = useState(false);
  const selectedFileRef = useRef(selectedFile);
  selectedFileRef.current = selectedFile;

  useEffect(() => {
    getRecentRepos().then(setRecentRepos);
  }, []);

  /** Mantém seleção alinhada ao status após stage/unstage (evita botão Stage fantasma). */
  useEffect(() => {
    const current = selectedFileRef.current;
    if (!status || !current) return;

    const nextStaged = reconcileStagedFlag(
      current.path,
      current.staged,
      status,
    );
    if (nextStaged === null) {
      setSelectedFile(null);
      setFileDiff(null);
      return;
    }
    if (nextStaged === current.staged) return;

    let cancelled = false;
    setFileDiff(null);
    setFileLoading(true);
    void getFileDiff(current.path, nextStaged)
      .then((d) => {
        if (cancelled) return;
        setSelectedFile({ path: current.path, staged: nextStaged });
        setFileDiff(d || "(sem diff)");
      })
      .catch((e) => {
        if (cancelled) return;
        setFileDiff(`Erro: ${e}`);
      })
      .finally(() => {
        if (!cancelled) setFileLoading(false);
      });

    return () => {
      cancelled = true;
    };
  }, [status]);

  const refreshStatus = useCallback(async () => {
    if (!repo) return;
    setStatus(await getRepoStatus());
  }, [repo]);

  const refreshRecents = useCallback(async () => {
    setRecentRepos(await getRecentRepos());
  }, []);

  const removeRecent = useCallback(
    async (path: string) => {
      if (repo && sameRepoPath(repo.path, path)) {
        await closeRepo();
        setRepo(null);
        setStatus(null);
        setSelectedFile(null);
        setFileDiff(null);
      }
      await removeRecentRepo(path);
      await refreshRecents();
    },
    [repo, refreshRecents],
  );

  async function open(path: string) {
    setLoading(true);
    setError(null);
    try {
      const repoInfo = await openRepo(path);
      setRepo(repoInfo);
      setSelectedFile(null);
      setFileDiff(null);
      await refreshRecents();
      setStatus(await getRepoStatus());
      return repoInfo;
    } catch (e) {
      setError(String(e));
      await refreshRecents();
      throw e;
    } finally {
      setLoading(false);
    }
  }

  async function close() {
    setLoading(true);
    setError(null);
    try {
      await closeRepo();
      setRepo(null);
      setStatus(null);
      setSelectedFile(null);
      setFileDiff(null);
    } catch (e) {
      setError(String(e));
    } finally {
      setLoading(false);
    }
  }

  async function selectFile(path: string, staged: boolean) {
    setSelectedFile({ path, staged });
    setFileDiff(null);
    setFileLoading(true);
    try {
      const d = await getFileDiff(path, staged);
      setFileDiff(d || "(sem diff)");
    } catch (e) {
      setFileDiff(`Erro: ${e}`);
    } finally {
      setFileLoading(false);
    }
  }

  function clearFileSelection() {
    setSelectedFile(null);
    setFileDiff(null);
  }

  return {
    repo,
    setRepo,
    recentRepos,
    status,
    loading,
    error,
    open,
    close,
    refreshStatus,
    refreshRecents,
    removeRecent,
    selectedFile,
    fileDiff,
    fileLoading,
    selectFile,
    clearFileSelection,
  };
}
