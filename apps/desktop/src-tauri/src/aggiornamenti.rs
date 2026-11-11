// Copyright (c) 2026 Andrea Re - info@digitare.net
// Autore: Andrea Re @ 2026 - info@digitare.net
//
// Controllo delle nuove versioni pubblicate su GitHub. È facoltativo e
// disattivato in modo predefinito: l'applicazione non si collega a Internet
// se l'utente non lo chiede nelle impostazioni. Non viene inviato alcun dato
// oltre alla normale richiesta HTTP.

use serde::Serialize;
use std::io::Read;

/// Indirizzo dell'ultima release pubblicata (le bozze non compaiono).
const URL_ULTIMA: &str = "https://api.github.com/repos/digitarenet/rastinvoice/releases/latest";
/// Pagina delle release da aprire per scaricare la nuova versione.
pub const URL_RELEASE: &str = "https://github.com/digitarenet/rastinvoice/releases/latest";
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
