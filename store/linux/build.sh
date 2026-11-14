#!/usr/bin/env bash
# Copyright (c) 2026 Andrea Re - info@digitare.net
# Autore: Andrea Re @ 2026 - info@digitare.net
#
# Pacchetti Linux (.deb, .rpm, AppImage) compilati in locale. Per compatibilità con le distribuzioni più diffuse
# la compilazione avviene in un container Debian 12 (Docker o Podman).
#
# Uso, dalla radice del repository:
#   store/linux/build.sh              # nel container Debian 12 (consigliato)
#   store/linux/build.sh --locale     # direttamente su questo computer: i pacchetti
#                                     # funzionano solo su distribuzioni recenti quanto questa
#
# Aggiornamento integrato: se store/configurazione.local.json indica la chiave
# privata (aggiornamenti.chiavePrivata) e la chiave pubblica
# (aggiornamenti.chiavePubblica), i pacchetti vengono firmati e si crea anche
# latest.json. I file da pubblicare finiscono in target/rilascio con nomi fissi
# (senza versione), a cui puntano i pulsanti del sito tramite
# releases/latest/download/<nome>. Poi: store/linux/pubblica.sh
set -euo pipefail

radice="$(cd "$(dirname "$0")/../.." && pwd)"
cd "$radice"

# Legge un valore dal file di configurazione locale (vuoto se manca).
config() {
  [ -f store/configurazione.local.json ] || return 0
  node -e '
    const c = require(process.argv[1]);
    const v = process.argv[2].split(".").reduce((o, k) => (o || {})[k], c);
    process.stdout.write(v ? String(v) : "");
  ' "$radice/store/configurazione.local.json" "$1"
}

if [ "${1:-}" != "--locale" ] && [ "${DENTRO_CONTAINER:-}" != "1" ]; then
  if command -v podman >/dev/null; then motore=podman; utente=(--userns=keep-id)
  elif command -v docker >/dev/null; then motore=docker; utente=(--user "$(id -u):$(id -g)")
  else
    echo "Serve Docker o Podman (sudo apt install podman), oppure usare --locale." >&2
    exit 1
  fi
  echo "== Ambiente Debian 12 ($motore)"
  "$motore" build -q -t rastinvoice-build -f store/linux/Containerfile store/linux >/dev/null

  # Chiave privata letta qui e passata al container come variabile.
  chiave_file="$(config aggiornamenti.chiavePrivata)"
  chiave_file="${chiave_file/#\~/$HOME}"
  chiave=""
  if [ -n "$chiave_file" ] && [ -f "$chiave_file" ]; then chiave="$(cat "$chiave_file")"; fi

  exec "$motore" run --rm -it "${utente[@]}" \
    -v "$radice:/src" -w /src \
    -e HOME=/tmp -e CARGO_HOME=/src/target/cargo-debian12 -e DENTRO_CONTAINER=1 \
    -e APPIMAGE_EXTRACT_AND_RUN=1 \
    -e TAURI_SIGNING_PRIVATE_KEY="$chiave" \
    -e TAURI_SIGNING_PRIVATE_KEY_PASSWORD="$(config aggiornamenti.passwordChiave)" \
    -e RASTINVOICE_CHIAVE_AGGIORNAMENTI="$(config aggiornamenti.chiavePubblica)" \
    rastinvoice-build store/linux/build.sh --dentro
fi

# --- Compilazione (nel container o su questo computer) -------------------
if [ "${DENTRO_CONTAINER:-}" != "1" ]; then
  for c in cargo node npm; do
    command -v "$c" >/dev/null || { echo "Manca $c: vedi store/README.md" >&2; exit 1; }
  done
  chiave_file="$(config aggiornamenti.chiavePrivata)"
  chiave_file="${chiave_file/#\~/$HOME}"
  if [ -n "$chiave_file" ] && [ -f "$chiave_file" ]; then
    export TAURI_SIGNING_PRIVATE_KEY="$(cat "$chiave_file")"
    export TAURI_SIGNING_PRIVATE_KEY_PASSWORD="$(config aggiornamenti.passwordChiave)"
    export RASTINVOICE_CHIAVE_AGGIORNAMENTI="$(config aggiornamenti.chiavePubblica)"
  fi
fi

versione="$(node -p "require('./apps/desktop/src-tauri/tauri.conf.json').version")"
echo "== RastInvoice $versione: dipendenze dell'interfaccia"
(cd apps/desktop && npm ci --no-audit --no-fund)

rm -rf target/release/bundle/{deb,rpm,appimage}
rm -f target/rilascio/RastInvoice-linux-* target/rilascio/latest.json target/rilascio/.versione-linux
mkdir -p target/rilascio
cd apps/desktop
if [ -n "${TAURI_SIGNING_PRIVATE_KEY:-}" ] && [ -n "${RASTINVOICE_CHIAVE_AGGIORNAMENTI:-}" ]; then
  echo "== Compilazione con firma per l'aggiornamento integrato"
  # La chiave pubblica va anche nella configurazione: la CLI di Tauri la usa per
  # verificare le firme appena create e si ferma se il campo pubkey è vuoto.
  npx tauri build --bundles deb,rpm,appimage --config \
    "{\"bundle\":{\"createUpdaterArtifacts\":true},\"plugins\":{\"updater\":{\"pubkey\":\"$RASTINVOICE_CHIAVE_AGGIORNAMENTI\"}}}"
  cd "$radice"
  firmati=1
else
  echo "== Compilazione (senza chiavi: niente aggiornamento integrato)"
  unset TAURI_SIGNING_PRIVATE_KEY RASTINVOICE_CHIAVE_AGGIORNAMENTI
  npx tauri build --bundles deb,rpm,appimage
  cd "$radice"
  firmati=0
fi

# Copia con i nomi fissi usati dal sito e da latest.json (insieme alle firme).
copia() {
  local origine=("$1"/*"$2")
  [ "${#origine[@]}" -eq 1 ] && [ -f "${origine[0]}" ] || { echo "Atteso un solo file $2 in $1" >&2; exit 1; }
  cp "${origine[0]}" "target/rilascio/$3"
  [ ! -f "${origine[0]}.sig" ] || cp "${origine[0]}.sig" "target/rilascio/$3.sig"
}
copia target/release/bundle/deb .deb RastInvoice-linux-amd64.deb
copia target/release/bundle/rpm .rpm RastInvoice-linux-x86_64.rpm
copia target/release/bundle/appimage .AppImage RastInvoice-linux-x86_64.AppImage
printf '%s' "$versione" > target/rilascio/.versione-linux

if [ "$firmati" = 1 ]; then
  node store/linux/manifesto-aggiornamenti.mjs "$versione" \
    "https://github.com/digitarenet/rastinvoice-fattura-elettronica/releases/download/v$versione" \
    > target/rilascio/latest.json
fi

echo
echo "Pacchetti pronti:"
ls -1 target/rilascio/RastInvoice-linux-* target/rilascio/latest.json 2>/dev/null | grep -v '\.sig$' || true
echo
echo "Per pubblicarli su GitHub: store/linux/pubblica.sh"
