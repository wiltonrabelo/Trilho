/**
 * Comparação de caminhos de repositório no frontend.
 *
 * As regras espelham `same_repo_path` (src-tauri/src/application/write_auth.rs):
 * espaços nas pontas ignorados, separador unificado, barra final ignorada e
 * comparação case-insensitive. Divergir do backend faz o app achar que trocou
 * de repositório quando só mudou a forma de escrever o caminho.
 */
export function normalizeRepoPath(path: string): string {
  return path
    .trim()
    .replace(/[\\/]+/g, "/")
    .replace(/\/+$/, "")
    .toLowerCase();
}

export function sameRepoPath(a: string, b: string): boolean {
  return normalizeRepoPath(a) === normalizeRepoPath(b);
}
