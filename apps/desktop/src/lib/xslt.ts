// Copyright (c) 2026 Andrea Re - info@digitare.net
// Autore: Andrea Re @ 2026 - info@digitare.net
//
// Applicazione di un foglio di stile XSL all'XML della fattura.
//
// Soluzione provvisoria: usa il motore XSLT integrato nella webview di
// sistema. Verrà sostituita da una trasformazione nativa lato Rust, perché i
// motori basati su Chromium stanno rimuovendo il supporto XSLT.

export function disponibile(): boolean {
  return typeof (window as any).XSLTProcessor !== "undefined";
}

export function trasforma(xml: string, xsl: string): string {
  const parser = new DOMParser();
  const docXml = parser.parseFromString(xml, "application/xml");
  const docXsl = parser.parseFromString(xsl, "application/xml");
  for (const d of [docXml, docXsl]) {
    const errore = d.getElementsByTagName("parsererror")[0];
    if (errore) throw new Error(errore.textContent ?? "XML non valido");
  }
  const processore = new XSLTProcessor();
  processore.importStylesheet(docXsl);
  const risultato = processore.transformToDocument(docXml);
  if (!risultato || !risultato.documentElement) {
    throw new Error("il foglio di stile non ha prodotto alcun risultato");
  }
  return "<!doctype html>" + new XMLSerializer().serializeToString(risultato);
}
