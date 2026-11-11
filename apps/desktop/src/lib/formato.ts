// Copyright (c) 2026 Andrea Re - info@digitare.net
// Autore: Andrea Re @ 2026 - info@digitare.net
//
// Formattazione all'italiana per l'interfaccia.

const euro = new Intl.NumberFormat("it-IT", {
  minimumFractionDigits: 2,
  maximumFractionDigits: 2,
});

/** Importo `1234.5` → `1.234,50`. Le stringhe vuote restano vuote. */
export function importo(valore: string, divisa = "EUR"): string {
  if (!valore) return "";
  const n = Number(valore);
  if (Number.isNaN(n)) return valore;
  const simbolo = divisa === "EUR" ? "€ " : `${divisa} `;
  return simbolo + euro.format(n);
}

/** Data ISO `2026-11-03` → `03/11/2026`. */
export function data(iso: string): string {
  if (!iso || iso.length < 10) return "";
  return `${iso.slice(8, 10)}/${iso.slice(5, 7)}/${iso.slice(0, 4)}`;
}

/** Iniziali per l'avatar di un soggetto. */
export function iniziali(nome: string): string {
  const parole = nome
    .replace(/[^\p{L}\p{N} ]/gu, " ")
    .split(/\s+/)
    .filter((p) => p.length > 1 && !/^(srl|spa|snc|sas|s|r|l|di|e|del|della)$/i.test(p));
  return ((parole[0]?.[0] ?? "?") + (parole[1]?.[0] ?? "")).toUpperCase();
}

/** Normalizza un testo di ricerca come fa il backend. */
export function normalizza(s: string): string {
  return s.toLowerCase().split(/\s+/).filter(Boolean).join(" ");
}

/** Oggi in formato ISO. */
export function oggi(): string {
  const d = new Date();
  const m = String(d.getMonth() + 1).padStart(2, "0");
  const g = String(d.getDate()).padStart(2, "0");
  return `${d.getFullYear()}-${m}-${g}`;
}
