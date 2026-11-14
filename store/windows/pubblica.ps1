# Copyright (c) 2026 Andrea Re - info@digitare.net
# Autore: Andrea Re @ 2026 - info@digitare.net
#
# Carica su GitHub l'eseguibile per Windows creato da store/windows/build.ps1.
# Se la release non esiste la crea in bozza, con le note prese da
# docs/CHANGELOG.md e store/note-release.md; se esiste aggiunge il file.
#
# Pubblicando, i file che mancano nella release (per esempio i pacchetti Linux
# di una versione solo per Windows) vengono copiati dall'ultima release
# pubblicata, con i nomi fissi e il suo latest.json: i pulsanti del sito, che
# puntano a releases/latest/download, continuano a funzionare.
#
# Prerequisito: GitHub CLI (winget install GitHub.cli) e "gh auth login" una volta.
#
# Uso, dalla radice del repository:
#   pwsh store/windows/pubblica.ps1                       # release della versione corrente, in bozza
#   pwsh store/windows/pubblica.ps1 -Pubblica             # ... e la pubblica
#   pwsh store/windows/pubblica.ps1 -Versione 1.3.1       # release scelta (v1.3.1)

param([switch]$Pubblica, [string]$Versione, [switch]$CopiaLinux)
# -CopiaLinux resta accettato per compatibilità: ora la copia è automatica.

$ErrorActionPreference = "Stop"
$radice = Resolve-Path (Join-Path $PSScriptRoot "../..")
Set-Location $radice
$repo = "digitarenet/rastinvoice-fattura-elettronica"
$file = "target/rilascio/RastInvoice-windows-x64.exe"
# Nomi fissi dei file che il sito scarica, per estensione.
$nomiFissi = [ordered]@{
    ".exe" = "RastInvoice-windows-x64.exe"
    ".deb" = "RastInvoice-linux-amd64.deb"
    ".rpm" = "RastInvoice-linux-x86_64.rpm"
    ".AppImage" = "RastInvoice-linux-x86_64.AppImage"
}

if (-not (Get-Command gh -ErrorAction SilentlyContinue)) {
    throw "Serve GitHub CLI: winget install GitHub.cli, poi gh auth login"
}
gh auth status *> $null
if ($LASTEXITCODE) { throw "Eseguire prima: gh auth login" }

$compilata = (Get-Content "apps/desktop/src-tauri/tauri.conf.json" -Raw | ConvertFrom-Json).version
if (-not $Versione) { $Versione = $compilata }
$Versione = $Versione.TrimStart("v")
$tag = "v$Versione"

$marcatore = "target/rilascio/.versione-windows"
if (-not (Test-Path $file) -or -not (Test-Path $marcatore)) {
    throw "Manca l'eseguibile: eseguire prima pwsh store/windows/build.ps1"
}
$versioneFile = (Get-Content $marcatore -Raw).Trim()
if ($versioneFile -ne $Versione) {
    Write-Warning "L'eseguibile è della versione $versioneFile, la release è $tag."
    if ((Read-Host "Caricarlo comunque? [s/N]") -ne "s") { exit 1 }
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
            $dentro = $riga -match "^## $([regex]::Escape($Versione))( |$)"
            continue
        }
        if ($dentro) { $righe += $riga }
    }
    if ($righe.Count -eq 0) { $righe = @("RastInvoice $Versione") }
    $righe += ""
    $righe += Get-Content "store/note-release.md" -Encoding UTF8 | Where-Object { $_ -notmatch "^<!--" }
    Set-Content -Path $note.FullName -Value $righe -Encoding utf8NoBOM
    gh release create $tag $file -R $repo --draft --target main --title "RastInvoice $Versione" --notes-file $note.FullName
    $esito = $LASTEXITCODE
    Remove-Item $note -Force
    if ($esito) { throw "creazione della release non riuscita" }
}

if (-not $Pubblica) {
    Write-Host "Bozza pronta. Per pubblicarla: pwsh store/windows/pubblica.ps1 -Versione $Versione -Pubblica" -ForegroundColor Green
    exit 0
}

# File mancanti copiati dall'ultima release pubblicata.
$presenti = @(gh release view $tag -R $repo --json assets --jq ".assets[].name")
$mancanti = @($nomiFissi.Values | Where-Object { $presenti -notcontains $_ })
$precedente = gh release view -R $repo --json tagName --jq ".tagName" 2>$null
# Una release più vecchia dell'ultima non diventa "latest" e non tocca il sito.
$recente = -not $precedente -or ([version]$Versione -ge [version]$precedente.TrimStart("v"))
if (-not $recente) { $mancanti = @() }
if ($mancanti.Count -gt 0 -and $precedente -and $precedente -ne $tag) {
    Write-Host "== Copio da $precedente: $($mancanti -join ', ')" -ForegroundColor Cyan
    $cartella = Join-Path ([System.IO.Path]::GetTempPath()) "rastinvoice-$precedente"
    Remove-Item $cartella -Recurse -Force -ErrorAction SilentlyContinue
    $modelli = @()
    foreach ($m in $mancanti) { $modelli += "-p"; $modelli += "*$([System.IO.Path]::GetExtension($m))" }
    # latest.json va con i pacchetti Linux: si copia se la release non lo ha.
    $copiaManifesto = ($presenti -notcontains "latest.json") -and ($mancanti | Where-Object { $_ -like "RastInvoice-linux-*" })
    if ($copiaManifesto) { $modelli += "-p"; $modelli += "latest.json" }
    gh release download $precedente -R $repo -D $cartella @modelli
    $daCaricare = @()
    foreach ($f in Get-ChildItem $cartella -File -ErrorAction SilentlyContinue) {
        if ($f.Name -eq "latest.json") { $daCaricare += $f.FullName; continue }
        $nuovo = $nomiFissi[$f.Extension]
        if (-not $nuovo -or $mancanti -notcontains $nuovo) { continue }
        $destinazione = Join-Path $cartella $nuovo
        if ($f.FullName -ne $destinazione) { Move-Item $f.FullName $destinazione -Force }
        $daCaricare += $destinazione
    }
    if ($daCaricare.Count -gt 0) {
        gh release upload $tag @daCaricare --clobber -R $repo
        if ($LASTEXITCODE) { throw "copia dei file non riuscita" }
    }
    Remove-Item $cartella -Recurse -Force -ErrorAction SilentlyContinue
    $presenti = @(gh release view $tag -R $repo --json assets --jq ".assets[].name")
    $mancanti = @($nomiFissi.Values | Where-Object { $presenti -notcontains $_ })
}
if ($mancanti.Count -gt 0) {
    Write-Warning "Mancano ancora: $($mancanti -join ', '). I relativi pulsanti del sito darebbero errore 404."
    if ((Read-Host "Pubblicare comunque? [s/N]") -ne "s") { exit 1 }
}

if ($recente) { gh release edit $tag -R $repo --draft=false --latest }
else { gh release edit $tag -R $repo --draft=false --latest=false }
if ($LASTEXITCODE) { throw "pubblicazione non riuscita" }
Write-Host "Release $tag pubblicata." -ForegroundColor Green
