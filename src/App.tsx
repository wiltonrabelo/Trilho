import { GitBranch, KeyRound, ScrollText, TrainFront, X } from "lucide-react";
import { useCallback, useEffect, useMemo, useState } from "react";

import { AuditLogDialog } from "@/components/AuditLogDialog";
import { BranchCompareDialog } from "@/components/BranchCompareDialog";
import { BranchOriginBadge } from "@/components/BranchOriginBadge";
import { PrStatusBadge } from "@/components/PrStatusBadge";
import { CommitCenterPanel } from "@/components/CommitCenterPanel";
import { ConnectDialog } from "@/components/ConnectDialog";
import { CloneDialog } from "@/components/CloneDialog";
import { CherryPickDialog } from "@/components/CherryPickDialog";
import { CommitForm } from "@/components/CommitForm";
import { CommitGraph } from "@/components/CommitGraph";
import { loadStoredTrailBase } from "@/components/TrailBaseSelector";
import { DetailPanel } from "@/components/DetailPanel";
import { OperationDialog } from "@/components/OperationDialog";
import { PublishDialog } from "@/components/PublishDialog";
import { RefsPanel } from "@/components/RefsPanel";
import { RepoPicker } from "@/components/RepoPicker";
import { ResizableColumns } from "@/components/ResizableColumns";
import { ResizableRows } from "@/components/ResizableRows";
import { StashDialog } from "@/components/StashDialog";
import { StatusPanel } from "@/components/StatusPanel";
import { ResetDialog } from "@/components/ResetDialog";
import { RewordDialog } from "@/components/RewordDialog";
import { TagDialog } from "@/components/TagDialog";
import { StatusBar } from "@/components/StatusBar";
import { SyncIndicator } from "@/components/SyncIndicator";
import { ThemeToggle } from "@/components/ThemeToggle";
import { useBlame } from "@/hooks/useBlame";
import { useBranchOrigin } from "@/hooks/useBranchOrigin";
import { useConnect } from "@/hooks/useConnect";
import { useClone } from "@/hooks/useClone";
import { useCommits } from "@/hooks/useCommits";
import { useFileSelection } from "@/hooks/useFileSelection";
import { useBranches } from "@/hooks/useBranches";
import { useOperations } from "@/hooks/useOperations";
import { usePrStatus } from "@/hooks/usePrStatus";
import { useRepo } from "@/hooks/useRepo";
import { useRepoChanged } from "@/hooks/useRepoChanged";
import { useStashes } from "@/hooks/useStashes";
import { useTags } from "@/hooks/useTags";
import { useSync } from "@/hooks/useSync";
import { getAppInfo, getRepoInfo, getRepoStatus, executeWriteOperation, listCommits, previewWriteOperation, runningInTauri } from "@/lib/api";
import type { AppInfo, AssistantWriteCompletedDto, RepoInfo } from "@/types";

