import type { WriteRequestDto } from "@/types";

/**
 * Duas escritas descrevem a mesma operação. Usado para tirar da lista de
 * pendências a proposta que acabou de ser executada — as instâncias vêm de
 * origens diferentes (resposta do LLM x eco da execução), então a comparação
 * é por identidade da operação, não por referência.
 */
export function writesMatch(a: WriteRequestDto, b: WriteRequestDto): boolean {
  if (a.kind !== b.kind) return false;
  switch (a.kind) {
    case "stage":
    case "unstage":
    case "discardWorktree":
    case "removeUntracked":
    case "saveWorktreeFile":
      return b.kind === a.kind && a.path === b.path;
    case "stageMany":
    case "unstageMany":
    case "discardWorktreeMany":
    case "removeUntrackedMany":
      return (
        b.kind === a.kind &&
        a.paths.length === b.paths.length &&
        a.paths.every((p, i) => p === b.paths[i])
      );
    case "commit":
      return b.kind === a.kind && a.summary === b.summary;
    case "revert":
    case "reword":
      return b.kind === a.kind && a.commitId === b.commitId;
    case "cherryPick":
      return b.kind === a.kind && JSON.stringify(a) === JSON.stringify(b);
    default:
      return JSON.stringify(a) === JSON.stringify(b);
  }
}
