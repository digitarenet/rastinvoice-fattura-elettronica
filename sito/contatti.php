<?php
/*
 * Copyright (c) 2026 Andrea Re - info@digitare.net
 * Autore: Andrea Re @ 2026 - info@digitare.net
 *
 * Modulo contatti di rastinvoice.it: controlla i dati e li invia per email con mail() di PHP.
 * Risponde in JSON se la richiesta lo chiede (invio da JavaScript), altrimenti reindirizza a
 * grazie.html o alla pagina iniziale. Non salva i messaggi sul server.
 */

declare(strict_types=1);

// Configurazione
const DESTINATARIO = 'rastinvoice@digitare.net';
const MITTENTE = 'sito@rastinvoice.it';      // indirizzo del dominio, per non finire nello spam
const INVII_PER_ORA = 5;                      // limite di invii per indirizzo IP
const SECONDI_MINIMI = 3;                     // un modulo compilato più in fretta è automatico

$json = str_contains($_SERVER['HTTP_ACCEPT'] ?? '', 'application/json');

function rispondi(bool $ok, string $errore = ''): never
{
    global $json;
    if ($json) {
        header('Content-Type: application/json; charset=utf-8');
        if (!$ok) {
            http_response_code(400);
        }
        echo json_encode(['ok' => $ok, 'errore' => $errore], JSON_UNESCAPED_UNICODE);
    } else {
        header('Location: ' . ($ok ? 'grazie.html' : 'index.html?errore=1#contatti'), true, 303);
    }
    exit;
}

if (($_SERVER['REQUEST_METHOD'] ?? '') !== 'POST') {
    header('Location: index.html#contatti', true, 303);
    exit;
}

// Testo su una riga, senza caratteri di controllo (evita l'inserimento di intestazioni email)
function riga(string $valore, int $massimo): string
{
    $valore = preg_replace('/[\x00-\x1F\x7F]+/u', ' ', $valore) ?? '';
    return mb_substr(trim($valore), 0, $massimo);
}

$nome = riga((string)($_POST['nome'] ?? ''), 100);
$email = riga((string)($_POST['email'] ?? ''), 200);
$argomento = riga((string)($_POST['argomento'] ?? 'Altro'), 60);
$messaggio = mb_substr(trim(str_replace("\0", '', (string)($_POST['messaggio'] ?? ''))), 0, 5000);
$privacy = ($_POST['privacy'] ?? '') === 'si';
$trappola = (string)($_POST['sito_web'] ?? '');
$inizio = (int)($_POST['inizio'] ?? 0);

// Invii automatici: fingiamo che sia andato tutto bene, senza inviare nulla
if ($trappola !== '' || ($inizio > 0 && time() - $inizio < SECONDI_MINIMI)) {
    rispondi(true);
}

$argomenti = [
    'Informazioni sul programma', 'Problema o errore', 'Proposta di nuova funzionalità',
    'Commercialisti e studi professionali', 'Collaborazione o integrazione', 'Altro',
];
if (!in_array($argomento, $argomenti, true)) {
    $argomento = 'Altro';
}

if ($nome === '' || $messaggio === '' || !filter_var($email, FILTER_VALIDATE_EMAIL)) {
    rispondi(false, 'Controlla i campi: nome, email valida e messaggio sono obbligatori.');
}
if (!$privacy) {
    rispondi(false, "Per inviare il messaggio serve il consenso all'informativa sulla privacy.");
}

// Limite di invii per IP, con un file nella cartella temporanea (l'IP è salvato solo come impronta)
$ip = hash('sha256', ($_SERVER['REMOTE_ADDR'] ?? '') . __FILE__);
$registro = sys_get_temp_dir() . '/rastinvoice-contatti-' . $ip;
$adesso = time();
$invii = [];
if (is_file($registro)) {
    $invii = array_filter(
        array_map('intval', explode("\n", (string)file_get_contents($registro))),
        fn(int $t) => $t > $adesso - 3600
    );
}
if (count($invii) >= INVII_PER_ORA) {
    rispondi(false, 'Hai già inviato diversi messaggi. Riprova tra un\'ora o scrivi a ' . DESTINATARIO . '.');
}

$oggetto = '=?UTF-8?B?' . base64_encode('[RastInvoice] ' . $argomento . ' - ' . $nome) . '?=';
$corpo = "Nuovo messaggio dal modulo contatti di rastinvoice.it\n\n"
    . "Nome: $nome\n"
    . "Email: $email\n"
    . "Argomento: $argomento\n"
    . 'Data: ' . date('d/m/Y H:i') . "\n\n"
    . $messaggio . "\n";
$intestazioni = implode("\r\n", [
    'From: RastInvoice <' . MITTENTE . '>',
    'Reply-To: ' . $email,
    'MIME-Version: 1.0',
    'Content-Type: text/plain; charset=UTF-8',
    'Content-Transfer-Encoding: 8bit',
]);

if (!mail(DESTINATARIO, $oggetto, $corpo, $intestazioni, '-f' . MITTENTE)) {
    rispondi(false, 'Non è stato possibile inviare il messaggio. Riprova più tardi o scrivi a ' . DESTINATARIO . '.');
}

$invii[] = $adesso;
@file_put_contents($registro, implode("\n", $invii), LOCK_EX);
rispondi(true);
