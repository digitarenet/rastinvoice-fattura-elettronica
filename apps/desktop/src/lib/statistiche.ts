// Copyright (c) 2026 Andrea Re - info@digitare.net
// Autore: Andrea Re @ 2026 - info@digitare.net
//
// Statistiche sui documenti: fatturato, acquisti e IVA per anno, mese e
// controparte. Le note di credito riducono i totali. Valori indicativi:
// non sostituiscono i registri IVA né la liquidazione periodica.

import type { Voce } from "./api";
import { controparte, direzione } from "./archivio";

export interface Somme {
  documenti: number;
  imponibile: number;
  imposta: number;
  totale: number;
  /** IVA in scissione dei pagamenti (versata dal cliente pubblico). */
  scissione: number;
}

export interface PerControparte extends Somme {
  chiave: string;
  nome: string;
}

export interface Statistiche {
  emesse: Somme;
  ricevute: Somme;
  /** Per mese (indice 0-11) se è scelto un anno, altrimenti per anno. */
  periodi: { etichetta: string; emesse: number; ricevute: number }[];
  clienti: PerControparte[];
  fornitori: PerControparte[];
  anni: number[];
  /** Vero se la maggior parte delle fatture emesse è in regime forfettario (RF19). */
  forfettario: boolean;
  /** Imponibile delle fatture emesse negli ultimi 12 mesi, a prescindere dal periodo scelto. */
  ultimi12: number;
  /** Data da cui partono gli ultimi 12 mesi (ISO). */
  inizio12: string;
}

/** Soglia dei ricavi per restare nel regime forfettario (legge 190/2014, dal 2023). */
export const SOGLIA_FORFETTARIO = 85_000;
/** Oltre questa soglia si esce dal regime forfettario già nell'anno in corso. */
export const SOGLIA_USCITA_IMMEDIATA = 100_000;

const MESI = ["gen", "feb", "mar", "apr", "mag", "giu", "lug", "ago", "set", "ott", "nov", "dic"];

function vuote(): Somme {
  return { documenti: 0, imponibile: 0, imposta: 0, totale: 0, scissione: 0 };
}

function aggiungi(s: Somme, v: Voce) {
  const segno = v.nota_credito ? -1 : 1;
  const n = (x: string) => segno * Math.abs(Number(x) || 0);
  s.documenti += 1;
  s.imponibile += n(v.imponibile);
  s.imposta += n(v.imposta);
  s.totale += n(v.totale);
  if (v.scissione) s.scissione += n(v.imposta);
}

export function calcola(voci: Voce[], propri: Set<string>, anno: number | null): Statistiche {
  const emesse = vuote();
  const ricevute = vuote();
  const clienti = new Map<string, PerControparte>();
  const fornitori = new Map<string, PerControparte>();
  // Ultimi 12 mesi rispetto a oggi e riconoscimento del regime forfettario.
  const oggi = new Date();
  const inizio = new Date(oggi.getFullYear() - 1, oggi.getMonth(), oggi.getDate() + 1);
  const inizio12 = `${inizio.getFullYear()}-${String(inizio.getMonth() + 1).padStart(2, "0")}-${String(inizio.getDate()).padStart(2, "0")}`;
  let ultimi12 = 0;
  let conRegime = 0;
  let forfettarie = 0;
  for (const v of voci) {
    if (direzione(v, propri) !== "emessa") continue;
    if (v.regime) {
      conRegime += 1;
      if (v.regime === "RF19") forfettarie += 1;
    }
    if (v.data >= inizio12) ultimi12 += (v.nota_credito ? -1 : 1) * Math.abs(Number(v.imponibile) || 0);
  }
  const forfettario = conRegime > 0 && forfettarie * 2 > conRegime;

  const anni = [...new Set(voci.map((v) => v.anno).filter((a) => a > 0))].sort((a, b) => a - b);
  const periodi =
    anno === null
      ? anni.map((a) => ({ etichetta: String(a), emesse: 0, ricevute: 0 }))
      : MESI.map((m) => ({ etichetta: m, emesse: 0, ricevute: 0 }));

  for (const v of voci) {
    if (anno !== null && v.anno !== anno) continue;
    const dir = direzione(v, propri);
    if (dir === "sconosciuta") continue;
    const somme = dir === "emessa" ? emesse : ricevute;
    aggiungi(somme, v);
    const altro = controparte(v, propri);
    const mappa = dir === "emessa" ? clienti : fornitori;
    let c = mappa.get(altro.chiave || altro.nome);
    if (!c) {
      c = { chiave: altro.chiave, nome: altro.nome || altro.chiave, ...vuote() };
      mappa.set(altro.chiave || altro.nome, c);
    }
    aggiungi(c, v);
    const indice = anno === null ? anni.indexOf(v.anno) : Number(v.data.slice(5, 7)) - 1;
    const p = periodi[indice];
    if (p) {
      const imp = (v.nota_credito ? -1 : 1) * Math.abs(Number(v.imponibile) || 0);
      if (dir === "emessa") p.emesse += imp;
      else p.ricevute += imp;
    }
  }
  const ordina = (m: Map<string, PerControparte>) => [...m.values()].sort((a, b) => b.imponibile - a.imponibile);
  return {
    emesse,
    ricevute,
    periodi,
    clienti: ordina(clienti),
    fornitori: ordina(fornitori),
    anni,
    forfettario,
    ultimi12,
    inizio12,
  };
}

/** Importo in euro senza simbolo, con due decimali. */
export function euro(n: number): string {
  return n.toLocaleString("it-IT", { minimumFractionDigits: 2, maximumFractionDigits: 2 });
}

/** Importo abbreviato per gli assi (12,5k). */
export function breve(n: number): string {
  const a = Math.abs(n);
  if (a >= 1_000_000) return `${(n / 1_000_000).toLocaleString("it-IT", { maximumFractionDigits: 1 })} mln`;
  if (a >= 1000) return `${(n / 1000).toLocaleString("it-IT", { maximumFractionDigits: 1 })}k`;
  return n.toLocaleString("it-IT", { maximumFractionDigits: 0 });
}
