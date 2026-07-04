import { GitBranch, TrainFront, X } from "lucide-react";
import { useCallback, useEffect, useState } from "react";

import { BranchOriginBadge } from "@/components/BranchOriginBadge";
import { CommitGraph } from "@/components/CommitGraph";
import { DetailPanel } from "@/components/DetailPanel";
import { RepoPicker } from "@/components/RepoPicker";
import { ResizableColumns } from "@/components/ResizableColumns";
import { ResizableRows } from "@/components/ResizableRows";
import { StatusPanel } from "@/components/StatusPanel";
import { SyncIndicator } from "@/components/SyncIndicator";
import { ThemeToggle } from "@/components/ThemeToggle";
import { useBlame } from "@/hooks/useBlame";
import { useBranchOrigin } from "@/hooks/useBranchOrigin";
import { useCommits } from "@/hooks/useCommits";
import { useRepo } from "@/hooks/useRepo";
import { useRepoChanged } from "@/hooks/useRepoChanged";
import { useSync } from "@/hooks/useSync";
import { getAppInfo, runningInTauri } from "@/lib/api";
import type { AppInfo } from "@/types";

function App() {
  const [info, setInfo] = useState<AppInfo | null>(null);
  const [webOnly, setWebOnly] = useState(false);

  const {
    repo,
    setRepo,
    recentRepos,
    status,
    loading: repoLoading,
    error,
    open,
    close,
    refreshStatus,
    selectedFile,
    fileDiff,
    fileLoading,
    selectFile,
    clearFileSelection,
  } = useRepo();

  const { origin, loading: originLoading, refresh: refreshOrigin } =
    useBranchOrigin(repo);

  const {
    view,
    setView,
    commits,
    trails,
    hasMore,
    loading: commitsLoading,
    selectedCommit,
    commitDiff,
    commitFiles,
    selectedCommitFile,
    commitFileDiff,
    diffLoading,
    refresh: refreshCommits,
    loadMore,
    selectCommit,
    selectCommitFile,
    clearSelection,
  } = useCommits(repo, origin?.candidate ?? null);

  const {
    source: blameSource,
    setSource: setBlameSource,
    lines: blameLines,
    focusLine: blameFocusLine,
    loading: blameLoading,
    error: blameError,
    selectLine: selectBlameLine,
  } = useBlame({
    path: selectedFile?.path ?? selectedCommitFile ?? null,
    staged: selectedFile?.staged ?? null,
    commit: selectedCommit,
  });

  const onAfterFetch = useCallback(async () => {
    await Promise.all([refreshCommits(), refreshStatus(), refreshOrigin()]);
  }, [refreshCommits, refreshStatus, refreshOrigin]);

  const {
    sync,
    credential,
    fetchLoading,
    fetchError,
    refresh: syncRefresh,
    fetch,
    refreshCredential,
  } = useSync(repo, setRepo, onAfterFetch);

  const refreshAll = useCallback(async () => {
    await Promise.all([
      refreshCommits(),
      refreshStatus(),
      syncRefresh(),
      refreshOrigin(),
    ]);
  }, [refreshCommits, refreshStatus, syncRefresh, refreshOrigin]);

  useRepoChanged(refreshAll);

  useEffect(() => {
    setWebOnly(!runningInTauri());
    getAppInfo().then(setInfo);
  }, []);

  async function handleOpenRepo(path: string) {
    try {
      await open(path);
      await refreshCommits();
      await syncRefresh();
      refreshCredential();
    } catch {
      /* erro já em useRepo.error */
    }
  }

  async function handleSelectCommit(commit: Parameters<typeof selectCommit>[0]) {
    clearFileSelection();
    await selectCommit(commit);
  }

  async function handleSelectFile(path: string, staged: boolean) {
    clearSelection();
    await selectFile(path, staged);
  }

  async function handleSelectCommitFile(path: string) {
    clearFileSelection();
    await selectCommitFile(path);
  }

  const loading = repoLoading || commitsLoading || diffLoading || fileLoading;
  const detailFilePath = selectedFile?.path ?? selectedCommitFile ?? null;
  const diff = selectedCommit
    ? selectedCommitFile
      ? commitFileDiff
      : commitDiff
    : fileDiff;

  return (
    <div className="flex h-full flex-col">
      <header className="flex items-center justify-between border-b border-border bg-surface px-5 py-3">
        <div className="flex items-center gap-2.5">
          <TrainFront className="text-accent" size={22} />
          <div className="flex items-baseline gap-2">
            <h1 className="text-lg font-semibold tracking-tight">Trilho</h1>
            <span className="text-xs text-muted">
              {info ? `v${info.version}` : "…"}
            </span>
          </div>
          {repo && (
            <div className="ml-4 flex items-center gap-1.5 text-xs text-muted">
              <GitBranch size={14} />
              {repo.isDetached ? (
                <span className="text-amber-500">detached HEAD</span>
              ) : (
                <span>{repo.branch ?? "—"}</span>
              )}
              <BranchOriginBadge origin={origin} loading={originLoading} />
            </div>
          )}
        </div>
        <div className="flex items-center gap-4">
          {repo && (
            <SyncIndicator
              sync={sync}
              credential={credential}
              onFetch={fetch}
              loading={fetchLoading}
              error={fetchError}
            />
          )}
          <ThemeToggle />
        </div>
      </header>

      {webOnly && (
        <div className="border-b border-amber-500/40 bg-amber-500/10 px-5 py-2 text-xs">
          Modo navegador — mocks locais. Use{" "}
          <code className="font-mono">npm run dev</code> para o app desktop.
        </div>
      )}

      {repo?.isDetached && (
        <div className="border-b border-amber-500/40 bg-amber-500/10 px-5 py-2 text-xs">
          Repositório em <strong>detached HEAD</strong> — grafo em leitura;
          operações de branch desabilitadas no MVP.
        </div>
      )}

      {repo &&
        repo.hasCommits &&
        !repo.isDetached &&
        repo.branch &&
        !repo.upstream && (
          <div className="border-b border-border bg-surface px-5 py-2 text-xs text-muted">
            Branch <strong>{repo.branch}</strong> sem upstream —
            ahead/behind e fetch remoto dependem de{" "}
            <code className="font-mono">git branch -u</code>.
          </div>
        )}

      {origin &&
        origin.confidence !== "indeterminate" &&
        origin.candidate &&
        !repo?.isDetached && (
          <div
            className="border-b border-border bg-surface px-5 py-2 text-xs text-muted"
            title={origin.signals.join(" · ")}
          >
            {origin.explanation}
          </div>
        )}

      {error && (
        <div className="border-b border-red-500/40 bg-red-500/10 px-5 py-2 text-sm text-red-500">
          {error}
        </div>
      )}

      {!repo ? (
        <main className="flex flex-1 items-center justify-center">
          <div className="w-72 rounded-xl border border-border bg-surface p-4 shadow-sm">
            <p className="mb-3 text-center text-sm text-muted">
              Abra um repositório Git para começar
            </p>
            <RepoPicker
              recentRepos={recentRepos}
              onOpen={handleOpenRepo}
              loading={repoLoading}
            />
          </div>
        </main>
      ) : (
        <main className="flex min-h-0 flex-1 flex-col">
          <ResizableColumns
            defaultRight={360}
            left={
              <>
                <RepoPicker
                  recentRepos={recentRepos}
                  onOpen={handleOpenRepo}
                  loading={repoLoading}
                />
                <button
                  type="button"
                  onClick={() => void close()}
                  disabled={repoLoading}
                  className="mx-3 mb-2 flex items-center justify-center gap-1.5 rounded-lg border border-border px-3 py-1.5 text-xs text-muted hover:bg-surface hover:text-text disabled:opacity-50"
                >
                  <X size={14} />
                  Fechar repositório
                </button>
                <div
                  className="mt-auto border-t border-border px-3 py-2 text-[10px] text-muted break-all"
                  title={repo.path}
                >
                  {repo.path}
                </div>
              </>
            }
            center={
              !repo.hasCommits ? (
                <div className="flex flex-1 items-center justify-center text-sm text-muted">
                  Repositório sem commits
                </div>
              ) : (
                <CommitGraph
                  commits={commits}
                  selectedId={selectedCommit?.id ?? null}
                  view={view}
                  onViewChange={setView}
                  trails={trails}
                  divergence={
                    origin?.mergeBaseId && origin.candidate
                      ? {
                          mergeBaseId: origin.mergeBaseId,
                          baseName: origin.candidate,
                        }
                      : null
                  }
                  onSelect={(c) => void handleSelectCommit(c)}
                  onLoadMore={() => void loadMore()}
                  hasMore={hasMore}
                  loading={commitsLoading}
                />
              )
            }
            right={
              <ResizableRows
                storageKey="trilho.rows.right.v1"
                defaultTop={280}
                minTop={140}
                minBottom={200}
                top={
                  <StatusPanel
                    staged={status?.staged ?? []}
                    unstaged={status?.unstaged ?? []}
                    untracked={status?.untracked ?? []}
                    selectedPath={selectedFile?.path ?? null}
                    selectedStaged={selectedFile?.staged ?? null}
                    onSelectFile={(p, s) => void handleSelectFile(p, s)}
                    commit={selectedCommit}
                    commitFiles={commitFiles}
                    selectedCommitFile={selectedCommitFile}
                    onSelectCommitFile={(p) => void handleSelectCommitFile(p)}
                  />
                }
                bottom={
                  <DetailPanel
                    commit={selectedCommit}
                    filePath={detailFilePath}
                    diff={diff}
                    loading={loading}
                    blameSource={blameSource}
                    onBlameSourceChange={setBlameSource}
                    blameLines={blameLines}
                    blameFocusLine={blameFocusLine}
                    blameLoading={blameLoading}
                    blameError={blameError}
                    onLineClick={selectBlameLine}
                  />
                }
              />
            }
          />
        </main>
      )}
    </div>
  );
}

export default App;
