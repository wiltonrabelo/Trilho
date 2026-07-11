import { useCallback, useEffect, useLayoutEffect, useRef, useState } from "react";

import { getBranchOrigin } from "@/lib/api";
import type { BranchOriginDto, RepoInfo } from "@/types";

export function useBranchOrigin(repo: RepoInfo | null) {
  const [origin, setOrigin] = useState<BranchOriginDto | null>(null);
  const [loading, setLoading] = useState(false);
  const hasOrigin = useRef(false);
  const repoPathRef = useRef<string | null>(null);

  // Limpa antes dos useEffects — evita candidata da origem do repo anterior.
  useLayoutEffect(() => {
    const path = repo?.path ?? null;
    if (path === repoPathRef.current) return;
    repoPathRef.current = path;
    hasOrigin.current = false;
    setOrigin(null);
  }, [repo?.path]);

  const refresh = useCallback(async () => {
    if (!repo || repo.isDetached) {
      hasOrigin.current = false;
      setOrigin(null);
      return;
    }
    // Só mostra "carregando" na primeira vez; refreshes mantêm o badge atual.
    if (!hasOrigin.current) setLoading(true);
    try {
      const result = await getBranchOrigin();
      hasOrigin.current = true;
      setOrigin(result);
    } catch {
      hasOrigin.current = false;
      setOrigin(null);
    } finally {
      setLoading(false);
    }
  }, [repo]);

  useEffect(() => {
    void refresh();
  }, [refresh]);

  return { origin, loading, refresh };
}
