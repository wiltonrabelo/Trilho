import React from "react";
import ReactDOM from "react-dom/client";
import App from "@/App";
import { initTheme } from "@/lib/theme";
import "@/index.css";

// Aplica o tema o quanto antes para evitar "flash" de cor errada.
initTheme();

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    <App />
  </React.StrictMode>
);
