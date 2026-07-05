import { useCallback, useEffect, useMemo, useState } from "react";

import {
  fileCheckKey,
  type FileCheckSection,
} from "@/lib/fileCheck";
import type { RepoStatusDto } from "@/types";

interface SelectedFile {
  path: string;
  staged: boolean;
}

interface UseFileSelectionOptions {
  repoPath: string | null | undefined;
  status: RepoStatusDto | null;
  selectedFile: SelectedFile | null;
  onSelectFile: (path: string, staged: boolean) => Promise<void>;
}

export function useFileSelection({
  repoPath,
  status,
  selectedFile,
  onSelectFile,
}: UseFileSelectionOptions) {
  const [checkedPaths, setCheckedPaths] = useState<Set<string>>(() => new Set());

  useEffect(() => {
    if (!repoPath) return;
    setCheckedPaths(new Set());
  }, [repoPath]);

  const toggleCheck = useCallback((path: string, section: FileCheckSection) => {
    const key = fileCheckKey(section, path);
    setCheckedPaths((prev) => {
      const next = new Set(prev);
      if (next.has(key)) next.delete(key);
      else next.add(key);
      return next;
    });
  }, []);

  const clearChecks = useCallback(() => setCheckedPaths(new Set()), []);

  const allStageablePaths = useMemo(
    () => [
      ...(status?.unstaged ?? []).map((f) => f.path),
      ...(status?.untracked ?? []).map((f) => f.path),
    ],
    [status?.unstaged, status?.untracked],
  );

  const allStagedPaths = useMemo(
    () => (status?.staged ?? []).map((f) => f.path),
    [status?.staged],
  );

  const handleSelectFile = useCallback(
    async (
      path: string,
      staged: boolean,
      meta?: { ctrlKey?: boolean; shiftKey?: boolean },
    ) => {
      if (meta?.ctrlKey) {
        toggleCheck(path, staged ? "staged" : "working");
        return;
      }
      if (meta?.shiftKey && selectedFile?.path) {
        const pool = staged ? allStagedPaths : allStageablePaths;
        const section: FileCheckSection = staged ? "staged" : "working";
        const anchor = selectedFile.path;
        const a = pool.indexOf(anchor);
        const b = pool.indexOf(path);
        if (a >= 0 && b >= 0) {
          const [from, to] = a < b ? [a, b] : [b, a];
          setCheckedPaths((prev) => {
            const next = new Set(prev);
            for (let i = from; i <= to; i++) {
              next.add(fileCheckKey(section, pool[i]!));
            }
            return next;
          });
          return;
        }
      }
      await onSelectFile(path, staged);
    },
    [
      allStageablePaths,
      allStagedPaths,
      onSelectFile,
      selectedFile?.path,
      toggleCheck,
    ],
  );

  return {
    checkedPaths,
    toggleCheck,
    clearChecks,
    handleSelectFile,
  };
}
