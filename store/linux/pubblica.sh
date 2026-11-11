#!/usr/bin/env bash
# Copyright (c) 2026 Andrea Re - info@digitare.net
# Autore: Andrea Re @ 2026 - info@digitare.net
#
# Carica su GitHub i pacchetti Linux creati da store/linux/build.sh, in una
# release in bozza da controllare e pubblicare dalla pagina Releases.
# Su GitHub vanno solo i pacchetti Linux: Windows e macOS si distribuiscono
# dagli store.
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

shopt -s nullglob
file=(target/release/bundle/deb/*_"$versione"_*.deb
      target/release/bundle/rpm/*-"$versione"-*.rpm
      target/release/bundle/appimage/*_"$versione"_*.AppImage)
[ -f target/release/bundle/latest.json ] && file+=(target/release/bundle/latest.json)
if [ "${#file[@]}" -eq 0 ]; then
  echo "Nessun pacchetto della versione $versione: eseguire prima store/linux/build.sh" >&2
  exit 1
fi

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
  gh release edit "$tag" -R "$repo" --draft=false --latest
  echo "Release $tag pubblicata."
else
  echo "Bozza pronta: controllarla e pubblicarla da https://github.com/$repo/releases"
fi
