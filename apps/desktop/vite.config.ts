// Copyright (c) 2026 Andrea Re - info@digitare.net
// Autore: Andrea Re @ 2026 - info@digitare.net
//
// Configurazione di Vite per l'interfaccia dell'applicazione desktop.

import { defineConfig } from "vite";
import { svelte } from "@sveltejs/vite-plugin-svelte";

export default defineConfig({
  plugins: [svelte()],
  // Tauri si aspetta una porta fissa e non deve nascondere gli errori di Rust.
  clearScreen: false,
  server: {
    port: 1420,
    strictPort: true,
    watch: { ignored: ["**/src-tauri/**"] },
    // Il registro delle modifiche è nella radice del repository.
    fs: { allow: ["..", "../../CHANGELOG.md"] },
  },
  build: {
    target: ["es2021", "chrome105", "safari15"],
    chunkSizeWarningLimit: 2000,
  },
});
