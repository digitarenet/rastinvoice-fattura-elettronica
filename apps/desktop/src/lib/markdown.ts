// Copyright (c) 2026 Andrea Re - info@digitare.net
// Autore: Andrea Re @ 2026 - info@digitare.net
//
// Conversione minima del Markdown del registro delle modifiche: titoli,
// paragrafi, elenchi puntati, grassetto e codice. Il testo viene sempre
// protetto prima di aggiungere i pochi elementi ammessi, quindi l'HTML
// prodotto non può contenere marcatori presenti nel sorgente.

export type Blocco =
  | { tipo: "titolo"; livello: number; html: string }
  | { tipo: "paragrafo"; html: string }
  | { tipo: "elenco"; voci: string[] };

function proteggi(s: string): string {
  return s.replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;").replace(/"/g, "&quot;");
}

function inline(s: string): string {
  return proteggi(s)
    .replace(/`([^`]+)`/g, "<code>$1</code>")
    .replace(/\*\*([^*]+)\*\*/g, "<strong>$1</strong>");
}

export function blocchiMarkdown(testo: string): Blocco[] {
  const blocchi: Blocco[] = [];
  let paragrafo: string[] = [];
  let elenco: string[] | null = null;

  const chiudi = () => {
    if (paragrafo.length) blocchi.push({ tipo: "paragrafo", html: inline(paragrafo.join(" ")) });
    paragrafo = [];
    if (elenco) blocchi.push({ tipo: "elenco", voci: elenco.map(inline) });
    elenco = null;
  };

  for (const riga of testo.split(/\r?\n/)) {
    const titolo = /^(#{1,6})\s+(.*)$/.exec(riga);
    if (titolo) {
      chiudi();
      blocchi.push({ tipo: "titolo", livello: titolo[1].length, html: inline(titolo[2]) });
    } else if (/^\s*[-*]\s+/.test(riga)) {
      if (paragrafo.length) chiudi();
      elenco ??= [];
      elenco.push(riga.replace(/^\s*[-*]\s+/, ""));
    } else if (!riga.trim()) {
      chiudi();
    } else if (elenco && /^\s+/.test(riga)) {
      // Continuazione della voce precedente.
      elenco[elenco.length - 1] += " " + riga.trim();
    } else {
      if (elenco) chiudi();
      paragrafo.push(riga.trim());
    }
  }
  chiudi();
  return blocchi;
}
