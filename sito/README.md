<!-- Copyright (c) 2026 Andrea Re - info@digitare.net - Autore: Andrea Re @ 2026 - info@digitare.net -->

# Sito rastinvoice.it

Sito di presentazione di RastInvoice, in HTML, CSS e JavaScript senza strumenti di compilazione. Il modulo
contatti usa un piccolo script PHP.

## Contenuto

| File | Cosa contiene |
| --- | --- |
| `index.html` | pagina unica: presentazione, funzionalità, immagini, download, domande frequenti, contatti |
| `privacy.html` | informativa sulla privacy del sito e del modulo contatti |
| `grazie.html` | conferma dopo l'invio del modulo quando JavaScript è disattivato |
| `contatti.php` | invio del modulo per email |
| `css/bootstrap-italia.min.css` | [Bootstrap Italia](https://italia.github.io/bootstrap-italia/) 2.18.0 (BSD-3-Clause), solo il foglio di stile |
| `css/sito.css` | stili propri del sito |
| `js/sito.js` | menu su schermi piccoli, galleria, invio del modulo |
| `fonts/` | Titillium Web (SIL Open Font License), ospitato localmente |
| `img/` | logo, icone, schermate in WebP, anteprima per i social, icone di Bootstrap Italia (`sprites.svg`) |
| `robots.txt`, `sitemap.xml` | indicazioni per i motori di ricerca |

Il sito non usa cookie né statistiche e non carica nulla da altri server, quindi non serve il banner dei
cookie.

## Pubblicazione

Serve un hosting con PHP 8.1 o successivo e la funzione `mail()` attiva (quasi tutti gli hosting condivisi
italiani la offrono). Su un VPS basta un server web con PHP-FPM e un servizio di invio della posta.

1. Copiare il contenuto della cartella `sito/` nella radice del dominio, escluso questo `README.md`.
2. Creare la casella o l'alias `sito@rastinvoice.it` (mittente dei messaggi) e verificare che
   `rastinvoice@digitare.net` riceva la posta. Se si cambiano gli indirizzi, modificarli in cima a
   `contatti.php`.
3. Configurare SPF e DKIM del dominio `rastinvoice.it` per il server di invio, altrimenti i messaggi possono
   finire nello spam.
4. Attivare HTTPS e il reindirizzamento da `rastinvoice.it` a `www.rastinvoice.it` (o il contrario,
   aggiornando allora gli indirizzi in `index.html`, `privacy.html`, `robots.txt` e `sitemap.xml`).
5. Inviare un messaggio di prova dal modulo.
6. Registrare il sito in Google Search Console e Bing Webmaster Tools e indicare `sitemap.xml`.

Se l'hosting non ha PHP, il modulo può puntare a un servizio esterno (per esempio Formspree): basta cambiare
l'attributo `action` del modulo in `index.html` e aggiornare l'informativa sulla privacy.

Prova in locale:

```sh
cd sito
php -S localhost:8000      # con PHP: anche il modulo risponde (senza inviare email reali, se manca sendmail)
python3 -m http.server     # senza PHP: tutto tranne il modulo
```

## Collegamenti da aggiornare

Alcuni collegamenti sono provvisori finché le schede negli store non sono pubblicate. Si trovano in
`index.html` (sezione Scarica e piè di pagina) e nel `README.md` del repository.

| Destinazione | Collegamento provvisorio | Collegamento definitivo |
| --- | --- | --- |
| Microsoft Store | `https://apps.microsoft.com/search?query=RastInvoice` | `https://apps.microsoft.com/detail/<ID prodotto>` |
| Mac App Store | `https://apps.apple.com/it/` | `https://apps.apple.com/it/app/rastinvoice/id<numero>` |
| Flathub | `https://flathub.org/apps/net.digitare.rastinvoice` | già definitivo |
| Snap Store | `https://snapcraft.io/rastinvoice` | già definitivo |
| Pacchetti Linux | release di GitHub | già definitivo, funziona quando il repository è pubblico |

Per i pulsanti degli store il sito usa icone e testi propri. Se si preferiscono i badge ufficiali
(Microsoft, Apple, Flathub, Snap), vanno scaricati dalle pagine dei rispettivi programmi per sviluppatori e
salvati in `img/`, senza collegarli dai server esterni, così il sito continua a non inviare dati ad altri.

## Aggiornamenti

- **Nuova versione del programma:** aggiornare `softwareVersion` nei dati strutturati in `index.html`.
- **Nuove schermate:** sostituire i file in `docs/immagini/` e rigenerare le versioni WebP (1600 e 800 pixel
  di larghezza) in `img/`.
- **Bootstrap Italia:** copiare `dist/css/bootstrap-italia.min.css` e `dist/svg/sprites.svg` dal pacchetto npm.
