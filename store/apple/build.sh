#!/usr/bin/env bash
# Copyright (c) 2026 Andrea Re - info@digitare.net
# Autore: Andrea Re @ 2026 - info@digitare.net
#
# Compilazione su Mac dell'app universale (Apple Silicon e Intel) e, se sono
# configurati i certificati, del pacchetto firmato per il Mac App Store.
# L'app non va su GitHub: si carica in App Store Connect con Transporter.
#
# Prerequisiti (una volta sola):
#   xcode-select --install          # strumenti di compilazione di Apple
#   Rust (https://rustup.rs) e Node.js 24 (brew install node@24)
#   Per il pacchetto: certificati "Apple Distribution" e "Mac Installer
#   Distribution" (nel Portachiavi appare come "3rd Party Mac Developer
#   Installer") e il profilo di
#   provisioning Mac App Store (vedi store/apple/README.md), indicati in
#   store/configurazione.local.json (sezione "apple").
#
# Uso, dalla radice del repository:
#   store/apple/build.sh            # app universale (+ pacchetto se configurato)
set -euo pipefail

radice="$(cd "$(dirname "$0")/../.." && pwd)"
cd "$radice"

config() {
  [ -f store/configurazione.local.json ] || return 0
  node -e '
    const c = require(process.argv[1]);
    const v = process.argv[2].split(".").reduce((o, k) => (o || {})[k], c);
    process.stdout.write(v ? String(v) : "");
  ' "$radice/store/configurazione.local.json" "$1"
}

for c in cargo rustc node npm xcrun; do
  command -v "$c" >/dev/null || { echo "Manca $c: vedi i prerequisiti in testa a questo script" >&2; exit 1; }
done

versione="$(node -p "require('./apps/desktop/src-tauri/tauri.conf.json').version")"
echo "== RastInvoice $versione per macOS"

# L'app universale richiede le librerie Rust per entrambe le architetture:
# con rustup si aggiungono da sole, con il Rust di Homebrew c'è solo quella
# del Mac in uso e si compila l'app per la sola architettura locale.
sysroot="$(rustc --print sysroot)"
if command -v rustup >/dev/null; then
  rustup target add aarch64-apple-darwin x86_64-apple-darwin >/dev/null
fi
if [ -d "$sysroot/lib/rustlib/aarch64-apple-darwin" ] && [ -d "$sysroot/lib/rustlib/x86_64-apple-darwin" ]; then
  universale=1
  bersaglio=(--target universal-apple-darwin)
  app="target/universal-apple-darwin/release/bundle/macos/RastInvoice.app"
else
  universale=0
  bersaglio=()
  app="target/release/bundle/macos/RastInvoice.app"
  echo "Attenzione: Rust senza i target aarch64/x86_64-apple-darwin (manca rustup)."
  echo "Compilo solo per $(uname -m). Per l'app universale installare rustup (https://rustup.rs)."
fi

(cd apps/desktop && npm ci --no-audit --no-fund)
(cd apps/desktop && npx tauri build ${bersaglio[@]+"${bersaglio[@]}"} --bundles app)

[ -d "$app" ] || { echo "App non trovata: $app" >&2; exit 1; }
echo "App pronta: $app"

team="$(config apple.teamId)"
id_app="$(config apple.identitaApp)"
id_inst="$(config apple.identitaInstaller)"
profilo="$(config apple.profilo)"; profilo="${profilo/#\~/$HOME}"
if [ "$universale" = 0 ]; then
  echo
  echo "App non universale: il pacchetto per il Mac App Store non viene creato."
  echo "Per provarla: open \"$app\""
  exit 0
fi
if [ -z "$team" ] || [ -z "$id_app" ] || [ -z "$id_inst" ] || [ ! -f "$profilo" ]; then
  # Copia firmata in locale (senza certificato) con la sandbox attiva, per
  # provare l'app come girerà dal Mac App Store: accesso solo ai file scelti
  # dall'utente e cartelle d'archivio ritrovate dopo il riavvio.
  prova="target/mac-app-store/prova-sandbox"
  rm -rf "$prova" && mkdir -p "$prova"
  cp -R "$app" "$prova/"
  cp store/apple/entitlements.plist "$prova/entitlements.plist"
  /usr/libexec/PlistBuddy -c "Delete :com.apple.application-identifier" "$prova/entitlements.plist"
  /usr/libexec/PlistBuddy -c "Delete :com.apple.developer.team-identifier" "$prova/entitlements.plist"
  codesign --force --sign - --entitlements "$prova/entitlements.plist" "$prova/RastInvoice.app"
  echo
  echo "Certificati Mac App Store non configurati: creata solo l'app, non il pacchetto."
  echo "Per provarla: open \"$app\""
  echo "Con la sandbox del Mac App Store: open \"$prova/RastInvoice.app\""
  exit 0
fi

echo "== Firma per il Mac App Store"
uscita="target/mac-app-store"
rm -rf "$uscita" && mkdir -p "$uscita"
cp -R "$app" "$uscita/"
app_mas="$uscita/RastInvoice.app"
cp "$profilo" "$app_mas/Contents/embedded.provisionprofile"
sed "s/TEAMID/$team/g" store/apple/entitlements.plist > "$uscita/entitlements.plist"
codesign --force --timestamp --sign "$id_app" --entitlements "$uscita/entitlements.plist" "$app_mas"
codesign --verify --strict --verbose=2 "$app_mas"

echo "== Pacchetto per App Store Connect"
pkg="$uscita/RastInvoice_${versione}_universale.pkg"
productbuild --component "$app_mas" /Applications --sign "$id_inst" "$pkg"

echo
echo "Pacchetto pronto: $pkg"
echo "Caricarlo con l'app Transporter (Mac App Store) usando l'Apple ID dello sviluppatore,"
echo "poi in App Store Connect sceglierlo nella nuova versione e inviarla in revisione."
