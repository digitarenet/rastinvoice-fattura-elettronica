// Copyright (c) 2026 Andrea Re - info@digitare.net
// Autore: Andrea Re @ 2026 - info@digitare.net
//
// Impostazioni e dati persistenti dell'applicazione.
//
// Modalità portable: se accanto all'eseguibile esiste una cartella `dati`
// scrivibile, tutto viene salvato lì (comodo su chiavetta USB). Altrimenti
// si usa la cartella dati standard dell'utente. Il registro di sistema non
// viene mai usato.

use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

const NOME_FILE: &str = "impostazioni.json";

/// Impostazioni salvate su disco.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Impostazioni {
    /// Cartelle di lavoro lette all'avvio.
    pub cartelle: Vec<String>,
    /// File aperti singolarmente e lasciati nella posizione originale.
    pub file_singoli: Vec<String>,
    /// Partite IVA o codici fiscali propri, per distinguere emesse e ricevute.
    pub identificativi_propri: Vec<String>,
    /// Documenti contrassegnati dall'utente (id della voce).
    pub contrassegnate: BTreeSet<String>,
    /// Documenti già aperti almeno una volta.
    pub lette: BTreeSet<String>,
    /// `true` se il benvenuto iniziale è già stato mostrato.
    pub benvenuto_visto: bool,
    /// Cosa fare con i file trascinati: `"chiedi"`, `"copia"` o `"lascia"`.
    pub file_trascinati: String,
    /// Vista predefinita: `"moderna"` o `"standard"`.
    pub vista: String,
    /// Tema: `"chiaro"`, `"scuro"` o `"sistema"`.
    pub tema: String,
}

/// Cartella dati dell'applicazione (portable o standard).
pub fn cartella_dati(standard: &Path) -> PathBuf {
    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            let portable = dir.join("dati");
            if portable.is_dir() && scrivibile(&portable) {
                return portable;
            }
        }
    }
    standard.to_path_buf()
}

fn scrivibile(dir: &Path) -> bool {
    let prova = dir.join(".prova_scrittura");
    let ok = std::fs::write(&prova, b"ok").is_ok();
    let _ = std::fs::remove_file(prova);
    ok
}

/// Legge le impostazioni; in caso di file mancante o rovinato usa i valori predefiniti.
pub fn carica(dir: &Path) -> Impostazioni {
    let mut i: Impostazioni = std::fs::read(dir.join(NOME_FILE))
        .ok()
        .and_then(|b| serde_json::from_slice(&b).ok())
        .unwrap_or_default();
    if i.file_trascinati.is_empty() {
        i.file_trascinati = "chiedi".into();
    }
    if i.vista.is_empty() {
        i.vista = "moderna".into();
    }
    if i.tema.is_empty() {
        i.tema = "sistema".into();
    }
    i
}

/// Salva le impostazioni in modo atomico (file temporaneo + rinomina).
pub fn salva(dir: &Path, i: &Impostazioni) -> std::io::Result<()> {
    std::fs::create_dir_all(dir)?;
    let testo = serde_json::to_vec_pretty(i).map_err(std::io::Error::other)?;
    let tmp = dir.join(format!("{NOME_FILE}.tmp"));
    std::fs::write(&tmp, testo)?;
    std::fs::rename(tmp, dir.join(NOME_FILE))
}
