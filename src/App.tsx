import { GitBranch, TrainFront, X } from "lucide-react";
import { useCallback, useEffect, useState } from "react";

import { BranchOriginBadge } from "@/components/BranchOriginBadge";
import { CloneDialog } from "@/components/CloneDialog";
import { CommitForm } from "@/components/CommitForm";
import { CommitGraph } from "@/components/CommitGraph";
import { CommitSummaryPanel } from "@/components/CommitSummaryPanel";
import { DetailPanel } from "@/components/DetailPanel";
import { OperationDialog } from "@/components/OperationDialog";
import { PublishDialog } from "@/components/PublishDialog";
import { RepoPicker } from "@/components/RepoPicker";
import { ResizableColumns } from "@/components/ResizableColumns";
import { ResizableRows } from "@/components/ResizableRows";
import { StatusPanel } from "@/components/StatusPanel";
import { SyncIndicator } from "@/components/SyncIndicator";
import { ThemeToggle } from "@/components/ThemeToggle";
import { useBlame } from "@/hooks/useBlame";
import { useBranchOrigin } from "@/hooks/useBranchOrigin";
import { useClone } from "@/hooks/useClone";
import { useCommits } from "@/hooks/useCommits";
import { useFileSelection } from "@/hooks/useFileSelection";
import { useOperations } from "@/hooks/useOperations";
import { useRepo } from "@/hooks/useRepo";
import { useRepoChanged } from "@/hooks/useRepoChanged";
import { useSync } from "@/hooks/useSync";
import { getAppInfo, getRepoInfo, runningInTauri } from "@/lib/api";
import type { AppInfo } from "@/types";

