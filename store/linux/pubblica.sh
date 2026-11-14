#!/usr/bin/env bash
# Copyright (c) 2026 Andrea Re - info@digitare.net
# Autore: Andrea Re @ 2026 - info@digitare.net
#
# Carica su GitHub i pacchetti Linux creati da store/linux/build.sh, in una
# release in bozza da controllare e pubblicare dalla pagina Releases. Se la
# release esiste già (per esempio creata da store/windows/pubblica.ps1) aggiunge
# i file. L'eseguibile per Windows si carica dal PC Windows con
# store/windows/pubblica.ps1; macOS si distribuisce solo dal Mac App Store.
#
# La release va resa pubblica solo quando contiene tutti i pacchetti (Windows e
# Linux): i pulsanti del sito puntano all'ultima release pubblicata e un file
# mancante darebbe errore 404.
#
# Prerequisito: GitHub CLI (sudo apt install gh) e `gh auth login` una volta.
#
# Uso, dalla radice del repository:
#   store/linux/pubblica.sh            # crea la bozza vX.Y.Z o aggiorna i file se esiste già
#   store/linux/pubblica.sh --pubblica # ... e la rende pubblica subito
set -euo pipefail

radice="$(cd "$(dirname "$0")/../.." && pwd)"
cd "$radice"
repo="digitarenet/rastinvoice-fattura-elettronica"

command -v gh >/dev/null || { echo "Serve GitHub CLI: sudo apt install gh, poi gh auth login" >&2; exit 1; }
gh auth status >/dev/null 2>&1 || { echo "Eseguire prima: gh auth login" >&2; exit 1; }

versione="$(node -p "require('./apps/desktop/src-tauri/tauri.conf.json').version")"
tag="v$versione"

# Pacchetti con i nomi fissi creati da build.sh, solo se della versione corrente.
if [ "$(cat target/rilascio/.versione-linux 2>/dev/null)" != "$versione" ]; then
  echo "Nessun pacchetto della versione $versione: eseguire prima store/linux/build.sh" >&2
  exit 1
fi
file=(target/rilascio/RastInvoice-linux-amd64.deb
      target/rilascio/RastInvoice-linux-x86_64.rpm
      target/rilascio/RastInvoice-linux-x86_64.AppImage)
[ -f target/rilascio/latest.json ] && file+=(target/rilascio/latest.json)

# La release deve corrispondere al codice pubblicato su main.
git fetch -q origin main
if [ "$(git rev-parse HEAD)" != "$(git rev-parse origin/main)" ]; then
  echo "Attenzione: il codice locale non coincide con origin/main." >&2
  read -r -p "Continuare comunque? [s/N] " r
  [ "$r" = "s" ] || exit 1
fi

# Note della release: la sezione della versione nel registro delle modifiche.
note="$(mktemp)"
awk -v v="$versione" '
  $0 ~ "^## " v "( |$)" { dentro = 1; next }
  dentro && /^## / { exit }
  dentro { print }
' docs/CHANGELOG.md > "$note"
[ -s "$note" ] || echo "RastInvoice $versione" > "$note"
# Parte fissa: download e avviso di SmartScreen per Windows.
{ echo; grep -v '^<!--' store/note-release.md; } >> "$note"

if gh release view "$tag" -R "$repo" >/dev/null 2>&1; then
  echo "== Aggiorno i file della release $tag"
  gh release upload "$tag" "${file[@]}" --clobber -R "$repo"
else
  echo "== Creo la release in bozza $tag"
  gh release create "$tag" "${file[@]}" -R "$repo" --draft --target main \
    --title "RastInvoice $versione" --notes-file "$note"
fi
rm -f "$note"

if [ "${1:-}" = "--pubblica" ]; then
  # Il sito scarica da releases/latest/download: servono tutti i file.
  presenti="$(gh release view "$tag" -R "$repo" --json assets --jq '.assets[].name')"
  for f in RastInvoice-windows-x64.exe RastInvoice-linux-amd64.deb \
           RastInvoice-linux-x86_64.rpm RastInvoice-linux-x86_64.AppImage; do
    grep -qx "$f" <<< "$presenti" || {
      echo "Manca $f nella release $tag: resta in bozza (il sito darebbe errore 404)." >&2
      exit 1
    }
  done
  gh release edit "$tag" -R "$repo" --draft=false --latest
  echo "Release $tag pubblicata."
else
  echo "Bozza pronta: controllarla e pubblicarla da https://github.com/$repo/releases"
fi
