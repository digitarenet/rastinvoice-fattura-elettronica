// Copyright (c) 2026 Andrea Re - info@digitare.net
// Autore: Andrea Re @ 2026 - info@digitare.net
//
// Sostegno al progetto: indirizzo per le donazioni ed elenco dei sostenitori
// che hanno chiesto di essere citati nei Credits.

/** Pagina PayPal per le donazioni (se vuota il pulsante è nascosto). */
export const URL_DONAZIONE = "https://www.paypal.biz/digitarenet";

/** Sostenitori da citare, nell'ordine in cui hanno contribuito. */
export const SOSTENITORI: string[] = [];

/**
 * Le donazioni esterne non sono ammesse nelle app del Mac App Store:
 * su macOS il pulsante non viene mostrato.
 */
export const donazioniAmmesse = !/Mac/i.test(navigator.userAgent);
