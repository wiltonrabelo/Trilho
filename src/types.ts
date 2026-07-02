/** DTOs espelhando o domínio Rust (ver src-tauri/src/domain). */
export interface CommitDto {
  id: string;
  shortId: string;
  summary: string;
  authorName: string;
  authoredAt: string; // ISO 8601
  isLocalOnly: boolean;
}

export interface AppInfo {
  name: string;
  version: string;
}
