/** DTOs espelhando o domínio Rust. */

export interface CommitDto {

  id: string;

  shortId: string;

  summary: string;

  /** Corpo da mensagem (linhas após o resumo), se houver. */
  body?: string | null;

  authorName: string;

  authoredAt: string;

  isLocalOnly: boolean;

  /** SHAs dos commits pais (para layout de lanes no M1-b). */

  parentIds: string[];
  /** Refs apontando p/ o commit (branches locais/remotas, tags). */
  refs: string[];

}



export interface AppInfo {

  name: string;

  version: string;

}



export interface RepoInfo {

  path: string;

  branch: string | null;

  upstream: string | null;

  hasRemote: boolean;

  /** URL do remoto principal — pré-preenche o Publicar. */
  remoteUrl: string | null;

  isDetached: boolean;

  hasCommits: boolean;

  /** Clone raso — histórico incompleto até completar. */
  isShallow: boolean;

}



export type FileChangeKind =

  | "modified"

  | "added"

  | "deleted"

  | "renamed"

  | "untracked"

  | "conflicted";



export interface FileChangeDto {

  path: string;

  kind: FileChangeKind;

  staged: boolean;

  /** RF-20 — blocos de conflito no arquivo (quando kind === conflicted). */
  conflictBlocks?: number | null;

}

export interface OperationInProgressDto {
  kind: "revert" | "merge" | "cherryPick";
  message: string;
  canContinue?: boolean;
  /** RF-20 — `git revert|cherry-pick --skip` (não aplica a merge). */
  canSkip?: boolean;
}

export interface RepoStatusDto {

  staged: FileChangeDto[];

  unstaged: FileChangeDto[];

  untracked: FileChangeDto[];

  operationInProgress?: OperationInProgressDto | null;

}



export interface SyncInfoDto {

  lastFetchAt: string | null;

  upstream: string | null;

  ahead: number;

  behind: number;

}



export interface GithubAccountDto {
  username: string;
  isActive: boolean;
}

export interface CredentialStatusDto {

  helperConfigured: boolean;

  gcmAvailable: boolean;

  helperSummary: string | null;

  hint: string | null;

  githubConnected: boolean;

  githubUsername: string | null;

  githubAccounts?: GithubAccountDto[];

  useHttpPath?: boolean;

  sshKeys: SshKeyInfoDto[];

}

export interface SshKeyInfoDto {
  name: string;
  hasPublic: boolean;
}

export interface SshTestResultDto {
  success: boolean;
  username: string | null;
  message: string;
}

export type OriginConfidence = "high" | "medium" | "low" | "indeterminate";

export interface RemoteBranchRefDto {
  remote: string;
  branch: string;
}

export interface StashEntryDto {
  index: number;
  reference: string;
  message: string;
}

export interface TagEntryDto {
  name: string;
  commitId: string;
  shortId: string;
}

/** RF-14 — modo de comparação entre branches. */
export type BranchDiffModeDto = "mergeBase" | "tips";

export interface BranchDiffFileDto {
  path: string;
  kind: FileChangeKind;
  additions: number;
  deletions: number;
}

export interface BranchDiffSummaryDto {
  left: string;
  right: string;
  mode: BranchDiffModeDto;
  range: string;
  files: BranchDiffFileDto[];
}

export interface BranchOriginDto {
  currentBranch: string | null;
  candidate: string | null;
  confidence: OriginConfidence;
  explanation: string;
  signals: string[];
  /** Ponto de divergência (merge-base) com a candidata — marca a Trilha. */
  mergeBaseId: string | null;
}

/** RF-12 — Pull Request(s) da branch no GitHub. */
export interface PrSummaryDto {
  number: number;
  title: string;
  url: string;
}

export interface BranchPrStatusDto {
  visible: boolean;
  open: PrSummaryDto[];
  merged: PrSummaryDto[];
  closed: PrSummaryDto[];
  notice: string | null;
}

/** RF-20 — visão 3-vias de um arquivo em conflito. */
export interface ConflictSideDto {
  available: boolean;
  content: string;
}

