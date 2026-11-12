<!-- Copyright (c) 2026 Andrea Re - info@digitare.net - Autore: Andrea Re @ 2026 - info@digitare.net -->

# Sito rastinvoice.it

Sito di presentazione di RastInvoice, in HTML, CSS e JavaScript senza strumenti di compilazione. Il modulo
contatti usa un piccolo script PHP.

## Contenuto

| File | Cosa contiene |
| --- | --- |
| `index.html` | pagina principale: presentazione, open source, Designers Italia, funzionalità, scheda tecnica, immagini, download, domande frequenti, contatti |
| `come-aprire-fattura-elettronica-xml.html`, `come-aprire-file-p7m.html`, `fattura-elettronica-in-pdf.html` | guide per le ricerche più frequenti, con dati strutturati HowTo e FAQ |
| `404.html` | pagina non trovata |
| `llms.txt` | riassunto del prodotto per gli assistenti AI (ChatGPT, Claude, Perplexity, Gemini) |
| `manifest.webmanifest` | nome e icone per browser e dispositivi |
| `privacy.html` | informativa sulla privacy del sito e del modulo contatti |
| `grazie.html` | conferma dopo l'invio del modulo quando JavaScript è disattivato |
| `contatti.php` | invio del modulo per email, con `mail()` di PHP o SMTP autenticato |
| `config-esempio.php` | modello della configurazione SMTP, da copiare come `rastinvoice-config.php` fuori dalla radice del sito |
| `prepara.sh` | prepara in `target/sito/` le cartelle `public_html/` e `privato/` da caricare sul server |
| `.htaccess` | per Apache: HTTPS e dominio unico www, compressione, cache, intestazioni di sicurezza, pagina 404, blocco dei file di configurazione |
| `css/bootstrap-italia.min.css` | [Bootstrap Italia](https://italia.github.io/bootstrap-italia/) 2.18.0 (BSD-3-Clause), ridotto alle sole regole usate (32 KB invece di 600 KB) |
| `css/sito.css` | stili propri del sito |
| `js/sito.js` | menu su schermi piccoli, galleria, invio del modulo |
| `fonts/` | Titillium Web (SIL Open Font License), ospitato localmente |
| `img/` | logo, icone, schermate in WebP, anteprima per i social, icone di Bootstrap Italia (`sprites.svg`) |
| `robots.txt`, `sitemap.xml` | indicazioni per i motori di ricerca |

Il sito non usa cookie né statistiche e non carica nulla da altri server, quindi non serve il banner dei
cookie.

## Pubblicazione

Serve un hosting con PHP 8.1 o successivo. Su un VPS basta un server web con PHP-FPM.

1. Eseguire `sito/prepara.sh`: crea in `target/sito/` la cartella `public_html/`, da copiare nella radice
   del dominio (`domains/rastinvoice.it/public_html/`), e la cartella `privato/` con la configurazione del
   modulo, da copiare accanto (`domains/rastinvoice.it/privato/`). Non usare `private_html` per i file
   riservati: sui server DirectAdmin è la radice del sito in HTTPS, quindi raggiungibile dal web.
2. Creare la casella `sito@rastinvoice.it`, che fa da mittente dei messaggi del modulo. I messaggi arrivano
   a `info@digitare.net`, con l'indirizzo di chi scrive come "Rispondi a".
3. Scegliere come inviare (vedi sotto) e mandare un messaggio di prova dal modulo.
4. Configurare SPF e DKIM del dominio `rastinvoice.it` per il server di invio, altrimenti i messaggi possono
   finire nello spam.
5. Attivare HTTPS e il reindirizzamento da `rastinvoice.it` a `www.rastinvoice.it` (o il contrario,
   aggiornando allora gli indirizzi in `index.html`, `privacy.html`, `robots.txt` e `sitemap.xml`).
6. Registrare il sito in Google Search Console e Bing Webmaster Tools e indicare `sitemap.xml`.

### Invio dei messaggi

**Senza credenziali (predefinito).** `contatti.php` usa la funzione `mail()` di PHP, cioè il server di posta
dell'hosting. Va bene sugli hosting condivisi che la offrono: non serve configurare nulla, se non mittente
e destinatario diversi da quelli predefiniti.

**Con SMTP autenticato (consigliato se `mail()` non c'è o i messaggi finiscono nello spam).** Copiare
`config-esempio.php` con il nome `rastinvoice-config.php` (in locale `sito/rastinvoice-config.php`, escluso
da git: `prepara.sh` lo mette in `privato/`) e caricarlo nella cartella `privato` accanto alla radice del
sito (`domains/rastinvoice.it/privato/`) o direttamente sopra la radice e indicare server, porta, utente e password della casella
`sito@rastinvoice.it`. In quella posizione il file con la password non è raggiungibile dal web e non finisce
nel repository. Se l'hosting non permette di scrivere sopra la radice, il file si può mettere accanto a
`contatti.php`: il file `.htaccess` ne blocca l'accesso sui server Apache.

### Protezione dallo spam

Il modulo non usa captcha, per non chiedere nulla ai visitatori e non caricare servizi esterni. Gli invii
automatici vengono scartati con tre controlli: un campo nascosto che solo i programmi compilano, il tempo
minimo di compilazione (3 secondi) e il limite di 5 invii all'ora per indirizzo IP. Se in futuro arrivasse
spam, si può aggiungere un captcha senza tracciamento come ALTCHA (ospitato sul sito) o Cloudflare
Turnstile, aggiornando l'informativa sulla privacy.

Se l'hosting non ha PHP, il modulo può puntare a un servizio esterno (per esempio Formspree): basta cambiare
l'attributo `action` del modulo in `index.html` e aggiornare l'informativa sulla privacy.

Prova in locale:

```sh
cd sito
php -S localhost:8000      # con PHP: anche il modulo risponde (senza inviare email reali, se manca sendmail)
python3 -m http.server     # senza PHP: tutto tranne il modulo
```

## Collegamenti da aggiornare

Finché le schede negli store non sono pubblicate e il repository GitHub è privato, il sito non ha link
che non funzionano: i pulsanti di download sono segnaposto "Prossimamente" (`<span>` con classe `presto`,
sezione Scarica e piè di pagina di `index.html`) e i link a GitHub sono stati tolti da tutte le pagine
(menu, contatti, piè di pagina, informativa, `llms.txt`, dati strutturati `sameAs`). Alla pubblicazione si
ripristinano dalla cronologia di git (commit "Sito: pulsanti Prossimamente") con i collegamenti definitivi
qui sotto.

La sezione "Open source" di `index.html` (`id="open-source"`) resta anche dopo la pubblicazione. Quando il
repository diventa pubblico: sostituire lo `<span id="pulsante-github">` con il link già pronto nel commento
accanto, togliere la nota `nota-os`, la voce "Codice sorgente su GitHub (prossimamente)" del piè di pagina
diventa un link, e nella domanda frequente sull'open source (anche nei dati strutturati FAQPage) e in
`llms.txt` si toglie "aperto a tutti a breve".

| Destinazione | Collegamento provvisorio | Collegamento definitivo |
| --- | --- | --- |
| Microsoft Store | `https://apps.microsoft.com/search?query=RastInvoice` | `https://apps.microsoft.com/detail/<ID prodotto>` |
| Mac App Store | `https://apps.apple.com/it/` | `https://apps.apple.com/it/app/rastinvoice/id<numero>` |
| Flathub | `https://flathub.org/apps/net.digitare.rastinvoice` | già definitivo |
| Pacchetti Linux | release di GitHub | già definitivo, funziona quando il repository è pubblico |

Per i pulsanti degli store il sito usa icone e testi propri. Se si preferiscono i badge ufficiali
(Microsoft, Apple, Flathub), vanno scaricati dalle pagine dei rispettivi programmi per sviluppatori e
salvati in `img/`, senza collegarli dai server esterni, così il sito continua a non inviare dati ad altri.

## Aggiornamenti

- **Nuova versione del programma:** aggiornare `softwareVersion` nei dati strutturati in `index.html`.
- **Nuove schermate:** sostituire i file in `docs/immagini/` e rigenerare le versioni WebP (1600 e 800 pixel
  di larghezza) in `img/`.
- **Bootstrap Italia:** copiare `dist/svg/sprites.svg` dal pacchetto npm e ridurre il foglio di stile alle
  regole usate: `npx purgecss --css bootstrap-italia.min.css --content 'sito/*.html' sito/js/sito.js --safelist aperto show active alert alert-success alert-danger alert-info alert-warning visually-hidden --output sito/css/`.
  Se si aggiungono pagine o classi nuove, ripetere la riduzione.
- **Nuove guide:** aggiungerle a `sitemap.xml`, a `llms.txt` e all'elenco Guide della pagina principale.

## SEO e motori di risposta AI

- Titolo, descrizione e H1 con le parole chiave principali (visualizzatore, fattura elettronica, XML, P7M, PDF).
- Dati strutturati schema.org: SoftwareApplication, FAQPage, WebSite e Person nella pagina principale; HowTo,
  FAQPage, BreadcrumbList e Article nelle guide. Si controllano con il
  [Test dei risultati multimediali](https://search.google.com/test/rich-results).
- Scheda tecnica con dati precisi (versione, prezzo, formati, licenza) e `llms.txt`: sono le parti che gli
  assistenti AI citano più volentieri.
- `robots.txt` ammette esplicitamente i crawler dei motori di ricerca e degli assistenti AI.
- Prestazioni: Lighthouse 96-99 in locale; sul server `.htaccess` aggiunge compressione e cache.
- Su Nginx servono le stesse regole di `.htaccess`: redirect a `https://www.`, gzip, `expires`, intestazioni,
  `error_page 404 /404.html` e il blocco di `rastinvoice-config.php`.
