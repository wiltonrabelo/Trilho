import { useCallback, useEffect, useState } from "react";

import { listStashes } from "@/lib/api";
import type { StashEntryDto } from "@/types";

export function useStashes(repoPath: string | undefined) {
  const [stashes, setStashes] = useState<StashEntryDto[]>([]);
  const [loading, setLoading] = useState(false);

  const refresh = useCallback(async () => {
    if (!repoPath) {
      setStashes([]);
      return;
    }
    setLoading(true);
    try {
      setStashes(await listStashes());
    } catch {
      setStashes([]);
    } finally {
      setLoading(false);
    }
  }, [repoPath]);

  useEffect(() => {
    void refresh();
  }, [refresh]);

  return { stashes, loading, refresh };
}
