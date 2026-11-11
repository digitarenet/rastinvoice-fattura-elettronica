// Copyright (c) 2026 Andrea Re - info@digitare.net
// Autore: Andrea Re @ 2026 - info@digitare.net
//
// Visualizzazione dei PDF con PDF.js. La libreria si carica solo al primo
// PDF aperto. Le opzioni escludono l'esecuzione di codice contenuto nel file
// (niente moduli XFA; PDF.js non esegue gli script dei documenti).

import type { PDFDocumentProxy } from "pdfjs-dist";

let modulo: Promise<typeof import("pdfjs-dist")> | null = null;

async function pdfjs() {
  modulo ??= (async () => {
    const m = await import("pdfjs-dist/legacy/build/pdf.mjs");
    const lavoratore = await import("pdfjs-dist/legacy/build/pdf.worker.min.mjs?url");
    m.GlobalWorkerOptions.workerSrc = lavoratore.default;
    return m as unknown as typeof import("pdfjs-dist");
  })();
  return modulo;
}

export async function apriPdf(dati: ArrayBuffer): Promise<PDFDocumentProxy> {
  const m = await pdfjs();
  return m.getDocument({
    data: new Uint8Array(dati),
    enableXfa: false,
    // Caratteri standard e decodificatori inclusi nell'applicazione: nessun
    // accesso alla rete.
    standardFontDataUrl: "/pdfjs/standard_fonts/",
    wasmUrl: "/pdfjs/wasm/",
  }).promise;
}

/** Disegna una pagina in un canvas alla larghezza indicata (in pixel CSS). */
export async function disegnaPagina(doc: PDFDocumentProxy, numero: number, canvas: HTMLCanvasElement, larghezza: number) {
  const pagina = await doc.getPage(numero);
  const base = pagina.getViewport({ scale: 1 });
  const scala = larghezza / base.width;
  const densita = window.devicePixelRatio || 1;
  const vista = pagina.getViewport({ scale: scala * densita });
  canvas.width = Math.floor(vista.width);
  canvas.height = Math.floor(vista.height);
  canvas.style.width = `${Math.floor(vista.width / densita)}px`;
  canvas.style.height = `${Math.floor(vista.height / densita)}px`;
  await pagina.render({ canvas, viewport: vista }).promise;
}
