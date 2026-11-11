// Copyright (c) 2026 Andrea Re - info@digitare.net
// Autore: Andrea Re @ 2026 - info@digitare.net
//
// Applicazione desktop: comandi esposti all'interfaccia.

mod archivio;
mod foglio_stile;
mod impostazioni;

use impostazioni::Impostazioni;
use rustinvoice_core::{apri_file, Contenuto};
use serde::Serialize;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use tauri::Manager;

/// Stato condiviso tra i comandi.
struct Stato {
    dir_dati: PathBuf,
    portable: bool,
    impostazioni: Mutex<Impostazioni>,
}

/// Informazioni di contesto restituite all'avvio.
#[derive(Serialize)]
struct Avvio {
    impostazioni: Impostazioni,
    cartella_dati: String,
    portable: bool,
    versione: String,
}

#[tauri::command]
fn avvio(stato: tauri::State<'_, Stato>) -> Avvio {
    Avvio {
        impostazioni: stato.impostazioni.lock().unwrap().clone(),
        cartella_dati: stato.dir_dati.to_string_lossy().to_string(),
        portable: stato.portable,
        versione: env!("CARGO_PKG_VERSION").to_owned(),
    }
}

#[tauri::command]
fn salva_impostazioni(stato: tauri::State<'_, Stato>, impostazioni: Impostazioni) -> Result<(), String> {
    impostazioni::salva(&stato.dir_dati, &impostazioni).map_err(|e| e.to_string())?;
    *stato.impostazioni.lock().unwrap() = impostazioni;
    Ok(())
}

#[tauri::command]
async fn scansiona(percorsi: Vec<String>) -> Result<archivio::Esito, String> {
    tauri::async_runtime::spawn_blocking(move || archivio::scansiona(&percorsi))
        .await
        .map_err(|e| e.to_string())
}

/// Dati per la visualizzazione di un documento.
#[derive(Serialize)]
struct Dettaglio {
    /// Pagina HTML della vista "Moderna".
    html: String,
    /// XML originale (già estratto dalla busta), per i fogli di stile XSL.
    xml: String,
    /// Modello dati completo.
    fattura: rustinvoice_core::FatturaElettronica,
    firmato: bool,
    livelli_firma: usize,
    certificati: usize,
    /// Esito della verifica di ciascuna firma.
    firme: Vec<rustinvoice_core::firma::Firmatario>,
}

#[tauri::command]
async fn dettaglio(file: String, corpo: usize) -> Result<Dettaglio, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let grezzo = rustinvoice_core::contenitori::leggi(&file).map_err(|e| e.to_string())?;
        let doc = rustinvoice_core::apri_byte(&grezzo).map_err(|e| e.to_string())?;
        let firmato = doc.firmato();
        let firme = if firmato {
            rustinvoice_core::firma::verifica(&grezzo)
        } else {
            Vec::new()
        };
        let (livelli_firma, certificati) = doc
            .busta
            .as_ref()
            .map(|b| (b.livelli, b.certificati.len()))
            .unwrap_or((0, 0));
        let Contenuto::Fattura(f) = doc.contenuto else {
            return Err("il file non contiene una fattura".to_owned());
        };
        // Per i lotti la vista Moderna mostra solo il documento selezionato.
        let mut singolo = (*f).clone();
        if corpo < singolo.corpi.len() && singolo.corpi.len() > 1 {
            singolo.corpi = vec![singolo.corpi[corpo].clone()];
        }
        Ok(Dettaglio {
            html: rustinvoice_render::html_moderna(&singolo),
            xml: doc.xml,
            fattura: *f,
            firmato,
            livelli_firma,
            certificati,
            firme,
        })
    })
    .await
    .map_err(|e| e.to_string())?
}

fn allegato(file: &str, corpo: usize, indice: usize) -> Result<(String, Vec<u8>), String> {
    let doc = apri_file(file).map_err(|e| e.to_string())?;
    let f = doc.fattura().ok_or("il file non contiene una fattura")?;
    let a = f
        .corpi
        .get(corpo)
        .and_then(|c| c.allegati.get(indice))
        .ok_or("allegato inesistente")?;
    let mut nome = a.nome.clone().unwrap_or_else(|| format!("allegato_{}", indice + 1));
    // Aggiunge l'estensione dichiarata se il nome ne è privo.
    if Path::new(&nome).extension().is_none() {
        if let Some(fmt) = &a.formato {
            nome = format!("{nome}.{}", fmt.to_lowercase());
        }
    }
    // Evita percorsi annidati nel nome scelto dal mittente.
    let nome = nome.replace(['/', '\\', ':'], "_");
    Ok((nome, a.dati.clone()))
}

