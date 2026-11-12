<?php
/*
 * Copyright (c) 2026 Andrea Re - info@digitare.net
 * Autore: Andrea Re @ 2026 - info@digitare.net
 *
 * Modulo contatti di rastinvoice.it: controlla i dati e li invia per email.
 * Risponde in JSON se la richiesta lo chiede (invio da JavaScript), altrimenti reindirizza a
 * grazie.html o alla pagina iniziale. Non salva i messaggi sul server.
 *
 * Invio: con la funzione mail() di PHP (nessuna credenziale) oppure, se configurato, con un server
 * SMTP autenticato. La configurazione sta nel file rastinvoice-config.php, da mettere fuori dalla
 * radice del sito, così non è raggiungibile dal web: nella cartella "privato" accanto alla radice
 * (domains/rastinvoice.it/privato/) oppure direttamente sopra la radice (modello: config-esempio.php).
 */

declare(strict_types=1);

// Configurazione predefinita, sovrascritta da rastinvoice-config.php se presente
$config = [
    'destinatario' => 'info@digitare.net',
    'mittente' => 'sito@rastinvoice.it',    // indirizzo del dominio, per non finire nello spam
    'smtp_host' => '',                       // vuoto: si usa mail() di PHP
    'smtp_porta' => 587,                     // 587 con STARTTLS, 465 con SSL
    'smtp_sicurezza' => 'tls',               // 'tls' (STARTTLS), 'ssl' oppure '' (sconsigliato)
    'smtp_utente' => '',
    'smtp_password' => '',
    'invii_per_ora' => 5,                    // limite di invii per indirizzo IP
    'secondi_minimi' => 3,                   // un modulo compilato più in fretta è automatico
];
$cartelle = [dirname(__DIR__) . '/privato', dirname(__DIR__), __DIR__];
foreach (array_map(fn($c) => $c . '/rastinvoice-config.php', $cartelle) as $file) {
    if (is_file($file)) {
        $config = array_merge($config, (array)require $file);
        break;
    }
}
define('DESTINATARIO', (string)$config['destinatario']);
define('MITTENTE', (string)$config['mittente']);
define('INVII_PER_ORA', (int)$config['invii_per_ora']);
define('SECONDI_MINIMI', (int)$config['secondi_minimi']);

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

/**
 * Invio con un server SMTP autenticato (AUTH LOGIN), senza librerie esterne.
 * Restituisce true se il server accetta il messaggio.
 */
