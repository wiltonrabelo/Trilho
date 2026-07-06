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

  | "untracked";



export interface FileChangeDto {

  path: string;

  kind: FileChangeKind;

  staged: boolean;

}



export interface RepoStatusDto {

  staged: FileChangeDto[];

  unstaged: FileChangeDto[];

  untracked: FileChangeDto[];

}



export interface SyncInfoDto {

  lastFetchAt: string | null;

  upstream: string | null;

  ahead: number;

  behind: number;

}



export interface CredentialStatusDto {

  helperConfigured: boolean;

  gcmAvailable: boolean;

  helperSummary: string | null;

  hint: string | null;

}

export type OriginConfidence = "high" | "medium" | "low" | "indeterminate";

export interface BranchOriginDto {
  currentBranch: string | null;
  candidate: string | null;
  confidence: OriginConfidence;
  explanation: string;
  signals: string[];
  /** Ponto de divergência (merge-base) com a candidata — marca a Trilha. */
  mergeBaseId: string | null;
}

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
  | { kind: "push" }
  | { kind: "pullFfOnly" }
  | { kind: "unshallowHistory" }
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
