import { useCallback, useEffect, useRef, useState } from "react";

import {
  getCommitDiff,
  getCommitFileDiff,
  getDualTrail,
  listCommitFiles,
  listCommits,
} from "@/lib/api";
import type { CommitDto, FileChangeDto, RepoInfo, TrailKindDto } from "@/types";

const PAGE_SIZE = 100;
const DUAL_TRAIL_LIMIT = 300;

/** Visão do grafo: "trail" = trilha da branch atual + linha da base (RF-02),
 *  legível em repositórios com muitos merges; "graph" = grafo completo. */
export type GraphView = "trail" | "graph";

export function useCommits(repo: RepoInfo | null, baseBranch: string | null) {
  // Grafo completo (lanes coloridas, estilo Git Graph do VS Code) é a visão
  // primária; "Trilha da branch" fica como recorte first-parent opcional.
  const [view, setView] = useState<GraphView>("graph");
  const [commits, setCommits] = useState<CommitDto[]>([]);
  // Paralelo a `commits` na trilha dupla: linha de cada commit (current/base/shared).
  const [trails, setTrails] = useState<TrailKindDto[] | null>(null);
  const [commitSkip, setCommitSkip] = useState(0);
  const [hasMore, setHasMore] = useState(false);
  const [loading, setLoading] = useState(false);
  const [selectedCommit, setSelectedCommit] = useState<CommitDto | null>(null);
  const [commitDiff, setCommitDiff] = useState<string | null>(null);
  const [diffLoading, setDiffLoading] = useState(false);
  // Detalhes do commit: arquivos alterados + arquivo selecionado + seu diff.
  const [commitFiles, setCommitFiles] = useState<FileChangeDto[]>([]);
  const [selectedCommitFile, setSelectedCommitFile] = useState<string | null>(
    null,
  );
  const [commitFileDiff, setCommitFileDiff] = useState<string | null>(null);

  // Repositório atual em ref: usado para descartar respostas atrasadas de um
  // repo que já não está aberto. Sem isso, uma leitura em voo disparada pelo
  // watcher (RF-19) do repo anterior pode resolver *depois* da troca e
  // sobrescrever a lista — exibindo commits de outro repositório.
  const repoRef = useRef(repo);
  repoRef.current = repo;

  const refresh = useCallback(async () => {
    if (!repo) return;
    const reqRepo = repo;
    if (view === "trail" && baseBranch) {
      // Trilha dupla: branch atual + base, divergência e trilho comum.
      const entries = await getDualTrail(baseBranch, DUAL_TRAIL_LIMIT);
      if (repoRef.current !== reqRepo) return;
      setCommits(entries.map((e) => e.commit));
      setTrails(entries.map((e) => e.trail));
      setCommitSkip(0);
      setHasMore(false);
      return;
    }
    const list = await listCommits(PAGE_SIZE, 0, view === "trail");
    if (repoRef.current !== reqRepo) return;
    setCommits(list);
    setTrails(null);
    setCommitSkip(0);
    setHasMore(list.length >= PAGE_SIZE);
  }, [repo, view, baseBranch]);

  // Troca de repositório: zera a lista imediatamente para não exibir commits do
  // repo anterior enquanto o novo carrega (chaveado por path, não por view).
  useEffect(() => {
    setCommits([]);
    setTrails(null);
    setCommitSkip(0);
    setHasMore(false);
    setSelectedCommit(null);
    setCommitDiff(null);
    setCommitFiles([]);
    setSelectedCommitFile(null);
    setCommitFileDiff(null);
  }, [repo?.path]);

  useEffect(() => {
    if (!repo) return;
    void refresh();
  }, [repo, refresh]);

  async function loadMore() {
    if (!repo) return;
    const reqRepo = repo;
    const nextSkip = commitSkip + PAGE_SIZE;
    setLoading(true);
    try {
      const more = await listCommits(PAGE_SIZE, nextSkip, view === "trail");
      if (repoRef.current !== reqRepo) return;
      setCommits((prev) => [...prev, ...more]);
      setCommitSkip(nextSkip);
      setHasMore(more.length >= PAGE_SIZE);
    } finally {
      setLoading(false);
    }
  }

  async function selectCommit(commit: CommitDto) {
    const reqRepo = repo;
    setSelectedCommit(commit);
    setSelectedCommitFile(null);
    setCommitFileDiff(null);
    setCommitDiff(null);
    setCommitFiles([]);
    setDiffLoading(true);
    try {
      const [d, files] = await Promise.all([
        getCommitDiff(commit.id),
        listCommitFiles(commit.id),
      ]);
      if (repoRef.current !== reqRepo) return;
      setCommitDiff(d || "(sem alterações)");
      setCommitFiles(files);
    } catch (e) {
      if (repoRef.current !== reqRepo) return;
      setCommitDiff(`Erro: ${e}`);
    } finally {
      setDiffLoading(false);
    }
  }

  async function selectCommitFile(path: string) {
    if (!selectedCommit) return;
    const commitId = selectedCommit.id;
    setSelectedCommitFile(path);
    setCommitFileDiff(null);
    setDiffLoading(true);
    try {
      const d = await getCommitFileDiff(commitId, path);
      setCommitFileDiff(d || "(sem alterações)");
    } catch (e) {
      setCommitFileDiff(`Erro: ${e}`);
    } finally {
      setDiffLoading(false);
    }
  }

  function clearSelection() {
    setSelectedCommit(null);
    setCommitDiff(null);
    setCommitFiles([]);
    setSelectedCommitFile(null);
    setCommitFileDiff(null);
  }

  return {
    view,
    setView,
    commits,
    trails,
    hasMore,
    loading,
    selectedCommit,
    commitDiff,
    commitFiles,
    selectedCommitFile,
    commitFileDiff,
    diffLoading,
    refresh,
    loadMore,
    selectCommit,
    selectCommitFile,
    clearSelection,
  };
}
