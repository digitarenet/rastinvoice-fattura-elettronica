// Copyright (c) 2026 Andrea Re - info@digitare.net
// Autore: Andrea Re @ 2026 - info@digitare.net
//
// Applicazione desktop: comandi esposti all'interfaccia.

mod aggiornamenti;
mod anteprima;
mod archivio;
mod foglio_stile;
mod impostazioni;
mod segnalibri;

use impostazioni::Impostazioni;
use rustinvoice_core::{apri_file, Contenuto};
use serde::Serialize;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use tauri::Manager;

/// Stato condiviso tra i comandi.
struct Stato {
    dir_dati: PathBuf,
    /// Cartella dati standard del profilo utente.
    dir_standard: PathBuf,
    portable: bool,
    primo_avvio: bool,
    impostazioni: Mutex<Impostazioni>,
}

/// Informazioni di contesto restituite all'avvio.
#[derive(Serialize)]
struct Avvio {
    impostazioni: Impostazioni,
    cartella_dati: String,
    portable: bool,
    versione: String,
    /// Al primo avvio l'interfaccia chiede dove salvare i dati.
    primo_avvio: bool,
    cartella_standard: String,
    cartella_portable: String,
    portable_possibile: bool,
    /// Canale da cui è stato installato il programma (vedi [`distribuzione`]).
    distribuzione: &'static str,
    /// Vero se installato da uno store: aggiornamenti e cartella dati li gestisce lo store.
    da_store: bool,
    /// Aggiornamento dall'applicazione: `automatico`, `pacchetto` o vuoto
    /// (vedi [`aggiornamenti::integrato`]).
    aggiornamento_integrato: &'static str,
}

/// Riconosce il canale di distribuzione: `microsoft-store`, `mac-app-store`,
/// `flathub`, `snap` oppure `diretta` (pacchetti scaricati o compilati in proprio).
fn distribuzione() -> &'static str {
    if std::env::var_os("FLATPAK_ID").is_some() {
        return "flathub";
    }
    if std::env::var_os("SNAP").is_some() {
        return "snap";
    }
    let Ok(exe) = std::env::current_exe() else {
        return "diretta";
    };
    if cfg!(windows)
        && exe
            .to_string_lossy()
            .to_lowercase()
            .contains("\\windowsapps\\")
    {
        return "microsoft-store";
    }
    // Le app del Mac App Store contengono la ricevuta in Contents/_MASReceipt
    // e girano nella sandbox, che imposta APP_SANDBOX_CONTAINER_ID (vale anche
    // per la copia di prova in sandbox, ancora senza ricevuta).
    if cfg!(target_os = "macos")
        && (std::env::var_os("APP_SANDBOX_CONTAINER_ID").is_some() || exe
            .parent()
            .and_then(|p| p.parent())
            .is_some_and(|c| c.join("_MASReceipt").exists()))
    {
        return "mac-app-store";
    }
    "diretta"
}

#[tauri::command]
fn avvio(stato: tauri::State<'_, Stato>) -> Avvio {
    Avvio {
        impostazioni: stato.impostazioni.lock().unwrap().clone(),
        cartella_dati: stato.dir_dati.to_string_lossy().to_string(),
        portable: stato.portable,
        versione: env!("CARGO_PKG_VERSION").to_owned(),
        primo_avvio: stato.primo_avvio,
        cartella_standard: stato.dir_standard.to_string_lossy().to_string(),
        cartella_portable: impostazioni::cartella_portable()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_default(),
        portable_possibile: impostazioni::portable_possibile(),
        distribuzione: distribuzione(),
        da_store: distribuzione() != "diretta",
        aggiornamento_integrato: if distribuzione() == "diretta" {
            aggiornamenti::integrato()
        } else {
            ""
        },
    }
}