export interface ConflictRegionDto {
  kind: "context" | "conflict" | string;
  ours: string;
  theirs: string;
  text: string;
}

export interface ConflictFileViewDto {
  path: string;
  base: ConflictSideDto;
  ours: ConflictSideDto;
  theirs: ConflictSideDto;
  worktree: string;
  regions: ConflictRegionDto[];
  conflictCount: number;
  hasMarkers: boolean;
}

export type ConflictChoiceDto =
  | "ours"
  | "theirs"
  | "both"
  | "bothTheirsFirst"
  | { custom: string };

/** Linha da trilha dupla a que o commit pertence. */
export type TrailKindDto = "current" | "base" | "shared";

export interface TrailEntryDto {
  commit: CommitDto;
  trail: TrailKindDto;
}

export type BlameSourceDto = "commit" | "workingTree" | "staging";

export interface BlameLineDto {
  line: number;
  commitId: string;
  shortId: string;
  author: string;
  authoredAt: string;
  summary: string;
  content: string;
}

/** RF-08 — pré-visualização de operação de escrita (M3). */
export interface OperationPreviewDto {
  commands: string[];
  description: string;
  repoPath: string;
  blocked: string | null;
}

export type WriteRequestDto =
  | { kind: "stage"; path: string }
  | { kind: "stageMany"; paths: string[] }
  | { kind: "stageAll" }
  | { kind: "unstage"; path: string }
  | { kind: "unstageMany"; paths: string[] }
  | { kind: "unstageAll" }
  | { kind: "commit"; summary: string; body?: string; amend?: boolean }
  | { kind: "uncommit" }
  | { kind: "revert"; commitId: string }
  | {
      kind: "cherryPick";
      commitId?: string;
      commitIds?: string[];
      recordOrigin?: boolean;
    }
  | { kind: "push" }
  | { kind: "pullFfOnly" }
  | { kind: "unshallowHistory" }
  | { kind: "switchBranch"; branch: string; trackRemote?: string | null }
  | {
      kind: "stashPush";
      message?: string | null;
      includeUntracked?: boolean;
    }
  | { kind: "stashApply"; index: number }
  | { kind: "stashPop"; index: number }
  | { kind: "stashDrop"; index: number }
  | {
      kind: "createTag";
      name: string;
      commitId: string;
      annotated?: boolean;
      message?: string | null;
      pushToRemote?: boolean;
    }
  | { kind: "deleteTag"; name: string }
  | { kind: "discardWorktree"; path: string }
  | { kind: "discardWorktreeMany"; paths: string[] }
  | { kind: "discardWorktreeAll" }
  | { kind: "removeUntracked"; path: string }
  | { kind: "removeUntrackedMany"; paths: string[] }
  | { kind: "discardHunk"; path: string; patch: string }
  | { kind: "resolveConflictSide"; path: string; side: "ours" | "theirs" }
  | { kind: "resolveConflictContent"; path: string; content: string }
  | { kind: "abortRevert" }
  | { kind: "continueRevert" }
  | { kind: "skipRevert" }
  | { kind: "abortMerge" }
  | { kind: "continueMerge" }
  | { kind: "abortCherryPick" }
  | { kind: "continueCherryPick" }
  | { kind: "skipCherryPick" }
  | {
      kind: "reword";
      commitId: string;
      summary: string;
      body?: string | null;
      forcePush?: boolean;
    }
  | {
      kind: "reset";
      commitId: string;
      mode?: "soft" | "mixed" | "hard";
      forcePush?: boolean;
    }
  | { kind: "pushForce" }
  | { kind: "publish"; url: string | null };

export interface CloneRequestDto {
  url: string;
  parentDir: string;
  folderName: string;
  branch?: string | null;
  depth?: number | null;
}

export interface CloneFormValues {
  url: string;
  parentDir: string;
  folderName: string;
  branch: string | null;
  depth: number | null;
}

export interface CloneResultDto {
  repo: RepoInfo;
  warning?: string | null;
}
