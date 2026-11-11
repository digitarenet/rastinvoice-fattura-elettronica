<!-- Copyright (c) 2026 Andrea Re - info@digitare.net - Autore: Andrea Re @ 2026 - info@digitare.net -->

# Pubblicazione su Linux

Quattro canali, in ordine di utilità e semplicità. I testi della scheda sono in [`../testi.md`](../testi.md);
la scheda AppStream, usata da tutti gli store grafici, è [`net.digitare.rastinvoice.metainfo.xml`](net.digitare.rastinvoice.metainfo.xml).

| Canale | Pacchetto | Costo | Chi lo usa |
| --- | --- | --- | --- |
| Pagina delle release GitHub | `.deb`, `.rpm`, `.AppImage` | gratuito | tutti, scaricando il file |
| Flathub | Flatpak | gratuito | Fedora, Linux Mint, Pop!_OS, elementary, Steam Deck e quasi tutte le distribuzioni |
| Snap Store | Snap | gratuito | Ubuntu (Centro software) |
| AUR (facoltativo) | PKGBUILD | gratuito | Arch e Manjaro |

## 1. Pacchetti .deb, .rpm e AppImage

Li crea il workflow **Rilascio** (Actions > Rilascio > Run workflow) e li allega alla release in bozza:

- `RastInvoice_<versione>_amd64.deb` per Debian, Ubuntu, Linux Mint, Pop!_OS, elementary, Zorin, MX Linux.
  È compilato su Ubuntu 22.04, quindi funziona su Debian 12 e Ubuntu 22.04 o più recenti.
  Installazione: `sudo apt install ./RastInvoice_<versione>_amd64.deb`.
- `RastInvoice-<versione>-1.x86_64.rpm` per Fedora, openSUSE, RHEL, Rocky e Alma.
  Installazione: `sudo dnf install ./RastInvoice-<versione>-1.x86_64.rpm`.
- `RastInvoice_<versione>_amd64.AppImage`: un solo file eseguibile, senza installazione.

Nessuna iscrizione richiesta. Basta pubblicare la release.

Si possono anche compilare sul PC Linux e caricare con `store/linux/build.sh` e `store/linux/pubblica.sh`:
vedi [Compilare in locale](../README.md#compilare-in-locale).

## 2. Flathub

È lo store predefinito della maggior parte delle distribuzioni, compresa la "Software" di Fedora e Mint.

1. Pubblicare la release `v1.0.0` su GitHub (il manifesto compila dal tag).
2. Generare gli elenchi delle dipendenze, dalla radice del repository, con
   [flatpak-builder-tools](https://github.com/flatpak/flatpak-builder-tools):
   ```bash
   python3 flatpak-cargo-generator.py Cargo.lock -o store/linux/flatpak/cargo-sources.json
   flatpak-node-generator npm apps/desktop/package-lock.json -o store/linux/flatpak/node-sources.json
   ```
3. Provare la compilazione in locale:
   ```bash
   flatpak install flathub org.gnome.Sdk//48 org.freedesktop.Sdk.Extension.rust-stable//24.08 org.freedesktop.Sdk.Extension.node24//24.08
   flatpak-builder --user --install --force-clean build store/linux/flatpak/net.digitare.rastinvoice.yml
   flatpak run net.digitare.rastinvoice
   ```
4. Fare il fork di https://github.com/flathub/flathub, creare un branch dal branch `new-pr`, aggiungere
   il manifesto e i due file json, aprire una pull request verso `new-pr` intitolata
   "Add net.digitare.rastinvoice".
5. I revisori di Flathub chiedono di solito qualche modifica (permessi, metainfo). Dopo l'approvazione
   viene creato il repository `flathub/net.digitare.rastinvoice`; da lì si pubblicano le versioni successive.
6. Verifica dello sviluppatore (spunta "verificato"): pubblicare un token su `digitare.net/.well-known/org.flathub.VerifiedApps.txt`,
   seguendo le istruzioni nella pagina dell'app su flathub.org.

Note: l'identificativo `net.digitare.rastinvoice` richiede di poter dimostrare il controllo del dominio
digitare.net. I permessi chiesti sono la cartella home e i dischi esterni (fatture e archivi di rete)
e la rete (foglio di stile e aggiornamenti, solo su richiesta): sono da motivare nella pull request.

## 3. Snap Store

1. Creare un account su https://snapcraft.io (account Ubuntu One, gratuito).
2. Su Ubuntu: `sudo snap install snapcraft --classic` e `snapcraft login`.
3. Registrare il nome: `snapcraft register rastinvoice`.
4. Copiare `store/linux/snap/snapcraft.yaml` in `snap/snapcraft.yaml` nella radice e lanciare `snapcraft`.
5. Provare: `sudo snap install --dangerous rastinvoice_1.0.0_amd64.snap`.
6. Pubblicare: `snapcraft upload --release=stable rastinvoice_1.0.0_amd64.snap`.
7. Nella pagina del pacchetto su snapcraft.io completare la scheda: icona, schermate, categoria
   **Productivity** e informativa privacy.

## 4. AUR (facoltativo)

Per Arch si può pubblicare un pacchetto `rastinvoice-bin` che riusa il `.deb` della release. Conviene farlo
solo se qualcuno lo chiede: la comunità Arch spesso lo crea da sé.

## Stato

Il manifesto Flatpak e il file snapcraft non sono ancora stati compilati: vanno provati una volta su un
computer Linux prima dell'invio. I pacchetti `.deb`, `.rpm` e AppImage vengono creati dal workflow Rilascio,
che non è ancora stato eseguito.
