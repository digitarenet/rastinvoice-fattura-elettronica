// Copyright (c) 2026 Andrea Re - info@digitare.net
// Autore: Andrea Re @ 2026 - info@digitare.net
//
// Conversione dei documenti Word (.docx) in pagina HTML con mammoth.
// Il risultato viene mostrato in una cornice senza script.

export async function wordInHtml(dati: ArrayBuffer): Promise<string> {
  const mammoth = (await import("mammoth/mammoth.browser.js")).default;
  const r = await mammoth.convertToHtml({ arrayBuffer: dati });
  return `<!doctype html><html><head><meta charset="utf-8"><style>
    body { font-family: "Titillium Web", Calibri, Arial, sans-serif; max-width: 820px; margin: 2rem auto; padding: 0 2rem; line-height: 1.5; color: #1a1a1a; }
    table { border-collapse: collapse; } td, th { border: 1px solid #c5c7c9; padding: 4px 8px; }
    img { max-width: 100%; }
  </style></head><body>${r.value}</body></html>`;
}
