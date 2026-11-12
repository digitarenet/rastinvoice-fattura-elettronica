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
