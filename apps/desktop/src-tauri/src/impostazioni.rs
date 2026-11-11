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

impl Impostazioni {
    /// Copia delle impostazioni con i percorsi sotto `da` spostati sotto `a`
    /// (file nell'archivio dell'applicazione e documenti letti o contrassegnati).
    pub fn con_radice_cambiata(&self, da: &Path, a: &Path) -> Impostazioni {
        let da = da.to_string_lossy().to_string();
        let a = a.to_string_lossy().to_string();
        let sposta = |p: &String| match p.strip_prefix(&da) {
            Some(resto) if resto.starts_with(['/', '\\']) => format!("{a}{resto}"),
            _ => p.clone(),
        };
        let mut i = self.clone();
        i.cartelle = i.cartelle.iter().map(sposta).collect();
        i.file_singoli = i.file_singoli.iter().map(sposta).collect();
        i.lette = i.lette.iter().map(sposta).collect();
        i.contrassegnate = i.contrassegnate.iter().map(sposta).collect();
        i
    }
}

/// Cartella `dati` accanto all'eseguibile (modalità portable).
pub fn cartella_portable() -> Option<PathBuf> {
    Some(std::env::current_exe().ok()?.parent()?.join("dati"))
}

/// Vero se accanto all'eseguibile si può creare la cartella `dati`
/// (non è così, per esempio, in "Programmi" su Windows).
pub fn portable_possibile() -> bool {
    let Some(dati) = cartella_portable() else {
        return false;
    };
    if dati.is_dir() {
        return scrivibile(&dati);
    }
    dati.parent().is_some_and(scrivibile)
}

/// Cartella dati dell'applicazione (portable o standard).
pub fn cartella_dati(standard: &Path) -> PathBuf {
    if let Some(portable) = cartella_portable() {
        if portable.is_dir() && scrivibile(&portable) {
            return portable;
        }
    }
    standard.to_path_buf()
}

/// Primo avvio: nella cartella dati non ci sono ancora impostazioni salvate.
pub fn primo_avvio(dir: &Path) -> bool {
    !dir.join(NOME_FILE).exists()
}

/// File e cartelle dell'applicazione da portare con sé cambiando cartella
/// dati. Il resto (per esempio la cache del motore web, che su Linux usa la
/// stessa cartella del profilo) non viene copiato.
const CONTENUTO_DATI: &[&str] = &[NOME_FILE, "indice.json", "fogli_stile", "archivio"];

/// Copia i dati dell'applicazione in un'altra cartella.
pub fn copia_dati(da: &Path, a: &Path) -> std::io::Result<()> {
    std::fs::create_dir_all(a)?;
    for nome in CONTENUTO_DATI {
        let sorgente = da.join(nome);
        if sorgente.is_dir() {
            copia_cartella(&sorgente, &a.join(nome))?;
        } else if sorgente.is_file() {
            std::fs::copy(&sorgente, a.join(nome))?;
        }
    }
    Ok(())
}

/// Copia ricorsivamente il contenuto di una cartella in un'altra
/// (i file temporanei non vengono copiati).
fn copia_cartella(da: &Path, a: &Path) -> std::io::Result<()> {
    std::fs::create_dir_all(a)?;
    for voce in std::fs::read_dir(da)? {
        let voce = voce?;
        let tipo = voce.file_type()?;
        let destinazione = a.join(voce.file_name());
        if tipo.is_dir() {
            copia_cartella(&voce.path(), &destinazione)?;
        } else if tipo.is_file() && !voce.file_name().to_string_lossy().ends_with(".tmp") {
            std::fs::copy(voce.path(), destinazione)?;
        }
    }
    Ok(())
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