/// Sceglie dove salvare i dati: accanto al programma (`portable`) o nel
/// profilo utente. Se la cartella cambia, i dati vengono copiati nella nuova
/// posizione e l'applicazione si riavvia. Lasciando la modalità portable, la
/// vecchia cartella `dati` viene rinominata (non cancellata), altrimenti al
/// riavvio verrebbe usata di nuovo.
#[tauri::command]
fn cambia_cartella_dati(
    app: tauri::AppHandle,
    stato: tauri::State<'_, Stato>,
    portable: bool,
) -> Result<(), String> {
    let destinazione = if portable {
        if !impostazioni::portable_possibile() {
            return Err("nella cartella del programma non si può scrivere".into());
        }
        impostazioni::cartella_portable().ok_or("cartella del programma non trovata")?
    } else {
        stato.dir_standard.clone()
    };
    let attuale = stato.dir_dati.clone();
    if destinazione == attuale {
        // Scelta confermata: si salvano le impostazioni, così non verrà più chiesta.
        std::fs::create_dir_all(&attuale).map_err(|e| e.to_string())?;
        let imp = stato.impostazioni.lock().unwrap().clone();
        return impostazioni::salva(&attuale, &imp).map_err(|e| e.to_string());
    }
    impostazioni::copia_dati(&attuale, &destinazione).map_err(|e| e.to_string())?;
    // I file copiati nell'archivio cambiano posizione: si aggiornano i percorsi.
    let imp = stato
        .impostazioni
        .lock()
        .unwrap()
        .con_radice_cambiata(&attuale, &destinazione);
    impostazioni::salva(&destinazione, &imp).map_err(|e| e.to_string())?;
    if stato.portable {
        let data = chrono::Local::now().format("%Y%m%d-%H%M%S");
        let accantonata = attuale.with_file_name(format!("dati (non usata {data})"));
        std::fs::rename(&attuale, &accantonata).map_err(|e| e.to_string())?;
    }
    app.restart()
}

/// Cartelle d'archivio e file singoli da riaprire al prossimo avvio.
fn percorsi_da_ricordare(imp: &Impostazioni) -> Vec<&String> {
    imp.cartelle.iter().chain(&imp.file_singoli).collect()
}

#[tauri::command]
fn salva_impostazioni(
    stato: tauri::State<'_, Stato>,
    impostazioni: Impostazioni,
) -> Result<(), String> {
    impostazioni::salva(&stato.dir_dati, &impostazioni).map_err(|e| e.to_string())?;
    segnalibri::aggiorna(&stato.dir_dati, &percorsi_da_ricordare(&impostazioni));
    *stato.impostazioni.lock().unwrap() = impostazioni;
    Ok(())
}

#[tauri::command]
async fn scansiona(
    stato: tauri::State<'_, Stato>,
    percorsi: Vec<String>,
) -> Result<archivio::Esito, String> {
    let dir = stato.dir_dati.clone();
    tauri::async_runtime::spawn_blocking(move || archivio::scansiona(&percorsi, &dir))
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
    /// Controlli formali e di calcolo sul documento selezionato.
    controlli: Vec<rustinvoice_core::controlli::Controllo>,
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
        let controlli = rustinvoice_core::controlli::controlla_documento(&f, corpo);
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
            controlli,
        })
    })
    .await
    .map_err(|e| e.to_string())?
}

/// Indice speciale degli allegati: indica il PDF della fattura stessa, generato
/// al momento. Così visualizzatore, salvataggio e apertura esterna degli
/// allegati funzionano anche per il PDF senza comandi a parte.
const INDICE_PDF_FATTURA: usize = 999_999;

fn allegato(file: &str, corpo: usize, indice: usize) -> Result<(String, Vec<u8>), String> {
    let doc = apri_file(file).map_err(|e| e.to_string())?;
    let f = doc.fattura().ok_or("il file non contiene una fattura")?;
    if indice == INDICE_PDF_FATTURA {
        return pdf_fattura(file, f, corpo);
    }
    let a = f
        .corpi
        .get(corpo)
        .and_then(|c| c.allegati.get(indice))
        .ok_or("allegato inesistente")?;
    let mut nome = a
        .nome
        .clone()
        .unwrap_or_else(|| format!("allegato_{}", indice + 1));
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

/// PDF del documento indicato (per i lotti solo quello), con il nome del file
/// senza estensioni.
fn pdf_fattura(
    file: &str,
    f: &rustinvoice_core::modello::FatturaElettronica,
    corpo: usize,
) -> Result<(String, Vec<u8>), String> {
    let mut singolo = f.clone();
    if corpo < singolo.corpi.len() && singolo.corpi.len() > 1 {
        singolo.corpi = vec![singolo.corpi[corpo].clone()];
    }
    let dati = rustinvoice_render::pdf::pdf_moderna(&singolo)?;
    let mut base = Path::new(file.rsplit("!/").next().unwrap_or(file))
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| "fattura".into());
    for estensione in [".p7m", ".P7M", ".xml", ".XML"] {
        if let Some(senza) = base.strip_suffix(estensione) {
            base = senza.to_owned();
        }
    }
    if f.corpi.len() > 1 {
        base = format!("{base}_{}", corpo + 1);
    }
    Ok((format!("{}.pdf", base.replace(['/', '\\', ':'], "_")), dati))
}

