import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";
import path from "node:path";

// Porta fixa para o Tauri saber onde encontrar o dev server.
const host = process.env.TAURI_DEV_HOST;

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
    host: host || false,
    hmr: host
      ? { protocol: "ws", host, port: 1421 }
      : undefined,
    watch: {
      // O Vite não precisa observar o backend Rust.
      ignored: ["**/src-tauri/**"],
    },
  },
});
