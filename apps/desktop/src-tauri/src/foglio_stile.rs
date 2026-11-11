// Copyright (c) 2026 Andrea Re - info@digitare.net
// Autore: Andrea Re @ 2026 - info@digitare.net
//
// Gestione dei fogli di stile XSL esterni.
//
// Il foglio di stile standard per la visualizzazione delle fatture è fornito
// da AssoSoftware, che ne consente il libero utilizzo citandone la fonte.
// Non viene distribuito con l'applicazione: lo scarica l'utente, su
// richiesta, dall'indirizzo ufficiale, così da avere sempre l'ultima
// versione. In alternativa l'utente può indicare un proprio file .xsl.

use serde::Serialize;
use std::io::Read;
use std::path::{Path, PathBuf};

/// Indirizzo ufficiale dell'archivio con il foglio di stile.
pub const URL_STANDARD: &str =
    "https://assosoftware.it/allegati/assoinvoice/FoglioStileAssoSoftware.zip";

const NOME_LOCALE: &str = "foglio_stile_standard.xsl";

/// Dimensione massima accettata per il download (protezione da risposte anomale).
const MAX_BYTE: u64 = 20 * 1024 * 1024;

/// Stato del foglio di stile locale.
#[derive(Debug, Clone, Serialize)]
pub struct Stato {
    pub presente: bool,
    /// Data dell'ultimo aggiornamento, `AAAA-MM-GG`.
    pub aggiornato_il: String,
    pub percorso: String,
}

fn percorso(dir: &Path) -> PathBuf {
    dir.join("fogli_stile").join(NOME_LOCALE)
}

pub fn stato(dir: &Path) -> Stato {
    let p = percorso(dir);
    let aggiornato_il = std::fs::metadata(&p)
        .and_then(|m| m.modified())
        .map(|t| {
            chrono::DateTime::<chrono::Local>::from(t)
                .format("%Y-%m-%d")
                .to_string()
        })
        .unwrap_or_default();
    Stato {
        presente: p.is_file(),
        aggiornato_il,
        percorso: p.to_string_lossy().to_string(),
    }
}

/// Contenuto del foglio di stile locale, se presente.
pub fn leggi(dir: &Path) -> Option<String> {
    let byte = std::fs::read(percorso(dir)).ok()?;
    rustinvoice_core::xml::decodifica(&byte).ok()
}

/// Scarica l'archivio ufficiale ed estrae il primo file `.xsl` che contiene.
pub fn aggiorna(dir: &Path) -> Result<Stato, String> {
    let risposta = ureq::get(URL_STANDARD)
        .call()
        .map_err(|e| format!("download non riuscito: {e}"))?;
    let mut byte = Vec::new();
    risposta
        .into_body()
        .into_reader()
        .take(MAX_BYTE)
        .read_to_end(&mut byte)
        .map_err(|e| format!("download interrotto: {e}"))?;
    let xsl = estrai_xsl(&byte)?;
    installa(dir, &xsl)
}

/// Importa un foglio di stile scelto dall'utente (file .xsl o .zip).
pub fn importa(dir: &Path, sorgente: &Path) -> Result<Stato, String> {
    let byte = std::fs::read(sorgente).map_err(|e| e.to_string())?;
    let xsl = if byte.starts_with(b"PK") {
        estrai_xsl(&byte)?
    } else {
        byte
    };
    installa(dir, &xsl)
}

fn installa(dir: &Path, xsl: &[u8]) -> Result<Stato, String> {
    // Verifica minima: deve essere un foglio di stile XSL ben formato.
    let testo = rustinvoice_core::xml::decodifica(xsl).map_err(|e| e.to_string())?;
    let doc = rustinvoice_core::xml::analizza(&testo).map_err(|e| e.to_string())?;
    if doc.root_element().tag_name().name() != "stylesheet" {
        return Err("il file non è un foglio di stile XSL".into());
    }
    let p = percorso(dir);
    std::fs::create_dir_all(p.parent().unwrap_or(dir)).map_err(|e| e.to_string())?;
    std::fs::write(&p, xsl).map_err(|e| e.to_string())?;
    Ok(stato(dir))
}

fn estrai_xsl(zip_byte: &[u8]) -> Result<Vec<u8>, String> {
    let mut archivio = zip::ZipArchive::new(std::io::Cursor::new(zip_byte))
        .map_err(|e| format!("archivio non valido: {e}"))?;
    for i in 0..archivio.len() {
        let mut f = archivio.by_index(i).map_err(|e| e.to_string())?;
        if f.is_file() && f.name().is_ok_and(|n| n.to_lowercase().ends_with(".xsl")) {
            let mut v = Vec::new();
            f.by_ref()
                .take(MAX_BYTE)
                .read_to_end(&mut v)
                .map_err(|e| e.to_string())?;
            return Ok(v);
        }
    }
    Err("l'archivio non contiene un file .xsl".into())
}
