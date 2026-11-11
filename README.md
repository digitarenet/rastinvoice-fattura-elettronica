# RustinVoice

> Nome provvisorio. Copyright (c) 2026 Andrea Re - info@digitare.net - [digitare.net](https://www.digitare.net)

Visualizzatore di fatture elettroniche italiane (formato **FatturaPA**), portable e multipiattaforma: Windows, Linux e macOS, senza installazione.

Apre fatture emesse e ricevute in tutti i formati in cui arrivano di solito: `.xml`, `.xml.p7m` firmati (anche in base64 o con più firme), lotti con più documenti, fatture semplificate. Riconosce ricevute e notifiche del Sistema di Interscambio senza confonderle con le fatture.

## Stato del progetto

Fase 1 in corso: nucleo di lettura e riga di comando. L'interfaccia grafica arriverà nella fase 2.

| Funzione | Stato |
| --- | --- |
| Modello dati FPA12 / FPR12 / FSM10 | fatto |
| Lettura XML con codifiche UTF-8, UTF-16, ISO-8859-1, Windows-1252 | fatto |
| Buste firmate `.p7m` (DER, BER, base64, annidate) | fatto |
| Riconoscimento ricevute e notifiche SdI | fatto |
| Tabelle codici (specifiche 1.9) | fatto |
| Riga di comando (`info`, `json`, `scansiona`) | fatto |
| Validazione XSD e controlli SdI | da fare |
| Verifica crittografica della firma | da fare |
| Email PEC `.eml`, Outlook `.msg`, archivi `.zip` | da fare |
| Fogli di stile, PDF, stampa | da fare |
| Applicazione desktop | da fare |

## Struttura

```
crates/
  rustinvoice-core/   nucleo: modello dati, lettura, buste firmate (nessuna dipendenza grafica)
  rustinvoice-cli/    riga di comando
```

## Compilazione

Serve [Rust](https://www.rust-lang.org/tools/install) 1.80 o successivo.

```sh
cargo build --release
cargo test
```

L'eseguibile si trova in `target/release/rustinvoice` (`rustinvoice.exe` su Windows).

## Uso da riga di comando

```sh
# riepilogo leggibile
rustinvoice info IT01234567890_0A1B2.xml.p7m

# intero contenuto in JSON (utile per integrazioni)
rustinvoice json IT01234567890_0A1B2.xml

# elenco CSV di tutti i documenti di una cartella e delle sottocartelle
rustinvoice scansiona ./Fatture > elenco.csv
```

## Riferimenti normativi

Il software è scritto a partire dalla documentazione pubblica dell'Agenzia delle Entrate sulla [fatturazione elettronica](https://www.agenziaentrate.gov.it/portale/fatturazione-elettronica-e-dati-fatture-transfrontaliere-new): specifiche tecniche del formato FatturaPA (versione 1.9) ed elenco dei controlli del Sistema di Interscambio.

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

L'elenco completo delle dipendenze, incluse quelle indirette, si ottiene con `cargo tree`.

## Licenza

In definizione. Fino alla pubblicazione della licenza tutti i diritti sono riservati all'autore.

## Autore

Andrea Re - [digitare.net](https://www.digitare.net) - info@digitare.net
