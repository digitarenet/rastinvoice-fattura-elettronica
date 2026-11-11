// Copyright (c) 2026 Andrea Re - info@digitare.net
// Autore: Andrea Re @ 2026 - info@digitare.net
//
// Controlli formali e di calcolo sulla fattura.
//
// Riprendono i principali controlli descritti nelle specifiche pubbliche del
// Sistema di Interscambio (codici errore 004xx) e aggiungono alcune verifiche
// formali su partite IVA, codici fiscali e IBAN. Non sostituiscono i controlli
// dello SdI: servono a segnalare subito le anomalie più comuni.

use rust_decimal::Decimal;
use serde::Serialize;

use crate::modello::{Corpo, FatturaElettronica, Formato, IdFiscale, Soggetto};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Gravita {
    /// Lo SdI scarterebbe il file o il dato è sicuramente errato.
    Errore,
    /// Dato sospetto o differenza minima da verificare.
    Avviso,
}

#[derive(Debug, Clone, Serialize)]
pub struct Controllo {
    pub gravita: Gravita,
    /// Codice errore SdI corrispondente, quando esiste.
    pub codice: Option<String>,
    pub messaggio: String,
    /// Corpo (documento del lotto) a cui si riferisce; `None` per l'intestazione.
    pub corpo: Option<usize>,
}

/// Esegue tutti i controlli sulla fattura.
pub fn controlla(f: &FatturaElettronica) -> Vec<Controllo> {
    let mut c = Vec::new();
    intestazione(f, &mut c);
    for (i, corpo) in f.corpi.iter().enumerate() {
        controlla_corpo(f.formato, i, corpo, &mut c);
    }
    c
}

/// Controlli dell'intestazione più quelli di un solo corpo.
pub fn controlla_documento(f: &FatturaElettronica, corpo: usize) -> Vec<Controllo> {
    controlla(f)
        .into_iter()
        .filter(|c| c.corpo.is_none() || c.corpo == Some(corpo))
        .collect()
}

fn aggiungi(
    v: &mut Vec<Controllo>,
    gravita: Gravita,
    codice: Option<&str>,
    corpo: Option<usize>,
    messaggio: String,
) {
    v.push(Controllo {
        gravita,
        codice: codice.map(str::to_owned),
        messaggio,
        corpo,
    });
}

fn intestazione(f: &FatturaElettronica, v: &mut Vec<Controllo>) {
    let h = &f.intestazione;
    let t = &h.dati_trasmissione;

    if let Some(cd) = t.codice_destinatario.as_deref().map(str::trim) {
        let atteso = match f.formato {
            Formato::Fpa12 => Some(6),
            Formato::Fpr12 | Formato::Fsm10 => Some(7),
            Formato::Altro => None,
        };
        if let Some(n) = atteso {
            if cd.chars().count() != n {
                aggiungi(
                    v,
                    Gravita::Errore,
                    Some("00427"),
                    None,
                    format!("Il codice destinatario \"{cd}\" deve avere {n} caratteri per questo formato di trasmissione"),
                );
            }
        }
        if !cd.chars().all(|c| c.is_ascii_alphanumeric()) {
            aggiungi(
                v,
                Gravita::Errore,
                None,
                None,
                format!("Il codice destinatario \"{cd}\" contiene caratteri non ammessi"),
            );
        }
    } else {
        aggiungi(
            v,
            Gravita::Errore,
            None,
            None,
            "Codice destinatario mancante".into(),
        );
    }

    soggetto("cedente/prestatore", &h.cedente_prestatore, true, v);
    soggetto(
        "cessionario/committente",
        &h.cessionario_committente,
        false,
        v,
    );
    if h.cedente_prestatore.regime_fiscale.is_none() && f.formato != Formato::Fsm10 {
        aggiungi(
            v,
            Gravita::Errore,
            None,
            None,
            "Regime fiscale del cedente/prestatore mancante".into(),
        );
    }
}

fn soggetto(ruolo: &str, s: &Soggetto, obbligo_iva: bool, v: &mut Vec<Controllo>) {
    match &s.id_fiscale_iva {
        Some(id) => partita_iva(ruolo, id, v),
        None if obbligo_iva => aggiungi(
            v,
            Gravita::Errore,
            None,
            None,
            format!("Partita IVA del {ruolo} mancante"),
        ),
        None if s.codice_fiscale.is_none() => aggiungi(
            v,
            Gravita::Errore,
            None,
            None,
            format!("Il {ruolo} non ha né partita IVA né codice fiscale"),
        ),
        None => {}
    }
    if let Some(cf) = s.codice_fiscale.as_deref() {
        if !codice_fiscale_valido(cf) {
            aggiungi(
                v,
                Gravita::Avviso,
                None,
                None,
                format!("Il codice fiscale del {ruolo} \"{cf}\" non supera il controllo formale"),
            );
        }
    }
    if s.denominazione.is_none() && (s.nome.is_none() || s.cognome.is_none()) {
        aggiungi(
            v,
            Gravita::Errore,
            None,
            None,
            format!("Denominazione o nome e cognome del {ruolo} mancanti"),
        );
    }
}

