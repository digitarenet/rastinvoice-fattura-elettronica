// Copyright (c) 2026 Andrea Re - info@digitare.net
// Autore: Andrea Re @ 2026 - info@digitare.net
//
// Notifiche del Sistema di Interscambio: significato delle sigle e
// collegamento alle fatture a cui si riferiscono.

import type { Voce, VoceMessaggio } from "./api";

export type Gravita = "ok" | "errore" | "avviso" | "info";

export interface InfoSigla {
  /** Stato della fattura in poche parole. */
  stato: string;
  /** Nome ufficiale della notifica. */
  nome: string;
  /** Spiegazione per l'utente. */
  spiegazione: string;
  gravita: Gravita;
}

const SIGLE: Record<string, InfoSigla> = {
  RC: {
    stato: "Consegnata",
    nome: "Ricevuta di consegna",
    spiegazione: "Lo SdI ha recapitato la fattura al destinatario.",
    gravita: "ok",
  },
  NS: {
    stato: "Scartata",
    nome: "Notifica di scarto",
    spiegazione: "Lo SdI ha rifiutato la fattura per errori formali: va corretta e reinviata entro 5 giorni. La fattura scartata si considera non emessa.",
    gravita: "errore",
  },
  MC: {
    stato: "Non consegnata",
    nome: "Notifica di mancata consegna",
    spiegazione: "La fattura è valida ma non è stato possibile recapitarla: il cliente la trova nella propria area riservata del sito dell'Agenzia delle Entrate. Conviene avvisarlo.",
    gravita: "avviso",
  },
  NE: {
    stato: "Esito del cliente",
    nome: "Notifica di esito",
    spiegazione: "Risposta della pubblica amministrazione destinataria: accettazione (EC01) o rifiuto (EC02).",
    gravita: "info",
  },
  DT: {
    stato: "Decorrenza termini",
    nome: "Notifica di decorrenza termini",
    spiegazione: "Sono passati 15 giorni senza risposta della pubblica amministrazione destinataria: la fattura si considera accettata.",
    gravita: "ok",
  },
  AT: {
    stato: "Trasmessa",
    nome: "Attestazione di avvenuta trasmissione",
    spiegazione: "Il recapito non è riuscito ma la fattura è stata trasmessa e messa a disposizione del destinatario.",
    gravita: "avviso",
  },
  MT: {
    stato: "Metadati",
    nome: "File dei metadati",
    spiegazione: "Accompagna le fatture ricevute: contiene i dati di trasmissione dello SdI. Non richiede azioni.",
    gravita: "info",
  },
  EC: {
    stato: "Esito inviato",
    nome: "Esito del committente",
    spiegazione: "Risposta inviata dal destinatario allo SdI su una fattura verso la pubblica amministrazione.",
    gravita: "info",
  },
  SE: {
    stato: "Esito scartato",
    nome: "Scarto dell'esito del committente",
    spiegazione: "Lo SdI ha rifiutato l'esito inviato dal committente.",
    gravita: "avviso",
  },
};

export function infoSigla(m: VoceMessaggio): InfoSigla {
  const base = SIGLE[m.sigla] ?? { stato: m.descrizione, nome: m.descrizione, spiegazione: "", gravita: "info" as Gravita };
  if (m.sigla === "NE") {
    if (m.esito === "EC01") return { ...base, stato: "Accettata dal cliente", gravita: "ok" };
    if (m.esito === "EC02") return { ...base, stato: "Rifiutata dal cliente", gravita: "errore" };
  }
  return base;
}

export const SIGLE_NOTE = Object.entries(SIGLE).map(([sigla, i]) => ({ sigla, ...i }));

/** Nome di file ridotto per il confronto: senza cartelle, .p7m e .xml. */
function chiave(nome: string): string {
  return (nome.split(/[\\/]|!\//).pop() ?? "")
    .toLowerCase()
    .replace(/\.p7m$/, "")
    .replace(/\.xml$/, "");
}

/** Nome della fattura ricavato dal nome della notifica (`IT..._00005_RC_001.xml`). */
function fatturaDaNomeNotifica(file: string): string {
  const n = chiave(file);
  const m = /^(.+)_[a-z]{2}_[a-z0-9]{1,3}$/.exec(n);
  return m ? m[1] : "";
}

export function chiaveFattura(m: VoceMessaggio): string {
  return chiave(m.riferito_a) || fatturaDaNomeNotifica(m.file);
}

/** Indice delle notifiche per fattura, in ordine cronologico. */
export function indiceNotifiche(messaggi: VoceMessaggio[]): Map<string, VoceMessaggio[]> {
  const mappa = new Map<string, VoceMessaggio[]>();
  for (const m of messaggi) {
    const k = chiaveFattura(m);
    if (!k) continue;
    const l = mappa.get(k) ?? [];
    l.push(m);
    mappa.set(k, l);
  }
  for (const l of mappa.values()) l.sort((a, b) => a.data.localeCompare(b.data));
  return mappa;
}

export function notificheDi(v: Voce, indice: Map<string, VoceMessaggio[]>): VoceMessaggio[] {
  return indice.get(chiave(v.nome_file)) ?? [];
}

/** Indice delle fatture per nome di file, per collegare le notifiche. */
export function indiceFatture(voci: Voce[]): Map<string, Voce> {
  const mappa = new Map<string, Voce>();
  for (const v of voci) mappa.set(chiave(v.nome_file), v);
  return mappa;
}

/** Ultima notifica significativa (i metadati non indicano uno stato). */
export function statoAttuale(notifiche: VoceMessaggio[]): VoceMessaggio | null {
  const utili = notifiche.filter((m) => m.sigla !== "MT");
  return utili.length ? utili[utili.length - 1] : null;
}
