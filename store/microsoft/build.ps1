# Copyright (c) 2026 Andrea Re - info@digitare.net
# Autore: Andrea Re @ 2026 - info@digitare.net
#
# Compilazione su Windows e pacchetto MSIX per il Microsoft Store. Il pacchetto non va su GitHub: si carica
# in Partner Center.
#
# Prerequisiti (una volta sola):
#   - Rust (https://rustup.rs) con il toolchain MSVC;
#   - Node.js 24;
#   - Visual Studio Installer: "Sviluppo di applicazioni desktop con C++"
#     e "Windows 11 SDK" (contiene makeappx.exe).
#
# Identità del pacchetto: da store/configurazione.local.json (vedi
# store/configurazione.esempio.json) oppure dai parametri.
#
# Uso, dalla radice del repository:
#   pwsh store/microsoft/build.ps1            # compila e crea l'MSIX
#   pwsh store/microsoft/build.ps1 -Prova     # ... e lo installa per provarlo
#                                              (richiede la Modalità sviluppatore)

param(
    [string]$NomePacchetto,
    [string]$Editore,
    [string]$EditoreVisualizzato,
    [switch]$Prova
)

$ErrorActionPreference = "Stop"
$radice = Resolve-Path (Join-Path $PSScriptRoot "../..")
Set-Location $radice

function Richiedi($comando, $suggerimento) {
    if (-not (Get-Command $comando -ErrorAction SilentlyContinue)) {
        throw "Manca '$comando': $suggerimento"
    }
}
Richiedi "cargo" "installare Rust da https://rustup.rs"
Richiedi "node" "installare Node.js 24"
Richiedi "npm" "installare Node.js 24"

# Valori mancanti presi dal file di configurazione locale.
$config = Join-Path $radice "store/configurazione.local.json"
if (Test-Path $config) {
    $c = (Get-Content $config -Raw | ConvertFrom-Json).microsoft
    if (-not $NomePacchetto) { $NomePacchetto = $c.nomePacchetto }
    if (-not $Editore) { $Editore = $c.editore }
    if (-not $EditoreVisualizzato) { $EditoreVisualizzato = $c.editoreVisualizzato }
}
if (-not $NomePacchetto -or -not $Editore -or $Editore -eq "CN=..." -or -not $EditoreVisualizzato) {
    throw "Identità del pacchetto mancante: compilare store/configurazione.local.json (valori da Partner Center > Identità del prodotto)."
}

Write-Host "== Dipendenze dell'interfaccia" -ForegroundColor Cyan
Push-Location apps/desktop
npm ci
if ($LASTEXITCODE) { throw "npm ci non riuscito" }

Write-Host "== Compilazione (solo eseguibile: il pacchetto è l'MSIX)" -ForegroundColor Cyan
npx tauri build --no-bundle
if ($LASTEXITCODE) { throw "compilazione non riuscita" }
Pop-Location

Write-Host "== Pacchetto MSIX" -ForegroundColor Cyan
& (Join-Path $PSScriptRoot "msix.ps1") -NomePacchetto $NomePacchetto -Editore $Editore -EditoreVisualizzato $EditoreVisualizzato

if ($Prova) {
    Write-Host "== Installazione di prova" -ForegroundColor Cyan
    Get-AppxPackage -Name $NomePacchetto | Remove-AppxPackage -ErrorAction SilentlyContinue
    Add-AppxPackage -Register (Join-Path $radice "target/msix/layout/AppxManifest.xml")
    Write-Host "Installato: si trova nel menu Start come RastInvoice."
}

$msix = Get-ChildItem target/msix/*.msix | Sort-Object LastWriteTime | Select-Object -Last 1
Write-Host ""
Write-Host "Pronto: $($msix.FullName)" -ForegroundColor Green
Write-Host "Caricarlo in Partner Center > RastInvoice > Invii > Pacchetti."
