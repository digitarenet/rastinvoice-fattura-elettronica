# Registro delle modifiche

Tutte le modifiche rilevanti di RastInvoice (fino alla 0.9.0 chiamato RustinVoice). Il numero di versione segue lo schema
MAGGIORE.MINORE.CORREZIONE: la versione minore cresce con le nuove funzioni, quella di
correzione con le sole correzioni.

## 0.11.0 - 2026-11-11

### Aggiunto
- Pacchetti Linux `.deb` (Debian, Ubuntu e derivate) e `.rpm` (Fedora, openSUSE), allegati alla release.
- Cartella `store/` con il materiale per la pubblicazione: testi comuni, logo e poster, procedura per
  Microsoft Store, scheda AppStream, file `.desktop`, manifesto Flathub, `snapcraft.yaml` per lo Snap Store
  e guida per Apple.
- Informativa privacy (`PRIVACY.md`) da indicare negli store.
- Firma e notarizzazione per macOS nel workflow Rilascio, attive solo quando sono presenti i segreti Apple.

### Modificato
- Schermate del README rifatte con il nuovo nome.
- Si lavora su un solo branch (`main`) e la CI parte solo a comando, per non consumare minuti.

## 0.10.3 - 2026-11-11

### Corretto
- Copiando nell'archivio un file con lo stesso nome di uno già presente ma diverso, il numero veniva aggiunto
  dopo l'estensione (`fatture.zip_1`) e il file non veniva più riconosciuto. Ora il numero va prima delle
  estensioni (`fatture_1.zip`).

### Modificato
- La finestra dei file trascinati spiega che archivi ZIP ed email vengono copiati interi, senza estrarli.

## 0.10.2 - 2026-11-11

### Corretto
- La casella "Ricorda la scelta" nella finestra dei file trascinati mostrava due quadrati: ora usa lo stile
  di Bootstrap Italia come le altre caselle.
- La barra dei pulsanti dell'anteprima va a capo quando lo spazio non basta, invece di nascondere gli ultimi
  pulsanti.
- Analisi statica conforme alla nuova versione di Clippy (lettura dei certificati con `as_chunks`).

### Aggiunto
- Informativa sulla privacy (`PRIVACY.md`), richiesta dagli store.
- Schermate aggiornate con il nuovo nome in `docs/immagini`.

## 0.10.1 - 2026-11-11

### Modificato
- Verifica e rilascio su GitHub si avviano solo a mano (scheda Actions), non più a ogni push o pull request,
  per non consumare minuti.
- Azioni aggiornate alle versioni per Node.js 24 (`actions/checkout@v5`, `actions/setup-node@v5`) e
  Node.js 24 per la compilazione dell'interfaccia.
- Il controllo delle vulnerabilità usa `cargo-audit` già compilato: alcuni minuti in meno per ogni verifica.

## 0.10.0 - 2026-11-11

### Modificato
- Nuovo nome del prodotto: **RastInvoice**. Cambiano il nome mostrato nell'applicazione, nella finestra, nei
  pacchetti e nei documenti. La cartella dei dati resta la stessa, quindi impostazioni e archivio vengono
  mantenuti.
- Il comando della riga di comando si chiama ora `rastinvoice` (prima `rustinvoice`): gli script che lo usano
  vanno aggiornati.

### Pianificato
- Supporto della fattura elettronica europea secondo la norma EN 16931: formati UBL e CII, documenti della
  rete Peppol e PDF con XML incorporato (ZUGFeRD e Factur-X).

## 0.9.0 - 2026-11-11

