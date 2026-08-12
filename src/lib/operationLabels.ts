import type { WriteRequestDto } from "@/types";

/** Rótulo curto da operação, para chips e listas de pendências. */
export function writeLabel(req: WriteRequestDto): string {
  switch (req.kind) {
    case "stage":
      return `Stage ${req.path}`;
    case "stageMany":
      return `Stage ${req.paths.length} arquivos`;
    case "stageAll":
      return "Stage all";
    case "unstage":
      return `Unstage ${req.path}`;
    case "unstageMany":
      return `Unstage ${req.paths.length} arquivos`;
    case "unstageAll":
      return "Unstage all";
    case "commit":
      return `Commit: ${req.summary}`;
    case "push":
      return "Push";
    case "pullFfOnly":
      return "Pull (--ff-only)";
    case "fetchRemote":
      return "Fetch (refs remotas)";
    case "revert":
      return `Revert ${req.commitId.slice(0, 7)}`;
    case "cherryPick":
      return `Cherry-pick ${commitIdsResumidos(req.commitIds, req.commitId)}`;
    default:
      return req.kind;
  }
}

/** Confirmação em primeira pessoa depois que a operação foi executada. */
export function writeSuccessMessage(req: WriteRequestDto): string {
  switch (req.kind) {
    case "stage":
      return `Executado: «${req.path}» está em stage.`;
    case "stageMany":
      return `Executado: ${req.paths.length} arquivo(s) em stage.`;
    case "stageAll":
      return "Executado: todos os arquivos alterados estão em stage.";
    case "unstage":
      return `Executado: «${req.path}» voltou para working tree (unstaged).`;
    case "unstageMany":
      return `Executado: ${req.paths.length} arquivo(s) voltaram para working tree.`;
    case "unstageAll":
      return "Executado: todos os arquivos voltaram para working tree (unstaged).";
    case "commit":
      return `Executado: commit «${req.summary}».`;
    case "push":
      return "Executado: push concluído.";
    case "pullFfOnly":
      return "Executado: pull (--ff-only) concluído.";
    case "fetchRemote":
      return "Executado: fetch das refs remotas concluído.";
    case "revert":
      return `Executado: revert do commit ${req.commitId.slice(0, 7)}.`;
    case "cherryPick":
      return `Executado: cherry-pick ${commitIdsResumidos(req.commitIds, req.commitId)}.`;
    default:
      return `Executado: ${writeLabel(req)}.`;
  }
}

function commitIdsResumidos(
  varios: string[] | undefined,
  unico: string | undefined
): string {
  const ids = varios && varios.length > 0 ? varios : unico ? [unico] : [];
  return ids.map((id) => id.slice(0, 7)).join(", ") || "…";
}

/**
 * Título do diálogo de confirmação (`OperationDialog`) conforme a operação
 * pendente. `undefined` mantém o título padrão do diálogo.
 */
export function operationDialogTitle(
  clonePending: boolean,
  pending: WriteRequestDto | null
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

/** Ids curtos do cherry-pick, aceitando tanto `commitIds` quanto `commitId`. */
function shortCherryPickIds(req: {
  commitId?: string;
  commitIds?: string[];
}): string {
  const ids =
    req.commitIds && req.commitIds.length > 0
      ? req.commitIds
      : req.commitId
        ? [req.commitId]
        : [];
  return ids.map((id) => id.slice(0, 7)).join(", ");
}

/** Rótulo curto de uma escrita proposta pelo assistente (lista de ações). */
export function writeLabel(req: WriteRequestDto): string {
  switch (req.kind) {
    case "stage":
      return `Stage ${req.path}`;
    case "stageMany":
      return `Stage ${req.paths.length} arquivos`;
    case "stageAll":
      return "Stage all";
    case "unstage":
      return `Unstage ${req.path}`;
    case "unstageMany":
      return `Unstage ${req.paths.length} arquivos`;
    case "unstageAll":
      return "Unstage all";
    case "commit":
      return `Commit: ${req.summary}`;
    case "push":
      return "Push";
    case "pullFfOnly":
      return "Pull (--ff-only)";
    case "fetchRemote":
      return "Fetch (refs remotas)";
    case "revert":
      return `Revert ${req.commitId.slice(0, 7)}`;
    case "cherryPick":
      return `Cherry-pick ${shortCherryPickIds(req) || "…"}`;
    default:
      return req.kind;
  }
}

/** Mensagem de sistema no chat depois que a escrita foi executada. */
export function writeSuccessMessage(req: WriteRequestDto): string {
  switch (req.kind) {
    case "stage":
      return `Executado: «${req.path}» está em stage.`;
    case "stageMany":
      return `Executado: ${req.paths.length} arquivo(s) em stage.`;
    case "stageAll":
      return "Executado: todos os arquivos alterados estão em stage.";
    case "unstage":
      return `Executado: «${req.path}» voltou para working tree (unstaged).`;
    case "unstageMany":
      return `Executado: ${req.paths.length} arquivo(s) voltaram para working tree.`;
    case "unstageAll":
      return "Executado: todos os arquivos voltaram para working tree (unstaged).";
    case "commit":
      return `Executado: commit «${req.summary}».`;
    case "push":
      return "Executado: push concluído.";
    case "pullFfOnly":
      return "Executado: pull (--ff-only) concluído.";
    case "fetchRemote":
      return "Executado: fetch das refs remotas concluído.";
    case "revert":
      return `Executado: revert do commit ${req.commitId.slice(0, 7)}.`;
    case "cherryPick":
      return `Executado: cherry-pick ${shortCherryPickIds(req) || "…"}.`;
    default:
      return `Executado: ${writeLabel(req)}.`;
  }
}
