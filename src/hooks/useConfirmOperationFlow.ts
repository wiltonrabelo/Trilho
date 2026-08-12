import { useCallback } from "react";

import {
  executeWriteOperation,
  getRepoInfo,
  getRepoStatus,
  listCommits,
  previewWriteOperation,
} from "@/lib/api";
import { errorMessage } from "@/lib/errors";
import type { RepoInfo, RepoStatusDto, WriteRequestDto } from "@/types";

import type { useClone } from "./useClone";
import type { useOperations } from "./useOperations";

type OperationsApi = ReturnType<typeof useOperations>;
type CloneApi = ReturnType<typeof useClone>;

interface ConfirmOperationFlowParams {
  ops: OperationsApi;
  clone: CloneApi;
  status: RepoStatusDto | null;
  syncUpstream: string | null | undefined;
  repoBranch: string | null | undefined;
  refreshStatus: () => Promise<void>;
  refreshAll: () => Promise<void>;
  setRepo: (repo: RepoInfo) => void;
  onAssistantWriteDone: (req: WriteRequestDto) => void;
}

/**
 * Confirmação do diálogo RF-08: executa a operação pendente (clone ou escrita)
 * e encadeia a continuação do revert quando o conflito acabou de ser resolvido.
 */
export function useConfirmOperationFlow({
  ops,
  clone,
  status,
  syncUpstream,
  repoBranch,
  refreshStatus,
  refreshAll,
  setRepo,
  onAssistantWriteDone,
}: ConfirmOperationFlowParams) {
  const confirmOperation = useCallback(async () => {
    const pendingKind = ops.pending?.kind;
    const pendingReq = ops.pending;
    const wasFromAssistant = ops.fromAssistant;
    const revertBefore = status?.operationInProgress?.kind === "revert";

    if (clone.pending) {
      await clone.confirmClone();
      return;
    }

    const ok = await ops.confirm();
    if (!ok) return;

    if (wasFromAssistant && pendingReq && pendingReq.kind !== "publish") {
      onAssistantWriteDone(pendingReq);
    }

    if (pendingKind === "push") {
      ops.setInfo(`Push concluído para ${syncUpstream ?? repoBranch ?? "remoto"}.`);
      return;
    }
    if (pendingKind === "pushForce") {
      ops.setInfo(
        `Force push concluído para ${syncUpstream ?? repoBranch ?? "remoto"}.`,
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
      const preview = await previewWriteOperation({ kind: "continueRevert" });
      if (preview.blocked) {
        throw new Error(preview.blocked);
      }
      const auth = preview.authorization?.trim();
      if (!auth) {
        throw new Error("Confirmação inválida: falta autorização do preview.");
      }
      const outcome = await executeWriteOperation(auth);
      await refreshAll();
      try {
        setRepo(await getRepoInfo());
      } catch {
        /* repo pode ter fechado */
      }
      // Outcome estruturado do backend: HEAD moveu = o revert criou commit.
      if (outcome.headMoved) {
        const latest = await listCommits(1);
        const summary = latest[0]?.summary ?? "";
        ops.setInfo(
          `Revert concluído: «${summary}». Use Push para enviar ao remoto.`,
        );
      } else {
        ops.setInfo(
          "Revert encerrado sem novo commit — «Aceitar atual» manteve o arquivo igual ao HEAD. Para desfazer o commit revertido, resolva o conflito com «Aceitar entrando».",
        );
      }
    } catch (e) {
      ops.setInfo(null);
      ops.setError(errorMessage(e));
    }
  }, [
    clone,
    ops,
    status?.operationInProgress?.kind,
    refreshStatus,
    refreshAll,
    setRepo,
    syncUpstream,
    repoBranch,
    onAssistantWriteDone,
  ]);

  const cancelOperation = useCallback(() => {
    if (clone.pending) clone.cancelPreview();
    else ops.cancel();
  }, [clone, ops]);

  return { confirmOperation, cancelOperation };
}
