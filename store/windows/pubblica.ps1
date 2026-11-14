# Copyright (c) 2026 Andrea Re - info@digitare.net
# Autore: Andrea Re @ 2026 - info@digitare.net
#
# Carica su GitHub l'eseguibile per Windows creato da store/windows/build.ps1,
# nella release della versione corrente. Se la release non esiste ancora la
# crea in bozza, con le note prese da docs/CHANGELOG.md e store/note-release.md;
# se esiste (per esempio creata da store/linux/pubblica.sh) aggiunge il file.
#
# La release va resa pubblica solo quando contiene tutti i pacchetti (Windows e
# Linux): i pulsanti del sito puntano all'ultima release pubblicata e un file
# mancante darebbe errore 404.
#
# Prerequisito: GitHub CLI (winget install GitHub.cli) e "gh auth login" una volta.
#
# Uso, dalla radice del repository:
#   pwsh store/windows/pubblica.ps1             # carica l'exe nella release (bozza se nuova)
#   pwsh store/windows/pubblica.ps1 -Pubblica   # ... e rende pubblica la release

param([switch]$Pubblica)

$ErrorActionPreference = "Stop"
$radice = Resolve-Path (Join-Path $PSScriptRoot "../..")
Set-Location $radice
$repo = "digitarenet/rastinvoice-fattura-elettronica"
$file = "target/rilascio/RastInvoice-windows-x64.exe"

if (-not (Get-Command gh -ErrorAction SilentlyContinue)) {
    throw "Serve GitHub CLI: winget install GitHub.cli, poi gh auth login"
}
gh auth status *> $null
if ($LASTEXITCODE) { throw "Eseguire prima: gh auth login" }

$versione = (Get-Content "apps/desktop/src-tauri/tauri.conf.json" -Raw | ConvertFrom-Json).version
$tag = "v$versione"

$marcatore = "target/rilascio/.versione-windows"
if (-not (Test-Path $file) -or -not (Test-Path $marcatore) -or (Get-Content $marcatore -Raw).Trim() -ne $versione) {
    throw "Manca l'eseguibile della versione ${versione}: eseguire prima pwsh store/windows/build.ps1"
}

# La release deve corrispondere al codice pubblicato su main.
git fetch -q origin main
if ((git rev-parse HEAD) -ne (git rev-parse origin/main)) {
    Write-Warning "Il codice locale non coincide con origin/main."
    if ((Read-Host "Continuare comunque? [s/N]") -ne "s") { exit 1 }
}

gh release view $tag -R $repo *> $null
if ($LASTEXITCODE -eq 0) {
    Write-Host "== Aggiungo l'eseguibile alla release $tag" -ForegroundColor Cyan
    gh release upload $tag $file --clobber -R $repo
    if ($LASTEXITCODE) { throw "caricamento non riuscito" }
} else {
    Write-Host "== Creo la release in bozza $tag" -ForegroundColor Cyan
    # Note: la sezione della versione nel registro delle modifiche, poi i download.
    $note = New-TemporaryFile
    $righe = @()
    $dentro = $false
    foreach ($riga in Get-Content "docs/CHANGELOG.md" -Encoding UTF8) {
        if ($riga -match "^## ") {
            if ($dentro) { break }
            $dentro = $riga -match "^## $([regex]::Escape($versione))( |$)"
            continue
        }
        if ($dentro) { $righe += $riga }
    }
    if ($righe.Count -eq 0) { $righe = @("RastInvoice $versione") }
    $righe += ""
    $righe += Get-Content "store/note-release.md" -Encoding UTF8 | Where-Object { $_ -notmatch "^<!--" }
    Set-Content -Path $note.FullName -Value $righe -Encoding utf8NoBOM
    gh release create $tag $file -R $repo --draft --target main --title "RastInvoice $versione" --notes-file $note.FullName
    $esito = $LASTEXITCODE
    Remove-Item $note -Force
    if ($esito) { throw "creazione della release non riuscita" }
}

if ($Pubblica) {
    # Il sito scarica da releases/latest/download: servono tutti i file.
    $presenti = gh release view $tag -R $repo --json assets --jq ".assets[].name"
    foreach ($f in "RastInvoice-windows-x64.exe", "RastInvoice-linux-amd64.deb",
                   "RastInvoice-linux-x86_64.rpm", "RastInvoice-linux-x86_64.AppImage") {
        if ($presenti -notcontains $f) {
            throw "Manca $f nella release ${tag}: resta in bozza (il sito darebbe errore 404)."
        }
    }
    gh release edit $tag -R $repo --draft=false --latest
    if ($LASTEXITCODE) { throw "pubblicazione non riuscita" }
    Write-Host "Release $tag pubblicata." -ForegroundColor Green
} else {
    Write-Host "Bozza pronta: controllarla e pubblicarla da https://github.com/$repo/releases" -ForegroundColor Green
}