/// Anteprima di un allegato (o di un file dentro un allegato zip, indicato
/// da `interno`). Il tipo si riconosce dal contenuto.
#[tauri::command]
async fn anteprima_allegato(
    file: String,
    corpo: usize,
    indice: usize,
    interno: Vec<String>,
) -> Result<anteprima::Anteprima, String> {
    let (nome, dati) = allegato(&file, corpo, indice)?;
    let (nome, dati) = anteprima::risolvi(&nome, dati, &interno)?;
    Ok(anteprima::prepara(&nome, &dati))
}

/// Byte di un allegato per i visualizzatori dell'interfaccia (PDF, immagini,
/// Word), trasferiti senza conversione in testo.
#[tauri::command]
async fn byte_allegato(
    file: String,
    corpo: usize,
    indice: usize,
    interno: Vec<String>,
) -> Result<tauri::ipc::Response, String> {
    let (nome, dati) = allegato(&file, corpo, indice)?;
    let (_, dati) = anteprima::risolvi(&nome, dati, &interno)?;
    Ok(tauri::ipc::Response::new(dati))
}

/// Salva un allegato dove sceglie l'utente. La finestra di salvataggio è
/// aperta dalla parte nativa: il percorso di destinazione non arriva mai
/// dall'interfaccia, quindi nessun contenuto web può scrivere dove vuole.
#[tauri::command]
async fn salva_allegato(
    app: tauri::AppHandle,
    file: String,
    corpo: usize,
    indice: usize,
) -> Result<bool, String> {
    let (nome, dati) = allegato(&file, corpo, indice)?;
    let Some(destinazione) = chiedi_destinazione(&app, &nome, None) else {
        return Ok(false);
    };
    std::fs::write(destinazione, dati).map_err(|e| e.to_string())?;
    Ok(true)
}

/// Tipi di file di testo che l'interfaccia può far salvare.
const TIPI_TESTO: &[(&str, &str, &str)] = &[
    ("csv", "CSV", "csv"),
    ("html", "Pagina HTML", "html"),
    ("xml", "File XML", "xml"),
];

/// Salva un testo (esportazione CSV, pagina HTML, XML estratto) dove sceglie
/// l'utente, con la finestra di salvataggio aperta dalla parte nativa.
#[tauri::command]
async fn salva_testo(
    app: tauri::AppHandle,
    nome: String,
    tipo: String,
    testo: String,
) -> Result<bool, String> {
    let Some(&(_, descrizione, estensione)) = TIPI_TESTO.iter().find(|(t, _, _)| *t == tipo) else {
        return Err("tipo di file non previsto".into());
    };
    let nome = nome_sicuro(&nome, estensione);
    let Some(destinazione) = chiedi_destinazione(&app, &nome, Some((descrizione, estensione)))
    else {
        return Ok(false);
    };
    std::fs::write(destinazione, testo).map_err(|e| e.to_string())?;
    Ok(true)
}

/// Nome proposto nella finestra di salvataggio: solo il nome del file, senza
/// cartelle, con l'estensione attesa.
fn nome_sicuro(nome: &str, estensione: &str) -> String {
    let base = Path::new(nome)
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_default();
    let base = if base.is_empty() {
        "documento".to_owned()
    } else {
        base
    };
    if base.to_lowercase().ends_with(&format!(".{estensione}")) {
        base
    } else {
        format!("{base}.{estensione}")
    }
}

/// Mostra la finestra di salvataggio e restituisce il percorso scelto.
fn chiedi_destinazione(
    app: &tauri::AppHandle,
    nome: &str,
    filtro: Option<(&str, &str)>,
) -> Option<PathBuf> {
    use tauri_plugin_dialog::DialogExt;
    let mut finestra = app.dialog().file().set_file_name(nome);
    if let Some((descrizione, estensione)) = filtro {
        finestra = finestra.add_filter(descrizione, &[estensione]);
    }
    finestra.blocking_save_file()?.into_path().ok()
}

