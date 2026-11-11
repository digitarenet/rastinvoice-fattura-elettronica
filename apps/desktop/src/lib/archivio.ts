// Copyright (c) 2026 Andrea Re - info@digitare.net
// Autore: Andrea Re @ 2026 - info@digitare.net
//
// Logica di filtro, ordinamento e aggregazione dei documenti, indipendente
// dall'interfaccia.

import type { SoggettoBreve, Voce } from "./api";

export type Direzione = "ricevuta" | "emessa" | "sconosciuta";
export type Sezione = "tutte" | "ricevute" | "emesse" | "da_leggere" | "contrassegnate" | "anomalie";
export type Ordinamento = "data_desc" | "data_asc" | "importo_desc" | "importo_asc" | "controparte";
export type Ruolo = "cliente" | "fornitore";

/**
 * Porta un identificativo inserito dall'utente nella forma usata come chiave
 * dei soggetti: `IT01234567890` per le partite IVA, `CF:...` per i codici
 * fiscali. Una partita IVA di 11 cifre senza paese è considerata italiana.
 */
export function normalizzaIdentificativo(s: string): string {
  const t = s.replace(/\s+/g, "").toUpperCase();
  if (!t) return "";
  if (t.startsWith("CF:")) return t;
  if (/^\d{11}$/.test(t)) return "IT" + t;
  if (/^[A-Z]{6}\d{2}[A-Z]\d{2}[A-Z]\d{3}[A-Z]$/.test(t)) return "CF:" + t;
  return t;
}

export function direzione(v: Voce, propri: Set<string>): Direzione {
  if (propri.has(v.cessionario.chiave)) return "ricevuta";
  if (propri.has(v.cedente.chiave)) return "emessa";
  return "sconosciuta";
}

/** Il soggetto "dall'altra parte" rispetto all'utente. */
export function controparte(v: Voce, propri: Set<string>): SoggettoBreve {
  return direzione(v, propri) === "emessa" ? v.cessionario : v.cedente;
}

/** Importo con segno: le note di credito riducono i totali. */
export function importoConSegno(v: Voce): number {
  const n = Number(v.totale) || 0;
  return v.nota_credito ? -Math.abs(n) : n;
}

export interface Filtri {
  sezione: Sezione;
  anno: number | null;
  ricerca: string;
  contatto: string | null;
  dal: string;
  al: string;
  ordinamento: Ordinamento;
}

export function filtra(
  voci: Voce[],
  f: Filtri,
  propri: Set<string>,
  lette: Set<string>,
  contrassegnate: Set<string>,
): Voce[] {
  const termini = f.ricerca.toLowerCase().split(/\s+/).filter(Boolean);
  const risultato = voci.filter((v) => {
    switch (f.sezione) {
      case "ricevute":
        if (direzione(v, propri) !== "ricevuta") return false;
        break;
      case "emesse":
        if (direzione(v, propri) !== "emessa") return false;
        break;
      case "da_leggere":
        if (lette.has(v.id)) return false;
        break;
      case "contrassegnate":
        if (!contrassegnate.has(v.id)) return false;
        break;
      case "anomalie":
        if (v.avvisi === 0 && v.controlli === 0) return false;
        break;
    }
    if (f.anno !== null && v.anno !== f.anno) return false;
    if (f.contatto && v.cedente.chiave !== f.contatto && v.cessionario.chiave !== f.contatto) return false;
    if (f.dal && v.data < f.dal) return false;
    if (f.al && v.data > f.al) return false;
    for (const t of termini) if (!v.testo.includes(t)) return false;
    return true;
  });
  const confronto: Record<Ordinamento, (a: Voce, b: Voce) => number> = {
    data_desc: (a, b) => b.data.localeCompare(a.data) || b.numero.localeCompare(a.numero, "it", { numeric: true }),
    data_asc: (a, b) => a.data.localeCompare(b.data) || a.numero.localeCompare(b.numero, "it", { numeric: true }),
    importo_desc: (a, b) => importoConSegno(b) - importoConSegno(a),
    importo_asc: (a, b) => importoConSegno(a) - importoConSegno(b),
    controparte: (a, b) =>
      controparte(a, propri).nome.localeCompare(controparte(b, propri).nome, "it", { sensitivity: "base" }),
  };
  return risultato.sort(confronto[f.ordinamento]);
}

