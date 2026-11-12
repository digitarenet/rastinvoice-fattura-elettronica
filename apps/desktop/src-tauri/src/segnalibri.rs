// Copyright (c) 2026 Andrea Re - info@digitare.net
// Autore: Andrea Re @ 2026 - info@digitare.net
//
// Segnalibri di sicurezza (security-scoped bookmark) per la sandbox del Mac
// App Store.
//
// Nella sandbox l'app può leggere solo i file e le cartelle che l'utente ha
// scelto nella sessione in corso (finestra Apri, trascinamento). Per ritrovare
// le cartelle d'archivio e i file singoli dopo il riavvio, per ognuno si salva
// un segnalibro in `segnalibri.json` nella cartella dati e lo si riattiva a
// ogni avvio. Fuori da macOS il modulo non fa nulla.

use std::path::Path;

#[cfg(target_os = "macos")]
mod mac {
    use objc2::runtime::Bool;
    use objc2_foundation::{
        NSData, NSString, NSURLBookmarkCreationOptions, NSURLBookmarkResolutionOptions, NSURL,
    };
    use std::collections::BTreeMap;
    use std::path::Path;

    const NOME_FILE: &str = "segnalibri.json";

    /// Segnalibri salvati: percorso -> dati del segnalibro in esadecimale.
    type Elenco = BTreeMap<String, String>;

    fn carica(dir_dati: &Path) -> Elenco {
        std::fs::read_to_string(dir_dati.join(NOME_FILE))
            .ok()
            .and_then(|t| serde_json::from_str(&t).ok())
            .unwrap_or_default()
    }

    fn salva(dir_dati: &Path, elenco: &Elenco) {
        if let Ok(testo) = serde_json::to_string_pretty(elenco) {
            let _ = std::fs::write(dir_dati.join(NOME_FILE), testo);
        }
    }

    fn esadecimale(byte: &[u8]) -> String {
        byte.iter().map(|b| format!("{b:02x}")).collect()
    }

    fn da_esadecimale(testo: &str) -> Option<Vec<u8>> {
        (0..testo.len())
            .step_by(2)
            .map(|i| u8::from_str_radix(testo.get(i..i + 2)?, 16).ok())
            .collect()
    }

    /// Crea il segnalibro di un percorso a cui l'app ha accesso adesso.
    fn crea(percorso: &str) -> Option<String> {
        let url = NSURL::fileURLWithPath(&NSString::from_str(percorso));
        let dati = url
            .bookmarkDataWithOptions_includingResourceValuesForKeys_relativeToURL_error(
                NSURLBookmarkCreationOptions::WithSecurityScope,
                None,
                None,
            )
            .ok()?;
        Some(esadecimale(&dati.to_vec()))
    }

    /// Riattiva un segnalibro. Restituisce `true` se va ricreato perché
    /// scaduto (per esempio dopo che il file è stato spostato).
    fn riattiva(dati: &str) -> Option<bool> {
        let dati = NSData::with_bytes(&da_esadecimale(dati)?);
        let mut scaduto = Bool::NO;
        // SAFETY: `scaduto` è un puntatore valido per tutta la chiamata.
        let url = unsafe {
            NSURL::URLByResolvingBookmarkData_options_relativeToURL_bookmarkDataIsStale_error(
                &dati,
                NSURLBookmarkResolutionOptions::WithSecurityScope,
                None,
                &mut scaduto,
            )
        }
        .ok()?;
        // L'accesso resta attivo finché l'app è aperta: le cartelle d'archivio
        // si rileggono in qualsiasi momento.
        // SAFETY: chiamata senza requisiti su un URL valido.
        unsafe { url.startAccessingSecurityScopedResource() }.then_some(scaduto.as_bool())
    }

    pub fn riattiva_tutti(dir_dati: &Path) {
        let mut elenco = carica(dir_dati);
        let mut cambiato = false;
        for (percorso, dati) in elenco.iter_mut() {
            if riattiva(dati) == Some(true) {
                if let Some(nuovo) = crea(percorso) {
                    *dati = nuovo;
                    cambiato = true;
                }
            }
        }
        if cambiato {
            salva(dir_dati, &elenco);
        }
    }

    pub fn aggiorna(dir_dati: &Path, percorsi: &[&String]) {
        let vecchio = carica(dir_dati);
        let mut nuovo = Elenco::new();
        for &percorso in percorsi {
            let dati = vecchio.get(percorso).cloned().or_else(|| crea(percorso));
            if let Some(dati) = dati {
                nuovo.insert(percorso.clone(), dati);
            }
        }
        if nuovo != vecchio {
            salva(dir_dati, &nuovo);
        }
    }

    #[cfg(test)]
    mod test {
        #[test]
        fn esadecimale_andata_e_ritorno() {
            let byte = [0u8, 1, 0x7f, 0xab, 0xff];
            let testo = super::esadecimale(&byte);
            assert_eq!(testo, "00017fabff");
            assert_eq!(super::da_esadecimale(&testo).unwrap(), byte);
        }
    }
}

/// All'avvio riattiva i segnalibri salvati, così le cartelle d'archivio e i
/// file singoli sono di nuovo leggibili nella sandbox.
pub fn riattiva_tutti(dir_dati: &Path) {
    #[cfg(target_os = "macos")]
    mac::riattiva_tutti(dir_dati);
    #[cfg(not(target_os = "macos"))]
    let _ = dir_dati;
}

/// Allinea i segnalibri ai percorsi indicati: crea quelli nuovi (l'utente li
/// ha appena scelti, quindi l'app vi ha accesso) e toglie quelli non più usati.
pub fn aggiorna(dir_dati: &Path, percorsi: &[&String]) {
    #[cfg(target_os = "macos")]
    mac::aggiorna(dir_dati, percorsi);
    #[cfg(not(target_os = "macos"))]
    let _ = (dir_dati, percorsi);
}