/// Scrive l'allegato in una cartella temporanea e ne restituisce il percorso,
/// così l'interfaccia può aprirlo con il programma predefinito.
/// Estensioni degli allegati che si possono aprire direttamente: documenti,
/// immagini e testo. Gli altri (eseguibili, script, collegamenti...) si
/// possono solo salvare, perché il nome e il contenuto li sceglie il mittente.
const ALLEGATI_APRIBILI: &[&str] = &[
    "pdf", "txt", "csv", "xml", "png", "jpg", "jpeg", "gif", "bmp", "tif", "tiff", "webp", "odt",
    "ods", "doc", "docx", "xls", "xlsx", "rtf", "p7m", "zip",
];

#[tauri::command]
fn apri_allegato(
    app: tauri::AppHandle,
    file: String,
    corpo: usize,
    indice: usize,
) -> Result<(), String> {
    let (nome, dati) = allegato(&file, corpo, indice)?;
    let estensione = Path::new(&nome)
        .extension()
        .map(|e| e.to_string_lossy().to_lowercase())
        .unwrap_or_default();
    if !ALLEGATI_APRIBILI.contains(&estensione.as_str()) {
        return Err(format!(
            "Per sicurezza i file .{estensione} non si aprono direttamente: usa \"Salva\" e controllalo prima di aprirlo."
        ));
    }
    // Cartella dell'utente (non quella temporanea condivisa) e nuova a ogni
    // apertura: nessun altro utente del computer può preparare in anticipo
    // file o collegamenti con lo stesso nome.
    let radice = app
        .path()
        .app_cache_dir()
        .map_err(|e| e.to_string())?
        .join("allegati");
    std::fs::create_dir_all(&radice).map_err(|e| e.to_string())?;
    let dir = cartella_nuova(&radice)?;
    let p = dir.join(nome);
    std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&p)
        .and_then(|mut f| std::io::Write::write_all(&mut f, &dati))
        .map_err(|e| e.to_string())?;
    segna_come_scaricato(&p);
    tauri_plugin_opener::open_path(&p, None::<&str>).map_err(|e| e.to_string())
}

/// Crea una sottocartella con nome mai usato prima.
fn cartella_nuova(radice: &Path) -> Result<PathBuf, String> {
    let base = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or_default();
    for n in 0..100u32 {
        let dir = radice.join(format!("{base:x}{n:02}"));
        if std::fs::create_dir(&dir).is_ok() {
            return Ok(dir);
        }
    }
    Err("impossibile preparare la cartella per l'allegato".into())
}

/// Su Windows marca il file come proveniente da Internet: Office lo apre in
/// visualizzazione protetta e il sistema avvisa prima di eseguire contenuti
/// attivi, come per gli allegati delle email.
fn segna_come_scaricato(p: &Path) {
    #[cfg(windows)]
    {
        let mut flusso = p.as_os_str().to_owned();
        flusso.push(":Zone.Identifier");
        let _ = std::fs::write(flusso, "[ZoneTransfer]\r\nZoneId=3\r\n");
    }
    #[cfg(not(windows))]
    let _ = p;
}

/// Apre in Esplora risorse una delle cartelle configurate o la cartella dati.
#[tauri::command]
fn apri_cartella(stato: tauri::State<'_, Stato>, percorso: String) -> Result<(), String> {
    let consentita = {
        let imp = stato.impostazioni.lock().unwrap();
        imp.cartelle.iter().any(|c| c == &percorso)
    } || Path::new(&percorso) == stato.dir_dati;
    if !consentita || !Path::new(&percorso).is_dir() {
        return Err("cartella non disponibile".into());
    }
    tauri_plugin_opener::open_path(&percorso, None::<&str>).map_err(|e| e.to_string())
}

/// Mostra un documento nella sua cartella (per i documenti dentro zip o email,
/// il file contenitore).
#[tauri::command]
fn mostra_nella_cartella(file: String) -> Result<(), String> {
    let (disco, _) = rustinvoice_core::contenitori::dividi(&file);
    tauri_plugin_opener::reveal_item_in_dir(disco).map_err(|e| e.to_string())
}

/// Apre il sito dell'autore nel browser predefinito (indirizzo fisso).
#[tauri::command]
fn apri_sito() -> Result<(), String> {
    tauri_plugin_opener::open_url(
        "https://www.digitare.net/?s=fatture&utm_source=rastinvoice&utm_medium=desktop&utm_campaign=credits",
        None::<&str>,
    )
    .map_err(|e| e.to_string())
}

