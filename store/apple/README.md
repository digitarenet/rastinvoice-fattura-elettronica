<!-- Copyright (c) 2026 Andrea Re - info@digitare.net - Autore: Andrea Re @ 2026 - info@digitare.net -->

# Pubblicazione per macOS

Ci sono due strade. Consiglio di partire dalla prima e valutare la seconda solo dopo.

| | dmg firmato e notarizzato | Mac App Store |
| --- | --- | --- |
| Costo | Apple Developer Program, 99 USD/anno | lo stesso |
| Distribuzione | download dal sito o da GitHub | dentro l'App Store |
| Limiti | nessuno | sandbox obbligatoria: l'archivio può leggere solo le cartelle scelte dall'utente, e serve memorizzarne i permessi |
| Lavoro | configurare 6 segreti su GitHub | modifiche all'app, profilo di provisioning, revisione Apple |

Senza iscrizione il dmg funziona lo stesso, ma al primo avvio macOS lo blocca: l'utente deve fare clic
destro > Apri, oppure andare in Impostazioni di Sistema > Privacy e sicurezza > Apri comunque.

## 1. Iscrizione

1. Andare su https://developer.apple.com/programs/enroll/ con un Apple ID con autenticazione a due fattori.
2. Scegliere **Individuale** (come persona fisica, a nome Andrea Re: la ditta individuale non ha un
   numero D-U-N-S da società). Costo 99 USD l'anno.

## 2. dmg firmato e notarizzato

1. Su un Mac, in Xcode > Settings > Accounts, creare il certificato **Developer ID Application**.
2. Esportarlo da Accesso Portachiavi come file `.p12` con una password, poi convertirlo:
   `base64 -i certificato.p12 | pbcopy`.
3. Su https://appleid.apple.com creare una **password specifica per app**.
4. In GitHub, Settings > Secrets and variables > Actions > **Secrets**, creare:

| Segreto | Valore |
| --- | --- |
| `APPLE_CERTIFICATE` | il testo base64 del file .p12 |
| `APPLE_CERTIFICATE_PASSWORD` | la password del .p12 |
| `APPLE_SIGNING_IDENTITY` | per esempio `Developer ID Application: Andrea Re (TEAMID)` |
| `APPLE_ID` | l'Apple ID |
| `APPLE_PASSWORD` | la password specifica per app |
| `APPLE_TEAM_ID` | il Team ID, visibile in developer.apple.com > Account |

5. Avviare Actions > Rilascio > Run workflow. Il passo "Firma Apple" attiva firma e notarizzazione
   solo se i segreti sono presenti; senza segreti il dmg resta non firmato come oggi.

## 3. Mac App Store (in un secondo momento)

Richiede, oltre all'iscrizione:

- certificati **Apple Distribution** e **Mac Installer Distribution** e un profilo di provisioning;
- un file di entitlements con `com.apple.security.app-sandbox`, `files.user-selected.read-write`,
  `network.client` e i segnalibri di sicurezza per ricordare le cartelle d'archivio tra un avvio e l'altro
  (oggi l'app ricorda solo il percorso: va modificata);
- la creazione dell'app in App Store Connect con i testi di [`../testi.md`](../testi.md), le schermate
  (1280×800 o 2880×1800) e l'informativa privacy;
- il caricamento con Transporter e la revisione di Apple, che di solito richiede da 1 a 3 giorni.

## Stato

Il workflow crea già un dmg universale (Intel e Apple Silicon), mai provato su un Mac reale. La firma
è predisposta ma non provata, perché servono i segreti.