fn partita_iva(ruolo: &str, id: &IdFiscale, v: &mut Vec<Controllo>) {
    if id.paese.eq_ignore_ascii_case("IT") && !partita_iva_valida(&id.codice) {
        aggiungi(
            v,
            Gravita::Avviso,
            None,
            None,
            format!("La partita IVA del {ruolo} \"{}\" non supera il controllo formale (cifra di controllo)", id.codice),
        );
    }
}

/// Chiave di raggruppamento del riepilogo IVA: aliquota normalizzata e natura.
fn chiave(aliquota: Option<Decimal>, natura: Option<&str>) -> (Decimal, String) {
    (
        aliquota.unwrap_or_default().normalize(),
        natura.unwrap_or_default().trim().to_uppercase(),
    )
}

fn euro(d: Decimal) -> String {
    format!("{:.2}", d.round_dp(2))
}

fn controlla_corpo(formato: Formato, i: usize, corpo: &Corpo, v: &mut Vec<Controllo>) {
    let doc = &corpo.dati_generali.documento;
    let n = Some(i);
    if doc
        .numero
        .as_deref()
        .map(str::trim)
        .unwrap_or_default()
        .is_empty()
    {
        aggiungi(
            v,
            Gravita::Errore,
            None,
            n,
            "Numero del documento mancante".into(),
        );
    }
    match doc.data {
        None => aggiungi(
            v,
            Gravita::Errore,
            None,
            n,
            "Data del documento mancante o non valida".into(),
        ),
        Some(d) if d > chrono::Local::now().date_naive() => aggiungi(
            v,
            Gravita::Avviso,
            Some("00403"),
            n,
            format!("La data del documento ({d}) è nel futuro"),
        ),
        _ => {}
    }

    // La fattura semplificata ha una struttura diversa: niente calcoli di riepilogo.
    if formato == Formato::Fsm10 {
        return;
    }

    let cent = Decimal::new(1, 2);
    let uno = Decimal::ONE;
    let mut somme: std::collections::BTreeMap<(Decimal, String), Decimal> = Default::default();

    for (k, l) in corpo.linee.iter().enumerate() {
        let num = l.numero_linea.map(|x| x as usize).unwrap_or(k + 1);
        let aliquota = l.aliquota_iva.unwrap_or_default();
        let natura = l.natura.as_deref().map(str::trim).unwrap_or_default();
        if aliquota.is_zero() && natura.is_empty() {
            aggiungi(
                v,
                Gravita::Errore,
                Some("00400"),
                n,
                format!("Linea {num}: aliquota IVA zero senza natura dell'operazione"),
            );
        }
        if !aliquota.is_zero() && !natura.is_empty() {
            aggiungi(
                v,
                Gravita::Errore,
                Some("00401"),
                n,
                format!("Linea {num}: natura {natura} indicata con aliquota IVA diversa da zero"),
            );
        }

        // PrezzoTotale = (PrezzoUnitario ± sconti/maggiorazioni) × Quantità.
        if let (Some(pu), Some(pt)) = (l.prezzo_unitario, l.prezzo_totale) {
            let mut prezzo = pu;
            for s in &l.sconti_maggiorazioni {
                let segno = if s.tipo.as_deref() == Some("MG") {
                    uno
                } else {
                    -uno
                };
                if let Some(p) = s.percentuale {
                    prezzo += segno * prezzo * p / Decimal::ONE_HUNDRED;
                } else if let Some(imp) = s.importo {
                    prezzo += segno * imp;
                }
            }
            let atteso = prezzo * l.quantita.unwrap_or(uno);
            let diff = (atteso - pt).abs();
            if diff > cent {
                aggiungi(
                    v,
                    Gravita::Errore,
                    Some("00423"),
                    n,
                    format!(
                        "Linea {num}: prezzo totale {} diverso dal calcolato {}",
                        euro(pt),
                        euro(atteso)
                    ),
                );
            }
        }
        if let Some(pt) = l.prezzo_totale {
            *somme
                .entry(chiave(l.aliquota_iva, l.natura.as_deref()))
                .or_default() += pt;
        }
    }

    // I contributi delle casse previdenziali concorrono all'imponibile della loro aliquota.
    for cassa in &doc.casse_previdenziali {
        if let Some(c) = cassa.importo_contributo {
            *somme
                .entry(chiave(cassa.aliquota_iva, cassa.natura.as_deref()))
                .or_default() += c;
        }
    }

    let mut chiavi_riepilogo = std::collections::BTreeSet::new();
    for r in &corpo.riepiloghi {
        let k = chiave(r.aliquota_iva, r.natura.as_deref());
        chiavi_riepilogo.insert(k.clone());
        let etichetta = if k.1.is_empty() {
            format!("aliquota {}%", k.0)
        } else {
            format!("natura {}", k.1)
        };
        let imponibile = r.imponibile_importo.unwrap_or_default();

        // ImponibileImporto = somma dei prezzi totali + spese accessorie + arrotondamento.
        let atteso = somme.get(&k).copied().unwrap_or_default()
            + r.spese_accessorie.unwrap_or_default()
            + r.arrotondamento.unwrap_or_default();
        let diff = (atteso - imponibile).abs();
        if diff > uno {
            aggiungi(
                v,
                Gravita::Errore,
                Some("00422"),
                n,
                format!(
                    "Riepilogo {etichetta}: imponibile {} diverso dalla somma delle linee {}",
                    euro(imponibile),
                    euro(atteso)
                ),
            );
        } else if diff > cent {
            aggiungi(
                v,
                Gravita::Avviso,
                Some("00422"),
                n,
                format!(
                    "Riepilogo {etichetta}: imponibile {} differisce di {} dalla somma delle linee",
                    euro(imponibile),
                    euro(diff)
                ),
            );
        }

        // Imposta = Imponibile × Aliquota.
        if let Some(imposta) = r.imposta {
            let atteso = imponibile * k.0 / Decimal::ONE_HUNDRED;
            let diff = (atteso - imposta).abs();
            if diff > uno {
                aggiungi(
                    v,
                    Gravita::Errore,
                    Some("00421"),
                    n,
                    format!(
                        "Riepilogo {etichetta}: imposta {} diversa dal calcolato {}",
                        euro(imposta),
                        euro(atteso)
                    ),
                );
            } else if diff > cent {
                aggiungi(
                    v,
                    Gravita::Avviso,
                    Some("00421"),
                    n,
                    format!(
                        "Riepilogo {etichetta}: imposta {} differisce di {} dal calcolato",
                        euro(imposta),
                        euro(diff)
                    ),
                );
            }
        }
    }
    for k in somme.keys() {
        if !chiavi_riepilogo.contains(k) {
            let etichetta = if k.1.is_empty() {
                format!("aliquota {}%", k.0)
            } else {
                format!("natura {}", k.1)
            };
            aggiungi(
                v,
                Gravita::Errore,
                Some("00419"),
                n,
                format!("Manca il riepilogo IVA per {etichetta} presente nelle linee"),
            );
        }
    }

    // Totale documento: confronto informativo (lo SdI non lo verifica).
    if let Some(totale) = doc.importo_totale_documento {
        let calcolato: Decimal = corpo
            .riepiloghi
            .iter()
            .map(|r| r.imponibile_importo.unwrap_or_default() + r.imposta.unwrap_or_default())
            .sum::<Decimal>()
            + doc.arrotondamento.unwrap_or_default();
        let sconti: Decimal = doc
            .sconti_maggiorazioni
            .iter()
            .filter_map(|s| {
                s.importo.map(|i| {
                    if s.tipo.as_deref() == Some("MG") {
                        i
                    } else {
                        -i
                    }
                })
            })
            .sum();
        let bollo = doc
            .bollo
            .as_ref()
            .and_then(|b| b.importo)
            .unwrap_or_default();
        // Alcuni gestionali includono il bollo nel totale, altri no: entrambe le forme sono accettate.
        let candidati = [
            calcolato + sconti,
            calcolato + sconti + bollo,
            calcolato,
            calcolato + bollo,
        ];
        if !candidati.iter().any(|c| (*c - totale).abs() <= cent) {
            aggiungi(
                v,
                Gravita::Avviso,
                None,
                n,
                format!(
                    "Totale documento {} diverso da imponibile più imposta ({})",
                    euro(totale),
                    euro(calcolato + sconti)
                ),
            );
        }
    }

    for p in corpo.pagamenti.iter().flat_map(|p| &p.dettagli) {
        if let Some(iban) = p.iban.as_deref() {
            if !iban_valido(iban) {
                aggiungi(
                    v,
                    Gravita::Avviso,
                    None,
                    n,
                    format!("L'IBAN {iban} non supera il controllo formale"),
                );
            }
        }
    }
}

