#!/usr/bin/env bash
# Copyright (c) 2026 Andrea Re - info@digitare.net
# Autore: Andrea Re @ 2026 - info@digitare.net
#
# Pacchetto Flatpak compilato in locale, per provarlo e per preparare l'invio
# a Flathub. Compila dai sorgenti della cartella di lavoro senza accesso alla
# rete, come fa Flathub.
#
# Uso, dalla radice del repository:
#   store/linux/flatpak.sh              # genera le dipendenze, compila, installa
#                                       # per l'utente e crea il file .flatpak
#   store/linux/flatpak.sh --flathub    # in più prepara in target/flatpak/flathub
#                                       # i file da proporre a Flathub
#
# Prerequisiti (una volta sola, su LMDE/Debian):
#   sudo apt install flatpak flatpak-builder python3-venv git
#   flatpak remote-add --user --if-not-exists flathub https://dl.flathub.org/repo/flathub.flatpakrepo

set -euo pipefail

radice="$(cd "$(dirname "$0")/../.." && pwd)"
cd "$radice"

flathub=0
[ "${1:-}" = "--flathub" ] && flathub=1

for comando in flatpak flatpak-builder python3 git; do
  command -v "$comando" >/dev/null || { echo "Manca $comando: vedi i prerequisiti in testa allo script." >&2; exit 1; }
done

id=net.digitare.rastinvoice
versione="$(python3 -c "import json; print(json.load(open('apps/desktop/src-tauri/tauri.conf.json'))['version'])")"
lavoro="$radice/target/flatpak"
mkdir -p "$lavoro"

# Strumenti di flatpak-builder-tools in un ambiente Python separato.
strumenti="$lavoro/flatpak-builder-tools"
if [ ! -d "$strumenti" ]; then
  git clone --depth 1 https://github.com/flatpak/flatpak-builder-tools.git "$strumenti"
fi
if [ ! -x "$lavoro/venv/bin/python" ]; then
  python3 -m venv "$lavoro/venv"
  "$lavoro/venv/bin/pip" install --quiet aiohttp tomlkit "$strumenti/node"
fi

# Elenchi delle dipendenze Rust e npm: flatpak-builder le scarica prima della
# compilazione, che poi avviene senza rete. Restano in target/flatpak, non in git.
echo "Generazione degli elenchi delle dipendenze..."
"$lavoro/venv/bin/python" "$strumenti/cargo/flatpak-cargo-generator.py" Cargo.lock \
  -o "$lavoro/cargo-sources.json"
"$lavoro/venv/bin/flatpak-node-generator" npm apps/desktop/package-lock.json \
  -o "$lavoro/node-sources.json"

# Manifesto per la prova locale: stessi passi di quello per Flathub, ma con i
# sorgenti presi dalla cartella di lavoro invece che dal tag su GitHub.
python3 - "$radice" "$lavoro/$id.yml" <<'PY'
import re, sys
radice, uscita = sys.argv[1], sys.argv[2]
testo = open(f"{radice}/store/linux/flatpak/net.digitare.rastinvoice.yml", encoding="utf-8").read()
locale = (
    "      - type: dir\n"
    f"        path: {radice}\n"
    "        skip: [target, .git, apps/desktop/node_modules, apps/desktop/dist, dati]\n"
)
testo, n = re.subn(r"      - type: git\n(        .*\n)+", locale, testo, count=1)
if n != 1:
    sys.exit("Sorgente git non trovata nel manifesto")
testo = testo.replace("      - cargo-sources.json", f"      - {radice}/target/flatpak/cargo-sources.json")
testo = testo.replace("      - node-sources.json", f"      - {radice}/target/flatpak/node-sources.json")
open(uscita, "w", encoding="utf-8").write(testo)
PY

echo "Compilazione Flatpak di RastInvoice $versione (la prima volta scarica i runtime GNOME)..."
# Senza rofiles-fuse: su alcuni sistemi il suo smontaggio fallisce a compilazione finita.
flatpak-builder --user --install --force-clean --install-deps-from=flathub --disable-rofiles-fuse \
  --state-dir="$lavoro/stato" --repo="$lavoro/repo" "$lavoro/build" "$lavoro/$id.yml"

pacchetto="$lavoro/RastInvoice_${versione}.flatpak"
flatpak build-bundle "$lavoro/repo" "$pacchetto" "$id"

if [ "$flathub" = 1 ]; then
  invio="$lavoro/flathub"
  rm -rf "$invio" && mkdir -p "$invio"
  cp "$lavoro/cargo-sources.json" "$lavoro/node-sources.json" "$invio/"
  commit="$(git rev-list -n 1 "v$versione" 2>/dev/null || true)"
  python3 - "$versione" "$commit" store/linux/flatpak/$id.yml "$invio/$id.yml" <<'PY'
import re, sys
versione, commit, sorgente, uscita = sys.argv[1:5]
testo = open(sorgente, encoding="utf-8").read()
riga = f"        tag: v{versione}\n" + (f"        commit: {commit}\n" if commit else "")
testo = re.sub(r"        tag: .*\n(        commit: .*\n)?", riga, testo, count=1)
open(uscita, "w", encoding="utf-8").write(testo)
PY
  [ -n "$commit" ] || echo "Attenzione: il tag v$versione non esiste ancora; pubblicare la release prima dell'invio a Flathub."
  grep -q "release version=\"$versione\"" store/linux/$id.metainfo.xml \
    || echo "Attenzione: in $id.metainfo.xml manca la voce <release version=\"$versione\">, richiesta da Flathub."
  echo "File per la pull request a Flathub: $invio"
fi

echo
echo "Installato per l'utente: flatpak run $id"
echo "Pacchetto: $pacchetto"
echo "  (si installa su un altro PC con: flatpak install --user $pacchetto)"