export interface Contatto {
  chiave: string;
  nome: string;
  comune: string;
  ruoli: Set<Ruolo>;
  documenti: number;
  totale: number;
  primo: string;
  ultimo: string;
}

/**
 * Raggruppa i soggetti presenti nei documenti, esclusi quelli dell'utente.
 * Il cedente è un fornitore, il cessionario un cliente.
 */
export function rubrica(voci: Voce[], propri: Set<string>, dal = "", al = ""): Contatto[] {
  const mappa = new Map<string, Contatto>();
  for (const v of voci) {
    if (dal && v.data < dal) continue;
    if (al && v.data > al) continue;
    const lati: [SoggettoBreve, Ruolo][] = [
      [v.cedente, "fornitore"],
      [v.cessionario, "cliente"],
    ];
    for (const [s, ruolo] of lati) {
      if (!s.chiave || propri.has(s.chiave)) continue;
      let c = mappa.get(s.chiave);
      if (!c) {
        c = { chiave: s.chiave, nome: s.nome, comune: s.comune, ruoli: new Set(), documenti: 0, totale: 0, primo: v.data, ultimo: v.data };
        mappa.set(s.chiave, c);
      }
      c.ruoli.add(ruolo);
      c.documenti += 1;
      c.totale += importoConSegno(v);
      if (v.data && (!c.primo || v.data < c.primo)) c.primo = v.data;
      if (v.data > c.ultimo) {
        c.ultimo = v.data;
        // Il nome più recente è il più affidabile (ragioni sociali cambiate).
        c.nome = s.nome || c.nome;
        c.comune = s.comune || c.comune;
      }
    }
  }
  return [...mappa.values()];
}

/**
 * Prova a indovinare l'identificativo dell'utente: il soggetto presente nella
 * maggior parte dei documenti, come cedente o come cessionario.
 */
export function indovinaPropri(voci: Voce[]): { chiave: string; nome: string } | null {
  if (voci.length < 3) return null;
  const conteggi = new Map<string, { n: number; nome: string }>();
  for (const v of voci) {
    for (const s of new Set([v.cedente, v.cessionario])) {
      if (!s.chiave) continue;
      const c = conteggi.get(s.chiave) ?? { n: 0, nome: s.nome };
      c.n += 1;
      conteggi.set(s.chiave, c);
    }
  }
  let migliore: [string, { n: number; nome: string }] | null = null;
  for (const e of conteggi) if (!migliore || e[1].n > migliore[1].n) migliore = e;
  if (!migliore || migliore[1].n < voci.length * 0.6) return null;
  return { chiave: migliore[0], nome: migliore[1].nome };
}

/** Elenco CSV (separatore `;`, compatibile con Excel in italiano). */
export function csv(voci: Voce[], propri: Set<string>): string {
  const campo = (s: string) => (/[;"\n]/.test(s) ? `"${s.replace(/"/g, '""')}"` : s);
  const righe = [
    ["Data", "Numero", "Tipo", "Direzione", "Cedente", "ID cedente", "Cessionario", "ID cessionario", "Totale", "Divisa", "Scadenza", "File"].join(";"),
  ];
  for (const v of voci) {
    righe.push(
      [
        v.data,
        v.numero,
        v.tipo_documento,
        direzione(v, propri),
        v.cedente.nome,
        v.cedente.chiave,
        v.cessionario.nome,
        v.cessionario.chiave,
        v.totale.replace(".", ","),
        v.divisa,
        v.prima_scadenza,
        v.file,
      ]
        .map((x) => campo(String(x ?? "")))
        .join(";"),
    );
  }
  // BOM per far riconoscere a Excel la codifica UTF-8.
  return "﻿" + righe.join("\r\n") + "\r\n";
}
