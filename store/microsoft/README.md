<!-- Copyright (c) 2026 Andrea Re - info@digitare.net - Autore: Andrea Re @ 2026 - info@digitare.net -->

# Pubblicazione su Microsoft Store

Guida passo passo per pubblicare RastInvoice su Microsoft Store come app gratuita, con un account
sviluppatore individuale. I testi della scheda sono in [`../testi.md`](../testi.md).

Pagina iniziale di Partner Center: https://partner.microsoft.com/dashboard/apps-and-games/overview

## 1. Iscrizione (una volta sola)

1. Su https://storedeveloper.microsoft.com scegliere **Individual developer** (gratuito) ed entrare con il
   proprio account Microsoft.
2. Completare la verifica dell'identità con documento e selfie (meglio dallo smartphone; in caso di blocco
   riprovare più tardi o aprire una richiesta da **? → Assistenza** in Partner Center).
3. Con l'account individuale l'editore mostrato nello Store è il proprio nome («Andrea Re»).

## 2. Prenotazione del nome

1. Partner Center → **App e giochi** → **+ Nuovo prodotto** → **App MSIX o PWA**.
2. Scrivere **RastInvoice** → **Verifica disponibilità** → **Prenota nome del prodotto**.

Il nome resta prenotato per tre mesi: entro quel periodo va inviata la prima versione.

## 3. Identità del pacchetto

Nella pagina dell'app: **Gestione prodotto** → **Identità del prodotto**. I tre valori vanno nella sezione
`microsoft` di `store/configurazione.local.json` (escluso da git; si crea copiando
`store/configurazione.esempio.json`):

```json
"microsoft": {
  "nomePacchetto": "<Package/Identity/Name, es. 12345AndreaRe.RastInvoice>",
  "editore": "<Package/Identity/Publisher, inizia con CN=>",
  "editoreVisualizzato": "<Package/Properties/PublisherDisplayName>"
}
```

I valori vanno copiati esattamente: se non coincidono, Partner Center rifiuta il pacchetto.

## 4. Creazione del pacchetto

