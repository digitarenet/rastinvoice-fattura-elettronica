<!-- Copyright (c) 2026 Andrea Re - info@digitare.net - Autore: Andrea Re @ 2026 - info@digitare.net -->

# Pubblicazione su Microsoft Store

Procedura completa per pubblicare RastInvoice su Microsoft Store come app gratuita. I testi della scheda
sono in [`../testi.md`](../testi.md).

## 1. Iscrizione (una volta sola)

1. Andare su https://storedeveloper.microsoft.com e scegliere l'account **individuale** (gratuito).
2. Completare la verifica dell'identità con un documento.
3. Come nome visualizzato dell'editore indicare "digitare.net", se disponibile, altrimenti "Andrea Re".

## 2. Prenotazione del nome

In Partner Center: **Apps e giochi > Nuovo prodotto > App MSIX o PWA**, nome **RastInvoice**.

## 3. Identità del pacchetto

In **Gestione prodotto > Identità del prodotto** si trovano tre valori. Vanno nella sezione `microsoft` di
`store/configurazione.local.json` (esclusa da git), non nel codice:

| Campo | Valore in Partner Center |
| --- | --- |
| `nomePacchetto` | Package/Identity/Name |
| `editore` | Package/Identity/Publisher (inizia con `CN=`) |
| `editoreVisualizzato` | Package/Properties/PublisherDisplayName |

## 4. Creazione del pacchetto

Sul PC Windows (requisiti in [Compilare in locale](../README.md#compilare-in-locale)):

```powershell
pwsh store/microsoft/build.ps1          # compila e crea target/msix/RastInvoice_<versione>.0_x64.msix
pwsh store/microsoft/build.ps1 -Prova   # in più installa il pacchetto sul PC per provarlo
```

Il pacchetto non è firmato: lo firma Microsoft Store. In attesa dello Store, su GitHub si pubblica anche
l'eseguibile senza firma creato da `store/windows/build.ps1`, che al primo avvio fa comparire l'avviso di
SmartScreen (vedi [Compilare in locale](../README.md#compilare-in-locale)).

## 5. Invio (Partner Center > Invii > Avvia l'invio)

| Sezione | Cosa inserire |
| --- | --- |
| Prezzi e disponibilità | Gratuito; mercati: tutti, oppure solo Italia; visibilità pubblica |
| Proprietà | Categoria **Business** (sottocategoria contabilità e finanza, se presente) o **Produttività**; informativa privacy: URL di `docs/PRIVACY.md` (oppure https://www.rastinvoice.it/privacy.html); sito; contatto: info@digitare.net |
| Proprietà > dichiarazioni | L'app non raccoglie dati personali; accede a Internet solo su richiesta dell'utente |
| Classificazioni per età | Questionario IARC: nessuna violenza, nessun contenuto sensibile, nessuna interazione tra utenti, nessun acquisto, nessuna condivisione della posizione. Risultato atteso: 3+ |
| Pacchetti | caricare il file `.msix`; famiglia di dispositivi: Windows 10/11 Desktop |
| Scheda dello store (italiano) | descrizione, funzionalità, parole chiave e novità da `testi.md`; schermate da `docs/immagini/` con le didascalie di `testi.md`; logo 1:1 `store/immagini/logo-1080.png` (300×300: `logo-300.png`); poster 2:3 `store/immagini/poster-720x1080.png` |
| Note per la certificazione | testo qui sotto |

### Note per la certificazione

> RastInvoice is a free, open source viewer for Italian electronic invoices (FatturaPA XML and signed
> P7M files). It works offline and does not require an account. To test it, open any FatturaPA sample
> invoice (for example from the Italian Revenue Agency website, https://www.fatturapa.gov.it) with
> "Apri file", or drag the file onto the window. The app connects to the Internet only when the user
> downloads the official stylesheet (Settings) or enables the optional update check.

## 6. Dopo la pubblicazione

- La certificazione richiede di solito da qualche ora a 3 giorni lavorativi.
- A ogni nuova versione: eseguire `store/microsoft/build.ps1`, creare un nuovo invio e caricare il nuovo `.msix`.
  Il numero di versione deve crescere ogni volta.
- Le recensioni si leggono e si rispondono in Partner Center.

## Requisiti verificati

- Iscrizione individuale gratuita dal 2025 ([annuncio Microsoft](https://blogs.windows.com/windowsdeveloper/2025/09/10/free-developer-registration-for-individual-developers-on-microsoft-store/)).
- I pacchetti MSIX vengono firmati dallo store ([opzioni di firma](https://learn.microsoft.com/en-us/windows/apps/package-and-deploy/code-signing-options)).
- L'app richiede WebView2, già presente in Windows 11 e nelle versioni aggiornate di Windows 10.
- Il pacchetto non è ancora stato provato su un computer Windows: va installato una volta in locale
  (`Add-AppxPackage` dopo averlo firmato con un certificato di prova) o inviato direttamente in
  certificazione, che segnala eventuali errori.
