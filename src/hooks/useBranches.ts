import { useCallback, useEffect, useState } from "react";

import { listLocalBranches, listRemoteBranches } from "@/lib/api";
import type { RemoteBranchRefDto } from "@/types";

export function useBranches(repoPath: string | undefined, currentBranch?: string | null) {
  const [branches, setBranches] = useState<string[]>([]);
  const [remoteBranches, setRemoteBranches] = useState<RemoteBranchRefDto[]>([]);
  const [loading, setLoading] = useState(false);

  const refresh = useCallback(async () => {
    if (!repoPath) {
      setBranches([]);
      setRemoteBranches([]);
      return;
    }
    setLoading(true);
    try {
      const [local, remote] = await Promise.all([
        listLocalBranches(),
        listRemoteBranches(),
      ]);
      setBranches(local);
      setRemoteBranches(remote);
    } catch {
      setBranches([]);
      setRemoteBranches([]);
    } finally {
      setLoading(false);
    }
  }, [repoPath]);

  useEffect(() => {
    void refresh();
  }, [refresh, currentBranch]);

  return { branches, remoteBranches, loading, refresh };
}
