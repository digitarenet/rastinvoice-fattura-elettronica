# Registro delle modifiche

Novità, miglioramenti e correzioni di RastInvoice (fino alla 0.9.0 chiamato RustinVoice). Il numero di
versione segue lo schema MAGGIORE.MINORE.CORREZIONE: la versione minore cresce con le nuove funzioni, quella
di correzione con le sole correzioni. Le versioni senza novità nel programma non sono elencate.

## 1.1.6 - 2026-11-12

### Modificato
- Su macOS i menu della barra in alto sono in italiano: RastInvoice, Archivio, Modifica, Vista e Finestra.

## 1.1.5 - 2026-11-12

### Modificato
- Nella barra in alto dell'applicazione compare il logo di RastInvoice, lo stesso dell'icona del programma,
  al posto dell'icona generica del documento.

## 1.1.0 - 2026-11-11

### Aggiunto
- Aggiornamento dall'applicazione per le versioni Linux scaricate dal sito (AppImage, .deb e .rpm). Con il
  controllo delle nuove versioni attivo, la versione AppImage scarica e installa da sola la nuova versione,
  che diventa attiva al riavvio; per i pacchetti .deb e .rpm basta un clic su **Installa** in Impostazioni,
  con la password di amministratore. Prima dell'installazione il programma verifica la firma digitale
  dell'autore, così si installano solo versioni originali. Le versioni scaricate dagli store continuano ad
  aggiornarsi tramite lo store.

## 1.0.0

Prima versione stabile, pronta per la pubblicazione negli store.

### Aggiunto
- Il pulsante **PDF** dell'anteprima crea direttamente il PDF della fattura, senza passare dalla finestra di
  stampa. Il PDF è in formato A4 con testo selezionabile e si apre nel visualizzatore integrato, da cui si
  salva o si apre con il programma predefinito. Le fatture lunghe vanno su più pagine, con le intestazioni
  delle tabelle ripetute e i numeri di pagina.

## 0.13.0

### Aggiunto
- Statistiche più complete:
  - vendite e acquisti con e senza IVA e la loro differenza;
  - vendite degli ultimi 12 mesi, con la media mensile.
- Regime forfettario riconosciuto dalle fatture emesse. In questo caso le statistiche mostrano quanto manca
  alla soglia di 85.000 € di ricavi e l'IVA pagata sugli acquisti, che non è detraibile. La differenza tra
  vendite e acquisti tiene conto di quell'IVA come costo. Il saldo IVA non viene mostrato.
- Sezione **Sostieni RastInvoice** nei Credits, con il pulsante per una donazione con PayPal (non nella versione
  per macOS), un modulo che prepara un'email nel programma di posta e
  l'elenco dei sostenitori che vogliono essere citati.

### Modificato
- Nelle versioni scaricate da uno store gli aggiornamenti li gestisce lo store: il controllo delle nuove
  versioni e la scelta della cartella dati accanto al programma non vengono mostrati.

## 0.12.0

### Aggiunto
- Pulsanti **Indietro** e **Avanti** accanto alla ricerca, come in un browser: ogni clic su una pagina, una
  sezione, un filtro, un cliente o un documento si può annullare. Funzionano anche con Alt+freccia sinistra e
  destra e con i tasti laterali del mouse. Mentre si scrive nella ricerca o si scorre l'elenco con le frecce
  viene registrato un solo passo.

## 0.11.2

### Migliorato
- Statistiche: i nomi di clienti e fornitori non sono più sottolineati e si leggono meglio. Passando con il
  mouse la riga si evidenzia e un clic su qualsiasi punto della riga apre i documenti del contatto.
- L'installer per Windows aggiunge da solo il componente WebView2, se manca.

## 0.11.0

### Aggiunto
- Pacchetti di installazione per Linux: `.deb` per Debian, Ubuntu e derivate, `.rpm` per Fedora e openSUSE.
- Informativa sulla privacy: il programma non raccoglie dati e si collega a Internet solo su richiesta.

## 0.10.3

### Corretto
- Copiando nell'archivio un file con lo stesso nome di uno già presente ma diverso, il numero veniva aggiunto
  dopo l'estensione (`fatture.zip_1`) e il file non veniva più riconosciuto. Ora il numero va prima delle
  estensioni (`fatture_1.zip`).

### Modificato
- La finestra dei file trascinati spiega che archivi ZIP ed email vengono copiati interi, senza estrarli.

## 0.10.2

### Corretto
- La casella "Ricorda la scelta" nella finestra dei file trascinati mostrava due quadrati.
- La barra dei pulsanti dell'anteprima va a capo quando lo spazio non basta, invece di nascondere gli ultimi
  pulsanti.

## 0.10.0

### Modificato
- Nuovo nome del prodotto: **RastInvoice**. La cartella dei dati resta la stessa, quindi impostazioni e
  archivio vengono mantenuti.
- Il comando della riga di comando si chiama ora `rastinvoice` (prima `rustinvoice`): gli script che lo usano
  vanno aggiornati.

### Pianificato
- Supporto della fattura elettronica europea secondo la norma EN 16931: formati UBL e CII, documenti della
  rete Peppol e PDF con XML incorporato (ZUGFeRD e Factur-X).

## 0.9.0