function App() {
  const [info, setInfo] = useState<AppInfo | null>(null);
  const [webOnly, setWebOnly] = useState(false);
  const [workingCopySelected, setWorkingCopySelected] = useState(true);
  const [amendIntent, setAmendIntent] = useState(0);
  const [publishOpen, setPublishOpen] = useState(false);

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
    refreshRecents,
    removeRecent,
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
    path: workingCopySelected
      ? selectedFile?.path ?? null
      : selectedFile?.path ?? selectedCommitFile ?? null,
    staged: selectedFile?.staged ?? null,
    commit: workingCopySelected ? null : selectedCommit,
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

  const { checkedPaths, clearChecks, toggleCheck, handleSelectFile } =
    useFileSelection({
    repoPath: repo?.path,
    status,
    selectedFile,
    onSelectFile: selectFile,
  });

  const refreshAll = useCallback(async () => {
    await Promise.all([
      refreshCommits(),
      refreshStatus(),
      syncRefresh(),
      refreshOrigin(),
    ]);
  }, [refreshCommits, refreshStatus, syncRefresh, refreshOrigin]);

  useRepoChanged(refreshAll);

  const refreshAfterWrite = useCallback(async () => {
    await refreshAll();
    try {
      setRepo(await getRepoInfo());
    } catch {
      /* repo pode ter fechado */
    }
    clearSelection();
    setWorkingCopySelected(true);
    clearChecks();
  }, [refreshAll, clearSelection, setRepo, clearChecks]);

  const ops = useOperations(refreshAfterWrite);

  const onCloneSuccess = useCallback(
    async (info: Awaited<ReturnType<typeof getRepoInfo>>) => {
      setRepo(info);
      await Promise.all([
        refreshCommits(),
        refreshStatus(),
        syncRefresh(),
        refreshOrigin(),
        refreshRecents(),
      ]);
      refreshCredential();
    },
    [
      setRepo,
      refreshCommits,
      refreshStatus,
      syncRefresh,
      refreshOrigin,
      refreshRecents,
      refreshCredential,
    ],
  );

  const clone = useClone(onCloneSuccess);

  const activePreview = clone.preview ?? ops.preview;
  const activeLoading = ops.loading || clone.loading;
  const confirmOperation = useCallback(() => {
    if (clone.pending) void clone.confirmClone();
    else void ops.confirm();
  }, [clone, ops]);
  const cancelOperation = useCallback(() => {
    if (clone.pending) clone.cancelPreview();
    else ops.cancel();
  }, [clone, ops]);

  const headCommit = commits[0] ?? null;
  const canAmend =
    Boolean(headCommit?.isLocalOnly) && !repo?.isDetached;
  const canUncommit =
    Boolean(
      selectedCommit &&
        headCommit &&
        selectedCommit.id === headCommit.id &&
        headCommit.isLocalOnly,
    ) && !repo?.isDetached;
  const writeDisabled = Boolean(repo?.isDetached);
  const upstreamConfigured = Boolean(repo?.upstream || sync?.upstream);
  const amendUnavailableReason =
    headCommit && !canAmend && !writeDisabled
      ? "Amend indisponível — o último commit já foi enviado ao remoto. Só é possível alterar a mensagem antes do push."
      : null;
  const isSelectedHead = Boolean(
    selectedCommit && headCommit && selectedCommit.id === headCommit.id,
  );
  const messageEditHint =
    selectedCommit && !workingCopySelected && !writeDisabled
      ? isSelectedHead && !canAmend
        ? "Este commit já está no remoto. Para corrigir a mensagem antes de enviar, use Amend em «Alterações locais» (só vale para o último commit local)."
        : !isSelectedHead
          ? "Alterar a mensagem de commits antigos (reword) ainda não está no MVP — previsto como RF-16 (pós-MVP)."
          : null
      : null;

  // Sempre abre o diálogo com a URL atual editável: quem publicou com a URL
  // errada (conta sem acesso) corrige aqui — o plano vira `remote set-url` + push.
  const handlePublish = useCallback(() => {
    if (!repo || writeDisabled) return;
    ops.cancel();
    setPublishOpen(true);
  }, [repo, writeDisabled, ops]);

  const handlePublishWithUrl = useCallback(
    async (remoteUrl: string) => {
      const url = remoteUrl.trim();
      if (!url) return;
      const preview = await ops.requestPublish(url);
      if (preview && !preview.blocked) {
        setPublishOpen(false);
      }
    },
    [ops],
  );

  const handleEditMessage = useCallback(() => {
    if (!canAmend || writeDisabled) return;
    setWorkingCopySelected(true);
    setAmendIntent((n) => n + 1);
  }, [canAmend, writeDisabled]);

  const changeCount =
    (status?.staged.length ?? 0) +
    (status?.unstaged.length ?? 0) +
    (status?.untracked.length ?? 0);

  useEffect(() => {
    setWebOnly(!runningInTauri());
    getAppInfo().then(setInfo);
  }, []);

  useEffect(() => {
    if (!repo?.path) return;
    setWorkingCopySelected(true);
    clearChecks();
    clearSelection();
  }, [repo?.path, clearSelection, clearChecks]);

  function handleSelectWorkingCopy() {
    clearSelection();
    clearFileSelection();
    setWorkingCopySelected(true);
  }

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
    setWorkingCopySelected(false);
    clearFileSelection();
    await selectCommit(commit);
  }

  async function handleSelectCommitFile(path: string) {
    clearFileSelection();
    await selectCommitFile(path);
  }

  const loading = repoLoading || commitsLoading || diffLoading || fileLoading;
  const selectedPath = selectedFile?.path ?? null;
  const fileInStaged = Boolean(
    selectedPath && status?.staged.some((f) => f.path === selectedPath),
  );
  const fileInUnstaged = Boolean(
    selectedPath && status?.unstaged.some((f) => f.path === selectedPath),
  );
  const fileInUntracked = Boolean(
    selectedPath && status?.untracked.some((f) => f.path === selectedPath),
  );
  const detailFilePath = workingCopySelected
    ? selectedFile?.path ?? null
    : selectedFile?.path ?? selectedCommitFile ?? null;
  const diff = workingCopySelected
    ? fileDiff
    : selectedFile
      ? fileDiff
      : selectedCommit
        ? selectedCommitFile
          ? commitFileDiff
          : commitDiff
        : fileDiff;

  return (
    <div className="flex h-full flex-col">
      <a href="#trilho-main" className="skip-to-main">
        Ir para o conteúdo principal
      </a>
      <OperationDialog
        preview={activePreview}
        loading={activeLoading}
        onConfirm={confirmOperation}
        onCancel={cancelOperation}
        progressLine={clone.progress}
        title={
          clone.pending
            ? "Confirmar clone"
            : ops.pending?.kind === "publish"
              ? "Confirmar publicação"
              : ops.pending?.kind === "unshallowHistory"
                ? "Completar histórico"
                : undefined
        }
      />
      <CloneDialog
        open={clone.cloneOpen}
        loading={clone.loading}
        error={clone.error}
        onCancel={clone.cancelCloneDialog}
        onContinue={(values) => void clone.requestClone(values)}
      />
      <PublishDialog
        open={publishOpen}
        branch={repo?.branch}
        initialUrl={repo?.remoteUrl}
        loading={ops.loading}
        error={publishOpen ? ops.error : null}
        onCancel={() => {
          setPublishOpen(false);
          ops.cancel();
        }}
        onContinue={(url) => void handlePublishWithUrl(url)}
      />
      {ops.error && (
        <div className="border-b border-red-500/40 bg-red-500/10 px-5 py-2 text-sm text-red-500">
          {ops.error}
        </div>
      )}
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
              branch={repo.branch}
              hasRemote={repo.hasRemote}
              upstreamConfigured={Boolean(repo.upstream || sync?.upstream)}
              isShallow={repo.isShallow}
              writeDisabled={writeDisabled}
              onFetch={fetch}
              onPublish={
                writeDisabled || upstreamConfigured ? undefined : handlePublish
              }
              onPush={
                writeDisabled
                  ? undefined
                  : () => void ops.request({ kind: "push" })
              }
              onPull={
                writeDisabled
                  ? undefined
                  : () => void ops.request({ kind: "pullFfOnly" })
              }
              onUnshallow={
                writeDisabled
                  ? undefined
                  : () => void ops.request({ kind: "unshallowHistory" })
              }
              loading={fetchLoading}
              pushLoading={ops.loading}
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
          operações de escrita desabilitadas.
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
        <main id="trilho-main" className="flex flex-1 items-center justify-center">
          <div className="w-72 rounded-xl border border-border bg-surface p-4 shadow-sm">
            <p className="mb-3 text-center text-sm text-muted">
              Abra um repositório Git para começar
            </p>
            <RepoPicker
              recentRepos={recentRepos}
              onOpen={handleOpenRepo}
              onRemoveRecent={
                runningInTauri() ? (path) => void removeRecent(path) : undefined
              }
              onClone={runningInTauri() ? clone.openClone : undefined}
              loading={repoLoading}
            />
          </div>
        </main>
      ) : (
        <main id="trilho-main" className="flex min-h-0 flex-1 flex-col">
          <ResizableColumns
            defaultRight={360}
            left={
              <>
                <RepoPicker
                  recentRepos={recentRepos}
                  onOpen={handleOpenRepo}
                  onRemoveRecent={
                    runningInTauri()
                      ? (path) => void removeRecent(path)
                      : undefined
                  }
                  onClone={runningInTauri() ? clone.openClone : undefined}
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
                <ResizableRows
                  storageKey="trilho.rows.center.v1"
                  defaultTop={360}
                  minTop={140}
                  minBottom={120}
                  top={
                    <CommitGraph
                      commits={commits}
                      selectedId={
                        workingCopySelected ? null : selectedCommit?.id ?? null
                      }
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
                      workingCopySelected={workingCopySelected}
                      changeCount={changeCount}
                      stagedCount={status?.staged.length ?? 0}
                      onSelectWorkingCopy={handleSelectWorkingCopy}
                      onSelect={(c) => void handleSelectCommit(c)}
                      onLoadMore={() => void loadMore()}
                      hasMore={hasMore}
                      loading={commitsLoading}
                    />
                  }
                  bottom={
                    <CommitSummaryPanel
                      commit={
                        workingCopySelected ? null : selectedCommit
                      }
                      canUncommit={canUncommit}
                      canEditMessage={
                        Boolean(
                          selectedCommit &&
                            headCommit &&
                            selectedCommit.id === headCommit.id &&
                            canAmend,
                        ) && !writeDisabled
                      }
                      onEditMessage={
                        canAmend && !writeDisabled
                          ? handleEditMessage
                          : undefined
                      }
                      messageEditHint={messageEditHint}
                      onRevert={
                        selectedCommit && !writeDisabled
                          ? () =>
                              void ops.request({
                                kind: "revert",
                                commitId: selectedCommit.id,
                              })
                          : undefined
                      }
                      onUncommit={
                        canUncommit && !writeDisabled
                          ? () => void ops.request({ kind: "uncommit" })
                          : undefined
                      }
                    />
                  }
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
                  <div className="flex h-full min-h-0 flex-col">
                    <div className="min-h-0 flex-1">
                      <StatusPanel
                        staged={status?.staged ?? []}
                        unstaged={status?.unstaged ?? []}
                        untracked={status?.untracked ?? []}
                        selectedPath={selectedFile?.path ?? null}
                        selectedStaged={selectedFile?.staged ?? null}
                        checkedPaths={checkedPaths}
                        onSelectFile={(p, s, meta) =>
                          void handleSelectFile(p, s, meta)
                        }
                        onToggleCheck={toggleCheck}
                        commit={
                          workingCopySelected ? null : selectedCommit
                        }
                        commitFiles={commitFiles}
                        selectedCommitFile={selectedCommitFile}
                        onSelectCommitFile={(p) =>
                          void handleSelectCommitFile(p)
                        }
                        onStage={
                          writeDisabled
                            ? undefined
                            : (p) =>
                                void ops.request({ kind: "stage", path: p })
                        }
                        onStageMany={
                          writeDisabled
                            ? undefined
                            : (paths) =>
                                void ops.request({ kind: "stageMany", paths })
                        }
                        onStageAll={
                          writeDisabled
                            ? undefined
                            : () => void ops.request({ kind: "stageAll" })
                        }
                        onUnstage={
                          writeDisabled
                            ? undefined
                            : (p) =>
                                void ops.request({ kind: "unstage", path: p })
                        }
                        onUnstageMany={
                          writeDisabled
                            ? undefined
                            : (paths) =>
                                void ops.request({
                                  kind: "unstageMany",
                                  paths,
                                })
                        }
                        onUnstageAll={
                          writeDisabled
                            ? undefined
                            : () => void ops.request({ kind: "unstageAll" })
                        }
                      />
                    </div>
                    {workingCopySelected &&
                      !writeDisabled &&
                      ((status?.staged.length ?? 0) > 0 || canAmend) && (
                      <div className="shrink-0">
                        <CommitForm
                        canAmend={canAmend}
                        amendUnavailableReason={amendUnavailableReason}
                        amendSeed={
                          headCommit
                            ? {
                                summary: headCommit.summary,
                                body: headCommit.body ?? "",
                              }
                            : null
                        }
                        amendIntent={amendIntent}
                        busy={ops.loading && ops.pending?.kind === "commit"}
                        onCommit={(summary, body, amend) => {
                          void ops.request({
                            kind: "commit",
                            summary,
                            body: body || undefined,
                            amend,
                          });
                        }}
                      />
                      </div>
                    )}
                  </div>
                }
                bottom={
                  <DetailPanel
                    commit={
                      workingCopySelected ? null : selectedCommit
                    }
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
                    workingTreeFile={Boolean(
                      selectedFile &&
                        (fileInStaged || fileInUnstaged || fileInUntracked),
                    )}
                    showStageFile={fileInUnstaged || fileInUntracked}
                    showUnstageFile={fileInStaged}
                    onStageFile={
                      selectedFile &&
                      (fileInUnstaged || fileInUntracked) &&
                      !writeDisabled
                        ? () =>
                            void ops.request({
                              kind: "stage",
                              path: selectedFile.path,
                            })
                        : undefined
                    }
                    onUnstageFile={
                      selectedFile && fileInStaged && !writeDisabled
                        ? () =>
                            void ops.request({
                              kind: "unstage",
                              path: selectedFile.path,
                            })
                        : undefined
                    }
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
