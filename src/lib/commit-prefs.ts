const STORAGE_KEY = "trilho.commit.prefillFileList";

/** `true` por padrão — usuário pode desligar (opt-out). */
export function getPrefillCommitFileList(): boolean {
  const value = localStorage.getItem(STORAGE_KEY);
  if (value === "false") return false;
  if (value === "true") return true;
  return true;
}

export function setPrefillCommitFileList(enabled: boolean): void {
  localStorage.setItem(STORAGE_KEY, String(enabled));
}