function invia_smtp(array $c, string $a, string $oggetto, string $corpo, string $rispondiA): bool
{
    $sicurezza = strtolower((string)$c['smtp_sicurezza']);
    $host = ($sicurezza === 'ssl' ? 'ssl://' : '') . $c['smtp_host'];
    $s = @stream_socket_client($host . ':' . (int)$c['smtp_porta'], $n, $e, 15);
    if (!$s) {
        error_log("contatti.php: connessione a $host:{$c['smtp_porta']} non riuscita: $e");
        return false;
    }
    stream_set_timeout($s, 15);
    // Ultimo comando e ultima risposta del server, per il registro degli errori
    $ultimo = 'connessione';
    $risposta = '';
    $leggi = function () use ($s, &$risposta): int {
        $riga = '';
        while (($r = fgets($s, 515)) !== false) {
            $riga = $r;
            $risposta = trim($r);
            if (strlen($r) < 4 || $r[3] !== '-') {
                break;
            }
        }
        return (int)substr($riga, 0, 3);
    };
    // $etichetta sostituisce il comando nel registro (utente, password, testo)
    $comando = function (string $cmd, array $attesi, string $etichetta = '') use ($s, $leggi, &$ultimo): bool {
        $ultimo = $etichetta !== '' ? $etichetta : $cmd;
        fwrite($s, $cmd . "\r\n");
        return in_array($leggi(), $attesi, true);
    };
    $dominio = substr(strrchr((string)$c['mittente'], '@') ?: '@localhost', 1);
    $ok = $leggi() === 220 && $comando('EHLO ' . $dominio, [250]);
    if ($ok && $sicurezza === 'tls') {
        $ok = $comando('STARTTLS', [220])
            && stream_socket_enable_crypto($s, true, STREAM_CRYPTO_METHOD_TLS_CLIENT)
            && $comando('EHLO ' . $dominio, [250]);
    }
    if ($ok && $c['smtp_utente'] !== '') {
        $ok = $comando('AUTH LOGIN', [334])
            && $comando(base64_encode((string)$c['smtp_utente']), [334], 'utente')
            && $comando(base64_encode((string)$c['smtp_password']), [235], 'password');
    }
    $ok = $ok && $comando('MAIL FROM:<' . $c['mittente'] . '>', [250])
        && $comando('RCPT TO:<' . $a . '>', [250, 251])
        && $comando('DATA', [354]);
    if ($ok) {
        $messaggio = implode("\r\n", [
            'Date: ' . date('r'),
            'From: RastInvoice <' . $c['mittente'] . '>',
            'To: <' . $a . '>',
            'Reply-To: ' . $rispondiA,
            'Subject: ' . $oggetto,
            'Message-ID: <' . bin2hex(random_bytes(12)) . '@' . $dominio . '>',
            'MIME-Version: 1.0',
            'Content-Type: text/plain; charset=UTF-8',
            'Content-Transfer-Encoding: base64',
        ]) . "\r\n\r\n" . chunk_split(base64_encode($corpo), 76, "\r\n") . '.';
        $ok = $comando($messaggio, [250], 'testo del messaggio');
    }
    if (!$ok) {
        error_log("contatti.php: invio SMTP non riuscito dopo \"$ultimo\": $risposta");
    }
    @fwrite($s, "QUIT\r\n");
    fclose($s);
    return $ok;
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

$inviato = $config['smtp_host'] !== ''
    ? invia_smtp($config, DESTINATARIO, $oggetto, $corpo, $email)
    : mail(DESTINATARIO, $oggetto, $corpo, $intestazioni, '-f' . MITTENTE);
if (!$inviato) {
    if ($config['smtp_host'] === '') {
        error_log('contatti.php: mail() non riuscita: ' . (error_get_last()['message'] ?? 'nessun dettaglio'));
    }
    rispondi(false, 'Non è stato possibile inviare il messaggio. Riprova più tardi o scrivi a ' . DESTINATARIO . '.');
}

// Conferma di ricezione a chi ha scritto. Il testo è fisso e non riporta nome né
// messaggio: chi usasse il modulo con l'indirizzo di un altro non può fargli
// arrivare contenuti scelti da lui. Se la conferma non parte, il messaggio è
// comunque arrivato e non si segnala errore.
$oggettoConferma = '=?UTF-8?B?' . base64_encode('RastInvoice: abbiamo ricevuto il tuo messaggio') . '?=';
$corpoConferma = "Ciao,\n\n"
    . "grazie per averci scritto dal sito rastinvoice.it: abbiamo ricevuto il tuo messaggio\n"
    . "e ti risponderemo il prima possibile.\n\n"
    . "Se non sei stato tu a scrivere, puoi ignorare questa email.\n\n"
    . "RastInvoice - https://www.rastinvoice.it\n"
    . "Questa email è inviata in automatico: per rispondere scrivi a " . DESTINATARIO . ".\n";
if ($config['smtp_host'] !== '') {
    invia_smtp($config, $email, $oggettoConferma, $corpoConferma, DESTINATARIO);
} else {
    $intestazioniConferma = implode("\r\n", [
        'From: RastInvoice <' . MITTENTE . '>',
        'Reply-To: ' . DESTINATARIO,
        'Auto-Submitted: auto-replied',
        'MIME-Version: 1.0',
        'Content-Type: text/plain; charset=UTF-8',
        'Content-Transfer-Encoding: 8bit',
    ]);
    mail($email, $oggettoConferma, $corpoConferma, $intestazioniConferma, '-f' . MITTENTE);
}

$invii[] = $adesso;
@file_put_contents($registro, implode("\n", $invii), LOCK_EX);
rispondi(true);
