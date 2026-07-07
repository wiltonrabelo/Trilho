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
      setHasMore(false);
      return;
    }
    const list = await listCommits(PAGE_SIZE, null, view === "trail");
    if (repoRef.current !== reqRepo) return;
    setCommits(list);
    setTrails(null);
    setHasMore(list.length >= PAGE_SIZE);
  }, [repo, view, baseBranch]);

  // Troca de repositório: zera a lista imediatamente para não exibir commits do
  // repo anterior enquanto o novo carrega (chaveado por path, não por view).
  useEffect(() => {
    setCommits([]);
    setTrails(null);
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
    if (!repo || commits.length === 0) return;
    const reqRepo = repo;
    const after = commits[commits.length - 1]!.id;
    setLoading(true);
    try {
      const more = await listCommits(PAGE_SIZE, after, view === "trail");
      if (repoRef.current !== reqRepo) return;
      setCommits((prev) => [...prev, ...more]);
      setHasMore(more.length >= PAGE_SIZE);
    } finally {
      setLoading(false);
    }
  }

  const selectCommit = useCallback(async (commit: CommitDto) => {
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
  }, [repo]);

  const selectedCommitRef = useRef<CommitDto | null>(null);
  selectedCommitRef.current = selectedCommit;

  const selectCommitFile = useCallback(async (path: string) => {
    const commitId = selectedCommitRef.current?.id;
    if (!commitId) return;
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
  }, []);

  const clearSelection = useCallback(() => {
    setSelectedCommit(null);
    setCommitDiff(null);
    setCommitFiles([]);
    setSelectedCommitFile(null);
    setCommitFileDiff(null);
  }, []);

  const selectCommitBySha = useCallback(
    async (sha: string) => {
      if (!repo) return;
      const reqRepo = repo;
      const matches = (c: CommitDto) =>
        c.id === sha ||
        c.id.startsWith(sha) ||
        sha.startsWith(c.id) ||
        c.shortId === sha.slice(0, 7);

      const found = commits.find(matches);
      if (found) {
        await selectCommit(found);
        return;
      }

      const stub: CommitDto = {
        id: sha,
        shortId: sha.slice(0, 7),
        summary: `Commit ${sha.slice(0, 7)}`,
        body: null,
        authorName: "",
        authoredAt: new Date().toISOString(),
        isLocalOnly: false,
        parentIds: [],
        refs: [],
      };
      if (repoRef.current !== reqRepo) return;
      await selectCommit(stub);
    },
    [repo, commits, selectCommit],
  );

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
    selectCommitBySha,
    selectCommitFile,
    clearSelection,
  };
}
