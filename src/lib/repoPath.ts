/**
 * Comparação de caminhos de repositório no frontend.
 *
 * As regras espelham `norm` (src-tauri/src/application/write_auth.rs): espaços
 * nas pontas ignorados, separador unificado, separadores repetidos colapsados
 * (mantendo o prefixo `\\` de UNC e a raiz Unix), barra final ignorada e
 * comparação case-insensitive apenas em ASCII. Divergir do backend faz o app
 * achar que trocou de repositório quando só mudou a forma de escrever o
 * caminho — ou o oposto, com a UI achando que é o mesmo repo e o gate de
 * escrita recusando o token.
 *
 * Única diferença consciente: o Rust só ignora maiúsculas no Windows. Aqui
 * ignoramos sempre, porque o frontend não conhece o sistema de arquivos e
 * errar para o lado permissivo só afeta qual item aparece selecionado.
 */
export function normalizeRepoPath(path: string): string {
  const bruto = path.trim().replace(/\//g, "\\");
  const prefixo = bruto.startsWith("\\\\")
    ? "\\\\"
    : bruto.startsWith("\\")
      ? "\\"
      : "";
  const corpo = bruto.split("\\").filter(Boolean).join("\\");
  return (prefixo + corpo).replace(/[A-Z]/g, (letra) => letra.toLowerCase());
}

export function sameRepoPath(a: string, b: string): boolean {
  return normalizeRepoPath(a) === normalizeRepoPath(b);
}
