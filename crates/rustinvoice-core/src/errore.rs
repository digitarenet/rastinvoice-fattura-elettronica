// Copyright (c) 2026 Andrea Re - info@digitare.net
// Autore: Andrea Re @ 2026 - info@digitare.net
//
// Tipi di errore del nucleo.

use thiserror::Error;

/// Errore generico restituito dalle funzioni del nucleo.
#[derive(Debug, Error)]
pub enum Errore {
    /// Il file non è leggibile come XML (sintassi errata, codifica non supportata, ...).
    #[error("XML non valido: {0}")]
    XmlNonValido(String),

    /// L'XML è corretto ma non è un documento FatturaPA riconosciuto.
    #[error("documento non riconosciuto: elemento radice <{0}>")]
    DocumentoNonRiconosciuto(String),

    /// Il contenitore firmato (.p7m) è malformato o non contiene dati.
    #[error("busta P7M non valida: {0}")]
    P7mNonValido(String),

    /// Errore di input/output sul file system.
    #[error("errore di lettura: {0}")]
    Io(#[from] std::io::Error),
}

/// Alias di comodo per i risultati del nucleo.
pub type Risultato<T> = std::result::Result<T, Errore>;
