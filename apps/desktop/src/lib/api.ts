// Copyright (c) 2026 Andrea Re - info@digitare.net
// Autore: Andrea Re @ 2026 - info@digitare.net
//
// Tipi e chiamate verso il backend Rust.

import { invoke } from "@tauri-apps/api/core";

export interface SoggettoBreve {
  nome: string;
  chiave: string;
  comune: string;
}

export interface Voce {
  id: string;
  file: string;
  nome_file: string;
  corpo: number;
  corpi_nel_file: number;
  tipo_documento: string;
  descrizione_tipo: string;
  nota_credito: boolean;
  numero: string;
  data: string;
  anno: number;
  cedente: SoggettoBreve;
  cessionario: SoggettoBreve;
  totale: string;
  imponibile: string;
  imposta: string;
  scissione: boolean;
  divisa: string;
  prima_scadenza: string;
  firmato: boolean;
  contenitore: string;
  allegati: number;
  avvisi: number;
  controlli: number;
  testo: string;
}

export interface VoceMessaggio {
  file: string;
  sigla: string;
  descrizione: string;
  riferito_a: string;
  identificativo_sdi: string;
  data: string;
  esito: string;
  note: string;
  errori: { codice: string; descrizione: string; suggerimento: string }[];
}

export interface FileScartato {
  file: string;
  motivo: string;
}

export interface Esito {
  voci: Voce[];
  messaggi: VoceMessaggio[];
  scartati: FileScartato[];
  non_raggiungibili: string[];
}

export interface Impostazioni {
  cartelle: string[];
  file_singoli: string[];
  identificativi_propri: string[];
  contrassegnate: string[];
  lette: string[];
  benvenuto_visto: boolean;
  file_trascinati: "chiedi" | "copia" | "lascia";
  vista: "moderna" | "standard";
  zoom: number;
  aggiornamenti: boolean;
}

export interface EsitoAggiornamenti {
  attuale: string;
  ultima: string;
  disponibile: boolean;
}

export interface Avvio {
  impostazioni: Impostazioni;
  cartella_dati: string;
  portable: boolean;
  versione: string;
  primo_avvio: boolean;
  cartella_standard: string;
  cartella_portable: string;
  portable_possibile: boolean;
}

export interface Allegato {
  nome: string | null;
  formato: string | null;
  descrizione: string | null;
  dimensione: number;
}

// Sottoinsieme del modello dati usato direttamente dall'interfaccia.
export interface Fattura {
  versione: string | null;
  corpi: { allegati: Allegato[] }[];
  avvisi: string[];
}

export interface Firmatario {
  livello: number;
  esito: "valida" | "non_valida" | "non_verificabile";
  dettaglio: string;
  nome: string;
  codice_fiscale: string;
  organizzazione: string;
  emittente: string;
  valido_dal: string;
  valido_al: string;
  data_firma: string;
  certificato_in_corso_di_validita: boolean;
  algoritmo: string;
}

export interface Controllo {
  gravita: "errore" | "avviso";
  codice: string | null;
  messaggio: string;
  corpo: number | null;
}

export interface Dettaglio {
  html: string;
  xml: string;
  fattura: Fattura;
  firmato: boolean;
  livelli_firma: number;
  certificati: number;
  firme: Firmatario[];
  controlli: Controllo[];
}

export interface FoglioTabella {
  nome: string;
  righe: string[][];
  troncato: boolean;
}

export type AnteprimaAllegato =
  | { tipo: "immagine"; mime: string }
  | { tipo: "pdf" }
  | { tipo: "word" }
  | { tipo: "testo"; testo: string; formato: "testo" | "xml" | "json"; troncato: boolean }
  | { tipo: "html"; html: string }
  | { tipo: "tabella"; fogli: FoglioTabella[] }
  | { tipo: "archivio"; voci: { nome: string; dimensione: number }[] }
  | { tipo: "non_disponibile"; motivo: string };

export interface StatoFoglio {
  presente: boolean;
  aggiornato_il: string;
  percorso: string;
}

export const api = {
  avvio: () => invoke<Avvio>("avvio"),
  salvaImpostazioni: (impostazioni: Impostazioni) =>
    invoke<void>("salva_impostazioni", { impostazioni }),
  scansiona: (percorsi: string[]) => invoke<Esito>("scansiona", { percorsi }),
  dettaglio: (file: string, corpo: number) =>
    invoke<Dettaglio>("dettaglio", { file, corpo }),
  /** Chiede dove salvare e salva; restituisce false se l'utente annulla. */
  salvaAllegato: (file: string, corpo: number, indice: number) =>
    invoke<boolean>("salva_allegato", { file, corpo, indice }),
  apriAllegato: (file: string, corpo: number, indice: number) =>
    invoke<void>("apri_allegato", { file, corpo, indice }),
  anteprimaAllegato: (file: string, corpo: number, indice: number, interno: string[]) =>
    invoke<AnteprimaAllegato>("anteprima_allegato", { file, corpo, indice, interno }),
  byteAllegato: (file: string, corpo: number, indice: number, interno: string[]) =>
    invoke<ArrayBuffer>("byte_allegato", { file, corpo, indice, interno }),
  apriCartella: (percorso: string) => invoke<void>("apri_cartella", { percorso }),
  mostraNellaCartella: (file: string) => invoke<void>("mostra_nella_cartella", { file }),
  /** Cambia la cartella dati; se cambia davvero, l'applicazione si riavvia. */
  cambiaCartellaDati: (portable: boolean) => invoke<void>("cambia_cartella_dati", { portable }),
  apriSito: () => invoke<void>("apri_sito"),
  controllaAggiornamenti: () => invoke<EsitoAggiornamenti>("controlla_aggiornamenti"),
  apriDownload: () => invoke<void>("apri_download"),
  copiaInArchivio: (percorsi: string[]) =>
    invoke<string[]>("copia_in_archivio", { percorsi }),
  cartellaArchivio: () => invoke<string>("cartella_archivio"),
  foglioStato: () => invoke<StatoFoglio>("foglio_stile_stato"),
  foglioAggiorna: () => invoke<StatoFoglio>("foglio_stile_aggiorna"),
  foglioImporta: (sorgente: string) =>
    invoke<StatoFoglio>("foglio_stile_importa", { sorgente }),
  foglioLeggi: () => invoke<string | null>("foglio_stile_leggi"),
  trasformaStandard: (xml: string) => invoke<string>("trasforma_standard", { xml }),
  /** Chiede dove salvare e salva; restituisce false se l'utente annulla. */
  salvaTesto: (nome: string, tipo: "csv" | "html" | "xml", testo: string) =>
    invoke<boolean>("salva_testo", { nome, tipo, testo }),
};
