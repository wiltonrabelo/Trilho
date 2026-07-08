import { invoke, isTauri } from "@tauri-apps/api/core";

import type {

  AppInfo,

  CommitDto,

  FileChangeDto,

  RepoInfo,

  RepoStatusDto,

  SyncInfoDto,

  CredentialStatusDto,
  SshTestResultDto,
  BranchOriginDto,
  BlameLineDto,
  BlameSourceDto,
  TrailEntryDto,
  CloneRequestDto,
  CloneResultDto,
  OperationPreviewDto,
  RemoteBranchRefDto,
  StashEntryDto,
  TagEntryDto,
  BranchDiffModeDto,
  BranchDiffSummaryDto,
  BranchPrStatusDto,
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

export async function removeRecentRepo(path: string): Promise<void> {
  if (!isTauri()) return;
  return invoke("remove_recent_repo", { path });
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

/** Commits em `branch` que não estão no histórico de HEAD (`git log branch --not HEAD`). */
export async function listBranchExclusiveCommits(
  branch: string,
  limit = 300,
  after: string | null = null,
): Promise<CommitDto[]> {
  if (!isTauri()) return MOCK_COMMITS.slice(0, 2);
  return invoke<CommitDto[]>("list_branch_exclusive_commits", {
    branch,
    limit,
    after,
  });
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

      githubConnected: false,

      githubUsername: null,

      githubAccounts: [],

      useHttpPath: false,

      sshKeys: [],

    };

  }

  return invoke<CredentialStatusDto>("get_credential_status");

}

export async function configureGcmHelper(): Promise<void> {
  if (!isTauri()) return;
  return invoke("configure_gcm_helper");
}

export async function triggerGithubLogin(
  remoteUrl?: string | null,
): Promise<void> {
  if (!isTauri()) return;
  return invoke("trigger_github_login", { remoteUrl: remoteUrl ?? null });
}

export async function storeGithubPat(pat: string): Promise<void> {
  if (!isTauri()) return;
  return invoke("store_github_pat", { pat });
}

export async function logoutGithubAccount(username: string): Promise<void> {
  if (!isTauri()) return;
  return invoke("logout_github_account", { username });
}

export async function enableGithubUseHttpPath(): Promise<void> {
  if (!isTauri()) return;
  return invoke("enable_github_use_http_path");
}

export async function testGithubSsh(): Promise<SshTestResultDto> {
  if (!isTauri()) {
    return { success: false, username: null, message: "Modo navegador — mock." };
  }
  return invoke<SshTestResultDto>("test_github_ssh");
}

export async function getSshPublicKey(name: string): Promise<string> {
  if (!isTauri()) return "ssh-ed25519 AAAA... mock";
  return invoke<string>("get_ssh_public_key", { name });
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

export async function getBranchPrStatus(): Promise<BranchPrStatusDto> {
  if (!isTauri()) {
    return {
      visible: true,
      open: [{ number: 42, title: "Mock PR aberto", url: "https://github.com/mock/repo/pull/42" }],
      merged: [],
      closed: [],
      notice: null,
    };
  }
  return invoke<BranchPrStatusDto>("get_branch_pr_status");
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

export function repoNameFromUrl(url: string): string {
  const trimmed = url.trim().replace(/\/$/, "");
  const segment = trimmed.split(/[/:]/).pop() ?? "";
  const name = segment.replace(/\.git$/i, "");
  return name || "repositorio";
}

export async function listCloneRemoteBranches(url: string): Promise<string[]> {
  if (!isTauri()) {
    return ["main", "develop"];
  }
  return invoke<string[]>("list_clone_remote_branches", { url });
}

export async function listLocalBranches(): Promise<string[]> {
  if (!isTauri()) {
    return ["main", "feature-mock"];
  }
  return invoke<string[]>("list_local_branches");
}

export async function listRemoteBranches(): Promise<RemoteBranchRefDto[]> {
  if (!isTauri()) {
    return [
      { remote: "origin", branch: "main" },
      { remote: "origin", branch: "develop" },
    ];
  }
  return invoke<RemoteBranchRefDto[]>("list_remote_branches");
}

export async function listStashes(): Promise<StashEntryDto[]> {
  if (!isTauri()) {
    return [
      {
        index: 0,
        reference: "stash@{0}",
        message: "WIP on main: mock stash",
      },
    ];
  }
  return invoke<StashEntryDto[]>("list_stashes");
}

export async function listTags(): Promise<TagEntryDto[]> {
  if (!isTauri()) {
    return [];
  }
  return invoke<TagEntryDto[]>("list_tags");
}

export async function listOrderedCompareRefs(
  refs: string[],
): Promise<string[]> {
  if (!isTauri()) {
    return refs;
  }
  return invoke<string[]>("list_ordered_compare_refs", { refs });
}

export async function listBranchDiffFiles(
  left: string,
  right: string,
  mode: BranchDiffModeDto = "mergeBase",
): Promise<BranchDiffSummaryDto> {
  if (!isTauri()) {
    return {
      left,
      right,
      mode,
      range: mode === "tips" ? `${left}..${right}` : `${left}...${right}`,
      files: [
        { path: "src/App.tsx", kind: "modified", additions: 3, deletions: 1 },
        { path: "README.md", kind: "added", additions: 10, deletions: 0 },
      ],
    };
  }
  return invoke<BranchDiffSummaryDto>("list_branch_diff_files", {
    left,
    right,
    mode,
  });
}

export async function getBranchFileDiff(
  left: string,
  right: string,
  path: string,
  mode: BranchDiffModeDto = "mergeBase",
): Promise<string> {
  if (!isTauri()) {
    return `diff --git a/${path} b/${path}\n--- a/${path}\n+++ b/${path}\n@@ -1 +1 @@\n-old\n+new\n`;
  }
  return invoke<string>("get_branch_file_diff_cmd", {
    left,
    right,
    path,
    mode,
  });
}

export async function previewCloneRemote(
  request: CloneRequestDto,
): Promise<OperationPreviewDto> {
  if (!isTauri()) {
    return {
      commands: [`git clone --progress ${request.url} ${request.parentDir}\\${request.folderName}`],
      description: "Mock — clonar repositório.",
      repoPath: request.parentDir,
      blocked: null,
    };
  }
  return invoke<OperationPreviewDto>("preview_clone_remote", { request });
}

export async function executeCloneRemote(
  request: CloneRequestDto,
): Promise<CloneResultDto> {
  if (!isTauri()) {
    return { repo: MOCK_REPO, warning: null };
  }
  return invoke<CloneResultDto>("execute_clone_remote", { request });
}

