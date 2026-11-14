<!-- Copyright (c) 2026 Andrea Re - info@digitare.net - Autore: Andrea Re @ 2026 - info@digitare.net -->

# Materiale per gli store

| Store | Cartella | Pacchetto | Costo | Cosa serve da Andrea |
| --- | --- | --- | --- | --- |
| Microsoft Store | [`microsoft/`](microsoft/README.md) | MSIX | gratuito | iscrizione individuale, identità del pacchetto |
| GitHub (Windows e Linux) | [`windows/`](windows/), [`linux/`](linux/README.md) | exe, deb, rpm, AppImage | gratuito | `build` e `pubblica` di ciascun sistema |
| Flathub | [`linux/`](linux/README.md) | Flatpak | gratuito | prova su Linux, pull request a Flathub |
| Mac App Store | [`apple/`](apple/README.md) | pkg | 99 USD/anno di iscrizione; app a 2,99 € | iscrizione, certificati, revisione Apple |

Materiale comune:

- [`testi.md`](testi.md): nome, descrizioni, funzionalità, parole chiave, didascalie e dati comuni.
- [`immagini/`](immagini/): logo 300×300 e 1080×1080, poster 720×1080.
- [`immagini/logo/`](immagini/logo/): file SVG del logo (marchio, logo completo e versione su fondo blu, icone),
  generati da `genera.py` (richiede `pip install fonttools`).
- `../docs/immagini/`: schermate a 2720×1720.
- `../docs/PRIVACY.md`: informativa privacy, da indicare come URL in ogni store.
- [`note-release.md`](note-release.md): parte fissa delle note di ogni release di GitHub (download e avviso di
  SmartScreen), aggiunta dagli script `pubblica`.

Distribuzione: Windows con il file `.exe` su GitHub (gratuito, senza firma: al primo avvio compare l'avviso di
SmartScreen) e, appena pubblicato, dal Microsoft Store (nessun avviso); macOS solo dal Mac App Store (2,99 €);
Linux da GitHub e Flathub. Il pulsante per le donazioni non compare nella
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

### Windows: eseguibile per GitHub

Stessi requisiti dell'MSIX, senza Windows SDK, più GitHub CLI (`winget install GitHub.cli`, poi `gh auth login`).

```powershell
pwsh store/windows/build.ps1               # crea target/rilascio/RastInvoice-windows-x64.exe
pwsh store/windows/pubblica.ps1            # lo carica sulla release v<versione> (in bozza se nuova)
pwsh store/windows/pubblica.ps1 -Pubblica  # ... e la rende pubblica, se contiene già i pacchetti Linux
```

Il file non richiede installazione e non ha firma digitale: al primo avvio Windows mostra «Windows ha protetto il
PC» (si apre con Ulteriori informazioni, Esegui comunque).

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
- I file da pubblicare vengono copiati in `target/rilascio` con nomi fissi, senza versione
  (`RastInvoice-linux-amd64.deb`, `RastInvoice-linux-x86_64.rpm`, `RastInvoice-linux-x86_64.AppImage`).
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

### Rilascio su GitHub

I pulsanti del sito e del README puntano a `releases/latest/download/<nome fisso>`, cioè all'ultima release
pubblicata: ogni release pubblicata deve contenere tutti i file, altrimenti il sito dà errore 404.

| File | Creato da |
| --- | --- |
| `RastInvoice-windows-x64.exe` | `store/windows/build.ps1` (PC Windows) |
| `RastInvoice-linux-amd64.deb`, `RastInvoice-linux-x86_64.rpm`, `RastInvoice-linux-x86_64.AppImage`, `latest.json` | `store/linux/build.sh` (PC Linux) |

Ordine: alzare la versione e fare push su `main`, poi su ciascun PC lo script `pubblica` (il primo crea la
bozza, l'altro aggiunge i file) e infine `-Pubblica`/`--pubblica`.

- **Versione per un solo sistema:** pubblicando, i file che mancano vengono copiati dall'ultima release
  pubblicata con i nomi fissi (per Linux anche `latest.json`), così il sito non dà errori.
  Esempio: `pwsh store/windows/pubblica.ps1 -Pubblica` con solo l'exe nuovo.
- **Release scelta:** `-Versione 1.3.1` (Windows) o `--versione 1.3.1` (Linux) carica i file nella release
  `v1.3.1` invece di quella della versione corrente. Una release più vecchia dell'ultima non diventa "latest".

### Controlli prima di un rilascio

```bash
cargo fmt --all --check
cargo clippy -p rustinvoice-core -p rustinvoice-render -p rustinvoice-cli -p rustinvoice-xslt --all-targets -- -D warnings
cargo test -p rustinvoice-core -p rustinvoice-render -p rustinvoice-cli -p rustinvoice-xslt
(cd apps/desktop && npm ci && npm run check)
```
