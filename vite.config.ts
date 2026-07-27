import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";
import path from "node:path";

// Porta fixa para o Tauri saber onde encontrar o dev server.
// Usa 127.0.0.1 (não "localhost") para evitar IPv6 (::1) → HTTP 400 no WebView2 após idle.
const host = process.env.TAURI_DEV_HOST || "127.0.0.1";

export default defineConfig({
  plugins: [react()],
  resolve: {
    alias: {
      "@": path.resolve(import.meta.dirname, "./src"),
    },
  },
  // Evita que o Vite limpe a tela e esconda erros do Rust.
  clearScreen: false,
  server: {
    port: 1420,
    strictPort: true,
    host,
    hmr: {
      protocol: "ws",
      host,
      port: 1421,
    },
    watch: {
      // O Vite não precisa observar o backend Rust.
      ignored: ["**/src-tauri/**"],
    },
  },
});
