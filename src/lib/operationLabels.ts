import type { WriteRequestDto } from "@/types";

/**
 * Título do diálogo de confirmação (`OperationDialog`) conforme a operação
 * pendente. `undefined` mantém o título padrão do diálogo.
 */
export function operationDialogTitle(
  clonePending: boolean,
  pending: WriteRequestDto | null,
): string | undefined {
  if (clonePending) return "Confirmar clone";
  if (!pending) return undefined;

  switch (pending.kind) {
    case "publish":
      return "Confirmar publicação";
    case "unshallowHistory":
      return "Completar histórico";
    case "switchBranch":
      return "Trocar de branch";
    case "deleteLocalBranch":
      return "Remover branch local";
    case "deleteRemoteBranch":
      return "Remover branch no remoto";
    case "stashPush":
      return "Guardar no stash";
    case "stashApply":
      return "Aplicar stash";
    case "stashPop":
      return "Aplicar e remover stash";
    case "stashDrop":
      return "Excluir stash";
    case "createTag":
      return "Criar tag";
    case "deleteTag":
      return "Excluir tag";
    case "reword":
      return pending.forcePush
        ? "Reescrever e enviar ao remoto"
        : "Reescrever mensagem";
    case "cherryPick":
      return "Cherry-pick";
    case "revert":
      return "Reverter commit";
    case "push":
      return "Enviar ao remoto";
    case "pushForce":
      return "Push forçado";
    case "discardWorktree":
    case "discardWorktreeMany":
    case "discardWorktreeAll":
    case "discardHunk":
      return "Descartar alterações";
    case "removeUntracked":
    case "removeUntrackedMany":
      return "Remover não rastreado";
    case "continueRevert":
      return "Finalizar revert";
    case "skipRevert":
      return "Pular revert";
    case "continueMerge":
      return "Finalizar merge";
    case "continueCherryPick":
      return "Finalizar cherry-pick";
    case "skipCherryPick":
      return "Pular cherry-pick";
    case "resolveConflictSide":
    case "resolveConflictContent":
      return "Resolver conflito";
    default:
      return undefined;
  }
}
