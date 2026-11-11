# Registro delle modifiche

Tutte le modifiche rilevanti di RustinVoice. Il numero di versione segue lo schema
MAGGIORE.MINORE.CORREZIONE: la versione minore cresce con le nuove funzioni, quella di
correzione con le sole correzioni.

## 0.5.0 - 2026-11-11

### Aggiunto
- **Scelta della cartella dati** al primo avvio e nelle Impostazioni: nel profilo utente
  oppure accanto al programma (modalità portable, la cartella `dati` viene creata
  automaticamente). Cambiando scelta, impostazioni, indice, foglio di stile e archivio
  vengono copiati nella nuova cartella e l'applicazione si riavvia; la vecchia cartella
  portable viene rinominata, non cancellata.

## 0.4.1 - 2026-11-11

### Modificato
- La sezione "Ricevute SdI" diventa **Notifiche SdI** ed è più chiara: ogni notifica mostra
  data, stato in parole ("Consegnata", "Scartata", "Rifiutata dal cliente"...), errori e
  motivazioni, con il collegamento alla fattura a cui si riferisce; una legenda spiega
  tutte le sigle.
- L'anteprima della fattura mostra lo stato presso lo SdI e la cronologia delle notifiche.
- I file non riconosciuti sono elencati a parte, con una spiegazione.

## 0.4.0 - 2026-11-11

### Aggiunto
- **Visualizzatore degli allegati** integrato: si apre facendo clic sul nome dell'allegato.
  - PDF (con PDF.js, senza accesso alla rete né esecuzione di script del documento).
  - Immagini PNG, JPEG, GIF, WebP, BMP e SVG.
  - Documenti Word .docx, convertiti in pagina.
  - Fogli di calcolo Excel (.xlsx, .xls, .xlsb) e OpenDocument (.ods), con un foglio per scheda.
  - CSV come tabella, testo, XML formattato, pagine HTML senza script.
  - Archivi .zip, sfogliabili, e buste firmate .p7m, aperte automaticamente.
  - Il tipo viene riconosciuto dal contenuto e non dal nome dato dal mittente; i formati
    senza anteprima (per esempio .doc, .rtf, .tiff) si aprono con il programma predefinito
    o si salvano.

### Modificato
- Registro delle modifiche nella pagina Credits con un carattere più piccolo.
- Le pagine Credits e Impostazioni occupano tutta la larghezza disponibile.

## 0.3.2 - 2026-11-11

### Sicurezza
- Le finestre di salvataggio (CSV, HTML, XML, allegati) sono aperte dalla parte nativa:
  l'interfaccia non può più indicare da sola dove scrivere un file.
- La finestra dell'applicazione non può essere portata su pagine esterne, nemmeno da
  collegamenti contenuti in una fattura.
- Gli allegati aperti vengono scritti in una cartella privata dell'utente, nuova a ogni
  apertura; su Windows sono marcati come provenienti da Internet, così Office li apre in
  visualizzazione protetta.
- L'esportazione CSV neutralizza i testi che Excel eseguirebbe come formule.
- Permessi dell'interfaccia ridotti al solo dialogo di apertura file.

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
