# Copyright (c) 2026 Andrea Re - info@digitare.net
# Autore: Andrea Re @ 2026 - info@digitare.net
#
# Eseguibile per Windows da pubblicare su GitHub e scaricare dal sito, in
# attesa della versione del Microsoft Store. È un solo file .exe che non
# richiede installazione: al primo avvio il programma chiede se salvare i dati
# nel profilo utente o accanto all'eseguibile (modalità portable).
#
# Il file non ha firma digitale: al primo avvio Windows SmartScreen mostra
# l'avviso "Windows ha protetto il PC" (note della release in
# store/note-release.md).
#
# Prerequisiti (una volta sola): Rust con il toolchain MSVC, Node.js 24 e
# Visual Studio o Build Tools con "Sviluppo di applicazioni desktop con C++"
# (vedi store/README.md). Il Windows SDK serve solo per l'MSIX.
#
# Uso, dalla radice del repository:
#   pwsh store/windows/build.ps1        # crea target/rilascio/RastInvoice-windows-x64.exe
#   pwsh store/windows/pubblica.ps1     # poi lo carica sulla release di GitHub

$ErrorActionPreference = "Stop"
$radice = Resolve-Path (Join-Path $PSScriptRoot "../..")
Set-Location $radice

# Nome fisso del file: il sito punta a releases/latest/download/<nome>, che
# deve restare uguale a ogni versione.
$nomeFile = "RastInvoice-windows-x64.exe"

foreach ($comando in "cargo", "node", "npm") {
    if (-not (Get-Command $comando -ErrorAction SilentlyContinue)) {
        throw "Manca '$comando': vedi store/README.md, sezione Compilare in locale."
    }
}

$versione = (Get-Content "apps/desktop/src-tauri/tauri.conf.json" -Raw | ConvertFrom-Json).version

Write-Host "== RastInvoice ${versione}: dipendenze dell'interfaccia" -ForegroundColor Cyan
Push-Location apps/desktop
npm ci --no-audit --no-fund
if ($LASTEXITCODE) { throw "npm ci non riuscito" }

Write-Host "== Compilazione dell'eseguibile" -ForegroundColor Cyan
npx tauri build --no-bundle
if ($LASTEXITCODE) { throw "compilazione non riuscita" }
Pop-Location

# Tauri può rinominare l'eseguibile con il nome del prodotto: si prova anche quello.
$eseguibile = "target/release/rustinvoice-desktop.exe"
if (-not (Test-Path $eseguibile) -and (Test-Path "target/release/RastInvoice.exe")) {
    $eseguibile = "target/release/RastInvoice.exe"
}
if (-not (Test-Path $eseguibile)) { throw "Eseguibile non trovato in target/release." }

$uscita = "target/rilascio"
New-Item -ItemType Directory -Force -Path $uscita | Out-Null
$destinazione = Join-Path $uscita $nomeFile
Copy-Item $eseguibile $destinazione -Force
# Versione del file, controllata da pubblica.ps1 prima del caricamento.
Set-Content -Path (Join-Path $uscita ".versione-windows") -Value $versione -NoNewline -Encoding ascii

$hash = (Get-FileHash $destinazione -Algorithm SHA256).Hash.ToLower()
Write-Host ""
Write-Host "Pronto: $((Resolve-Path $destinazione).Path)" -ForegroundColor Green
Write-Host "SHA-256: $hash"
Write-Host "Per caricarlo su GitHub: pwsh store/windows/pubblica.ps1"
