# Copyright (c) 2026 Andrea Re - info@digitare.net
# Autore: Andrea Re @ 2026 - info@digitare.net
#
# Versione PowerShell di sito/prepara.sh, con lo stesso risultato: prepara in
# target/sito/ le cartelle da caricare sul server.
#   public_html/  contenuto del sito, da copiare in domains/rastinvoice.it/public_html/
#   privato/      configurazione del modulo contatti, da copiare in domains/rastinvoice.it/privato/
#
# Uso, dalla radice del repository:
#   pwsh sito/prepara.ps1

$ErrorActionPreference = "Stop"
$sito = $PSScriptRoot
$uscita = Join-Path $sito "../target/sito"
Remove-Item $uscita -Recurse -Force -ErrorAction SilentlyContinue
$pub = Join-Path $uscita "public_html"
$privato = Join-Path $uscita "privato"
New-Item -ItemType Directory -Force -Path $pub, $privato | Out-Null
$utf8 = New-Object System.Text.UTF8Encoding($false)

# Tutto il sito tranne documentazione, modello e script di servizio.
Copy-Item (Join-Path $sito "*") $pub -Recurse -Force
Copy-Item (Join-Path $sito ".htaccess") $pub -Force
foreach ($f in "README.md", "config-esempio.php", "rastinvoice-config.php", "prepara.sh", "prepara.ps1") {
    Remove-Item (Join-Path $pub $f) -Force -ErrorAction SilentlyContinue
}
Get-ChildItem $pub -Recurse -Force -Include ".DS_Store", "Thumbs.db", "desktop.ini" | Remove-Item -Force

# CSS incorporato nelle pagine, senza ritorni a capo di Windows (\r): il browser
# li normalizza e l'impronta non corrisponderebbe. I percorsi dei caratteri
# diventano assoluti, validi per ogni pagina (anche la 404).
$bi = [IO.File]::ReadAllText((Join-Path $pub "css/bootstrap-italia.min.css"))
$proprio = [IO.File]::ReadAllText((Join-Path $pub "css/sito.css")).Replace('url("../fonts/', 'url("/fonts/')
$stile = ($bi + "`n" + $proprio).Replace("`r", "")
if (-not $stile.EndsWith("`n")) { $stile += "`n" }

# Impronta SHA-256 del blocco <style> per la Content-Security-Policy (style-src).
$sha = [System.Security.Cryptography.SHA256]::Create()
$impronta = "sha256-" + [Convert]::ToBase64String($sha.ComputeHash($utf8.GetBytes($stile)))

foreach ($pagina in Get-ChildItem $pub -Filter *.html) {
    $html = [IO.File]::ReadAllText($pagina.FullName)
    $html = [regex]::Replace($html, '(?m)^[ \t]*<link rel="stylesheet" href="/?css/bootstrap-italia\.min\.css">[ \t]*(\r?\n)',
        { param($m) "  <style>" + $stile + "</style>" + $m.Groups[1].Value })
    $html = [regex]::Replace($html, '(?m)^[ \t]*<link rel="stylesheet" href="/?css/sito\.css">[ \t]*\r?\n', "")
    if ($html.Contains('rel="stylesheet"')) { throw "Fogli di stile non incorporati in $($pagina.Name)" }
    [IO.File]::WriteAllText($pagina.FullName, $html, $utf8)
}
$htaccess = Join-Path $pub ".htaccess"
$regole = [IO.File]::ReadAllText($htaccess).Replace("style-src 'self'", "style-src 'self' '$impronta'")
if (-not $regole.Contains($impronta)) { throw "Impronta CSS non inserita in .htaccess" }
[IO.File]::WriteAllText($htaccess, $regole, $utf8)
Remove-Item (Join-Path $pub "css") -Recurse -Force

$config = Join-Path $sito "rastinvoice-config.php"
if (Test-Path $config) {
    Copy-Item $config $privato
    Write-Host "Configurazione copiata da sito/rastinvoice-config.php"
} else {
    Copy-Item (Join-Path $sito "config-esempio.php") (Join-Path $privato "rastinvoice-config.php")
    Write-Host "Configurazione di esempio in privato/rastinvoice-config.php: compilarla prima di caricarla"
    Write-Host "(oppure non caricarla affatto per usare mail() di PHP)."
}

$uscita = (Resolve-Path $uscita).Path
Write-Host ""
Write-Host "Pronte in ${uscita}:" -ForegroundColor Green
Write-Host "  public_html/ -> domains/rastinvoice.it/public_html/"
Write-Host "  privato/     -> domains/rastinvoice.it/privato/"
