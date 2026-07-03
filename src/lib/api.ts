import { invoke, isTauri } from "@tauri-apps/api/core";
import type {
  AppInfo,
  CommitDto,
  RepoInfo,
  RepoStatusDto,
  SyncInfoDto,
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
  skip = 0,
): Promise<CommitDto[]> {
  if (!isTauri()) return MOCK_COMMITS;
  return invoke<CommitDto[]>("list_commits", { limit, skip });
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
