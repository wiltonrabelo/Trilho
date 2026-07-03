/** DTOs espelhando o domínio Rust. */

export interface CommitDto {

  id: string;

  shortId: string;

  summary: string;

  authorName: string;

  authoredAt: string;

  isLocalOnly: boolean;

  /** SHAs dos commits pais (para layout de lanes no M1-b). */

  parentIds: string[];

}



export interface AppInfo {

  name: string;

  version: string;

}



export interface RepoInfo {

  path: string;

  branch: string | null;

  upstream: string | null;

  isDetached: boolean;

  hasCommits: boolean;

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


