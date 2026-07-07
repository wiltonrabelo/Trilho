import { useCallback, useEffect, useState } from "react";

import { listTags } from "@/lib/api";
import type { TagEntryDto } from "@/types";

export function useTags(repoPath: string | undefined) {
  const [tags, setTags] = useState<TagEntryDto[]>([]);
  const [loading, setLoading] = useState(false);

  const refresh = useCallback(async () => {
    if (!repoPath) {
      setTags([]);
      return;
    }
    setLoading(true);
    try {
      setTags(await listTags());
    } catch {
      setTags([]);
    } finally {
      setLoading(false);
    }
  }, [repoPath]);

  useEffect(() => {
    void refresh();
  }, [refresh]);

  return { tags, loading, refresh };
}
