# RustinVoice

> Nome provvisorio. Copyright (c) 2026 Andrea Re - info@digitare.net - [digitare.net](https://www.digitare.net)

Visualizzatore di fatture elettroniche italiane (formato **FatturaPA**), portable e multipiattaforma: Windows, Linux e macOS, senza installazione.

Apre fatture emesse e ricevute in tutti i formati in cui arrivano di solito: `.xml`, `.xml.p7m` firmati (anche in base64 o con più firme), lotti con più documenti, fatture semplificate. Riconosce ricevute e notifiche del Sistema di Interscambio senza confonderle con le fatture.

## Stato del progetto

Fase 2 in corso: applicazione desktop con interfaccia grafica, costruita sul nucleo di lettura della fase 1.

| Funzione | Stato |
| --- | --- |
| Modello dati FPA12 / FPR12 / FSM10 | fatto |
| Lettura XML con codifiche UTF-8, UTF-16, ISO-8859-1, Windows-1252 | fatto |
| Buste firmate `.p7m` (DER, BER, base64, annidate) | fatto |
| Riconoscimento ricevute e notifiche SdI | fatto |
| Tabelle codici (specifiche 1.9.1) | fatto |
| Riga di comando (`info`, `json`, `scansiona`) | fatto |
| Controlli di validità con codici errore SdI (calcoli, partite IVA, codici fiscali, IBAN) | fatto |
| Validazione XSD completa | da fare |
| Verifica crittografica della firma (integrità e firmatario) | fatto |
| Verifica della catena di certificazione e revoca | da fare |
| Email PEC `.eml` e archivi `.zip` | fatto |
| Outlook `.msg` | da fare |
| Indice locale per avvii rapidi, cartelle di rete e NAS | fatto |
| Applicazione desktop (Windows, Linux, macOS) | prima versione |
| Vista "Moderna" e foglio di stile ufficiale AssoSoftware (scaricabile) | fatto |
| Ricerca a testo libero, filtri per sezione, anno, date, contatto | fatto |
| Rubrica clienti e fornitori | fatto |
| Statistiche: fatturato, acquisti, IVA per anno, mese e controparte | fatto |
| Trascina e rilascia, modalità portable | fatto |
| Stampa, salvataggio HTML e XML, allegati, esportazione CSV | fatto |
| Visualizzatore allegati (PDF, immagini, Word, Excel, ODS, CSV, testo, zip) | fatto |
| Esportazione PDF diretta | da fare |
| Trasformazione XSL nativa (indipendente dal motore web di sistema) | da fare |

## Struttura

```
crates/
  rustinvoice-core/     nucleo: modello dati, lettura, buste firmate (nessuna dipendenza grafica)
  rustinvoice-render/   vista "Moderna" in HTML a partire dal modello dati
  rustinvoice-cli/      riga di comando
apps/
  desktop/              applicazione desktop (Tauri 2 + Svelte 5 + Bootstrap Italia)
    src/                interfaccia
    src-tauri/          parte nativa in Rust
```

## Compilazione

Serve [Rust](https://www.rust-lang.org/tools/install) 1.80 o successivo.

```sh
cargo build --release
cargo test
```

L'eseguibile si trova in `target/release/rustinvoice` (`rustinvoice.exe` su Windows).

### Applicazione desktop

Servono anche [Node.js](https://nodejs.org) 20 o successivo e i prerequisiti di Tauri per il proprio sistema
([elenco](https://v2.tauri.app/start/prerequisites/)): su Windows WebView2, già presente in Windows 10 e 11;
su Linux i pacchetti di sviluppo di WebKitGTK 4.1.

```sh
cargo run -p rustinvoice-desktop                    # avvio (anche dal pulsante Run dell'IDE)
cargo build --release -p rustinvoice-desktop        # eseguibile in target/release

cd apps/desktop
npm run app:dev     # sviluppo con ricarica automatica dell'interfaccia
npm run app:build   # eseguibile singolo in target/release
npx tauri build     # anche installer/pacchetti (NSIS, AppImage, DMG)
```

Con `cargo build` e `cargo run` l'interfaccia viene ricompilata automaticamente (con `npm`) quando i suoi
sorgenti sono più recenti dell'ultima compilazione. Per disattivare questo passaggio impostare la variabile
d'ambiente `RUSTINVOICE_SENZA_NPM=1`. In RustRover sono già pronte le configurazioni di avvio
"RustinVoice desktop", "Crea EXE" e "Test".

L'eseguibile `target/release/rustinvoice-desktop` non richiede installazione. Al primo avvio chiede dove
salvare i dati: nel profilo utente oppure accanto al programma (**modalità portable**, per esempio su una
chiavetta USB, con la cartella `dati` creata automaticamente). La scelta si può cambiare dalle Impostazioni.

Il foglio di stile ufficiale AssoSoftware non è incluso: si scarica dalle impostazioni dell'applicazione
(dal sito ufficiale) oppure si importa da un file `.xsl` o `.zip`.

## Uso da riga di comando

```sh
# riepilogo leggibile
rustinvoice info IT01234567890_0A1B2.xml.p7m

# intero contenuto in JSON (utile per integrazioni)
rustinvoice json IT01234567890_0A1B2.xml

# elenco CSV di tutti i documenti di una cartella e delle sottocartelle
rustinvoice scansiona ./Fatture > elenco.csv
```

## Versioni

Le modifiche di ogni versione sono descritte nel [registro delle modifiche](CHANGELOG.md), consultabile
anche dalla voce **Credits** dell'applicazione.

## Riferimenti normativi

Il software è scritto a partire dalla documentazione pubblica dell'Agenzia delle Entrate sulla [fatturazione elettronica](https://www.agenziaentrate.gov.it/portale/fatturazione-elettronica-e-dati-fatture-transfrontaliere-new): specifiche tecniche del formato FatturaPA (versione 1.9.1, in vigore dal 15 maggio 2026) ed elenco dei controlli del Sistema di Interscambio.

## Librerie utilizzate

Si ringraziano gli autori delle librerie open source su cui si basa il progetto.

| Libreria | Autori | Licenza |
| --- | --- | --- |
| [roxmltree](https://github.com/RazrFalcon/roxmltree) | Yevhenii Reizner | MIT / Apache-2.0 |
| [encoding_rs](https://github.com/hsivonen/encoding_rs) | Henri Sivonen | (MIT / Apache-2.0) e BSD-3-Clause |
| [rust_decimal](https://github.com/paupino/rust-decimal) | Paul Mason e contributori | MIT |
| [chrono](https://github.com/chronotope/chrono) | Kang Seonghoon, Brandon W Maister e contributori | MIT / Apache-2.0 |
| [serde](https://serde.rs), [serde_json](https://github.com/serde-rs/json) | Erick Tryzelaar, David Tolnay | MIT / Apache-2.0 |
| [thiserror](https://github.com/dtolnay/thiserror) | David Tolnay | MIT / Apache-2.0 |
| [base64](https://github.com/marshallpierce/rust-base64) | Marshall Pierce, Alice Maz | MIT / Apache-2.0 |
| [clap](https://github.com/clap-rs/clap) | Kevin K., Ed Page e contributori | MIT / Apache-2.0 |
| [walkdir](https://github.com/BurntSushi/walkdir) | Andrew Gallant | MIT / Unlicense |
| [Tauri](https://tauri.app) e plugin dialog, opener | Tauri Programme within The Commons Conservancy | MIT / Apache-2.0 |
| [rayon](https://github.com/rayon-rs/rayon) | Niko Matsakis, Josh Stone | MIT / Apache-2.0 |
| [ureq](https://github.com/algesten/ureq) | Martin Algesten | MIT / Apache-2.0 |
| [zip](https://github.com/zip-rs/zip2) | Mathijs van de Nes, Marli Frost, Ryan Levick, Chris Hennick | MIT |
| [rsa](https://github.com/RustCrypto/RSA), [sha1, sha2](https://github.com/RustCrypto/hashes), [p256, p384](https://github.com/RustCrypto/elliptic-curves) | RustCrypto Developers | MIT / Apache-2.0 |
| [calamine](https://github.com/tafia/calamine) | Johann Tuffe e contributori | MIT |
| [PDF.js](https://mozilla.github.io/pdf.js/) | Mozilla e contributori | Apache-2.0 |
| [mammoth](https://github.com/mwilliamson/mammoth.js) | Michael Williamson | BSD-2-Clause |
| [Svelte](https://svelte.dev) | Rich Harris e contributori | MIT |
| [Vite](https://vite.dev) | Evan You e contributori | MIT |
| [TypeScript](https://www.typescriptlang.org) | Microsoft | Apache-2.0 |
| [Bootstrap Italia](https://italia.github.io/bootstrap-italia/) | Designers Italia / Dipartimento per la trasformazione digitale | BSD-3-Clause |
| [Titillium Web](https://fonts.google.com/specimen/Titillium+Web) | Accademia di Belle Arti di Urbino | SIL Open Font License 1.1 |
| [Roboto Mono](https://fonts.google.com/specimen/Roboto+Mono) | Christian Robertson | Apache-2.0 |

L'interfaccia segue il [design system di Designers Italia](https://designers.italia.it/design-system/).

L'elenco completo delle dipendenze, incluse quelle indirette, si ottiene con `cargo tree` e `npm ls`.

## Segnalazioni di sicurezza

Vedere [SECURITY.md](SECURITY.md).

## Licenza

In definizione. Fino alla pubblicazione della licenza tutti i diritti sono riservati all'autore.

## Autore

Andrea Re - [digitare.net](https://www.digitare.net) - info@digitare.net