### Aggiunto
- Licenza open source **EUPL-1.2** (licenza pubblica dell'Unione europea), indicata anche nei Credits.
- Pulsante **Salva in PDF** nell'anteprima: apre la finestra di stampa del sistema con il nome del file già
  proposto e ricorda di scegliere la stampante PDF.
- Controllo facoltativo delle nuove versioni (Impostazioni > Aggiornamenti), disattivato in modo predefinito.
- Versione portatile per Windows in un file `.zip`, da usare anche su chiavetta USB.

### Sicurezza
- Lettura degli archivi ZIP aggiornata a una versione che elimina quattro vulnerabilità note.

## 0.8.0

### Aggiunto
- Accessibilità secondo le WCAG 2.1 livello AA (EN 301 549, Linee guida AgID):
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
- Sezione Accessibilità nei Credits.

### Modificato
- Contrasti dei testi secondari e dei colori di stato portati ad almeno 4,5:1 anche sulle righe
  selezionate; stella dei contrassegni e testo segnaposto della ricerca resi più leggibili.
- Vista Moderna: titoli di sezione in ordine gerarchico e area principale dichiarata.

### Rimosso
- Tema scuro, non previsto dal design system di Designers Italia.

## 0.7.0

### Migliorato
- La vista AssoSoftware usa un motore di trasformazione interno e non dipende più dal browser integrato nel
  sistema, che in futuro potrebbe non supportarla più.

## 0.6.1

### Modificato
- Interfaccia allineata al design system di Designers Italia (componenti, tipografia, colori, contrasti).
- Il visualizzatore degli allegati porta il fuoco nella finestra all'apertura e lo restituisce alla chiusura.

## 0.6.0

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

## 0.5.0

### Aggiunto
- **Scelta della cartella dati** al primo avvio e nelle Impostazioni: nel profilo utente
  oppure accanto al programma (modalità portable, la cartella `dati` viene creata
  automaticamente). Cambiando scelta, impostazioni, indice, foglio di stile e archivio
  vengono copiati nella nuova cartella e l'applicazione si riavvia; la vecchia cartella
  portable viene rinominata, non cancellata.

## 0.4.1

### Modificato
- La sezione "Ricevute SdI" diventa **Notifiche SdI** ed è più chiara: ogni notifica mostra
  data, stato in parole ("Consegnata", "Scartata", "Rifiutata dal cliente"...), errori e
  motivazioni, con il collegamento alla fattura a cui si riferisce; una legenda spiega
  tutte le sigle.
- L'anteprima della fattura mostra lo stato presso lo SdI e la cronologia delle notifiche.
- I file non riconosciuti sono elencati a parte, con una spiegazione.

## 0.4.0

### Aggiunto
- **Visualizzatore degli allegati** integrato: si apre facendo clic sul nome dell'allegato.
  - PDF, senza accesso alla rete né esecuzione di script del documento.
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

## 0.3.2

### Sicurezza
- Le finestre di salvataggio (CSV, HTML, XML, allegati) sono gestite direttamente dal programma, che decide
  dove scrivere i file.
- La finestra dell'applicazione non può essere portata su pagine esterne, nemmeno da
  collegamenti contenuti in una fattura.
- Gli allegati aperti vengono scritti in una cartella privata dell'utente, nuova a ogni
  apertura; su Windows sono marcati come provenienti da Internet, così Office li apre in
  visualizzazione protetta.
- L'esportazione CSV neutralizza i testi che Excel eseguirebbe come formule.

## 0.3.1

### Migliorato
- Avvio con l'indice circa 2,8 volte più rapido (10.000 file: da 175 a 63 millisecondi).
- Le cartelle vengono esplorate in parallelo: molto più veloce su dischi di rete e NAS.
- Scorrimento e ricerca restano fluidi anche con decine di migliaia di fatture.

## 0.3.0

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

### Modificato
- La scheda del foglio di stile ufficiale si chiama ora "AssoSoftware".
- Gli allegati si aprono solo se di tipo sicuro (documenti, immagini, testo); gli altri
  si possono salvare.

### Corretto
- Barra verticale nera sulle voci selezionate in Windows.
- La vista con foglio di stile poteva mostrare il documento precedente durante lo scorrimento.
- Apertura degli allegati e delle cartelle bloccata dai permessi dell'applicazione.

## 0.2.0

### Aggiunto
- Applicazione desktop: elenco con ricerca a testo libero, sezioni ricevute, emesse, da leggere,
  contrassegnate e con anomalie, filtri per anno, date e contatto.
- Anteprima nella vista Moderna e con il foglio di stile ufficiale AssoSoftware,
  scaricabile dal sito dell'associazione.
- Rubrica di clienti e fornitori con totali e documenti per contatto.
- Trascina e rilascia, modalità portable, stampa, salvataggio HTML e XML, allegati,
  esportazione CSV.

## 0.1.0

### Aggiunto
- Lettura delle fatture FatturaPA (specifiche 1.9.1): fatture ordinarie, verso la Pubblica
  Amministrazione e semplificate, lotti, vecchi file con codifiche diverse, buste firmate .p7m
  (anche in base64 o con più firme).
- Riconoscimento delle ricevute e notifiche del Sistema di Interscambio.
- Riga di comando con i comandi `info`, `json` e `scansiona`.
