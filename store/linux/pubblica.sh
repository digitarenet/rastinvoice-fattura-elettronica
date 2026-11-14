#!/usr/bin/env bash
# Copyright (c) 2026 Andrea Re - info@digitare.net
# Autore: Andrea Re @ 2026 - info@digitare.net
#
# Carica su GitHub i pacchetti Linux creati da store/linux/build.sh. Se la
# release non esiste la crea in bozza, con le note prese da docs/CHANGELOG.md e
# store/note-release.md; se esiste (per esempio creata da
# store/windows/pubblica.ps1) aggiunge i file.
#
# Pubblicando, i file che mancano nella release (per esempio l'eseguibile per
# Windows di una versione solo per Linux) vengono copiati dall'ultima release
# pubblicata con i nomi fissi: i pulsanti del sito, che puntano a
# releases/latest/download, continuano a funzionare.
#
# Prerequisito: GitHub CLI (sudo apt install gh) e `gh auth login` una volta.
#
# Uso, dalla radice del repository:
#   store/linux/pubblica.sh                      # release della versione corrente, in bozza
#   store/linux/pubblica.sh --pubblica           # ... e la pubblica
#   store/linux/pubblica.sh --versione 1.3.1     # release scelta (v1.3.1)
set -euo pipefail

radice="$(cd "$(dirname "$0")/../.." && pwd)"
cd "$radice"
repo="digitarenet/rastinvoice-fattura-elettronica"
# Nomi fissi dei file che il sito scarica.
nomi_fissi=(RastInvoice-windows-x64.exe RastInvoice-linux-amd64.deb
            RastInvoice-linux-x86_64.rpm RastInvoice-linux-x86_64.AppImage)

pubblica=0
versione=""
while [ $# -gt 0 ]; do
  case "$1" in
    --pubblica) pubblica=1 ;;
    --versione) versione="${2#v}"; shift ;;
    *) echo "Opzione sconosciuta: $1" >&2; exit 1 ;;
  esac
  shift
done

command -v gh >/dev/null || { echo "Serve GitHub CLI: sudo apt install gh, poi gh auth login" >&2; exit 1; }
gh auth status >/dev/null 2>&1 || { echo "Eseguire prima: gh auth login" >&2; exit 1; }

compilata="$(cat target/rilascio/.versione-linux 2>/dev/null || true)"
[ -n "$compilata" ] || { echo "Nessun pacchetto: eseguire prima store/linux/build.sh" >&2; exit 1; }
[ -n "$versione" ] || versione="$(node -p "require('./apps/desktop/src-tauri/tauri.conf.json').version")"
tag="v$versione"
if [ "$compilata" != "$versione" ]; then
  echo "Attenzione: i pacchetti sono della versione $compilata, la release è $tag." >&2
  read -r -p "Caricarli comunque? [s/N] " r
  [ "$r" = "s" ] || exit 1
fi

file=(target/rilascio/RastInvoice-linux-amd64.deb
      target/rilascio/RastInvoice-linux-x86_64.rpm
      target/rilascio/RastInvoice-linux-x86_64.AppImage)
lavoro="$(mktemp -d)"
trap 'rm -rf "$lavoro"' EXIT
if [ -f target/rilascio/latest.json ]; then
  # Gli indirizzi di latest.json devono puntare alla release in cui vanno i file.
  sed "s#/releases/download/v$compilata/#/releases/download/$tag/#g" target/rilascio/latest.json > "$lavoro/latest.json"
  file+=("$lavoro/latest.json")
fi

# La release deve corrispondere al codice pubblicato su main.
git fetch -q origin main
if [ "$(git rev-parse HEAD)" != "$(git rev-parse origin/main)" ]; then
  echo "Attenzione: il codice locale non coincide con origin/main." >&2
  read -r -p "Continuare comunque? [s/N] " r
  [ "$r" = "s" ] || exit 1
fi

if gh release view "$tag" -R "$repo" >/dev/null 2>&1; then
  echo "== Aggiungo i pacchetti Linux alla release $tag"
  gh release upload "$tag" "${file[@]}" --clobber -R "$repo"
else
  echo "== Creo la release in bozza $tag"
  # Note: la sezione della versione nel registro delle modifiche, poi i download.
  awk -v v="$versione" '
    $0 ~ "^## " v "( |$)" { dentro = 1; next }
    dentro && /^## / { exit }
    dentro { print }
  ' docs/CHANGELOG.md > "$lavoro/note.md"
  [ -s "$lavoro/note.md" ] || echo "RastInvoice $versione" > "$lavoro/note.md"
  { echo; grep -v '^<!--' store/note-release.md; } >> "$lavoro/note.md"
  gh release create "$tag" "${file[@]}" -R "$repo" --draft --target main \
    --title "RastInvoice $versione" --notes-file "$lavoro/note.md"
fi

if [ "$pubblica" = 0 ]; then
  echo "Bozza pronta. Per pubblicarla: store/linux/pubblica.sh --versione $versione --pubblica"
  exit 0
fi

mancanti_in() {
  local presenti
  presenti="$(gh release view "$tag" -R "$repo" --json assets --jq '.assets[].name')"
  for f in "${nomi_fissi[@]}"; do grep -qx "$f" <<< "$presenti" || echo "$f"; done
}
mapfile -t mancanti < <(mancanti_in)

# Una release più vecchia dell'ultima non diventa "latest" e non tocca il sito.
precedente="$(gh release view -R "$repo" --json tagName --jq '.tagName' 2>/dev/null || true)"
recente=1
if [ -n "$precedente" ] && [ "$(printf '%s\n%s\n' "${precedente#v}" "$versione" | sort -V | tail -1)" != "$versione" ]; then
  recente=0
  mancanti=()
fi

# File mancanti copiati dall'ultima release pubblicata, con i nomi fissi.
if [ "${#mancanti[@]}" -gt 0 ] && [ -n "$precedente" ] && [ "$precedente" != "$tag" ]; then
  echo "== Copio da $precedente: ${mancanti[*]}"
  mkdir -p "$lavoro/copia"
  modelli=()
  for m in "${mancanti[@]}"; do modelli+=(-p "*.${m##*.}"); done
  gh release download "$precedente" -R "$repo" -D "$lavoro/copia" "${modelli[@]}" || true
  da_caricare=()
  for m in "${mancanti[@]}"; do
    for f in "$lavoro/copia"/*."${m##*.}"; do
      [ -f "$f" ] || continue
      [ "$f" = "$lavoro/copia/$m" ] || mv "$f" "$lavoro/copia/$m"
      da_caricare+=("$lavoro/copia/$m")
      break
    done
  done
  [ "${#da_caricare[@]}" -eq 0 ] || gh release upload "$tag" "${da_caricare[@]}" --clobber -R "$repo"
  mapfile -t mancanti < <(mancanti_in)
fi
if [ "${#mancanti[@]}" -gt 0 ]; then
  echo "Mancano ancora: ${mancanti[*]}. I relativi pulsanti del sito darebbero errore 404." >&2
  read -r -p "Pubblicare comunque? [s/N] " r
  [ "$r" = "s" ] || exit 1
fi

if [ "$recente" = 1 ]; then
  gh release edit "$tag" -R "$repo" --draft=false --latest
else
  gh release edit "$tag" -R "$repo" --draft=false --latest=false
fi
echo "Release $tag pubblicata."
