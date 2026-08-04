import React from "react";
import ReactDOM from "react-dom/client";
import App from "@/App";
import { AppErrorBoundary } from "@/components/AppErrorBoundary";
import { installDevBlankRecovery } from "@/lib/dev-blank-recovery";
import { initTheme } from "@/lib/theme";
import "@/index.css";

// Aplica o tema o quanto antes para evitar "flash" de cor errada.
initTheme();
installDevBlankRecovery();

// App desktop: bloqueia o menu nativo do WebView (Atualizar, Inspecionar…).
// Menus customizados (ramos, commits, arquivos…) continuam via onContextMenu próprio.
document.addEventListener(
  "contextmenu",
  (e) => {
    e.preventDefault();
  },
  true,
);

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    <AppErrorBoundary>
      <App />
    </AppErrorBoundary>
  </React.StrictMode>,
);
