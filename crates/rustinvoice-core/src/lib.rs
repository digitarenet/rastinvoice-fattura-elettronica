// Copyright (c) 2026 Andrea Re - info@digitare.net
// Autore: Andrea Re @ 2026 - info@digitare.net
//
// Nucleo di RustinVoice: lettura, interpretazione e verifica delle fatture
// elettroniche italiane nel formato FatturaPA.
//
// Il nucleo non dipende da alcuna interfaccia grafica: è usato dalla riga
// di comando, dall'applicazione desktop e, in futuro, dal servizio web.

pub mod codici;
pub mod contenitori;
pub mod documento;
pub mod errore;
pub mod firma;
pub mod modello;
pub mod p7m;
pub mod parser;
pub mod xml;

pub use documento::{apri_byte, apri_file, Contenuto, Documento};
pub use errore::{Errore, Risultato};
pub use modello::FatturaElettronica;
