/**
 * Em `tauri dev`, após idle/sleep a WebView2 às vezes cai em página Edge
 * (HTTP 400 / "não está funcionando"). Recarrega quando o Vite responde.
 */
export function installDevBlankRecovery(): void {
  if (!import.meta.env.DEV) return;

  const DEV_ORIGIN = "http://127.0.0.1:1420";
  let reloading = false;

  const looksBroken = () => {
    const root = document.getElementById("root");
    const text = document.body?.innerText ?? "";
    return (
      !root ||
      root.childElementCount === 0 ||
      /HTTP ERROR/i.test(text) ||
      /não está funcionando/i.test(text) ||
      /This page isn't working/i.test(text)
    );
  };

  const tryRecover = async () => {
    if (reloading || document.visibilityState === "hidden") return;
    try {
      const res = await fetch(`${DEV_ORIGIN}/@vite/client`, {
        cache: "no-store",
      });
      if (!res.ok) return;
    } catch {
      return;
    }
    if (!looksBroken()) return;
    reloading = true;
    window.location.replace(`${DEV_ORIGIN}/`);
  };

  document.addEventListener("visibilitychange", () => {
    if (document.visibilityState === "visible") void tryRecover();
  });
  window.addEventListener("focus", () => void tryRecover());
  window.setInterval(() => void tryRecover(), 8_000);

  if (import.meta.hot) {
    import.meta.hot.on("vite:ws:disconnect", () => {
      console.warn(
        "Vite HMR desconectado — a UI pode falhar até o server voltar.",
      );
    });
    import.meta.hot.on("vite:ws:connect", () => {
      if (looksBroken()) window.location.replace(`${DEV_ORIGIN}/`);
    });
  }
}
