import { useCallback, useEffect, useState } from "react";

import { getBranchPrStatus } from "@/lib/api";
import type { BranchPrStatusDto, CredentialStatusDto, RepoInfo } from "@/types";

function isGithubRemote(url: string | null | undefined): boolean {
  if (!url) return false;
  const lower = url.toLowerCase();
  return lower.includes("github.com");
}

export function usePrStatus(
  repo: RepoInfo | null,
  credential: CredentialStatusDto | null,
) {
  const [status, setStatus] = useState<BranchPrStatusDto | null>(null);
  const [loading, setLoading] = useState(false);

  const refresh = useCallback(async () => {
    if (
      !repo?.branch ||
      !repo.remoteUrl ||
      !isGithubRemote(repo.remoteUrl) ||
      !credential?.githubConnected
    ) {
      setStatus(null);
      return;
    }
    setLoading(true);
    try {
      setStatus(await getBranchPrStatus());
    } catch {
      setStatus({
        visible: true,
        open: [],
        merged: [],
        closed: [],
        notice: "Status de PR indisponível.",
      });
    } finally {
      setLoading(false);
    }
  }, [repo?.branch, repo?.remoteUrl, credential?.githubConnected]);

  useEffect(() => {
    void refresh();
  }, [refresh]);

  return { prStatus: status, prLoading: loading, refreshPrStatus: refresh };
}
