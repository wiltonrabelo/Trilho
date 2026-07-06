import { invoke, isTauri } from "@tauri-apps/api/core";

import type {

  AppInfo,

  CommitDto,

  FileChangeDto,

  RepoInfo,

  RepoStatusDto,

  SyncInfoDto,

  CredentialStatusDto,
  BranchOriginDto,
  BlameLineDto,
  BlameSourceDto,
  TrailEntryDto,
  OperationPreviewDto,
  WriteRequestDto,
} from "@/types";

import { MOCK_APP_INFO, MOCK_COMMITS, MOCK_REPO, MOCK_STATUS } from "@/lib/mock-data";



export function runningInTauri(): boolean {

  return isTauri();

}



export async function getAppInfo(): Promise<AppInfo> {

  if (!isTauri()) return MOCK_APP_INFO;

  return invoke<AppInfo>("get_app_info");

}



export async function listCommitsMock(): Promise<CommitDto[]> {

  if (!isTauri()) return MOCK_COMMITS;

  return invoke<CommitDto[]>("list_commits_mock");

}



export async function validateRepoPath(path: string): Promise<void> {

  if (!isTauri()) return;

  return invoke("validate_repo_path", { path });

}



export async function openRepo(path: string): Promise<RepoInfo> {

  if (!isTauri()) return MOCK_REPO;

  return invoke<RepoInfo>("open_repo", { path });

}



export async function closeRepo(): Promise<void> {

  if (!isTauri()) return;

  return invoke("close_repo");

}



export async function getRepoInfo(): Promise<RepoInfo> {

  if (!isTauri()) return MOCK_REPO;

  return invoke<RepoInfo>("get_repo_info");

}



export async function getRecentRepos(): Promise<string[]> {

  if (!isTauri()) return [];

  return invoke<string[]>("get_recent_repos");

}



export async function listCommits(
  limit = 100,
  after: string | null = null,
  firstParent = true,
): Promise<CommitDto[]> {
  if (!isTauri()) return MOCK_COMMITS;
  return invoke<CommitDto[]>("list_commits", { limit, after, firstParent });
}



/** Trilha dupla: branch atual + base até o merge-base + trilho comum. */
export async function getDualTrail(
  base: string,
  limit = 300,
): Promise<TrailEntryDto[]> {
  if (!isTauri()) {
    return MOCK_COMMITS.map((commit) => ({ commit, trail: "current" }));
  }
  return invoke<TrailEntryDto[]>("get_dual_trail", { base, limit });
}

export async function getRepoStatus(): Promise<RepoStatusDto> {

  if (!isTauri()) return MOCK_STATUS;

  return invoke<RepoStatusDto>("get_repo_status");

}



export async function getFileDiff(

  path: string,

  staged: boolean,

): Promise<string> {

  if (!isTauri()) return `diff mock — ${path} (staged=${staged})`;

  return invoke<string>("get_file_diff", { path, staged });

}



export async function getCommitDiff(commitId: string): Promise<string> {

  if (!isTauri()) return `commit diff mock — ${commitId}`;

  return invoke<string>("get_commit_diff", { commitId });

}



/** Arquivos alterados por um commit (detalhes de commit, M1). */
export async function listCommitFiles(
  commitId: string,
): Promise<FileChangeDto[]> {
  if (!isTauri()) {
    return [
      { path: "src/App.tsx", kind: "modified", staged: false },
      { path: "src/lib/graph/layout-lanes.ts", kind: "added", staged: false },
    ];
  }
  return invoke<FileChangeDto[]>("list_commit_files", { commitId });
}



/** Diff de um arquivo específico dentro de um commit. */
export async function getCommitFileDiff(
  commitId: string,
  path: string,
): Promise<string> {
  if (!isTauri()) return `commit file diff mock — ${commitId} · ${path}`;
  return invoke<string>("get_commit_file_diff", { commitId, path });
}



export async function getSyncInfo(): Promise<SyncInfoDto> {

  if (!isTauri()) {

    return { lastFetchAt: null, upstream: null, ahead: 0, behind: 0 };

  }

  return invoke<SyncInfoDto>("get_sync_info");

}



export async function fetchRemote(): Promise<SyncInfoDto> {

  if (!isTauri()) {

    return {

      lastFetchAt: new Date().toISOString(),

      upstream: "origin/main",

      ahead: 0,

      behind: 0,

    };

  }

  return invoke<SyncInfoDto>("fetch_remote");

}



export async function getCredentialStatus(): Promise<CredentialStatusDto> {

  if (!isTauri()) {

    return {

      helperConfigured: true,

      gcmAvailable: true,

      helperSummary: "mock",

      hint: null,

    };

  }

  return invoke<CredentialStatusDto>("get_credential_status");

}

export async function getBranchOrigin(): Promise<BranchOriginDto> {
  if (!isTauri()) {
    return {
      currentBranch: "master",
      candidate: "main",
      confidence: "medium",
      explanation: "Mock — origem inferida de main.",
      signals: ["mock"],
      mergeBaseId: null,
    };
  }
  return invoke<BranchOriginDto>("get_branch_origin");
}

export async function getFileBlame(
  path: string,
  source: BlameSourceDto,
  startLine: number,
  endLine: number,
  commitId?: string,
): Promise<BlameLineDto[]> {
  if (!isTauri()) {
    return [
      {
        line: startLine,
        commitId: "1b2c3d4e5f60718293a4b5c6d7e8f90a1b2c3d4e",
        shortId: "1b2c3d4",
        author: "Mock",
        authoredAt: "2026-07-02T11:05:00-03:00",
        summary: `mock blame — ${path}`,
        content: `linha ${startLine}`,
      },
    ];
  }
  return invoke<BlameLineDto[]>("get_file_blame", {
    path,
    source,
    commitId: commitId ?? null,
    startLine,
    endLine,
  });
}

export async function previewPublishOperation(
  remoteUrl?: string | null,
): Promise<OperationPreviewDto> {
  const url = remoteUrl?.trim() || null;
  if (!isTauri()) {
    return {
      commands: [
        "git -C /mock remote add origin https://github.com/user/repo.git",
        "git -C /mock push -u origin master",
      ],
      description: "Mock — publicar branch.",
      repoPath: "/mock",
      blocked: null,
    };
  }
  // Contrato com o backend: SÓ `url`. Mandar `url` + `remoteUrl` (aliases do
  // mesmo campo serde) causava `duplicate field 'url'` na deserialização.
  return invoke<OperationPreviewDto>("preview_write_operation", {
    request: { kind: "publish", url },
  });
}

export async function executePublishOperation(
  remoteUrl?: string | null,
): Promise<void> {
  const url = remoteUrl?.trim() || null;
  if (!isTauri()) return;
  return invoke("execute_write_operation", {
    request: { kind: "publish", url },
  });
}

export async function previewWriteOperation(
  request: WriteRequestDto,
): Promise<OperationPreviewDto> {
  if (!isTauri()) {
    return {
      commands: ["git -C /mock restore --staged -- file"],
      description: "Mock — preview de operação.",
      repoPath: "/mock",
      blocked: null,
    };
  }
  return invoke<OperationPreviewDto>("preview_write_operation", { request });
}

export async function executeWriteOperation(
  request: WriteRequestDto,
): Promise<void> {
  if (!isTauri()) return;
  return invoke("execute_write_operation", { request });
}

