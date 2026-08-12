import type { WriteRequestDto } from "@/types";

// Lista canônica dos `kind` do contrato de escrita — precisa bater com
// `shared/write-request-kinds.json` (validado em writeRequestKinds.test.ts)
// e com o enum Rust `WriteRequest` (validado em cargo test).
export const WRITE_REQUEST_KINDS = [
  "stage",
  "stageMany",
  "stageAll",
  "unstage",
  "unstageMany",
  "unstageAll",
  "commit",
  "uncommit",
  "revert",
  "cherryPick",
  "push",
  "pullFfOnly",
  "fetchRemote",
  "unshallowHistory",
  "switchBranch",
  "deleteLocalBranch",
  "deleteRemoteBranch",
  "stashPush",
  "stashApply",
  "stashPop",
  "stashDrop",
  "createTag",
  "deleteTag",
  "discardWorktree",
  "discardWorktreeMany",
  "discardWorktreeAll",
  "removeUntracked",
  "removeUntrackedMany",
  "discardHunk",
  "resolveConflictSide",
  "resolveConflictContent",
  "saveWorktreeFile",
  "abortRevert",
  "continueRevert",
  "abortMerge",
  "continueMerge",
  "abortCherryPick",
  "continueCherryPick",
  "skipRevert",
  "skipCherryPick",
  "reword",
  "reset",
  "pushForce",
  "publish",
] as const;

// Asserções em tempo de compilação: a lista cobre exatamente a união
// `WriteRequestDto["kind"]` — nem a mais, nem a menos.
type ListedKind = (typeof WRITE_REQUEST_KINDS)[number];
type KindsFaltandoNaLista = Exclude<WriteRequestDto["kind"], ListedKind>;
type KindsSobrandoNaLista = Exclude<ListedKind, WriteRequestDto["kind"]>;

const _cobreTodosOsKinds: KindsFaltandoNaLista extends never ? true : never = true;
const _semKindsExtras: KindsSobrandoNaLista extends never ? true : never = true;
void _cobreTodosOsKinds;
void _semKindsExtras;