#[tauri::command]
fn salva_allegato(file: String, corpo: usize, indice: usize, destinazione: String) -> Result<(), String> {
    let (_, dati) = allegato(&file, corpo, indice)?;
    std::fs::write(destinazione, dati).map_err(|e| e.to_string())
}

#[tauri::command]
fn nome_allegato(file: String, corpo: usize, indice: usize) -> Result<String, String> {
    allegato(&file, corpo, indice).map(|(n, _)| n)
}

/// Scrive l'allegato in una cartella temporanea e ne restituisce il percorso,
/// così l'interfaccia può aprirlo con il programma predefinito.
#[tauri::command]
fn apri_allegato(file: String, corpo: usize, indice: usize) -> Result<String, String> {
    let (nome, dati) = allegato(&file, corpo, indice)?;
    let dir = std::env::temp_dir().join("rustinvoice-allegati");
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    let p = dir.join(nome);
    std::fs::write(&p, dati).map_err(|e| e.to_string())?;
    Ok(p.to_string_lossy().to_string())
}

/// Copia i file indicati nella cartella "archivio" dei dati dell'applicazione.
#[tauri::command]
fn copia_in_archivio(stato: tauri::State<'_, Stato>, percorsi: Vec<String>) -> Result<Vec<String>, String> {
    let dest = stato.dir_dati.join("archivio");
    std::fs::create_dir_all(&dest).map_err(|e| e.to_string())?;
    let mut copiati = Vec::new();
    for p in archivio::elenca(&percorsi) {
        let Some(nome) = p.file_name() else { continue };
        let mut obiettivo = dest.join(nome);
        // Non sovrascrive file diversi con lo stesso nome.
        let mut n = 1;
        while obiettivo.exists() {
            if std::fs::read(&obiettivo).ok() == std::fs::read(&p).ok() {
                break;
            }
            obiettivo = dest.join(format!("{}_{n}", nome.to_string_lossy()));
            n += 1;
        }
        if !obiettivo.exists() {
            std::fs::copy(&p, &obiettivo).map_err(|e| e.to_string())?;
        }
        copiati.push(obiettivo.to_string_lossy().to_string());
    }
    Ok(copiati)
}

#[tauri::command]
fn cartella_archivio(stato: tauri::State<'_, Stato>) -> String {
    stato.dir_dati.join("archivio").to_string_lossy().to_string()
}

#[tauri::command]
fn foglio_stile_stato(stato: tauri::State<'_, Stato>) -> foglio_stile::Stato {
    foglio_stile::stato(&stato.dir_dati)
}

#[tauri::command]
async fn foglio_stile_aggiorna(stato: tauri::State<'_, Stato>) -> Result<foglio_stile::Stato, String> {
    let dir = stato.dir_dati.clone();
    tauri::async_runtime::spawn_blocking(move || foglio_stile::aggiorna(&dir))
        .await
        .map_err(|e| e.to_string())?
}

#[tauri::command]
fn foglio_stile_importa(stato: tauri::State<'_, Stato>, sorgente: String) -> Result<foglio_stile::Stato, String> {
    foglio_stile::importa(&stato.dir_dati, Path::new(&sorgente))
}

#[tauri::command]
fn foglio_stile_leggi(stato: tauri::State<'_, Stato>) -> Option<String> {
    foglio_stile::leggi(&stato.dir_dati)
}

/// Scrive un file di testo (esportazioni CSV, HTML).
#[tauri::command]
fn scrivi_testo(percorso: String, testo: String) -> Result<(), String> {
    std::fs::write(percorso, testo).map_err(|e| e.to_string())
}

/// Avvia l'applicazione.
pub fn avvia() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let standard = app
                .path()
                .app_data_dir()
                .unwrap_or_else(|_| std::env::temp_dir().join("rustinvoice"));
            let dir_dati = impostazioni::cartella_dati(&standard);
            let portable = dir_dati != standard;
            let _ = std::fs::create_dir_all(&dir_dati);
            let imp = impostazioni::carica(&dir_dati);
            app.manage(Stato {
                dir_dati,
                portable,
                impostazioni: Mutex::new(imp),
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            avvio,
            salva_impostazioni,
            scansiona,
            dettaglio,
            salva_allegato,
            nome_allegato,
            apri_allegato,
            copia_in_archivio,
            cartella_archivio,
            foglio_stile_stato,
            foglio_stile_aggiorna,
            foglio_stile_importa,
            foglio_stile_leggi,
            scrivi_testo
        ])
        .run(tauri::generate_context!())
        .expect("errore durante l'avvio dell'applicazione");
}
