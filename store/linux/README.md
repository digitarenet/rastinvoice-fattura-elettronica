<!-- Copyright (c) 2026 Andrea Re - info@digitare.net - Autore: Andrea Re @ 2026 - info@digitare.net -->

# Pubblicazione su Linux

Tre canali, in ordine di utilità e semplicità. I testi della scheda sono in [`../testi.md`](../testi.md);
la scheda AppStream, usata da tutti gli store grafici, è [`net.digitare.rastinvoice.metainfo.xml`](net.digitare.rastinvoice.metainfo.xml).

| Canale | Pacchetto | Costo | Chi lo usa |
| --- | --- | --- | --- |
| Pagina delle release GitHub | `.deb`, `.rpm`, `.AppImage` | gratuito | tutti, scaricando il file |
| Flathub | Flatpak | gratuito | Fedora, Linux Mint, Pop!_OS, elementary, Steam Deck e quasi tutte le distribuzioni |
| AUR (facoltativo) | PKGBUILD | gratuito | Arch e Manjaro |

## 1. Pacchetti .deb, .rpm e AppImage

Si compilano sul PC Linux con `store/linux/build.sh` e si caricano su una release di GitHub con
`store/linux/pubblica.sh` (vedi [Compilare in locale](../README.md#compilare-in-locale)):

- `RastInvoice_<versione>_amd64.deb` per Debian, Ubuntu, Linux Mint, Pop!_OS, elementary, Zorin, MX Linux.
  È compilato in un container Debian 12, quindi funziona su Debian 12 e Ubuntu 22.04 o più recenti.
  Installazione: `sudo apt install ./RastInvoice_<versione>_amd64.deb`.
- `RastInvoice-<versione>-1.x86_64.rpm` per Fedora, openSUSE, RHEL, Rocky e Alma.
  Installazione: `sudo dnf install ./RastInvoice-<versione>-1.x86_64.rpm`.
- `RastInvoice_<versione>_amd64.AppImage`: un solo file eseguibile, senza installazione.

Nessuna iscrizione richiesta. Basta pubblicare la release.

## 2. Flathub

È lo store predefinito della maggior parte delle distribuzioni, compresa la "Software" di Fedora e Mint.

Requisiti sul PC Linux, una volta sola:

```bash
sudo apt install flatpak flatpak-builder python3-venv git
flatpak remote-add --user --if-not-exists flathub https://dl.flathub.org/repo/flathub.flatpakrepo
```

1. Provare in locale: `store/linux/flatpak.sh` genera gli elenchi delle dipendenze, compila dai sorgenti
   senza rete (come Flathub), installa l'app per l'utente (`flatpak run net.digitare.rastinvoice`) e crea
   `target/flatpak/RastInvoice_<versione>.flatpak`. La prima volta scarica i runtime GNOME (circa 1 GB).
2. Pubblicare la release `v<versione>` su GitHub: il manifesto per Flathub compila dal tag.
3. `store/linux/flatpak.sh --flathub` prepara in `target/flatpak/flathub/` il manifesto con tag e commit
   aggiornati e i due file json delle dipendenze. Avvisa se manca il tag o la voce della versione in
   `net.digitare.rastinvoice.metainfo.xml`.
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

## 3. AUR (facoltativo)

Per Arch si può pubblicare un pacchetto `rastinvoice-bin` che riusa il `.deb` della release. Conviene farlo
solo se qualcuno lo chiede: la comunità Arch spesso lo crea da sé.

## Stato

Gli script `build.sh` e `flatpak.sh` non sono ancora stati eseguiti: vanno provati una volta sul
PC Linux prima del primo invio.
