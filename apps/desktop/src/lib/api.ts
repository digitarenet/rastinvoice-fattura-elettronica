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
  tema: "chiaro" | "scuro" | "sistema";
}

export interface Avvio {
  impostazioni: Impostazioni;
  cartella_dati: string;
  portable: boolean;
  versione: string;
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
  nomeAllegato: (file: string, corpo: number, indice: number) =>
    invoke<string>("nome_allegato", { file, corpo, indice }),
  salvaAllegato: (file: string, corpo: number, indice: number, destinazione: string) =>
    invoke<void>("salva_allegato", { file, corpo, indice, destinazione }),
  apriAllegato: (file: string, corpo: number, indice: number) =>
    invoke<void>("apri_allegato", { file, corpo, indice }),
  apriCartella: (percorso: string) => invoke<void>("apri_cartella", { percorso }),
  mostraNellaCartella: (file: string) => invoke<void>("mostra_nella_cartella", { file }),
  apriSito: () => invoke<void>("apri_sito"),
  copiaInArchivio: (percorsi: string[]) =>
    invoke<string[]>("copia_in_archivio", { percorsi }),
  cartellaArchivio: () => invoke<string>("cartella_archivio"),
  foglioStato: () => invoke<StatoFoglio>("foglio_stile_stato"),
  foglioAggiorna: () => invoke<StatoFoglio>("foglio_stile_aggiorna"),
  foglioImporta: (sorgente: string) =>
    invoke<StatoFoglio>("foglio_stile_importa", { sorgente }),
  foglioLeggi: () => invoke<string | null>("foglio_stile_leggi"),
  scriviTesto: (percorso: string, testo: string) =>
    invoke<void>("scrivi_testo", { percorso, testo }),
};
