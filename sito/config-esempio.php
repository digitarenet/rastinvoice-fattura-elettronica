<?php
/*
 * Copyright (c) 2026 Andrea Re - info@digitare.net
 * Autore: Andrea Re @ 2026 - info@digitare.net
 *
 * Modello di configurazione del modulo contatti. Copiarlo con il nome rastinvoice-config.php nella
 * cartella "privato" accanto alla radice del sito (per esempio domains/rastinvoice.it/privato/, accanto
 * a public_html), così il file con la password non è raggiungibile dal web. Non usare private_html:
 * sui server DirectAdmin è la radice del sito in HTTPS, quindi pubblica. Non va mai pubblicato nel
 * repository.
 *
 * Lasciando vuoto smtp_host il modulo usa la funzione mail() di PHP, senza credenziali.
 */

return [
    'destinatario' => 'info@digitare.net',
    'mittente' => 'sito@rastinvoice.it',
    'smtp_host' => 'smtp.esempio.it',
    'smtp_porta' => 587,
    'smtp_sicurezza' => 'tls',
    'smtp_utente' => 'sito@rastinvoice.it',
    'smtp_password' => 'password-della-casella',

    // Protezione dagli invii automatici (facoltativo: questi sono i valori predefiniti)
    // 'invii_per_ora' => 5,          // per indirizzo IP
    // 'invii_al_giorno' => 50,       // in tutto
    // 'invii_senza_gettone' => 10,   // al giorno, dai browser senza JavaScript
    // 'link_massimi' => 3,           // oltre, il messaggio è scartato come spam
    // 'segreto' => '',               // vuoto: chiave generata in privato/rastinvoice-segreto
];