function App() {
  const [info, setInfo] = useState<AppInfo | null>(null);
  const [webOnly, setWebOnly] = useState(false);
  const [workingCopySelected, setWorkingCopySelected] = useState(true);
  const [amendIntent, setAmendIntent] = useState(0);
  const [publishOpen, setPublishOpen] = useState(false);
  const [stashOpen, setStashOpen] = useState(false);
  const [tagOpen, setTagOpen] = useState(false);
  const [rewordOpen, setRewordOpen] = useState(false);
  const [cherryPickOpen, setCherryPickOpen] = useState(false);
  const [resetOpen, setResetOpen] = useState(false);
  const [branchCompareOpen, setBranchCompareOpen] = useState(false);
  const [auditLogOpen, setAuditLogOpen] = useState(false);
  const [cloneSetupWarning, setCloneSetupWarning] = useState<string | null>(null);
  const [githubPatNotice, setGithubPatNotice] = useState<string | null>(null);
  const [assistantWriteDone, setAssistantWriteDone] =
    useState<AssistantWriteCompletedDto | null>(null);
  /** Base manual da trilha comparada (`null` = auto / origem inferida). */
  const [trailBase, setTrailBase] = useState<string | null>(null);

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
    refreshSelectedFile,
  } = useRepo();

  useEffect(() => {
    if (!repo) setCloneSetupWarning(null);
  }, [repo]);

  const { origin, loading: originLoading, refresh: refreshOrigin } =
    useBranchOrigin(repo);

  useEffect(() => {
    setTrailBase(loadStoredTrailBase(repo?.path ?? null));
  }, [repo?.path]);

  const effectiveTrailBase = useMemo(() => {
    if (trailBase) return trailBase;
    if (
      origin?.candidate &&
      origin.currentBranch &&
      repo?.branch &&
      origin.currentBranch === repo.branch
    ) {
      return origin.candidate;
    }
    return null;
  }, [trailBase, origin?.candidate, origin?.currentBranch, repo?.branch]);

  const {
    view,
    setView,
    commits,
    trails,
    hasMore,
    loading: commitsLoading,
    focusedBranch,
    focusBranch,
    clearFocusedBranch,
    checkoutHeadCommit,
    selectedCommit,
    commitDiff,
    commitFiles,
    selectedCommitFile,
    commitFileDiff,
    diffLoading,
    refresh: refreshCommits,
    loadMore,
    selectCommit,
    selectCommitBySha,
    selectCommitFile,
    clearSelection,
  } = useCommits(repo, effectiveTrailBase);

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
  });

  const branchList = useBranches(repo?.path, repo?.branch);
  const stashList = useStashes(repo?.path);
  const tagList = useTags(repo?.path);

  const onAfterFetch = useCallback(async () => {
    await Promise.all([
      refreshCommits(),
      refreshStatus(),
      refreshOrigin(),
      branchList.refresh(),
      stashList.refresh(),
      tagList.refresh(),
    ]);
  }, [refreshCommits, refreshStatus, refreshOrigin, branchList.refresh, stashList.refresh, tagList.refresh]);

  const {
    sync,
    credential,
    fetchLoading,
    fetchError,
    refresh: syncRefresh,
    fetch,
    refreshCredential,
  } = useSync(repo, setRepo, onAfterFetch);

  const { prStatus, prLoading, refreshPrStatus } = usePrStatus(repo, credential);
  const prOpen = prStatus?.open[0] ?? null;
  const prBaseBranch = prOpen?.baseBranch ?? null;

  const onCredentialSaved = useCallback(() => {
    refreshCredential();
    void refreshPrStatus();
    setGithubPatNotice("Token GitHub salvo — atualizando status de PR…");
    window.setTimeout(() => setGithubPatNotice(null), 10_000);
  }, [refreshCredential, refreshPrStatus]);

  const connect = useConnect(repo?.remoteUrl, onCredentialSaved);

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
      branchList.refresh(),
      stashList.refresh(),
      tagList.refresh(),
    ]);
  }, [refreshCommits, refreshStatus, syncRefresh, refreshOrigin, branchList.refresh, stashList.refresh, tagList.refresh]);

  useRepoChanged(refreshAll);

  const refreshAfterWrite = useCallback(async () => {
    if (focusedBranch) {
      clearFocusedBranch();
      setView("graph");
    }
    await refreshAll();
    try {
      setRepo(await getRepoInfo());
    } catch {
      /* repo pode ter fechado */
    }
    await refreshSelectedFile();
    clearSelection();
    setWorkingCopySelected(true);
    clearChecks();
  }, [
    focusedBranch,
    clearFocusedBranch,
    setView,
    refreshAll,
    refreshSelectedFile,
    clearSelection,
    setRepo,
    clearChecks,
  ]);

  // Cherry-pick/revert/merge alteram a working tree da branch em checkout —
  // sair da visão de commits exclusivos e mostrar o grafo da branch atual.
  useEffect(() => {
    const op = status?.operationInProgress;
    if (
      op &&
      (op.kind === "cherryPick" ||
        op.kind === "revert" ||
        op.kind === "merge") &&
      focusedBranch
    ) {
      clearFocusedBranch();
      setView("graph");
    }
  }, [
    status?.operationInProgress,
    focusedBranch,
    clearFocusedBranch,
    setView,
  ]);

  const ops = useOperations(refreshAfterWrite);

  const onCloneSuccess = useCallback(
    async (info: RepoInfo, warning: string | null) => {
      setRepo(info);
      setCloneSetupWarning(warning);
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
  const confirmOperation = useCallback(async () => {
    const pendingKind = ops.pending?.kind;
    const pendingReq = ops.pending;
    const wasFromAssistant = ops.fromAssistant;
    const revertBefore = status?.operationInProgress?.kind === "revert";
    const headBefore =
      checkoutHeadCommit?.id ?? commits[0]?.id ?? null;

    if (clone.pending) {
      await clone.confirmClone();
      return;
    }

    const ok = await ops.confirm();
    if (!ok) return;

    if (
      wasFromAssistant &&
      pendingReq &&
      pendingReq.kind !== "publish"
    ) {
      setAssistantWriteDone({ key: Date.now(), req: pendingReq });
    }

    if (pendingKind === "push") {
      ops.setInfo(
        `Push concluído para ${sync?.upstream ?? repo?.branch ?? "remoto"}.`,
      );
      return;
    }
    if (pendingKind === "pushForce") {
      ops.setInfo(
        `Force push concluído para ${sync?.upstream ?? repo?.branch ?? "remoto"}.`,
      );
      return;
    }

    const resolvingConflict =
      pendingKind === "resolveConflictSide" ||
      pendingKind === "resolveConflictContent";
    if (!resolvingConflict || !revertBefore) return;

    await refreshStatus();
    const fresh = await getRepoStatus();
    const op = fresh.operationInProgress;
    if (op?.kind !== "revert" || !op.canContinue) return;

    try {
      await executeWriteOperation({ kind: "continueRevert" });
      await refreshAll();
      try {
        setRepo(await getRepoInfo());
      } catch {
        /* repo pode ter fechado */
      }
      const latest = await listCommits(1);
      const newHead = latest[0];
      const createdRevert =
        newHead &&
        headBefore &&
        newHead.id !== headBefore &&
        newHead.summary.toLowerCase().includes("revert");
      if (createdRevert) {
        ops.setInfo(`Revert concluído: «${newHead.summary}». Use Push para enviar ao remoto.`);
      } else {
        ops.setInfo(
          "Revert encerrado sem novo commit — «Aceitar atual» manteve o arquivo igual ao HEAD. Para desfazer o commit revertido, resolva o conflito com «Aceitar entrando».",
        );
      }
    } catch (e) {
      ops.setInfo(null);
      ops.setError(e instanceof Error ? e.message : String(e));
    }
  }, [
    clone,
    ops,
    status?.operationInProgress?.kind,
    checkoutHeadCommit?.id,
    commits,
    refreshStatus,
    refreshAll,
    setRepo,
    sync?.upstream,
    repo?.branch,
  ]);
  const cancelOperation = useCallback(() => {
    if (clone.pending) clone.cancelPreview();
    else ops.cancel();
  }, [clone, ops]);

  useEffect(() => {
    if (ops.preview && ops.pending?.kind === "stashPush") {
      setStashOpen(false);
    }
  }, [ops.preview, ops.pending]);

  useEffect(() => {
    if (ops.preview && ops.pending?.kind === "createTag") {
      setTagOpen(false);
    }
  }, [ops.preview, ops.pending]);

  useEffect(() => {
    if (ops.preview && ops.pending?.kind === "reword") {
      setRewordOpen(false);
    }
  }, [ops.preview, ops.pending]);

  useEffect(() => {
    if (ops.preview && ops.pending?.kind === "cherryPick") {
      setCherryPickOpen(false);
    }
  }, [ops.preview, ops.pending]);

  useEffect(() => {
    if (ops.preview && ops.pending?.kind === "reset") {
      setResetOpen(false);
    }
  }, [ops.preview, ops.pending]);

  const headCommit = checkoutHeadCommit ?? commits[0] ?? null;
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

  const handleSaveWorktreeFile = useCallback(
    async (content: string) => {
      if (!selectedFile || writeDisabled) return;
      const { path, staged } = selectedFile;
      const preview = await previewWriteOperation({
        kind: "saveWorktreeFile",
        path,
        content,
      });
      if (preview.blocked) {
        throw new Error(preview.blocked);
      }
      await executeWriteOperation({ kind: "saveWorktreeFile", path, content });
      await refreshStatus();
      await selectFile(path, staged);
    },
    [selectedFile, writeDisabled, refreshStatus, selectFile],
  );

  const upstreamConfigured = Boolean(repo?.upstream || sync?.upstream);
  const amendUnavailableReason =
    headCommit && !canAmend && !writeDisabled
      ? "Amend indisponível — o último commit já foi enviado ao remoto. Só é possível alterar a mensagem antes do push."
      : null;
  const isSelectedHead = Boolean(
    selectedCommit && headCommit && selectedCommit.id === headCommit.id,
  );
  const canReword =
    Boolean(
      selectedCommit &&
        !isSelectedHead &&
        !workingCopySelected &&
        !focusedBranch &&
        (selectedCommit.isLocalOnly || upstreamConfigured),
    ) && !writeDisabled;
  const rewordRequiresForcePush = Boolean(
    selectedCommit && !selectedCommit.isLocalOnly && upstreamConfigured,
  );
  const canReset = Boolean(
    selectedCommit &&
      headCommit &&
      selectedCommit.id !== headCommit.id &&
      !workingCopySelected &&
      !writeDisabled &&
      !focusedBranch,
  );
  const resetRequiresForcePush = Boolean(
    canReset && headCommit && !headCommit.isLocalOnly && upstreamConfigured,
  );
  const resetHint =
    canReset && headCommit && !headCommit.isLocalOnly
      ? "Reset reescreve o histórico local. Se os commits já estão no remoto, use Force push no sync quando quiser publicar (não é automático)."
      : null;
  const canRevert = Boolean(
    selectedCommit &&
      headCommit &&
      selectedCommit.id !== headCommit.id &&
      selectedCommit.parentIds.length <= 1 &&
      !workingCopySelected &&
      !writeDisabled &&
      !focusedBranch,
  );
  const revertBlockedReason =
    selectedCommit && !workingCopySelected && !writeDisabled && !canRevert
      ? focusedBranch
        ? "Saia do foco de branch para reverter commits."
        : isSelectedHead
          ? "Não é possível reverter o HEAD (último commit). Selecione um commit anterior — não merge."
          : selectedCommit.parentIds.length > 1
            ? "Revert de commit de merge não está disponível nesta versão. Reverta os commits individuais da branch mesclada."
            : null
      : null;
  const revertInfoHint = canRevert
    ? "Revert cria um commit local que desfaz o selecionado. Não envia ao remoto — use Push depois."
    : null;
  const canEditMessageOnHead = Boolean(isSelectedHead && canAmend) && !writeDisabled;
  const messageEditHint =
    selectedCommit && !workingCopySelected && !writeDisabled
      ? isSelectedHead && !canAmend
        ? "Este commit já está no remoto. Para corrigir a mensagem antes de enviar, use Amend em «Alterações locais» (só vale para o último commit local)."
        : !isSelectedHead && !selectedCommit.isLocalOnly && !upstreamConfigured
          ? "Este commit já foi enviado, mas a branch não tem upstream — configure o remoto antes do reword."
          : null
      : null;

  // Sempre abre o diálogo com a URL atual editável: quem publicou com a URL
  // errada (conta sem acesso) corrige aqui — o plano vira `remote set-url` + push.
  const handleCherryPick = useCallback(() => {
    if (writeDisabled || !selectedCommit || !headCommit) return;
    if (selectedCommit.id === headCommit.id) return;
    setCherryPickOpen(true);
  }, [writeDisabled, selectedCommit, headCommit]);

  const handleReset = useCallback(() => {
    if (!canReset || !selectedCommit) return;
    setResetOpen(true);
  }, [canReset, selectedCommit]);

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
    if (writeDisabled || !selectedCommit) return;
    if (isSelectedHead) {
      if (!canAmend) return;
      setWorkingCopySelected(true);
      setAmendIntent((n) => n + 1);
      return;
    }
    if (canReword) setRewordOpen(true);
  }, [canAmend, canReword, isSelectedHead, selectedCommit, writeDisabled]);

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
    setCloneSetupWarning(null);
    try {
      await open(path);
      await syncRefresh();
      refreshCredential();
    } catch {
      /* erro já em useRepo.error; useCommits recarrega via repo.path */
    }
  }

  async function handleSelectCommit(commit: Parameters<typeof selectCommit>[0]) {
    setWorkingCopySelected(false);
    clearFileSelection();
    await selectCommit(commit);
  }

  async function handleSelectTag(commitId: string) {
    setWorkingCopySelected(false);
    clearFileSelection();
    await selectCommitBySha(commitId);
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
  const fileConflicted = Boolean(
    selectedPath &&
      (status?.staged.some(
        (f) => f.path === selectedPath && f.kind === "conflicted",
      ) ||
        status?.unstaged.some(
          (f) => f.path === selectedPath && f.kind === "conflicted",
        )),
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
        error={ops.error}
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
                : ops.pending?.kind === "switchBranch"
                  ? "Trocar de branch"
                  : ops.pending?.kind === "stashPush"
                    ? "Guardar no stash"
                    : ops.pending?.kind === "stashApply"
                      ? "Aplicar stash"
                      : ops.pending?.kind === "stashPop"
                        ? "Aplicar e remover stash"
                        : ops.pending?.kind === "stashDrop"
                          ? "Excluir stash"
                          : ops.pending?.kind === "createTag"
                            ? "Criar tag"
                            : ops.pending?.kind === "deleteTag"
                              ? "Excluir tag"
                              : ops.pending?.kind === "reword"
                                ? ops.pending.forcePush
                                  ? "Reescrever e enviar ao remoto"
                                  : "Reescrever mensagem"
                                : ops.pending?.kind === "cherryPick"
                                  ? "Cherry-pick"
                                  : ops.pending?.kind === "revert"
                                    ? "Reverter commit"
                                    : ops.pending?.kind === "push"
                                      ? "Enviar ao remoto"
                                      : ops.pending?.kind === "pushForce"
                                    ? "Push forçado"
                                    : ops.pending?.kind === "discardWorktree" ||
                                  ops.pending?.kind === "discardWorktreeMany" ||
                                  ops.pending?.kind === "discardWorktreeAll" ||
                                  ops.pending?.kind === "discardHunk"
                                ? "Descartar alterações"
                                : ops.pending?.kind === "removeUntracked" ||
                                    ops.pending?.kind === "removeUntrackedMany"
                                  ? "Remover não rastreado"
                                  : ops.pending?.kind === "continueRevert"
                                    ? "Finalizar revert"
                                    : ops.pending?.kind === "skipRevert"
                                      ? "Pular revert"
                                    : ops.pending?.kind === "continueMerge"
                                      ? "Finalizar merge"
                                      : ops.pending?.kind === "continueCherryPick"
                                        ? "Finalizar cherry-pick"
                                        : ops.pending?.kind === "skipCherryPick"
                                          ? "Pular cherry-pick"
                                        : ops.pending?.kind ===
                                              "resolveConflictSide" ||
                                            ops.pending?.kind ===
                                              "resolveConflictContent"
                                          ? "Resolver conflito"
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
      <StashDialog
        open={stashOpen}
        counts={{
          staged: status?.staged.length ?? 0,
          unstaged: status?.unstaged.length ?? 0,
          untracked: status?.untracked.length ?? 0,
        }}
        loading={ops.loading}
        error={stashOpen && !ops.preview ? ops.error : null}
        onCancel={() => {
          setStashOpen(false);
          ops.cancel();
        }}
        onContinue={(message, includeUntracked) => {
          void ops.request({
            kind: "stashPush",
            message: message || null,
            includeUntracked,
          });
        }}
      />
      <CherryPickDialog
        open={cherryPickOpen}
        primaryCommit={selectedCommit}
        candidates={
          focusedBranch && selectedCommit
            ? commits
            : selectedCommit
              ? [selectedCommit]
              : []
        }
        loading={ops.loading}
        error={cherryPickOpen && !ops.preview ? ops.error : null}
        onCancel={() => {
          setCherryPickOpen(false);
          ops.cancel();
        }}
        onContinue={(commitIds, recordOrigin) => {
          void ops.request({
            kind: "cherryPick",
            commitIds,
            recordOrigin,
          });
        }}
      />
      <RewordDialog
        open={rewordOpen}
        shortId={selectedCommit?.shortId ?? "—"}
        initialSummary={selectedCommit?.summary ?? ""}
        initialBody={selectedCommit?.body ?? ""}
        requiresForcePush={rewordRequiresForcePush}
        loading={ops.loading}
        error={rewordOpen && !ops.preview ? ops.error : null}
        onCancel={() => {
          setRewordOpen(false);
          ops.cancel();
        }}
        onContinue={(summary, body, forcePush) => {
          if (!selectedCommit) return;
          void ops.request({
            kind: "reword",
            commitId: selectedCommit.id,
            summary,
            body: body || null,
            forcePush,
          });
        }}
      />
      <ResetDialog
        open={resetOpen}
        shortId={selectedCommit?.shortId ?? "—"}
        remoteWillDiverge={resetRequiresForcePush}
        loading={ops.loading}
        error={resetOpen && !ops.preview ? ops.error : null}
        onCancel={() => {
          setResetOpen(false);
          ops.cancel();
        }}
        onContinue={(mode) => {
          if (!selectedCommit) return;
          void ops.request({
            kind: "reset",
            commitId: selectedCommit.id,
            mode,
            forcePush: false,
          });
        }}
      />
      <BranchCompareDialog
        open={branchCompareOpen}
        localBranches={branchList.branches}
        remoteBranches={branchList.remoteBranches}
        currentBranch={repo?.branch}
        onClose={() => setBranchCompareOpen(false)}
      />
      <TagDialog
        open={tagOpen}
        commitShortId={selectedCommit?.shortId ?? "—"}
        hasRemote={repo?.hasRemote}
        loading={ops.loading}
        error={tagOpen && !ops.preview ? ops.error : null}
        onCancel={() => {
          setTagOpen(false);
          ops.cancel();
        }}
        onContinue={({ name, annotated, message, pushToRemote }) => {
          if (!selectedCommit) return;
          void ops.request({
            kind: "createTag",
            name,
            commitId: selectedCommit.id,
            annotated,
            message: message || null,
            pushToRemote,
          });
        }}
      />
      <ConnectDialog
        open={connect.open}
        credential={credential}
        remoteUrl={repo?.remoteUrl}
        loading={connect.loading}
        error={connect.error}
        sshTest={connect.sshTest}
        copyHint={connect.copyHint}
        onCancel={connect.cancel}
        onGcmLogin={() => void connect.loginGcm(repo?.remoteUrl)}
        onSavePat={(pat) => void connect.savePat(pat)}
        onConfigureGcm={() => void connect.configureGcm()}
        onTestSsh={() => void connect.testSsh()}
        onCopyPublicKey={(name) => void connect.copyPublicKey(name)}
        onLogoutAccount={(username) => void connect.logoutAccount(username)}
        onEnableUseHttpPath={() => void connect.enableUseHttpPath()}
      />
      <AuditLogDialog open={auditLogOpen} onClose={() => setAuditLogOpen(false)} />
      {githubPatNotice && (
        <div className="flex items-start justify-between gap-3 border-b border-emerald-500/40 bg-emerald-500/10 px-5 py-2 text-sm text-emerald-800 dark:text-emerald-200">
          <p>{githubPatNotice}</p>
          <button
            type="button"
            onClick={() => setGithubPatNotice(null)}
            className="shrink-0 text-emerald-700 hover:text-emerald-900 dark:text-emerald-300"
            aria-label="Fechar aviso"
          >
            <X size={14} />
          </button>
        </div>
      )}
      {ops.error && (
        <div className="border-b border-red-500/40 bg-red-500/10 px-5 py-2 text-sm text-red-500">
          {ops.error}
        </div>
      )}
      {ops.info && (
        <div className="flex items-start justify-between gap-3 border-b border-amber-500/40 bg-amber-500/10 px-5 py-2 text-sm text-amber-800 dark:text-amber-200">
          <p>{ops.info}</p>
          <button
            type="button"
            onClick={() => ops.clearInfo()}
            className="shrink-0 text-amber-700 hover:text-amber-900 dark:text-amber-300"
            aria-label="Fechar aviso"
          >
            <X size={14} />
          </button>
        </div>
      )}
      <header className="flex shrink-0 items-center justify-between border-b border-border bg-header px-4 py-2 text-headerFg">
        <div className="flex items-center gap-3">
          <TrainFront className="text-accent" size={20} />
          <div className="flex items-baseline gap-2">
            <h1 className="text-sm font-semibold tracking-tight">Trilho</h1>
            <span className="text-[11px] text-muted">
              {info ? `v${info.version}` : "…"}
            </span>
          </div>
          {repo && (
            <div className="ml-2 flex items-center gap-1.5 border-l border-border pl-3 text-xs text-muted">
              <GitBranch size={13} />
              {repo.isDetached ? (
                <span className="text-amber-600 dark:text-amber-400">detached HEAD</span>
              ) : (
                <span className="font-medium text-text">{repo.branch ?? "—"}</span>
              )}
              <BranchOriginBadge
                origin={origin}
                loading={originLoading}
                prBaseBranch={prBaseBranch}
                prNumber={prOpen?.number ?? null}
              />
              <PrStatusBadge status={prStatus} loading={prLoading} />
            </div>
          )}
        </div>
        <div className="flex items-center gap-4">
          {repo && (
            <SyncIndicator
              sync={sync}
              credential={credential}
              branch={repo.branch}
              remoteUrl={repo.remoteUrl}
              sshUsername={
                connect.sshTest?.success ? connect.sshTest.username : null
              }
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
              onPushForce={
                writeDisabled
                  ? undefined
                  : () => void ops.request({ kind: "pushForce" })
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
              error={fetchError || ops.error}
              onConnect={connect.openDialog}
            />
          )}
          {!repo && !webOnly && (
            <button
              type="button"
              onClick={connect.openDialog}
              className="flex items-center gap-1 rounded border border-border px-2 py-1 text-xs text-muted hover:bg-surface"
              title="Conectar conta GitHub"
            >
              <KeyRound size={14} />
              GitHub
            </button>
          )}
          {!webOnly && (
            <button
              type="button"
              onClick={() => setAuditLogOpen(true)}
              className="flex items-center gap-1 rounded border border-border px-2 py-1 text-xs text-muted hover:bg-surface"
              title="Histórico de ações (auditoria)"
            >
              <ScrollText size={14} />
              Ações
            </button>
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

      {cloneSetupWarning && (
        <div className="border-b border-amber-500/40 bg-amber-500/10 px-5 py-2 text-sm text-amber-700 dark:text-amber-300">
          {cloneSetupWarning}
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
              currentPath={null}
            />
          </div>
        </main>
      ) : (
        <main id="trilho-main" className="flex min-h-0 flex-1 flex-col">
          <ResizableColumns
            defaultRight={360}
            left={
              <div className="flex min-h-0 flex-1 flex-col overflow-hidden">
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
                  currentPath={repo?.path ?? null}
                />
                <RefsPanel
                  branches={branchList.branches}
                  remoteBranches={branchList.remoteBranches}
                  tags={tagList.tags}
                  stashes={stashList.stashes}
                  currentBranch={repo.branch}
                  focusedBranch={focusedBranch}
                  loading={branchList.loading}
                  tagsLoading={tagList.loading}
                  stashesLoading={stashList.loading}
                  writeDisabled={writeDisabled}
                  onFocusBranch={focusBranch}
                  onSwitchLocal={(branch) =>
                    void ops.request({ kind: "switchBranch", branch })
                  }
                  onSwitchRemote={(remote, branch) =>
                    void ops.request({
                      kind: "switchBranch",
                      branch,
                      trackRemote: remote,
                    })
                  }
                  onStashApply={(index) =>
                    void ops.request({ kind: "stashApply", index })
                  }
                  onStashPop={(index) =>
                    void ops.request({ kind: "stashPop", index })
                  }
                  onStashDrop={(index) =>
                    void ops.request({ kind: "stashDrop", index })
                  }
                  onTagSelect={(commitId) => void handleSelectTag(commitId)}
                  onTagDelete={(name) =>
                    void ops.request({ kind: "deleteTag", name })
                  }
                  onCompareBranches={() => setBranchCompareOpen(true)}
                />
                <div className="mt-auto shrink-0 border-t border-border pt-3">
                  <button
                    type="button"
                    onClick={() => void close()}
                    disabled={repoLoading}
                    className="mx-3 mb-2 flex shrink-0 items-center justify-center gap-1.5 rounded-lg border border-border px-3 py-1.5 text-xs text-muted hover:bg-surface hover:text-text disabled:opacity-50"
                  >
                    <X size={14} />
                    Fechar repositório
                  </button>
                  <div
                    className="px-3 pb-2 text-[10px] text-muted break-all"
                    title={repo.path}
                  >
                    {repo.path}
                  </div>
                </div>
              </div>
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
                      divergence={(() => {
                        if (focusedBranch || !effectiveTrailBase) return null;
                        // Na trilha dupla, o 1º commit «shared» é o merge-base
                        // (inclui base manual ≠ origem inferida).
                        let mergeBaseId: string | null = null;
                        if (trails && trails.length === commits.length) {
                          const i = trails.findIndex((t) => t === "shared");
                          if (i >= 0) mergeBaseId = commits[i]?.id ?? null;
                        }
                        if (
                          !mergeBaseId &&
                          effectiveTrailBase === origin?.candidate
                        ) {
                          mergeBaseId = origin?.mergeBaseId ?? null;
                        }
                        return mergeBaseId
                          ? {
                              mergeBaseId,
                              baseName: effectiveTrailBase,
                            }
                          : null;
                      })()}
                      focusedBranch={focusedBranch}
                      currentBranch={repo.branch}
                      checkoutHeadId={checkoutHeadCommit?.id ?? null}
                      onClearFocusedBranch={clearFocusedBranch}
                      workingCopySelected={workingCopySelected}
                      changeCount={changeCount}
                      stagedCount={status?.staged.length ?? 0}
                      onSelectWorkingCopy={handleSelectWorkingCopy}
                      onSelect={(c) => void handleSelectCommit(c)}
                      onLoadMore={() => void loadMore()}
                      hasMore={hasMore}
                      loading={commitsLoading}
                      repoPath={repo.path}
                      suggestedBase={origin?.candidate ?? null}
                      trailBase={trailBase}
                      onTrailBaseChange={setTrailBase}
                    />
                  }
                  bottom={
                    <CommitCenterPanel
                      commit={
                        workingCopySelected ? null : selectedCommit
                      }
                      canUncommit={canUncommit}
                      canEditMessage={canEditMessageOnHead || canReword}
                      onEditMessage={
                        (canEditMessageOnHead || canReword) && !writeDisabled
                          ? handleEditMessage
                          : undefined
                      }
                      messageEditHint={messageEditHint}
                      onRevert={
                        canRevert
                          ? () =>
                              void ops.request({
                                kind: "revert",
                                commitId: selectedCommit!.id,
                              })
                          : undefined
                      }
                      revertBlockedReason={revertBlockedReason}
                      revertInfoHint={revertInfoHint}
                      onReset={
                        canReset && !writeDisabled ? handleReset : undefined
                      }
                      resetHint={resetHint}
                      cherryPickHint={
                        selectedCommit &&
                        headCommit &&
                        selectedCommit.id !== headCommit.id
                          ? selectedCommit.parentIds.length > 1
                            ? "Cherry-pick não está disponível para commits de merge nesta versão."
                            : "Cherry-pick traz as alterações de um commit de outra branch para a branch atual (checkout)."
                          : null
                      }
                      onCherryPick={
                        selectedCommit &&
                        !writeDisabled &&
                        headCommit &&
                        selectedCommit.id !== headCommit.id &&
                        selectedCommit.parentIds.length <= 1
                          ? handleCherryPick
                          : undefined
                      }
                      onCreateTag={
                        selectedCommit && !workingCopySelected
                          ? () => setTagOpen(true)
                          : undefined
                      }
                      onUncommit={
                        canUncommit && !writeDisabled
                          ? () => void ops.request({ kind: "uncommit" })
                          : undefined
                      }
                      writeDisabled={writeDisabled}
                      uiContext={{
                        selectedCommitId: workingCopySelected
                          ? null
                          : selectedCommit?.id ?? null,
                        selectedCommitSummary: workingCopySelected
                          ? null
                          : selectedCommit?.summary ?? null,
                        selectedFilePath: detailFilePath,
                        blameFocusLine: blameFocusLine,
                        workingCopySelected,
                      }}
                      onProposeWrite={(req) =>
                        void ops.request(req, { fromAssistant: true })
                      }
                      writeCompleted={assistantWriteDone}
                      onWriteCompletedAck={() => setAssistantWriteDone(null)}
                    />
                  }
                />
              )
            }
            right={
              <ResizableRows
                storageKey="trilho.rows.right.v1"
                defaultTop={fileConflicted && workingCopySelected ? 420 : 280}
                minTop={140}
                minBottom={120}
                top={
                  <div className="flex h-full min-h-0 flex-col">
                    <div className="min-h-0 flex-1">
                      <StatusPanel
                        staged={status?.staged ?? []}
                        unstaged={status?.unstaged ?? []}
                        untracked={status?.untracked ?? []}
                        operationInProgress={status?.operationInProgress}
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
                        onStash={
                          writeDisabled
                            ? undefined
                            : () => setStashOpen(true)
                        }
                        onDiscard={
                          writeDisabled
                            ? undefined
                            : (p) =>
                                void ops.request({
                                  kind: "discardWorktree",
                                  path: p,
                                })
                        }
                        onDiscardMany={
                          writeDisabled
                            ? undefined
                            : (paths) =>
                                void ops.request({
                                  kind: "discardWorktreeMany",
                                  paths,
                                })
                        }
                        onDiscardAll={
                          writeDisabled
                            ? undefined
                            : () => void ops.request({ kind: "discardWorktreeAll" })
                        }
                        onRemoveUntracked={
                          writeDisabled
                            ? undefined
                            : (p) =>
                                void ops.request({
                                  kind: "removeUntracked",
                                  path: p,
                                })
                        }
                        onRemoveUntrackedMany={
                          writeDisabled
                            ? undefined
                            : (paths) =>
                                void ops.request({
                                  kind: "removeUntrackedMany",
                                  paths,
                                })
                        }
                        onAbortOperation={
                          writeDisabled
                            ? undefined
                            : (kind) => {
                                const req =
                                  kind === "revert"
                                    ? { kind: "abortRevert" as const }
                                    : kind === "merge"
                                      ? { kind: "abortMerge" as const }
                                      : { kind: "abortCherryPick" as const };
                                void ops.request(req);
                              }
                        }
                        onContinueOperation={
                          writeDisabled
                            ? undefined
                            : (kind) => {
                                const req =
                                  kind === "revert"
                                    ? { kind: "continueRevert" as const }
                                    : kind === "merge"
                                      ? { kind: "continueMerge" as const }
                                      : { kind: "continueCherryPick" as const };
                                void ops.request(req);
                              }
                        }
                        onSkipOperation={
                          writeDisabled
                            ? undefined
                            : (kind) => {
                                if (kind === "merge") return;
                                const req =
                                  kind === "revert"
                                    ? { kind: "skipRevert" as const }
                                    : { kind: "skipCherryPick" as const };
                                void ops.request(req);
                              }
                        }
                      />
                    </div>
                    {workingCopySelected &&
                      !writeDisabled &&
                      !status?.operationInProgress &&
                      ((status?.staged.length ?? 0) > 0 || canAmend) && (
                      <div className="shrink-0">
                        <CommitForm
                        canAmend={canAmend}
                        stagedCount={status?.staged.length ?? 0}
                        stagedFiles={status?.staged ?? []}
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
                    conflicted={fileConflicted && workingCopySelected}
                    conflictOperationKind={status?.operationInProgress?.kind ?? null}
                    writeDisabled={writeDisabled}
                    onResolveConflictSide={
                      fileConflicted && selectedFile && !writeDisabled
                        ? (side) =>
                            void ops.request({
                              kind: "resolveConflictSide",
                              path: selectedFile.path,
                              side,
                            })
                        : undefined
                    }
                    onResolveConflictContent={
                      fileConflicted && selectedFile && !writeDisabled
                        ? (content) =>
                            void ops.request({
                              kind: "resolveConflictContent",
                              path: selectedFile.path,
                              content,
                            })
                        : undefined
                    }
                    onDiscardHunk={
                      selectedFile &&
                      (fileInUnstaged || fileInStaged) &&
                      !writeDisabled
                        ? (patch) =>
                            void ops.request({
                              kind: "discardHunk",
                              path: selectedFile.path,
                              patch,
                              staged: Boolean(selectedFile.staged),
                            })
                        : undefined
                    }
                    canRevertHunks={
                      Boolean(
                        selectedFile &&
                          (fileInUnstaged || fileInStaged) &&
                          !writeDisabled,
                      )
                    }
                    onSaveWorktreeFile={
                      selectedFile &&
                      (fileInUnstaged || fileInStaged || fileInUntracked) &&
                      !fileConflicted &&
                      !writeDisabled
                        ? handleSaveWorktreeFile
                        : undefined
                    }
                    fileReloadKey={fileDiff}
                    branchName={repo?.branch ?? null}
                  />
                }
              />
            }
          />
        </main>
      )}

      {repo && (
        <StatusBar
          branch={repo.branch}
          isDetached={repo.isDetached}
          repoPath={repo.path}
          sync={sync}
          changeCount={changeCount}
          upstreamConfigured={Boolean(repo.upstream || sync?.upstream)}
        />
      )}
    </div>
  );
}

export default App;