/// Controllo della cifra di una partita IVA italiana (11 cifre).
pub fn partita_iva_valida(piva: &str) -> bool {
    let c: Vec<u32> = piva.trim().chars().filter_map(|c| c.to_digit(10)).collect();
    if c.len() != 11 || piva.trim().len() != 11 {
        return false;
    }
    let mut somma = 0;
    for (i, d) in c.iter().take(10).enumerate() {
        if i % 2 == 0 {
            somma += d;
        } else {
            let x = d * 2;
            somma += if x > 9 { x - 9 } else { x };
        }
    }
    (10 - somma % 10) % 10 == c[10]
}

/// Controllo formale del codice fiscale: 16 caratteri con carattere di
/// controllo, oppure 11 cifre (soggetti diversi dalle persone fisiche).
pub fn codice_fiscale_valido(cf: &str) -> bool {
    let cf = cf.trim().to_uppercase();
    if cf.len() == 11 {
        return partita_iva_valida(&cf);
    }
    if cf.len() != 16 || !cf.chars().all(|c| c.is_ascii_alphanumeric()) {
        return false;
    }
    // Valori dei caratteri in posizione dispari (1ª, 3ª, ...), per cifra o lettera.
    const DISPARI: [u32; 26] = [
        1, 0, 5, 7, 9, 13, 15, 17, 19, 21, 2, 4, 18, 20, 11, 3, 6, 8, 12, 14, 16, 10, 22, 25, 24,
        23,
    ];
    let mut somma = 0;
    for (i, ch) in cf.chars().take(15).enumerate() {
        let indice = if ch.is_ascii_digit() {
            ch as u32 - '0' as u32
        } else {
            ch as u32 - 'A' as u32
        };
        somma += if i % 2 == 0 {
            DISPARI[indice as usize]
        } else {
            indice
        };
    }
    let atteso = char::from_u32('A' as u32 + somma % 26).unwrap_or('?');
    cf.chars().nth(15) == Some(atteso)
}

