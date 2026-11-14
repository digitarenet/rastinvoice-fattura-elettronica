// Copyright (c) 2026 Andrea Re - info@digitare.net
// Autore: Andrea Re @ 2026 - info@digitare.net
//
// Controllo delle nuove versioni pubblicate su GitHub. È facoltativo e
// disattivato in modo predefinito: l'applicazione non si collega a Internet
// se l'utente non lo chiede nelle impostazioni. Non viene inviato alcun dato
// oltre alla normale richiesta HTTP.
//
// Su Linux, per i pacchetti scaricati da GitHub (AppImage, .deb, .rpm), la
// nuova versione può essere anche scaricata e installata dall'applicazione.
// Il file scaricato viene installato solo se la sua firma corrisponde alla
// chiave pubblica inclusa nel programma in fase di compilazione. Le versioni
// degli store (Microsoft Store, Mac App Store, Flathub, Snap) si aggiornano
// tramite lo store e non usano questo meccanismo.

use serde::Serialize;
use std::io::Read;

/// Indirizzo dell'ultima release pubblicata (le bozze non compaiono).
const URL_ULTIMA: &str = "https://api.github.com/repos/digitarenet/rastinvoice-fattura-elettronica/releases/latest";
/// Pagina del sito da aprire per scaricare la nuova versione: ha i link
/// diretti ai file e agli store, più semplice della pagina di GitHub.
pub const URL_DOWNLOAD: &str = "https://www.rastinvoice.it/#scarica";
/// Descrizione delle versioni pubblicate per l'aggiornamento integrato, con
/// indirizzo e firma di ogni pacchetto.
#[cfg(target_os = "linux")]
const URL_MANIFESTO: &str = "https://github.com/digitarenet/rastinvoice-fattura-elettronica/releases/latest/download/latest.json";
/// Chiave pubblica con cui verificare la firma dei pacchetti. La imposta il
/// workflow di rilascio; senza chiave l'aggiornamento integrato è disattivato
/// e resta solo il collegamento alla pagina di download.
const CHIAVE_PUBBLICA: Option<&str> = option_env!("RASTINVOICE_CHIAVE_AGGIORNAMENTI");
/// Dimensione massima della risposta letta.
const MAX_BYTE: u64 = 1024 * 1024;

/// Esito del controllo.
#[derive(Debug, Clone, Serialize)]
pub struct Esito {
    /// Versione in uso.
    pub attuale: String,
    /// Ultima versione pubblicata.
    pub ultima: String,
    /// `true` se l'ultima versione è più recente di quella in uso.
    pub disponibile: bool,
}

/// Legge l'ultima versione pubblicata e la confronta con quella in uso.
pub fn controlla() -> Result<Esito, String> {
    let risposta = ureq::get(URL_ULTIMA)
        .header("User-Agent", "RastInvoice")
        .header("Accept", "application/vnd.github+json")
        .call()
        .map_err(|e| format!("controllo non riuscito: {e}"))?;
    let mut testo = String::new();
    risposta
        .into_body()
        .into_reader()
        .take(MAX_BYTE)
        .read_to_string(&mut testo)
        .map_err(|e| format!("risposta non leggibile: {e}"))?;
    let json: serde_json::Value =
        serde_json::from_str(&testo).map_err(|e| format!("risposta non valida: {e}"))?;
    let etichetta = json
        .get("tag_name")
        .and_then(|v| v.as_str())
        .ok_or("nessuna versione pubblicata")?;
    let attuale = env!("CARGO_PKG_VERSION").to_string();
    let ultima = etichetta.trim_start_matches(['v', 'V']).to_string();
    let disponibile = piu_recente(&ultima, &attuale);
    Ok(Esito {
        attuale,
        ultima,
        disponibile,
    })
}

/// Modalità di aggiornamento integrato del pacchetto in uso:
/// - `automatico`: AppImage, la nuova versione sostituisce il file senza
///   chiedere nulla e diventa attiva al riavvio;
/// - `pacchetto`: .deb o .rpm, l'installazione chiede la password di
///   amministratore, quindi parte solo su richiesta dell'utente;
/// - stringa vuota: nessun aggiornamento integrato (store, compilazione in
///   proprio, Windows e macOS), si apre la pagina di download.
pub fn integrato() -> &'static str {
    if CHIAVE_PUBBLICA.is_none_or(|c| c.trim().is_empty()) {
        return "";
    }
    #[cfg(target_os = "linux")]
    {
        use tauri::utils::{config::BundleType, platform::bundle_type};
        match bundle_type() {
            Some(BundleType::AppImage) if std::env::var_os("APPIMAGE").is_some() => {
                return "automatico";
            }
            Some(BundleType::Deb | BundleType::Rpm) => return "pacchetto",
            _ => {}
        }
    }
    ""
}

/// Scarica e installa l'ultima versione, se più recente di quella in uso.
/// Restituisce la versione installata, oppure `None` se non c'è nulla da fare.
#[cfg(target_os = "linux")]
pub async fn installa(app: tauri::AppHandle) -> Result<Option<String>, String> {
    use tauri_plugin_updater::UpdaterExt;

    if integrato().is_empty() {
        return Err("aggiornamento integrato non disponibile per questa versione".into());
    }
    let chiave = CHIAVE_PUBBLICA.unwrap_or_default().trim();
    let url = URL_MANIFESTO
        .parse()
        .map_err(|e| format!("indirizzo non valido: {e}"))?;
    let aggiornatore = app
        .updater_builder()
        .pubkey(chiave)
        .endpoints(vec![url])
        .and_then(|b| b.build())
        .map_err(|e| format!("aggiornamento non disponibile: {e}"))?;
    let Some(nuova) = aggiornatore
        .check()
        .await
        .map_err(|e| format!("controllo non riuscito: {e}"))?
    else {
        return Ok(None);
    };
    nuova
        .download_and_install(|_, _| {}, || {})
        .await
        .map_err(|e| format!("installazione non riuscita: {e}"))?;
    Ok(Some(nuova.version))
}

/// Su Windows e macOS gli aggiornamenti passano dagli store.
#[cfg(not(target_os = "linux"))]
pub async fn installa(_app: tauri::AppHandle) -> Result<Option<String>, String> {
    Err("aggiornamento integrato non disponibile per questa versione".into())
}

/// Confronta due versioni nella forma `x.y.z` (eventuali suffissi ignorati).
fn piu_recente(a: &str, b: &str) -> bool {
    let numeri = |v: &str| -> Vec<u64> {
        v.split(['-', '+'])
            .next()
            .unwrap_or("")
            .split('.')
            .map(|p| p.parse().unwrap_or(0))
            .collect()
    };
    let (mut x, mut y) = (numeri(a), numeri(b));
    let n = x.len().max(y.len());
    x.resize(n, 0);
    y.resize(n, 0);
    x > y
}

#[cfg(test)]
mod test {
    use super::piu_recente;

    #[test]
    fn confronto_versioni() {
        assert!(piu_recente("0.10.0", "0.9.3"));
        assert!(piu_recente("1.0", "0.99.99"));
        assert!(!piu_recente("0.9.0", "0.9.0"));
        assert!(!piu_recente("0.9.0-beta", "0.9.0"));
        assert!(!piu_recente("0.8.9", "0.9.0"));
    }
}
