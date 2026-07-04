import { useCallback, useEffect, useState } from "react";

import { getFileBlame } from "@/lib/api";
import type { BlameLineDto, BlameSourceDto, CommitDto } from "@/types";

interface UseBlameOptions {
  path: string | null;
  staged: boolean | null;
  commit: CommitDto | null;
}

export function useBlame({ path, staged, commit }: UseBlameOptions) {
  const [source, setSource] = useState<BlameSourceDto>("workingTree");
  const [lines, setLines] = useState<BlameLineDto[]>([]);
  const [focusLine, setFocusLine] = useState<number | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const activePath = path ?? null;

  useEffect(() => {
    if (commit && activePath) {
      // Arquivo dentro de um commit: blame sempre na fonte "commit" (naquele SHA).
      setSource("commit");
    } else if (activePath && staged === true) {
      setSource("staging");
    } else if (activePath) {
      setSource("workingTree");
    }
  }, [activePath, staged, commit]);

  const loadBlame = useCallback(
    async (startLine: number, endLine: number) => {
      // Blame exige um caminho de arquivo. Commit sem arquivo escolhido não
      // tem blame (a UI mostra apenas o diff agregado do commit).
      if (!activePath) {
        setLines([]);
        return;
      }
      setLoading(true);
      setError(null);
      try {
        const useCommitSource = Boolean(commit);
        const result = await getFileBlame(
          activePath,
          useCommitSource ? "commit" : source,
          startLine,
          endLine,
          useCommitSource ? commit!.id : undefined,
        );
        setLines(result);
      } catch (e) {
        setError(String(e));
        setLines([]);
      } finally {
        setLoading(false);
      }
    },
    [activePath, commit, source],
  );

  useEffect(() => {
    if (!activePath) {
      setLines([]);
      setFocusLine(null);
      return;
    }
    void loadBlame(1, 80);
  }, [activePath, source, loadBlame]);

  function selectLine(lineNo: number) {
    setFocusLine(lineNo);
    void loadBlame(Math.max(1, lineNo - 2), lineNo + 2);
  }

  return {
    source,
    setSource,
    lines,
    focusLine,
    loading,
    error,
    selectLine,
    reload: () => loadBlame(1, 80),
  };
}
