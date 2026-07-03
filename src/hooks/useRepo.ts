import { useCallback, useEffect, useState } from "react";

import {
  closeRepo,
  getFileDiff,
  getRecentRepos,
  getRepoStatus,
  openRepo,
} from "@/lib/api";
import type { RepoInfo, RepoStatusDto } from "@/types";

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

  useEffect(() => {
    getRecentRepos().then(setRecentRepos);
  }, []);

  const refreshStatus = useCallback(async () => {
    if (!repo) return;
    setStatus(await getRepoStatus());
  }, [repo]);

  async function open(path: string) {
    setLoading(true);
    setError(null);
    try {
      const repoInfo = await openRepo(path);
      setRepo(repoInfo);
      setSelectedFile(null);
      setFileDiff(null);
      const recents = await getRecentRepos();
      setRecentRepos(recents);
      setStatus(await getRepoStatus());
      return repoInfo;
    } catch (e) {
      setError(String(e));
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
    selectedFile,
    fileDiff,
    fileLoading,
    selectFile,
    clearFileSelection,
  };
}