/// Apre la pagina delle donazioni. Sono ammessi solo indirizzi PayPal.me
/// o del profilo PayPal Business, così l'interfaccia non può aprire pagine qualsiasi.
#[tauri::command]
fn apri_donazione(url: String) -> Result<(), String> {
    let ammesso = [
        "https://www.paypal.me/",
        "https://paypal.me/",
        "https://www.paypal.biz/",
    ]
    .iter()
    .any(|p| url.starts_with(p) && url.len() > p.len() && url.len() < 120);
    if !ammesso {
        return Err("indirizzo per le donazioni non valido".into());
    }
    tauri_plugin_opener::open_url(url, None::<&str>).map_err(|e| e.to_string())
}

/// Indirizzo a cui scrivono sostenitori e utenti (fisso: l'interfaccia sceglie solo oggetto e testo).
const EMAIL_CONTATTO: &str = "rastinvoice@digitare.net";

/// Codifica un testo per un indirizzo `mailto:` (RFC 6068): tutto tranne i caratteri non riservati.
fn codifica_mailto(testo: &str) -> String {
    let mut esito = String::with_capacity(testo.len() * 3);
    for b in testo.replace("\r\n", "\n").replace('\n', "\r\n").bytes() {
        if b.is_ascii_alphanumeric() || b"-._~".contains(&b) {
            esito.push(b as char);
        } else {
            esito.push_str(&format!("%{b:02X}"));
        }
    }
    esito
}

/// Apre il programma di posta predefinito con un messaggio già compilato.
/// Il programma non invia nulla: l'utente rivede e spedisce l'email.
#[tauri::command]
fn scrivi_email(oggetto: String, testo: String) -> Result<(), String> {
    // Limiti prudenti: alcuni programmi di posta rifiutano indirizzi troppo lunghi.
    let oggetto: String = oggetto.chars().take(200).collect();
    let testo: String = testo.chars().take(4000).collect();
    let url = format!(
        "mailto:{EMAIL_CONTATTO}?subject={}&body={}",
        codifica_mailto(&oggetto),
        codifica_mailto(&testo)
    );
    tauri_plugin_opener::open_url(url, None::<&str>).map_err(|e| e.to_string())
}

/// Controlla se è stata pubblicata una versione più recente.
#[tauri::command]
async fn controlla_aggiornamenti() -> Result<aggiornamenti::Esito, String> {
    tauri::async_runtime::spawn_blocking(aggiornamenti::controlla)
        .await
        .map_err(|e| e.to_string())?
}

/// Scarica e installa l'ultima versione (solo pacchetti Linux scaricati da
/// GitHub). Restituisce la versione installata, se ce n'era una più recente.
#[tauri::command]
async fn installa_aggiornamento(app: tauri::AppHandle) -> Result<Option<String>, String> {
    if distribuzione() != "diretta" {
        return Err("gli aggiornamenti li gestisce lo store".into());
    }
    aggiornamenti::installa(app).await
}

/// Riavvia l'applicazione, per usare la versione appena installata.
#[tauri::command]
fn riavvia(app: tauri::AppHandle) {
    app.restart();
}

/// Apre nel browser la pagina di download dell'ultima versione.
#[tauri::command]
fn apri_download() -> Result<(), String> {
    tauri_plugin_opener::open_url(aggiornamenti::URL_RELEASE, None::<&str>)
        .map_err(|e| e.to_string())
}

/// Copia i file indicati nella cartella "archivio" dei dati dell'applicazione.
#[tauri::command]
fn copia_in_archivio(
    stato: tauri::State<'_, Stato>,
    percorsi: Vec<String>,
) -> Result<Vec<String>, String> {
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
            // Il numero va prima dell'estensione (fatture_1.zip), così il file
            // resta riconoscibile dal tipo.
            obiettivo = dest.join(nome_con_numero(Path::new(nome), n));
            n += 1;
        }
        if !obiettivo.exists() {
            std::fs::copy(&p, &obiettivo).map_err(|e| e.to_string())?;
        }
        copiati.push(obiettivo.to_string_lossy().to_string());
    }
    Ok(copiati)
}

