# Copyright (c) 2026 Andrea Re - info@digitare.net
# Autore: Andrea Re @ 2026 - info@digitare.net
#
# Crea il pacchetto MSIX per il Microsoft Store a partire dall'eseguibile
# compilato. Il pacchetto non viene firmato: lo firma il Microsoft Store al
# momento della pubblicazione.
#
# Uso (da Windows, con il Windows SDK installato):
#   pwsh scripts/msix.ps1 -NomePacchetto "<Package/Identity/Name>" `
#        -Editore "<Package/Identity/Publisher>" -EditoreVisualizzato "<nome>"
#
# I tre valori si leggono in Partner Center, nella pagina "Identità del
# prodotto" dell'app. Non vanno scritti nel repository.

param(
    [Parameter(Mandatory = $true)][string]$NomePacchetto,
    [Parameter(Mandatory = $true)][string]$Editore,
    [Parameter(Mandatory = $true)][string]$EditoreVisualizzato,
    [string]$NomeVisualizzato = "RastInvoice",
    [string]$Eseguibile = "target/release/rustinvoice-desktop.exe",
    [string]$Uscita = "target/msix"
)

$ErrorActionPreference = "Stop"
$radice = Resolve-Path (Join-Path $PSScriptRoot "..")
Set-Location $radice

# Versione dal file di configurazione di Tauri; il Microsoft Store vuole
# quattro numeri con l'ultimo uguale a zero.
$conf = Get-Content "apps/desktop/src-tauri/tauri.conf.json" -Raw | ConvertFrom-Json
$versione = "$($conf.version).0"

if (-not (Test-Path $Eseguibile)) {
    throw "Eseguibile non trovato: $Eseguibile. Compilare prima con 'npx tauri build'."
}

# Struttura del pacchetto.
$layout = Join-Path $Uscita "layout"
Remove-Item $layout -Recurse -Force -ErrorAction SilentlyContinue
New-Item -ItemType Directory -Force -Path (Join-Path $layout "Assets") | Out-Null
Copy-Item $Eseguibile (Join-Path $layout "RastInvoice.exe")
foreach ($logo in "Square44x44Logo", "Square150x150Logo", "Square310x310Logo", "StoreLogo") {
    Copy-Item "apps/desktop/src-tauri/icons/$logo.png" (Join-Path $layout "Assets/$logo.png")
}

# Manifesto con i valori di Partner Center (escape XML dei valori).
function Xml([string]$t) { [System.Security.SecurityElement]::Escape($t) }
$manifesto = Get-Content "apps/desktop/src-tauri/msix/AppxManifest.xml" -Raw
$manifesto = $manifesto.Replace("{{NOME_PACCHETTO}}", (Xml $NomePacchetto)).
    Replace("{{EDITORE}}", (Xml $Editore)).
    Replace("{{EDITORE_VISUALIZZATO}}", (Xml $EditoreVisualizzato)).
    Replace("{{NOME_VISUALIZZATO}}", (Xml $NomeVisualizzato)).
    Replace("{{VERSIONE}}", $versione)
Set-Content -Path (Join-Path $layout "AppxManifest.xml") -Value $manifesto -Encoding UTF8

# makeappx.exe dal Windows SDK più recente installato.
$makeappx = Get-ChildItem "${env:ProgramFiles(x86)}\Windows Kits\10\bin\*\x64\makeappx.exe" |
    Sort-Object FullName -Descending | Select-Object -First 1
if (-not $makeappx) { throw "makeappx.exe non trovato: installare il Windows SDK." }

$pacchetto = Join-Path $Uscita "RastInvoice_$($versione)_x64.msix"
& $makeappx.FullName pack /o /d $layout /p $pacchetto
if ($LASTEXITCODE -ne 0) { throw "makeappx non riuscito ($LASTEXITCODE)." }
Write-Host "Pacchetto creato: $pacchetto"
