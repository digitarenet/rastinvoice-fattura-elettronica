# Registro delle modifiche

Tutte le modifiche rilevanti di RustinVoice. Il numero di versione segue lo schema
MAGGIORE.MINORE.CORREZIONE: la versione minore cresce con le nuove funzioni, quella di
correzione con le sole correzioni.

## 0.3.1 - 2026-11-11

### Migliorato
- Avvio con l'indice circa 2,8 volte più rapido (10.000 file: da 175 a 63 millisecondi):
  i dati già letti non vengono più copiati e l'indice si riscrive solo se cambia qualcosa.
- Le cartelle vengono esplorate in parallelo, riusando i dati di dimensione e data forniti
  dalla lettura della cartella: molto più veloce su dischi di rete e NAS.
- L'elenco dei documenti disegna solo le righe visibili: scorrimento e ricerca restano
  fluidi anche con decine di migliaia di fatture.

## 0.3.0 - 2026-11-11

### Aggiunto
- Apertura di fatture contenute in archivi **.zip** ed email **.eml**, comprese le PEC
  del Sistema di Interscambio con il messaggio originale annidato (`postacert.eml`).
- **Verifica della firma** dei file .p7m: integrità del documento, corrispondenza con
  il certificato del firmatario (RSA ed ECDSA), dati del firmatario e validità del
  certificato alla data della firma.
- **Controlli di validità**: codice destinatario, partite IVA, codici fiscali, IBAN,
  calcolo di righe, riepiloghi IVA e totale, con i codici errore dello SdI.
- **Indice locale** dei documenti letti: all'avvio si rileggono solo i file nuovi o
  modificati, utile con cartelle di rete e NAS.
- Avviso per le cartelle non raggiungibili, con i documenti già letti ancora consultabili.
- Scheda XML con formattazione leggibile (solo a video, il file non viene modificato).
- Zoom delle viste Moderna e AssoSoftware (anche con Ctrl +, Ctrl -, Ctrl 0).
- Voce di menu **Credits** con informazioni, librerie utilizzate e questo registro.
- Verifica automatica del codice a ogni modifica pubblicata (formattazione, analisi, test).

### Modificato
- La scheda del foglio di stile ufficiale si chiama ora "AssoSoftware".
- L'interfaccia è inclusa nell'eseguibile in modo predefinito: `cargo run` e il pulsante
  Run dell'IDE funzionano senza server di sviluppo (`npm run app:dev` per lo sviluppo).
- L'interfaccia viene ricompilata automaticamente quando i sorgenti cambiano, così
  l'eseguibile avviato dall'IDE mostra sempre le ultime modifiche.
- Gli allegati si aprono solo se di tipo sicuro (documenti, immagini, testo); gli altri
  si possono salvare.

### Corretto
- Barra verticale nera sulle voci selezionate in Windows.
- La vista con foglio di stile poteva mostrare il documento precedente durante lo scorrimento.
- Apertura degli allegati e delle cartelle bloccata dai permessi dell'applicazione.

## 0.2.0 - 2026-11-11

### Aggiunto
- Applicazione desktop (Tauri 2, Svelte 5, Bootstrap Italia): elenco con ricerca a testo
  libero, sezioni ricevute, emesse, da leggere, contrassegnate e con anomalie, filtri per
  anno, date e contatto.
- Anteprima nella vista Moderna e con il foglio di stile ufficiale AssoSoftware,
  scaricabile dal sito dell'associazione.
- Rubrica di clienti e fornitori con totali e documenti per contatto.
- Trascina e rilascia, modalità portable, stampa, salvataggio HTML e XML, allegati,
  esportazione CSV, tema chiaro e scuro.
- Configurazioni di avvio condivise per RustRover.

## 0.1.0 - 2026-11-11

### Aggiunto
- Nucleo di lettura FatturaPA (specifiche 1.9.1): formati FPA12, FPR12 e FSM10, lotti,
  codifiche dei vecchi file, buste firmate .p7m (DER, BER, base64, annidate).
- Riconoscimento delle ricevute e notifiche del Sistema di Interscambio.
- Riga di comando con i comandi `info`, `json` e `scansiona`.
