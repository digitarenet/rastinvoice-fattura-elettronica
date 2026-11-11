// Copyright (c) 2026 Andrea Re - info@digitare.net
// Autore: Andrea Re @ 2026 - info@digitare.net
//
// Punto di ingresso dell'applicazione desktop.

// In modalità release su Windows non apre la console accanto alla finestra.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    rustinvoice_desktop_lib::avvia();
}
