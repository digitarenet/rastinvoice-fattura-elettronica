<!-- Copyright (c) 2026 Andrea Re - info@digitare.net - Autore: Andrea Re @ 2026 - info@digitare.net -->

# Materiale per gli store

| Store | Cartella | Pacchetto | Costo | Cosa serve da Andrea |
| --- | --- | --- | --- | --- |
| Microsoft Store | [`microsoft/`](microsoft/README.md) | MSIX | gratuito | iscrizione individuale, identità del pacchetto |
| GitHub (solo Linux) | [`linux/`](linux/README.md) | deb, rpm, AppImage | gratuito | `store/linux/build.sh` e `pubblica.sh` |
| Flathub | [`linux/`](linux/README.md) | Flatpak | gratuito | prova su Linux, pull request a Flathub |
| Mac App Store | [`apple/`](apple/README.md) | pkg | 99 USD/anno di iscrizione; app a 2,99 € | iscrizione, certificati, revisione Apple |

Materiale comune:

- [`testi.md`](testi.md): nome, descrizioni, funzionalità, parole chiave, didascalie e dati comuni.
- [`immagini/`](immagini/): logo 300×300 e 1080×1080, poster 720×1080.
- [`immagini/logo/`](immagini/logo/): file SVG del logo (marchio, logo completo e versione su fondo blu, icone),
  generati da `genera.py` (richiede `pip install fonttools`).
- `../docs/immagini/`: schermate a 2720×1720.
- `../docs/PRIVACY.md`: informativa privacy, da indicare come URL in ogni store.

Distribuzione: Windows solo dal Microsoft Store (gratuito, nessun avviso di SmartScreen), macOS solo dal Mac
App Store (2,99 €), Linux da GitHub e Flathub. Il pulsante per le donazioni non compare nella
versione per macOS, perché Apple non ammette donazioni esterne.

Ordine consigliato: Microsoft Store e release Linux su GitHub, poi Flathub, infine Mac App
Store.

## Compilare in locale

Tutti i pacchetti si compilano sui propri PC: il repository non ha CI su GitHub. Gli script si eseguono dalla
cartella principale del repository.

### Configurazione

Copiare `store/configurazione.esempio.json` in `store/configurazione.local.json` (escluso da git) e compilare
solo le sezioni che servono:

| Sezione | Valori | Serve per |
| --- | --- | --- |
| `microsoft` | nome del pacchetto, editore `CN=...`, nome visualizzato (da Partner Center > Identità del prodotto) | MSIX |
| `aggiornamenti` | percorso della chiave privata, password, chiave pubblica | aggiornamenti automatici su Linux |
| `apple` | Team ID, nomi dei due certificati, percorso del profilo di provisioning | pacchetto per il Mac App Store |

### Windows: MSIX per il Microsoft Store

Requisiti, una volta sola:

- Rust (`winget install Rustlang.Rustup`) e Node.js 24 (`winget install OpenJS.NodeJS`);
- Visual Studio 2022/2026 o Build Tools con il carico di lavoro **Sviluppo di applicazioni desktop con C++**
  e il **Windows SDK** (contiene `makeappx.exe`);
- PowerShell 7 (`winget install Microsoft.PowerShell`).

```powershell
pwsh store/microsoft/build.ps1          # crea target/msix/RastInvoice_<versione>.0_x64.msix
pwsh store/microsoft/build.ps1 -Prova   # in più lo installa sul PC (serve la Modalità sviluppatore)
```

Il file `.msix` si carica in Partner Center. Su GitHub non si pubblica.

### Linux (LMDE 7): deb, rpm e AppImage

I pacchetti si compilano in un container Debian 12: compilati direttamente su LMDE 7 (Debian 13) non
partirebbero su Ubuntu 22.04/24.04 e Debian 12.

Requisiti, una volta sola: `sudo apt install podman` (va bene anche docker).

```bash
store/linux/build.sh              # crea il container la prima volta, poi compila deb, rpm e AppImage
store/linux/pubblica.sh           # carica i pacchetti su una release in bozza v<versione>
store/linux/pubblica.sh --pubblica   # carica e rende pubblica la release
```

- La prima esecuzione scarica l'immagine e compila tutto (qualche minuto); le successive riusano la cache in
  `target/cargo-debian12` e sono molto più rapide.
- Se `aggiornamenti` è compilato, `build.sh` firma i pacchetti e crea `latest.json` per gli aggiornamenti
  automatici; altrimenti crea pacchetti senza aggiornamenti.
- `store/linux/build.sh --locale` compila direttamente sul PC, senza container (solo per prove).
- Il Flatpak ha uno script a parte, con i suoi prerequisiti: `store/linux/flatpak.sh` (dettagli in
  [`linux/`](linux/README.md)). Va su Flathub, non su GitHub.
- `pubblica.sh` richiede GitHub CLI con accesso (`sudo apt install gh`, poi `gh auth login`). Prende le note
  della versione da `docs/CHANGELOG.md` e avvisa se il commit locale non è quello su `origin/main`.

### macOS: pacchetto per il Mac App Store

Requisiti, una volta sola:

- `xcode-select --install` (o Xcode completo, utile per creare i certificati);
- Rust (`curl https://sh.rustup.rs -sSf | sh`) e Node.js 24 (`brew install node@24` o dal sito);
- per il pacchetto firmato: certificati **Apple Distribution** e **Mac Installer Distribution** nel
  Portachiavi e il profilo di provisioning Mac App Store (vedi [`apple/`](apple/README.md)).

```bash
store/apple/build.sh
```

Crea l'app universale (Apple Silicon e Intel). Con la sezione `apple` compilata la firma e crea
`target/mac-app-store/RastInvoice_<versione>_universale.pkg`, da caricare con l'app Transporter. Prima del
primo invio vanno completate le modifiche per la sandbox descritte in [`apple/`](apple/README.md).

### Controlli prima di un rilascio

```bash
cargo fmt --all --check
cargo clippy -p rustinvoice-core -p rustinvoice-render -p rustinvoice-cli -p rustinvoice-xslt --all-targets -- -D warnings
cargo test -p rustinvoice-core -p rustinvoice-render -p rustinvoice-cli -p rustinvoice-xslt
(cd apps/desktop && npm ci && npm run check)
```
