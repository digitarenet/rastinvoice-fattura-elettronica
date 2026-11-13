#!/usr/bin/env bash
# Copyright (c) 2026 Andrea Re - info@digitare.net
# Autore: Andrea Re @ 2026 - info@digitare.net
#
# Prepara le cartelle da caricare sul server in target/sito/:
#   public_html/  contenuto del sito, da copiare in domains/rastinvoice.it/public_html/
#   privato/      configurazione del modulo contatti, da copiare in domains/rastinvoice.it/privato/
#                 (fuori dalla radice del sito: non è raggiungibile dal web)
#
# La configurazione si prende da sito/rastinvoice-config.php (escluso da git) se esiste,
# altrimenti si copia il modello config-esempio.php da compilare.
#
# Uso, dalla radice del repository (su Windows dalla Git Bash):
#   sito/prepara.sh
set -euo pipefail

sito="$(cd "$(dirname "$0")" && pwd)"
uscita="$sito/../target/sito"
rm -rf "$uscita" && mkdir -p "$uscita/public_html" "$uscita/privato"

# Tutto il sito tranne documentazione, modello e script di servizio. Solo cp e
# rm, senza rsync: lo script gira anche nella Git Bash di Windows.
cp -R "$sito/." "$uscita/public_html/"
(cd "$uscita/public_html" && rm -f README.md config-esempio.php rastinvoice-config.php prepara.sh)
find "$uscita/public_html" \( -name .DS_Store -o -name Thumbs.db -o -name desktop.ini \) -delete

# CSS incorporato nelle pagine: i due fogli di stile non bloccano più il rendering
# con richieste separate (consiglio di PageSpeed). Nei sorgenti restano file distinti,
# così il sito si può ancora aprire e modificare direttamente da sito/.
# I percorsi dei caratteri diventano assoluti, validi per ogni pagina (anche la 404).
pub="$uscita/public_html"
stile="$uscita/stile.css"
# Senza ritorni a capo di Windows (\r): il browser li normalizza e l'impronta non corrisponderebbe.
{ cat "$pub/css/bootstrap-italia.min.css"; printf '\n'; sed 's#url("\.\./fonts/#url("/fonts/#g' "$pub/css/sito.css"; } | tr -d '\r' > "$stile"
# L'impronta SHA-256 del blocco <style> va nella Content-Security-Policy (style-src),
# che altrimenti bloccherebbe gli stili incorporati.
impronta="sha256-$(openssl dgst -sha256 -binary "$stile" | openssl base64 -A)"
for pagina in "$pub"/*.html; do
  awk -v stile="$stile" '
    /<link rel="stylesheet" href="\/?css\/bootstrap-italia\.min\.css">/ {
      printf "  <style>"
      while ((getline riga < stile) > 0) print riga
      close(stile)
      print "</style>"
      next
    }
    /<link rel="stylesheet" href="\/?css\/sito\.css">/ { next }
    { print }
  ' "$pagina" > "$pagina.tmp" && mv "$pagina.tmp" "$pagina"
  if grep -q 'rel="stylesheet"' "$pagina"; then
    echo "Errore: fogli di stile non incorporati in $pagina" >&2; exit 1
  fi
done
sed -i.bak "s|style-src 'self'|style-src 'self' '$impronta'|" "$pub/.htaccess" && rm -f "$pub/.htaccess.bak"
grep -q "$impronta" "$pub/.htaccess" || { echo "Errore: impronta CSS non inserita in .htaccess" >&2; exit 1; }
rm -rf "$pub/css" "$stile"

if [ -f "$sito/rastinvoice-config.php" ]; then
  cp "$sito/rastinvoice-config.php" "$uscita/privato/"
  echo "Configurazione copiata da sito/rastinvoice-config.php"
else
  cp "$sito/config-esempio.php" "$uscita/privato/rastinvoice-config.php"
  echo "Configurazione di esempio in privato/rastinvoice-config.php: compilarla prima di caricarla"
  echo "(oppure non caricarla affatto per usare mail() di PHP)."
fi

uscita="$(cd "$uscita" && pwd)"
echo
echo "Pronte in $uscita:"
echo "  public_html/ -> domains/rastinvoice.it/public_html/"
echo "  privato/     -> domains/rastinvoice.it/privato/"
