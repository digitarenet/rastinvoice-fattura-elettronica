// Copyright (c) 2026 Andrea Re - info@digitare.net
// Autore: Andrea Re @ 2026 - info@digitare.net
//
// Crea latest.json, il file che l'applicazione legge per l'aggiornamento
// integrato dei pacchetti Linux: per ogni pacchetto (AppImage, .deb, .rpm)
// indica l'indirizzo di download nella release e la firma prodotta da
// `tauri build` con la chiave privata di rilascio. Legge i pacchetti già
// copiati da store/linux/build.sh in target/rilascio con i nomi fissi.
//
// Uso: node store/linux/manifesto-aggiornamenti.mjs <versione> <url-base-release> > latest.json

import { readdirSync, readFileSync } from "node:fs";
import { join } from "node:path";

const [versione, base] = process.argv.slice(2);
if (!versione || !base) {
  console.error("uso: manifesto-aggiornamenti.mjs <versione> <url-base-release>");
  process.exit(1);
}

const cartella = "target/rilascio";
// Tipo di pacchetto -> estensione del file.
const tipi = { appimage: ".AppImage", deb: ".deb", rpm: ".rpm" };
const piattaforme = {};

for (const [tipo, estensione] of Object.entries(tipi)) {
  const file = readdirSync(cartella).filter(
    (f) => f.startsWith("RastInvoice-linux-") && f.endsWith(estensione),
  );
  if (file.length === 0) continue;
  if (file.length !== 1) {
    console.error(`atteso un solo pacchetto ${estensione} in ${cartella}, trovati ${file.length}`);
    process.exit(1);
  }
  const nome = file[0];
  const firma = readFileSync(join(cartella, `${nome}.sig`), "utf8").trim();
  piattaforme[`linux-x86_64-${tipo}`] = {
    url: `${base}/${encodeURIComponent(nome)}`,
    signature: firma,
  };
}

if (Object.keys(piattaforme).length === 0) {
  console.error("nessun pacchetto firmato trovato");
  process.exit(1);
}

const manifesto = {
  version: versione,
  notes: `RastInvoice ${versione}`,
  pub_date: new Date().toISOString().replace(/\.\d{3}Z$/, "Z"),
  platforms: piattaforme,
};
console.log(JSON.stringify(manifesto, null, 2));
