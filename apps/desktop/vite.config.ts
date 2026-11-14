// Copyright (c) 2026 Andrea Re - info@digitare.net
// Autore: Andrea Re @ 2026 - info@digitare.net
//
// Configurazione di Vite per l'interfaccia dell'applicazione desktop.

import { defineConfig } from "vite";
import { svelte } from "@sveltejs/vite-plugin-svelte";
import { cpSync, existsSync, readFileSync } from "node:fs";
import { join, normalize } from "node:path";
import type { Plugin } from "vite";

// File di supporto di PDF.js (caratteri standard e decodificatori) serviti
// dall'applicazione stessa in /pdfjs/: nessun accesso alla rete.
const PDFJS = join(import.meta.dirname, "node_modules", "pdfjs-dist");
const CARTELLE_PDFJS = ["standard_fonts", "wasm"];

function risorsePdfjs(): Plugin {
  return {
    name: "risorse-pdfjs",
    configureServer(server) {
      server.middlewares.use("/pdfjs", (req, res, next) => {
        const relativo = normalize(decodeURIComponent((req.url ?? "").split("?")[0])).replace(/^[/\\]+/, "");
        const [cartella] = relativo.split(/[/\\]/);
        const percorso = join(PDFJS, relativo);
        if (!CARTELLE_PDFJS.includes(cartella) || !percorso.startsWith(PDFJS) || !existsSync(percorso)) return next();
        res.setHeader("Content-Type", percorso.endsWith(".wasm") ? "application/wasm" : "application/octet-stream");
        res.end(readFileSync(percorso));
      });
    },
    writeBundle(opzioni) {
      const uscita = opzioni.dir ?? "dist";
      for (const c of CARTELLE_PDFJS) cpSync(join(PDFJS, c), join(uscita, "pdfjs", c), { recursive: true });
    },
  };
}

export default defineConfig({
  plugins: [svelte(), risorsePdfjs()],
  // Tauri si aspetta una porta fissa e non deve nascondere gli errori di Rust.
  clearScreen: false,
  server: {
    port: 1420,
    strictPort: true,
    watch: { ignored: ["**/src-tauri/**"] },
    // Il registro delle modifiche è nella radice del repository.
    fs: { allow: ["..", "../../docs/CHANGELOG.md"] },
  },
  build: {
    target: ["es2021", "chrome105", "safari15"],
    chunkSizeWarningLimit: 2000,
  },
});
