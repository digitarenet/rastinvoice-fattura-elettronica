<!-- Copyright (c) 2026 Andrea Re - info@digitare.net - Autore: Andrea Re @ 2026 - info@digitare.net -->

# Pubblicazione sul Mac App Store

Su macOS RastInvoice si distribuisce solo dal Mac App Store, al prezzo di **2,99 €**. Su GitHub non si
pubblica il dmg. Chi vuole può comunque compilare il programma dai sorgenti.

## Conti

| Voce | Importo |
| --- | --- |
| Apple Developer Program | 99 USD l'anno |
| Prezzo di vendita | 2,99 € IVA inclusa |
| Commissione Apple (Small Business Program, ricavi sotto 1 milione l'anno) | 15% |
| Ricavo per copia | circa 2,08 € (2,99 € meno IVA 22% e commissione) |
| Copie per coprire l'iscrizione | circa 45-50 l'anno |

Per il Mac App Store Apple fa da rivenditore: incassa e versa l'IVA ai clienti e paga lo sviluppatore ogni mese.
Gli incassi sono da dichiarare: conviene sentire il commercialista.

## 1. Iscrizione

1. Andare su https://developer.apple.com/programs/enroll/ con un Apple ID con autenticazione a due fattori.
2. Scegliere **Individuale**, a nome Andrea Re. La ditta individuale non ha il numero D-U-N-S che Apple chiede
   alle società.
3. In App Store Connect, alla voce **Accordi, imposte e banche**, firmare l'accordo per le **app a pagamento**
   e inserire conto bancario e dati fiscali. Senza questo passo l'app non può essere venduta.
4. Iscriversi all'**App Store Small Business Program**: commissione al 15% invece del 30%.

## 2. Modifiche all'app (da fare prima del primo invio)

Le app del Mac App Store girano nella sandbox di Apple. Per RastInvoice significa:

- un file di entitlements con `com.apple.security.app-sandbox`,
  `com.apple.security.files.user-selected.read-write`, `com.apple.security.network.client` e
  `com.apple.security.files.bookmarks.app-scope`;
- salvare per ogni cartella d'archivio un **segnalibro di sicurezza** (security-scoped bookmark) e
  riattivarlo a ogni avvio: oggi l'app ricorda solo il percorso, e dopo il riavvio la sandbox negherebbe
  l'accesso;
- nessun pulsante per le donazioni: Apple non le ammette. L'app lo nasconde già su macOS.

L'app riconosce già di essere installata dal Mac App Store: nasconde il controllo degli aggiornamenti e la
scelta della cartella dati accanto al programma.

## 3. Certificati e pacchetto

1. In Xcode > Settings > Accounts creare i certificati **Apple Distribution** e
   **Mac Installer Distribution**.
2. In developer.apple.com creare l'identificativo dell'app (`net.digitare.rustinvoice`) e un profilo di
   provisioning **Mac App Store**.
3. Sul Mac eseguire `store/apple/build.sh`: compila l'app universale, la firma con il certificato di
   distribuzione e gli entitlements (`entitlements.plist`), crea il pacchetto con `productbuild` e lo lascia in
   `target/mac-app-store/`. Il pacchetto si carica con l'app **Transporter**. Senza certificati lo script crea
   solo `RastInvoice.app`, utile per le prove. Dettagli in [Compilare in locale](../README.md#compilare-in-locale).

## 4. Scheda in App Store Connect

- Nome, sottotitolo, descrizione e parole chiave da [`../testi.md`](../testi.md).
- Categoria **Economia** (Business), secondaria **Produttività**.
- Prezzo: fascia da 2,99 €, disponibile in tutti i paesi o solo in Italia.
- Schermate 2880×1800 o 1280×800: quelle in `docs/immagini/` vanno ridimensionate a 2880×1800.
- Informativa privacy: URL di `docs/PRIVACY.md` (oppure https://www.rastinvoice.it/privacy.html); nella sezione "Privacy dell'app" indicare **nessun dato raccolto**.
- La revisione di Apple richiede di solito da 1 a 3 giorni.

## Stato

Le modifiche per la sandbox (punto 2) non sono ancora state fatte e non c'è ancora un Mac su cui provarle.
