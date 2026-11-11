#!/usr/bin/env bash
# Copyright (c) 2026 Andrea Re - info@digitare.net
# Autore: Andrea Re @ 2026 - info@digitare.net
#
# Pacchetto Snap compilato in locale con snapcraft, per provarlo e caricarlo
# sullo Snap Store. snapcraft compila in una macchina virtuale LXD con Ubuntu
# 22.04 e usa i file presenti in git (le modifiche non salvate in un commit
# non vengono incluse).
#
# Uso, dalla radice del repository:
#   store/linux/snap.sh             # crea target/snap/rastinvoice_<versione>_amd64.snap
#   store/linux/snap.sh --carica    # in più lo carica sullo Snap Store (canale stable)
#
# Prerequisiti (una volta sola, su LMDE/Debian):
#   sudo rm -f /etc/apt/preferences.d/nosnap.pref   # solo se presente (Mint blocca snapd)
#   sudo apt install snapd && sudo snap install core
#   sudo snap install snapcraft --classic
#   sudo snap install lxd && sudo lxd init --auto && sudo usermod -aG lxd "$USER"   # poi uscire e rientrare
#   snapcraft login   # solo per --carica

set -euo pipefail

radice="$(cd "$(dirname "$0")/../.." && pwd)"
cd "$radice"

carica=0
[ "${1:-}" = "--carica" ] && carica=1

command -v snapcraft >/dev/null || { echo "Manca snapcraft: vedi i prerequisiti in testa allo script." >&2; exit 1; }

versione="$(python3 -c "import json; print(json.load(open('apps/desktop/src-tauri/tauri.conf.json'))['version'])")"

if [ -n "$(git status --porcelain)" ]; then
  echo "Attenzione: ci sono modifiche non salvate in un commit; il pacchetto usa l'ultimo commit."
fi

# snapcraft cerca il file in snap/snapcraft.yaml: lo si copia con la versione
# corrente e lo si rimuove alla fine.
mkdir -p snap
trap 'rm -rf "$radice/snap"' EXIT
sed "s/^version: .*/version: '$versione'/" store/linux/snap/snapcraft.yaml > snap/snapcraft.yaml

echo "Compilazione Snap di RastInvoice $versione..."
if snapcraft pack --help >/dev/null 2>&1; then snapcraft pack; else snapcraft; fi

mkdir -p target/snap
pacchetto="target/snap/rastinvoice_${versione}_amd64.snap"
mv -f rastinvoice_"${versione}"_*.snap "$pacchetto"

echo
echo "Pacchetto: $pacchetto"
echo "Prova:     sudo snap install --dangerous $pacchetto"

if [ "$carica" = 1 ]; then
  snapcraft upload --release=stable "$pacchetto"
fi
