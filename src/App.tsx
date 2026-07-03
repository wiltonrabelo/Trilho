import { GitBranch, TrainFront, X } from "lucide-react";

import { useCallback, useEffect, useState } from "react";

import { CommitGraph } from "@/components/CommitGraph";
import { DetailPanel } from "@/components/DetailPanel";
import { RepoPicker } from "@/components/RepoPicker";
import { ResizableColumns } from "@/components/ResizableColumns";
import { StatusPanel } from "@/components/StatusPanel";

import { SyncIndicator } from "@/components/SyncIndicator";

import { ThemeToggle } from "@/components/ThemeToggle";

import { useRepoChanged } from "@/hooks/useRepoChanged";

import {

  fetchRemote,

  getAppInfo,

  getCommitDiff,

  getFileDiff,

  getRecentRepos,

  getRepoInfo,

  getRepoStatus,

  getSyncInfo,

  listCommits,

  openRepo,
  closeRepo,
  runningInTauri,

} from "@/lib/api";

import type {

  AppInfo,

  CommitDto,

  RepoInfo,

  RepoStatusDto,

  SyncInfoDto,

} from "@/types";



const PAGE_SIZE = 100;



function App() {

  const [info, setInfo] = useState<AppInfo | null>(null);

  const [webOnly, setWebOnly] = useState(false);

  const [repo, setRepo] = useState<RepoInfo | null>(null);

  const [recentRepos, setRecentRepos] = useState<string[]>([]);

  const [commits, setCommits] = useState<CommitDto[]>([]);

  const [commitSkip, setCommitSkip] = useState(0);

  const [hasMoreCommits, setHasMoreCommits] = useState(false);

  const [status, setStatus] = useState<RepoStatusDto | null>(null);

  const [sync, setSync] = useState<SyncInfoDto | null>(null);

  const [selectedCommit, setSelectedCommit] = useState<CommitDto | null>(null);

  const [selectedFile, setSelectedFile] = useState<{

    path: string;

    staged: boolean;

  } | null>(null);

  const [diff, setDiff] = useState<string | null>(null);

  const [loading, setLoading] = useState(false);

  const [fetchLoading, setFetchLoading] = useState(false);

  const [fetchError, setFetchError] = useState<string | null>(null);

  const [error, setError] = useState<string | null>(null);



  const refreshRepoData = useCallback(async () => {

    if (!repo) return;

    const [commitList, repoStatus, syncInfo] = await Promise.all([

      listCommits(PAGE_SIZE, 0),

      getRepoStatus(),

      getSyncInfo(),

    ]);

    setCommits(commitList);

    setCommitSkip(0);

    setHasMoreCommits(commitList.length >= PAGE_SIZE);

    setStatus(repoStatus);

    setSync(syncInfo);

  }, [repo]);



  useRepoChanged(refreshRepoData);



  useEffect(() => {

    setWebOnly(!runningInTauri());

    getAppInfo().then(setInfo);

    getRecentRepos().then(setRecentRepos);

  }, []);



  async function handleOpenRepo(path: string) {

    setLoading(true);

    setError(null);

    try {

      const repoInfo = await openRepo(path);

      setRepo(repoInfo);

      setSelectedCommit(null);

      setSelectedFile(null);

      setDiff(null);

      const recents = await getRecentRepos();

      setRecentRepos(recents);

      const [commitList, repoStatus, syncInfo] = await Promise.all([

        listCommits(PAGE_SIZE, 0),

        getRepoStatus(),

        getSyncInfo(),

      ]);

      setCommits(commitList);

      setHasMoreCommits(commitList.length >= PAGE_SIZE);

      setStatus(repoStatus);

      setSync(syncInfo);

    } catch (e) {

      setError(String(e));

    } finally {

      setLoading(false);

    }

  }



  async function loadMoreCommits() {

    const nextSkip = commitSkip + PAGE_SIZE;

    setLoading(true);

    try {

      const more = await listCommits(PAGE_SIZE, nextSkip);

      setCommits((prev) => [...prev, ...more]);

      setCommitSkip(nextSkip);

      setHasMoreCommits(more.length >= PAGE_SIZE);

    } finally {

      setLoading(false);

    }

  }



  async function handleSelectCommit(commit: CommitDto) {

    setSelectedCommit(commit);

    setSelectedFile(null);

    setDiff(null);

    setLoading(true);

    try {

      const d = await getCommitDiff(commit.id);

      setDiff(d || "(sem alterações)");

    } catch (e) {

      setDiff(`Erro: ${e}`);

    } finally {

      setLoading(false);

    }

  }



  async function handleSelectFile(path: string, staged: boolean) {

    setSelectedFile({ path, staged });

    setSelectedCommit(null);

    setDiff(null);

    setLoading(true);

    try {

      const d = await getFileDiff(path, staged);

      setDiff(d || "(sem diff)");

    } catch (e) {

      setDiff(`Erro: ${e}`);

    } finally {

      setLoading(false);

    }

  }



  async function handleCloseRepo() {
    setLoading(true);
    setError(null);
    try {
      await closeRepo();
      setRepo(null);
      setCommits([]);
      setStatus(null);
      setSync(null);
      setSelectedCommit(null);
      setSelectedFile(null);
      setDiff(null);
      setFetchError(null);
    } catch (e) {
      setError(String(e));
    } finally {
      setLoading(false);
    }
  }

  async function handleFetch() {

    setFetchLoading(true);

    setFetchError(null);

    try {

      const info = await fetchRemote();

      setSync(info);

      await refreshRepoData();

      const updatedRepo = await getRepoInfo();

      setRepo(updatedRepo);

    } catch (e) {

      setFetchError(String(e));

    } finally {

      setFetchLoading(false);

    }

  }



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

            </div>

          )}

        </div>

        <div className="flex items-center gap-4">

          {repo && (

            <SyncIndicator

              sync={sync}

              onFetch={handleFetch}

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
          Repositório em <strong>detached HEAD</strong> — grafo em leitura; operações
          de branch desabilitadas no MVP.
        </div>
      )}

      {repo &&
        repo.hasCommits &&
        !repo.isDetached &&
        repo.branch &&
        !repo.upstream && (
          <div className="border-b border-border bg-surface px-5 py-2 text-xs text-muted">
            Branch <strong>{repo.branch}</strong> sem upstream — ahead/behind e fetch
            remoto dependem de <code className="font-mono">git branch -u</code>.
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

              loading={loading}

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
                  loading={loading}
                />
                <button
                  type="button"
                  onClick={handleCloseRepo}
                  disabled={loading}
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
                  onSelect={handleSelectCommit}
                  onLoadMore={loadMoreCommits}
                  hasMore={hasMoreCommits}
                  loading={loading}
                />
              )
            }
            right={
              <div className="grid h-full grid-rows-[minmax(0,1fr)_minmax(0,1.2fr)]">
                <StatusPanel
                  staged={status?.staged ?? []}
                  unstaged={status?.unstaged ?? []}
                  untracked={status?.untracked ?? []}
                  selectedPath={selectedFile?.path ?? null}
                  onSelectFile={handleSelectFile}
                />
                <div className="border-t border-border">
                  <DetailPanel
                    commit={selectedCommit}
                    diff={diff}
                    loading={loading}
                  />
                </div>
              </div>
            }
          />
        </main>

      )}

    </div>

  );

}



export default App;


