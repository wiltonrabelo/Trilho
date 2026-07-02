import { invoke, isTauri } from "@tauri-apps/api/core";
import type { AppInfo, CommitDto } from "@/types";
import { MOCK_APP_INFO, MOCK_COMMITS } from "@/lib/mock-data";

/** Verifica se estamos dentro do WebView do Tauri (app desktop). */
export function runningInTauri(): boolean {
  return isTauri();
}

/**
 * Camada fina de acesso ao backend via IPC do Tauri.
 * Fora do Tauri (ex.: `npm run dev:web` no navegador), usa dados mock locais.
 */
export async function getAppInfo(): Promise<AppInfo> {
  if (!isTauri()) {
    return MOCK_APP_INFO;
  }
  return invoke<AppInfo>("get_app_info");
}

/** M0: retorna commits de exemplo do adaptador mock (`GitReader`). */
export async function listCommitsMock(): Promise<CommitDto[]> {
  if (!isTauri()) {
    return MOCK_COMMITS;
  }
  return invoke<CommitDto[]>("list_commits_mock");
}
