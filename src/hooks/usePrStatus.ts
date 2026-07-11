import { useCallback, useEffect, useState } from "react";

import { getBranchPrStatus } from "@/lib/api";
import type { BranchPrStatusDto, CredentialStatusDto, RepoInfo } from "@/types";

/** github.com ou GitHub Enterprise (`github.*`). */
function isGithubRemote(url: string | null | undefined): boolean {
  if (!url) return false;
  const lower = url.toLowerCase();
  if (lower.includes("github.com")) return true;
  // GHE: host começa com github. (ex.: github.empresa.com)
  try {
    if (lower.startsWith("http://") || lower.startsWith("https://")) {
      const host = new URL(url).hostname.toLowerCase();
      return host === "github.com" || host.startsWith("github.");
    }
  } catch {
    /* fall through */
  }
  const ssh = lower.match(/^git@([^:]+):/);
  if (ssh) {
    const host = ssh[1];
    return host === "github.com" || host.startsWith("github.");
  }
  return false;
}

export function usePrStatus(
  repo: RepoInfo | null,
  _credential: CredentialStatusDto | null,
) {
  const [status, setStatus] = useState<BranchPrStatusDto | null>(null);
  const [loading, setLoading] = useState(false);

  const refresh = useCallback(async () => {
    if (!repo?.branch || !repo.remoteUrl || !isGithubRemote(repo.remoteUrl)) {
      setStatus(null);
      return;
    }
    setLoading(true);
    try {
      setStatus(await getBranchPrStatus());
    } catch (e) {
      const msg = e instanceof Error ? e.message : String(e);
      setStatus({
        visible: true,
        open: [],
        merged: [],
        closed: [],
        notice: msg || "Status de PR indisponível.",
      });
    } finally {
      setLoading(false);
    }
  }, [repo?.branch, repo?.remoteUrl]);

  useEffect(() => {
    void refresh();
  }, [refresh]);

  return { prStatus: status, prLoading: loading, refreshPrStatus: refresh };
}
