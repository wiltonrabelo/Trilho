/**
 * Em `tauri dev`, se o Vite cair (porta 1420) a WebView fica branca.
 * Quando o servidor voltar e a UI estiver vazia, recarrega a página.
 */
export function installDevBlankRecovery(): void {
  if (!import.meta.env.DEV) return;

  let reloading = false;

  const rootEmpty = () => {
    const root = document.getElementById("root");
    return !root || root.childElementCount === 0;
  };

  const tryRecover = async () => {
    if (reloading || document.visibilityState === "hidden") return;
    try {
      const res = await fetch(`${window.location.origin}/@vite/client`, {
        cache: "no-store",
      });
      if (!res.ok) return;
    } catch {
      return;
    }
    if (!rootEmpty()) return;
    reloading = true;
    window.location.reload();
  };

  document.addEventListener("visibilitychange", () => {
    if (document.visibilityState === "visible") void tryRecover();
  });
  window.setInterval(() => void tryRecover(), 12_000);

  if (import.meta.hot) {
    import.meta.hot.on("vite:ws:disconnect", () => {
      console.warn("Vite HMR desconectado — a UI pode ficar em branco até o server voltar.");
    });
    import.meta.hot.on("vite:ws:connect", () => {
      if (rootEmpty()) window.location.reload();
    });
  }
}