/// `fatture.xml.p7m` con `n = 2` diventa `fatture_2.xml.p7m`: il numero va
/// prima di tutte le estensioni.
fn nome_con_numero(nome: &Path, n: u32) -> String {
    let testo = nome.to_string_lossy();
    match testo.find('.') {
        Some(i) if i > 0 => format!("{}_{n}{}", &testo[..i], &testo[i..]),
        _ => format!("{testo}_{n}"),
    }
}

#[tauri::command]
fn cartella_archivio(stato: tauri::State<'_, Stato>) -> String {
    stato
        .dir_dati
        .join("archivio")
        .to_string_lossy()
        .to_string()
}

#[tauri::command]
fn foglio_stile_stato(stato: tauri::State<'_, Stato>) -> foglio_stile::Stato {
    foglio_stile::stato(&stato.dir_dati)
}

#[tauri::command]
async fn foglio_stile_aggiorna(
    stato: tauri::State<'_, Stato>,
) -> Result<foglio_stile::Stato, String> {
    let dir = stato.dir_dati.clone();
    tauri::async_runtime::spawn_blocking(move || foglio_stile::aggiorna(&dir))
        .await
        .map_err(|e| e.to_string())?
}

#[tauri::command]
fn foglio_stile_importa(
    stato: tauri::State<'_, Stato>,
    sorgente: String,
) -> Result<foglio_stile::Stato, String> {
    foglio_stile::importa(&stato.dir_dati, Path::new(&sorgente))
}

#[tauri::command]
fn foglio_stile_leggi(stato: tauri::State<'_, Stato>) -> Option<String> {
    foglio_stile::leggi(&stato.dir_dati)
}

/// Applica il foglio di stile standard con il motore XSLT nativo, che non
/// dipende dal supporto XSLT della webview.
#[tauri::command]
async fn trasforma_standard(stato: tauri::State<'_, Stato>, xml: String) -> Result<String, String> {
    let foglio = foglio_stile::leggi(&stato.dir_dati).ok_or("foglio di stile non presente")?;
    tauri::async_runtime::spawn_blocking(move || rustinvoice_xslt::trasforma(&foglio, &xml))
        .await
        .map_err(|e| e.to_string())?
        .map_err(|e| e.to_string())
}

/// La finestra può mostrare solo l'interfaccia dell'applicazione e le
/// anteprime generate al suo interno: qualsiasi altra pagina (per esempio un
/// collegamento contenuto in una fattura) viene bloccata.
fn navigazione_consentita(url: &tauri::Url) -> bool {
    match url.scheme() {
        "tauri" | "about" | "data" | "blob" => true,
        "http" | "https" => {
            url.host_str() == Some("tauri.localhost")
                // Server di sviluppo (npm run app:dev), solo senza interfaccia inclusa.
                || (!cfg!(feature = "interfaccia-inclusa")
                    && url.host_str() == Some("localhost")
                    && url.port() == Some(1420))
        }
        _ => false,
    }
}

/// Barra dei menu di macOS in italiano: quella predefinita di Tauri è in
/// inglese. Le voci sono quelle standard, con le azioni native del sistema.
#[cfg(target_os = "macos")]
fn menu_macos(app: &tauri::AppHandle) -> tauri::Result<tauri::menu::Menu<tauri::Wry>> {
    use tauri::menu::{AboutMetadata, Menu, PredefinedMenuItem as Voce, Submenu};

    let info = AboutMetadata {
        name: Some("RastInvoice".into()),
        version: Some(app.package_info().version.to_string()),
        copyright: app.config().bundle.copyright.clone(),
        ..Default::default()
    };
    let applicazione = Submenu::with_items(
        app,
        "RastInvoice",
        true,
        &[
            &Voce::about(app, Some("Informazioni su RastInvoice"), Some(info))?,
            &Voce::separator(app)?,
            &Voce::services(app, Some("Servizi"))?,
            &Voce::separator(app)?,
            &Voce::hide(app, Some("Nascondi RastInvoice"))?,
            &Voce::hide_others(app, Some("Nascondi altre"))?,
            &Voce::show_all(app, Some("Mostra tutte"))?,
            &Voce::separator(app)?,
            &Voce::quit(app, Some("Esci da RastInvoice"))?,
        ],
    )?;
    let archivio = Submenu::with_items(
        app,
        "Archivio",
        true,
        &[&Voce::close_window(app, Some("Chiudi finestra"))?],
    )?;
    let modifica = Submenu::with_items(
        app,
        "Modifica",
        true,
        &[
            &Voce::undo(app, Some("Annulla"))?,
            &Voce::redo(app, Some("Ripristina"))?,
            &Voce::separator(app)?,
            &Voce::cut(app, Some("Taglia"))?,
            &Voce::copy(app, Some("Copia"))?,
            &Voce::paste(app, Some("Incolla"))?,
            &Voce::select_all(app, Some("Seleziona tutto"))?,
        ],
    )?;
    let vista = Submenu::with_items(
        app,
        "Vista",
        true,
        &[&Voce::fullscreen(app, Some("Entra/esci da schermo intero"))?],
    )?;
    let finestra = Submenu::with_items(
        app,
        "Finestra",
        true,
        &[
            &Voce::minimize(app, Some("Contrai"))?,
            &Voce::maximize(app, Some("Ingrandisci"))?,
            &Voce::separator(app)?,
            &Voce::close_window(app, Some("Chiudi"))?,
        ],
    )?;
    Menu::with_items(app, &[&applicazione, &archivio, &modifica, &vista, &finestra])
}

