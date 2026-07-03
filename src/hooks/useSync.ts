import { useCallback, useEffect, useState } from "react";

import {
  fetchRemote,
  getCredentialStatus,
  getRepoInfo,
  getSyncInfo,
} from "@/lib/api";
import type { CredentialStatusDto, RepoInfo, SyncInfoDto } from "@/types";

export function useSync(
  repo: RepoInfo | null,
  setRepo: (info: RepoInfo) => void,
  onAfterFetch?: () => Promise<void>,
) {
  const [sync, setSync] = useState<SyncInfoDto | null>(null);
  const [credential, setCredential] = useState<CredentialStatusDto | null>(
    null,
  );
  const [fetchLoading, setFetchLoading] = useState(false);
  const [fetchError, setFetchError] = useState<string | null>(null);

  const refreshCredential = useCallback(() => {
    getCredentialStatus()
      .then(setCredential)
      .catch(() => null);
  }, []);

  const refresh = useCallback(async () => {
    if (!repo) return;
    setSync(await getSyncInfo());
  }, [repo]);

  useEffect(() => {
    refreshCredential();
  }, [refreshCredential]);

  useEffect(() => {
    if (!repo) {
      setSync(null);
      setFetchError(null);
      return;
    }
    void refresh();
    refreshCredential();
  }, [repo, refresh, refreshCredential]);

  async function fetch() {
    setFetchLoading(true);
    setFetchError(null);
    try {
      const info = await fetchRemote();
      setSync(info);
      await onAfterFetch?.();
      const updatedRepo = await getRepoInfo();
      setRepo(updatedRepo);
    } catch (e) {
      setFetchError(String(e));
    } finally {
      setFetchLoading(false);
    }
  }

  return {
    sync,
    credential,
    fetchLoading,
    fetchError,
    refresh,
    fetch,
    refreshCredential,
  };
}