Sul PC Windows, dalla radice del repository aggiornato (`git pull`), con Rust, Node.js 24 e il Windows SDK
installati (requisiti in [Compilare in locale](../README.md#compilare-in-locale)):

```powershell
pwsh store/microsoft/build.ps1          # compila e crea target/msix/RastInvoice_<versione>.0_x64.msix
pwsh store/microsoft/build.ps1 -Prova   # in più installa il pacchetto sul PC per provarlo
```

`-Prova` richiede la **Modalità sviluppatore** (Impostazioni → Sistema → Per sviluppatori). Dopo la prova
l'app si disinstalla dal menu Start, così non entra in conflitto con quella scaricata dallo Store.

Il pacchetto non è firmato: lo firma Microsoft Store. In attesa dello Store, su GitHub si pubblica anche
l'eseguibile senza firma creato da `store/windows/build.ps1`, che al primo avvio fa comparire l'avviso di
SmartScreen (vedi [Compilare in locale](../README.md#compilare-in-locale)).

## 5. Invio

Pagina dell'app → **Panoramica dell'applicazione** → **Avvia l'invio**. Le sezioni si compilano in
qualsiasi ordine; ognuna va salvata.

### 5.1 Prezzi e disponibilità

| Campo | Valore |
| --- | --- |
| Mercati | tutti i mercati (oppure solo Italia) |
| Visibilità | Pubblica, rendi il prodotto disponibile e individuabile nello Store |
| Pianificazione | appena possibile dopo la certificazione |
| Prezzo base | **Gratuito** |
| Versione di valutazione | nessuna |

### 5.2 Proprietà

| Campo | Valore |
| --- | --- |
| Categoria | **Business** (sottocategoria Contabilità e finanza, se presente); in alternativa Produttività |
| Informativa sulla privacy | https://www.rastinvoice.it/privacy.html |
| Sito Web | https://www.rastinvoice.it |
| Recapiti del supporto | info@digitare.net |
| Accesso a dati personali | **No**: l'app non raccoglie né invia dati personali |
| Requisiti di sistema | lasciare i valori predefiniti; nota: «Windows 10 o 11 a 64 bit con WebView2» |
| Dichiarazioni del prodotto | non selezionare nulla di particolare (nessuna pubblicità, nessun acquisto in app) |

### 5.3 Classificazioni per età

Compilare il questionario IARC: categoria **App (non gioco)**, indirizzo email di contatto, poi rispondere
**No** a tutte le domande (violenza, contenuti sessuali, linguaggio, sostanze, gioco d'azzardo,
interazione o condivisione tra utenti, condivisione della posizione, acquisti digitali). Risultato atteso:
**3+ / PEGI 3**.

### 5.4 Pacchetti

1. Trascinare il file `target/msix/RastInvoice_<versione>.0_x64.msix`.
2. Famiglie di dispositivi: solo **Windows 10/11 Desktop**.
3. Se compare l'avviso sulla funzionalità `runFullTrust`, è normale: serve alle app desktop
   (motivazione da indicare, se richiesta: «Desktop application built with a native runtime; it needs full
   trust to open invoice files and folders chosen by the user»).

### 5.5 Schede dello Store

**Aggiungi/rimuovi lingue** → lasciare solo **Italiano (Italia)**, poi aprire la scheda italiana:

| Campo | Da [`testi.md`](../testi.md) |
| --- | --- |
| Descrizione del prodotto | «Descrizione completa» |
| Novità di questa versione | «Novità di questa versione» |
| Funzionalità del prodotto | «Funzionalità principali», una per riga |
| Schermate (desktop) | i sei file di `docs/immagini/` (2720×1720), ognuno con la sua didascalia |
| Logo dello Store 1:1 | `store/immagini/logo-1080.png` (o `logo-300.png`) |
| Poster 2:3 (consigliato) | `store/immagini/poster-720x1080.png` |
| Descrizione breve | «Descrizione breve» |
| Termini di ricerca (fino a 7) | fattura elettronica · visualizzatore fatture · fatturapa · fattura pa · p7m · fattura xml · lettore fatture |
| Copyright e marchio | © 2026 Andrea Re |
| Condizioni di licenza aggiuntive | Licenza EUPL-1.2: https://github.com/digitarenet/rastinvoice-fattura-elettronica/blob/main/LICENSE |
| Requisiti di sistema / note | Windows 10 o 11 a 64 bit; su Windows 10 serve Microsoft Edge WebView2 |

### 5.6 Opzioni di invio → Note per la certificazione

> RastInvoice is a free, open source viewer for Italian electronic invoices (FatturaPA XML and signed
> P7M files). It works offline and does not require an account. To test it, open any FatturaPA sample
> invoice (for example from the Italian Revenue Agency website, https://www.fatturapa.gov.it) with
> "Apri file", or drag the file onto the window. The app connects to the Internet only when the user
> downloads the official stylesheet (Settings).

Pubblicazione: lasciare «Pubblica il prodotto non appena supera la certificazione».

### 5.7 Invio

**Invia allo Store**. La certificazione richiede di solito da qualche ora a 3 giorni lavorativi; l'esito
arriva per email e in Partner Center.

## 6. Dopo la pubblicazione

- Il link pubblico è `https://apps.microsoft.com/detail/<ID prodotto>`; l'ID (12 caratteri, inizia con `9`)
  è in **Gestione prodotto → Identità del prodotto**. Va inserito nel README e nei pulsanti del sito.
- A ogni nuova versione: `git pull`, `pwsh store/microsoft/build.ps1`, poi in Partner Center
  **Aggiorna** l'invio, sostituire il pacchetto e aggiornare «Novità di questa versione». Il numero di
  versione deve crescere ogni volta.
- Recensioni e statistiche di installazione: sezione **Approfondimenti** di Partner Center.

## Requisiti verificati

- Iscrizione individuale gratuita dal 2025 ([annuncio Microsoft](https://blogs.windows.com/windowsdeveloper/2025/09/10/free-developer-registration-for-individual-developers-on-microsoft-store/)).
- I pacchetti MSIX vengono firmati dallo store ([opzioni di firma](https://learn.microsoft.com/en-us/windows/apps/package-and-deploy/code-signing-options)).
- L'app richiede WebView2, già presente in Windows 11 e nelle versioni aggiornate di Windows 10.