/// Avvia l'applicazione.
pub fn avvia() {
    let costruttore = tauri::Builder::default();
    #[cfg(target_os = "macos")]
    let costruttore = costruttore.menu(menu_macos);
    // Aggiornamento integrato dei pacchetti Linux (vedi `aggiornamenti`).
    #[cfg(target_os = "linux")]
    let costruttore = costruttore.plugin(tauri_plugin_updater::Builder::new().build());
    costruttore
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(
            tauri::plugin::Builder::<tauri::Wry, ()>::new("navigazione")
                .on_navigation(|_, url| navigazione_consentita(url))
                .build(),
        )
        .setup(|app| {
            let standard = app
                .path()
                .app_data_dir()
                .unwrap_or_else(|_| std::env::temp_dir().join("rustinvoice"));
            impostazioni::migra_cartella_precedente(&standard);
            let dir_dati = impostazioni::cartella_dati(&standard);
            let portable = dir_dati != standard;
            let primo_avvio = impostazioni::primo_avvio(&dir_dati);
            let _ = std::fs::create_dir_all(&dir_dati);
            let imp = impostazioni::carica(&dir_dati);
            // Sandbox del Mac App Store: accesso alle cartelle scelte in passato.
            segnalibri::riattiva_tutti(&dir_dati);
            segnalibri::aggiorna(&dir_dati, &percorsi_da_ricordare(&imp));
            app.manage(Stato {
                dir_dati,
                dir_standard: standard,
                portable,
                primo_avvio,
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
            apri_allegato,
            apri_cartella,
            mostra_nella_cartella,
            apri_sito,
            copia_in_archivio,
            cartella_archivio,
            foglio_stile_stato,
            foglio_stile_aggiorna,
            foglio_stile_importa,
            foglio_stile_leggi,
            trasforma_standard,
            salva_testo,
            anteprima_allegato,
            byte_allegato,
            cambia_cartella_dati,
            controlla_aggiornamenti,
            installa_aggiornamento,
            riavvia,
            apri_download,
            scrivi_email,
            apri_donazione
        ])
        .run(tauri::generate_context!())
        .expect("errore durante l'avvio dell'applicazione");
}

#[cfg(test)]
mod test {
    use super::{codifica_mailto, nome_con_numero};
    use std::path::Path;

    #[test]
    fn mailto_codificato() {
        assert_eq!(codifica_mailto("Ciao & grazie"), "Ciao%20%26%20grazie");
        assert_eq!(codifica_mailto("a\nb"), "a%0D%0Ab");
        assert_eq!(codifica_mailto("è"), "%C3%A8");
    }

    #[test]
    fn numero_prima_delle_estensioni() {
        assert_eq!(
            nome_con_numero(Path::new("fatture.zip"), 1),
            "fatture_1.zip"
        );
        assert_eq!(
            nome_con_numero(Path::new("IT01_A.xml.p7m"), 2),
            "IT01_A_2.xml.p7m"
        );
        assert_eq!(nome_con_numero(Path::new("senza"), 3), "senza_3");
        assert_eq!(nome_con_numero(Path::new(".nascosto"), 1), ".nascosto_1");
    }
}
