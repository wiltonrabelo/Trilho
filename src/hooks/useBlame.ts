import { useCallback, useEffect, useState } from "react";

import { getFileBlame } from "@/lib/api";
import type { BlameLineDto, BlameSourceDto } from "@/types";

/** Carrega até 20k linhas (limite do backend) — arquivos .sdm costumam ter milhares. */
const FULL_BLAME_END = 20_000;

interface UseBlameOptions {
  path: string | null;
  staged: boolean | null;
}

export function useBlame({ path, staged }: UseBlameOptions) {
  const [source, setSource] = useState<BlameSourceDto>("workingTree");
  const [lines, setLines] = useState<BlameLineDto[]>([]);
  const [focusLine, setFocusLine] = useState<number | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const activePath = path ?? null;

  useEffect(() => {
    if (staged !== null) {
      setSource(staged ? "staging" : "workingTree");
    }
  }, [activePath, staged]);

  const loadBlame = useCallback(
    async (startLine: number, endLine: number) => {
      if (!activePath) {
        setLines([]);
        return;
      }
      setLoading(true);
      setError(null);
      try {
        // Sempre blame no checkout (WT/staging/HEAD) — não no commit selecionado no histórico.
        const result = await getFileBlame(
          activePath,
          source,
          startLine,
          endLine,
        );
        setLines(result);
      } catch (e) {
        setError(String(e));
        setLines([]);
      } finally {
        setLoading(false);
      }
    },
    [activePath, source],
  );

  useEffect(() => {
    if (!activePath) {
      setLines([]);
      setFocusLine(null);
      return;
    }
    void loadBlame(1, FULL_BLAME_END);
  }, [activePath, source, loadBlame]);

  function selectLine(lineNo: number) {
    setFocusLine(lineNo);
  }

  return {
    source,
    setSource,
    lines,
    focusLine,
    loading,
    error,
    selectLine,
    reload: () => loadBlame(1, FULL_BLAME_END),
  };
}