### Aggiunto
- Licenza open source **EUPL-1.2** (licenza pubblica dell'Unione europea), nel file `LICENSE`, nei metadati
  dei pacchetti, nel README e nei Credits.
- Scheda `publiccode.yml` per il catalogo del software open source di Developers Italia.
- Pulsante **Salva in PDF** nell'anteprima: apre la finestra di stampa del sistema con il nome del file già
  proposto e ricorda di scegliere la stampante PDF.
- Controllo facoltativo delle nuove versioni (Impostazioni > Aggiornamenti), disattivato in modo predefinito;
  legge solo il numero dell'ultima versione pubblicata su GitHub.
- Rilascio automatico (`.github/workflows/rilascio.yml`): a ogni tag `v*` produce installer e versione
  portatile per Windows, app universale per macOS, AppImage per Linux e li allega a una release in bozza.
- Pacchetto MSIX per il Microsoft Store (`scripts/msix.ps1` e `apps/desktop/src-tauri/msix/`), non firmato
  perché lo firma lo Store; i dati di Partner Center si passano come variabili del repository.
- Controllo delle vulnerabilità note nelle dipendenze Rust e JavaScript nella verifica automatica, con gli
  avvisi non applicabili motivati in `.cargo/audit.toml`.

### Modificato
- Libreria `zip` aggiornata alla versione 9, che corregge quattro vulnerabilità note nell'estrazione degli
  archivi (RustinVoice non usava le funzioni interessate, ma l'aggiornamento elimina il rischio).
- Versione minima di Rust portata a 1.88, richiesta dalla nuova libreria `zip`.

## 0.8.2 - 2026-11-11

### Modificato
- README con l'elenco completo delle funzionalità, una galleria di immagini aggiornate (archivio dimostrativo
  con dati di fantasia, in `docs/immagini`) e la guida completa della riga di comando: comandi, colonne del
  CSV, opzioni e codici di uscita.

## 0.8.1 - 2026-11-11

### Modificato
- README aggiornato con le novità principali, la versione attuale e lo stato delle nuove funzioni
  (notifiche SdI, design system, accessibilità).

## 0.8.0 - 2026-11-11

### Aggiunto
- Accessibilità secondo le WCAG 2.1 livello AA (EN 301 549, Linee guida AgID), descritta in
  `ACCESSIBILITA.md`:
  - collegamento "Vai al contenuto principale";
  - nomi accessibili per tutti i pulsanti con sola icona;
  - schede navigabili con le frecce;
  - finestre modali che trattengono il fuoco e lo restituiscono alla chiusura;
  - elenco dei documenti che annuncia posizione, voce corrente e documenti da leggere;
  - fuoco che segue le frecce, con un indicatore ben visibile;
  - messaggi di stato annunciati;
  - pulsanti di almeno 24 px.
- Impostazione **Dimensione dell'interfaccia** (100-200%), con disposizione adattiva e menu a scomparsa
  quando lo spazio è poco.
- Pulsante **Segna come da leggere** nell'anteprima, utilizzabile anche da tastiera (prima solo con il
  doppio clic).
- Sezione Accessibilità in README e Credits.

### Modificato
- Contrasti dei testi secondari e dei colori di stato portati ad almeno 4,5:1 anche sulle righe
  selezionate; stella dei contrassegni e testo segnaposto della ricerca resi più leggibili.
- Vista Moderna: titoli di sezione in ordine gerarchico e area principale dichiarata.

### Rimosso
- Tema scuro, non previsto dal design system di Designers Italia.

## 0.7.0 - 2026-11-11

### Aggiunto
- Motore XSLT 1.0 / XPath 1.0 nativo (`rustinvoice-xslt`) per la vista AssoSoftware: non
  dipende più dal supporto XSLT della webview, che i motori basati su Chromium stanno
  rimuovendo. Il motore della webview resta come riserva.
- Programma di esempio `trasforma` e prove automatiche del motore, con risultati confrontati
  con un motore XSLT di riferimento su tutte le fatture di prova.

## 0.6.1 - 2026-11-11

### Modificato
- Verificata l'aderenza al design system di Designers Italia (componenti Bootstrap Italia,
  tipografia, palette, contrasti dei testi WCAG 2.1 AA, lingua, tastiera); README e
  Credits lo descrivono con precisione.
- Il visualizzatore degli allegati porta il fuoco nella finestra all'apertura e lo
  restituisce alla chiusura.

## 0.6.0 - 2026-11-11

### Aggiunto
- Sezione **Statistiche** per anno o per tutto il periodo: fatturato e acquisti
  (imponibile e totale), IVA a debito e a credito, IVA in scissione dei pagamenti, saldo
  IVA indicativo, grafico mensile o annuale di emesse e ricevute (con tabella dei valori)
  e classifica di clienti e fornitori con imponibile e IVA.
- Nella scheda di ogni cliente e fornitore: documenti, imponibile, IVA e totale per anno.

### Modificato
- Menu riorganizzato: Statistiche e Clienti e fornitori in cima, poi i documenti, gli anni
  e gli strumenti.

### Corretto
- Icona mancante nella scheda Clienti e fornitori.

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
