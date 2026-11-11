<!-- Copyright (c) 2026 Andrea Re - info@digitare.net - Autore: Andrea Re @ 2026 - info@digitare.net -->

# Accessibilità di RastInvoice

Questo documento descrive lo stato di accessibilità dell'applicazione desktop RastInvoice, la sua
aderenza al design system di Designers Italia e le verifiche effettuate. Segue la struttura del modello di
dichiarazione di accessibilità di AgID. RastInvoice è un software privato e non è soggetto all'obbligo di
pubblicare la dichiarazione: questo documento è una scelta volontaria di trasparenza.

## Riferimenti

- [WCAG 2.1](https://www.w3.org/TR/WCAG21/), livello AA.
- Norma europea [EN 301 549](https://www.etsi.org/deliver/etsi_en/301500_301599/301549/), che le richiama
  per il software.
- [Linee guida AgID sull'accessibilità degli strumenti informatici](https://www.agid.gov.it/it/design-servizi/accessibilita),
  in attuazione della Legge 4/2004 (Legge Stanca).
- [Design system di Designers Italia](https://designers.italia.it/design-system/), tramite il kit
  [Bootstrap Italia](https://italia.github.io/bootstrap-italia/).

## Stato di conformità

Versione valutata: 0.8.0, novembre 2026.

**Parzialmente conforme** ai requisiti WCAG 2.1 livello AA. I controlli automatici non rilevano alcuna
violazione e le verifiche manuali eseguite sono superate. Mancano però le prove con i lettori di schermo,
e la vista AssoSoftware non dipende dall'applicazione (vedi i punti seguenti). Lo stato diventerà
"conforme" dopo quelle prove.

### Contenuti non accessibili o non verificati

- **Prove con lettori di schermo.** La struttura è predisposta: ruoli, nomi accessibili, annunci e gestione
  del fuoco. Restano da eseguire le prove complete con NVDA (Windows) e VoiceOver (macOS).
- **Vista AssoSoftware.** È prodotta dal foglio di stile pubblicato da AssoSoftware, che usa tabelle di
  impaginazione e testi di piccole dimensioni. Il contenuto è di terzi e non viene modificato. Per una
  lettura accessibile della fattura si usa la vista **Moderna**, che contiene gli stessi dati.
- **Allegati.** I documenti allegati alle fatture (PDF, Word, fogli di calcolo) sono mostrati così come sono.
  La loro accessibilità dipende da chi li ha prodotti.

## Verifiche effettuate

| Verifica | Esito |
|---|---|
| axe-core 4.10 (regole WCAG 2.0, 2.1, 2.2 A/AA e buone pratiche) su tutte le schermate, sulle finestre modali e sulla vista Moderna | nessuna violazione |
| Contrasto dei testi (1.4.3) | almeno 4,5:1 anche su sfondi colorati e righe selezionate |
| Contrasto di icone, grafici e indicatori (1.4.11) | almeno 3:1 |
| Uso solo da tastiera (2.1.1, 2.4.3, 2.4.7) | ordine logico, fuoco sempre visibile, nessuna trappola; frecce nell'elenco e nelle schede |
| Collegamento "Vai al contenuto principale" (2.4.1) | presente, primo elemento raggiungibile con Tab |
| Finestre modali | il fuoco entra all'apertura, resta nella finestra con Tab e torna al punto di partenza alla chiusura; Esc chiude |
| Nomi accessibili (4.1.2) | tutti i pulsanti con sola icona hanno un nome; schede, pulsanti a due stati e voce corrente sono dichiarati |
| Messaggi di stato (4.1.3) | avvisi, errori, numero di documenti trovati e caricamenti sono annunciati |
| Ridimensionamento (1.4.4) e ridisposizione (1.4.10) | interfaccia ingrandibile fino al 200% dalle Impostazioni; la disposizione si adatta e il menu diventa a scomparsa |
| Dimensione dei pulsanti (2.5.8) | almeno 24 × 24 px |
| Grafici (1.1.1) | descrizione testuale e tabella dei valori sempre disponibile |
| Lingua (3.1.1) | italiano dichiarato |

## Aderenza a Designers Italia

- Interfaccia costruita con Bootstrap Italia 2: tipografia Titillium Web e Roboto Mono, icone ufficiali,
  componenti per pulsanti, campi, selettori, avvisi, tabelle e indicatori di caricamento.
- Palette del design system, con i colori di stato scuriti dove serve a garantire il contrasto AA su sfondi
  colorati.
- Solo tema chiaro: il design system non prevede un tema scuro.
- Gli elementi tipici di un'applicazione desktop sono costruiti con gli stessi colori, caratteri e spaziature
  del kit. Sono l'elenco virtuale dei documenti, l'anteprima a schede e i grafici.

## Segnalazioni

Problemi di accessibilità e richieste di miglioramento si segnalano a **info@digitare.net**, indicando la
versione dell'applicazione, il sistema operativo e, se usata, la tecnologia assistiva. Le segnalazioni
ricevono risposta entro 30 giorni.