/// Controllo modulo 97 dell'IBAN.
pub fn iban_valido(iban: &str) -> bool {
    let s: String = iban
        .chars()
        .filter(|c| !c.is_whitespace())
        .collect::<String>()
        .to_uppercase();
    if s.len() < 15 || s.len() > 34 || !s.chars().all(|c| c.is_ascii_alphanumeric()) {
        return false;
    }
    let riordinato = format!("{}{}", &s[4..], &s[..4]);
    let mut resto: u32 = 0;
    for c in riordinato.chars() {
        let v = if c.is_ascii_digit() {
            c as u32 - '0' as u32
        } else {
            c as u32 - 'A' as u32 + 10
        };
        resto = if v >= 10 {
            (resto * 100 + v) % 97
        } else {
            (resto * 10 + v) % 97
        };
    }
    resto == 1
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn identificativi() {
        // Partita IVA di fantasia con cifra di controllo corretta e sbagliata.
        assert!(partita_iva_valida("12345678903"));
        assert!(!partita_iva_valida("12345678904"));
        assert!(codice_fiscale_valido("RSSMRA80A01H501U"));
        assert!(!codice_fiscale_valido("RSSMRA80A01H501X"));
        assert!(iban_valido("IT60 X054 2811 1010 0000 0123 456"));
        assert!(!iban_valido("IT61 X054 2811 1010 0000 0123 456"));
    }

    #[test]
    fn calcoli_su_fattura_di_prova() {
        let xml = include_str!("../tests/dati/ordinaria_fpr12.xml");
        let doc = crate::apri_byte(xml.as_bytes()).unwrap();
        let crate::Contenuto::Fattura(f) = doc.contenuto else {
            panic!()
        };
        let errori: Vec<_> = controlla(&f)
            .into_iter()
            .filter(|c| c.gravita == Gravita::Errore)
            .collect();
        assert!(errori.is_empty(), "{errori:#?}");
    }

    #[test]
    fn rileva_imposta_errata() {
        let xml = include_str!("../tests/dati/ordinaria_fpr12.xml").replacen(
            "<Imposta>272.80</Imposta>",
            "<Imposta>300.00</Imposta>",
            1,
        );
        let doc = crate::apri_byte(xml.as_bytes()).unwrap();
        let crate::Contenuto::Fattura(f) = doc.contenuto else {
            panic!()
        };
        let c = controlla(&f);
        assert!(
            c.iter().any(|x| x.codice.as_deref() == Some("00421")),
            "{c:#?}"
        );
    }
}
