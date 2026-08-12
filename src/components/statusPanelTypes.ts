import type { FileChangeKind } from "@/types";

/** Contexto do clique direito em arquivo da working tree. */
export type WorktreeFileSection = "staged" | "unstaged" | "untracked";

export interface WorktreeFileContext {
  path: string;
  section: WorktreeFileSection;
  kind: FileChangeKind;
  clientX: number;
  clientY: number;
}

/** Contexto do clique direito em arquivo de um commit. */
export interface CommitFileContext {
  path: string;
  kind: FileChangeKind;
  clientX: number;
  clientY: number;
}
