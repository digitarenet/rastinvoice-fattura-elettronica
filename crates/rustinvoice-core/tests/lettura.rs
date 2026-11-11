// Copyright (c) 2026 Andrea Re - info@digitare.net
// Autore: Andrea Re @ 2026 - info@digitare.net
//
// Test di integrazione sulla lettura delle fatture di prova in `tests/dati`.

use rust_decimal::Decimal;
use rustinvoice_core::modello::Formato;
use rustinvoice_core::{apri_byte, apri_file, codici, Contenuto, FatturaElettronica};
use std::path::PathBuf;
use std::str::FromStr;

fn dati(nome: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/dati")
        .join(nome)
}

fn fattura(nome: &str) -> FatturaElettronica {
    let doc = apri_file(dati(nome)).expect("file di prova leggibile");
    match doc.contenuto {
        Contenuto::Fattura(f) => *f,
        Contenuto::MessaggioSdi(_) => panic!("attesa una fattura"),
    }
}

fn d(s: &str) -> Decimal {
    Decimal::from_str(s).unwrap()
}

#[test]
fn ordinaria_intestazione() {
    let f = fattura("ordinaria_fpr12.xml");
    assert_eq!(f.formato, Formato::Fpr12);
    assert!(f.avvisi.is_empty(), "avvisi inattesi: {:?}", f.avvisi);

    let h = &f.intestazione;
    assert_eq!(
        h.dati_trasmissione.progressivo_invio.as_deref(),
        Some("0A1B2")
    );
    assert_eq!(
        h.dati_trasmissione.pec_destinatario.as_deref(),
        Some("cliente@pec.esempio.it")
    );

    let c = &h.cedente_prestatore;
    // Gli spazi multipli della ragione sociale vengono normalizzati.
    assert_eq!(c.nome_visualizzato(), "Fornitore Esempio S.r.l.");
    assert_eq!(c.chiave().as_deref(), Some("IT01234567890"));
    assert_eq!(c.regime_fiscale.as_deref(), Some("RF01"));
    assert_eq!(
        c.sede.as_ref().unwrap().su_una_riga(),
        "Via Roma 1, 10100 Torino (TO)"
    );
    assert_eq!(
        c.iscrizione_rea.as_ref().unwrap().capitale_sociale,
        Some(d("10000.00"))
    );
    assert_eq!(
        h.cessionario_committente.chiave().as_deref(),
        Some("IT09876543210")
    );
}

#[test]
fn ordinaria_corpo() {
    let f = fattura("ordinaria_fpr12.xml");
    assert_eq!(f.corpi.len(), 1);
    let corpo = &f.corpi[0];
    let doc = &corpo.dati_generali.documento;
    assert_eq!(doc.tipo_documento.as_deref(), Some("TD01"));
    assert_eq!(doc.numero.as_deref(), Some("2026/1184"));
    assert_eq!(doc.data.unwrap().to_string(), "2026-11-03");
    assert_eq!(doc.causale.len(), 2);
    assert!(doc.bollo.as_ref().unwrap().virtuale);

    // Totale assente: ricavato dal riepilogo.
    assert_eq!(doc.importo_totale_documento, None);
    assert_eq!(corpo.totale(), Some(d("1512.80")));

    assert_eq!(corpo.linee.len(), 2);
    let l1 = &corpo.linee[0];
    assert_eq!(l1.codici_articolo[0].valore.as_deref(), Some("CONS-01"));
    assert_eq!(
        l1.altri_dati_gestionali[0].riferimento_testo.as_deref(),
        Some("R&D")
    );
    let l2 = &corpo.linee[1];
    assert_eq!(l2.sconti_maggiorazioni[0].percentuale, Some(d("20.00")));

    assert_eq!(corpo.dati_generali.ddt[0].riferimento_numero_linea, vec![2]);
    assert_eq!(
        corpo.dati_generali.ordini_acquisto[0].codice_cig.as_deref(),
        Some("Z123456789")
    );

    let pagamento = &corpo.pagamenti[0].dettagli[0];
    assert_eq!(
        pagamento.iban.as_deref(),
        Some("IT60X0542811101000000123456")
    );
    assert_eq!(corpo.prima_scadenza().unwrap().to_string(), "2026-12-03");

    let allegato = &corpo.allegati[0];
    assert_eq!(allegato.nome.as_deref(), Some("dettaglio.txt"));
    assert_eq!(allegato.dati, b"Dettaglio ore: 40 ");
}

#[test]
fn lotto_in_windows_1252_con_anomalie() {
    let f = fattura("lotto_fpa12_latin1.xml");
    assert_eq!(f.formato, Formato::Fpa12);
    assert_eq!(f.corpi.len(), 2, "un lotto contiene due documenti");

    let cedente = &f.intestazione.cedente_prestatore;
    assert_eq!(cedente.nome_visualizzato(), "Ing. Mario Rossi");
    assert_eq!(cedente.regime_fiscale.as_deref(), Some("RF19"));

    // Caratteri accentati e trattino lungo decodificati correttamente.
    let cessionario = &f.intestazione.cessionario_committente;
    assert_eq!(
        cessionario.nome_visualizzato(),
        "Comune di Esempio – Ufficio Tecnico"
    );
    assert_eq!(cessionario.chiave().as_deref(), Some("CF:80000000000"));
    assert!(f.corpi[0].linee[0]
        .descrizione
        .as_deref()
        .unwrap()
        .contains("qualità"));

    let cassa = &f.corpi[0].dati_generali.documento.casse_previdenziali[0];
    assert_eq!(cassa.tipo.as_deref(), Some("TC04"));
    assert_eq!(f.corpi[0].totale(), Some(d("1040.00")));

    // Seconda fattura del lotto: nota di credito collegata.
    let nc = &f.corpi[1].dati_generali;
    assert!(codici::e_nota_di_credito(
        nc.documento.tipo_documento.as_deref().unwrap()
    ));
    assert_eq!(nc.fatture_collegate[0].id_documento.as_deref(), Some("12"));

    // Virgola decimale tollerata e data scritta male: entrambe segnalate.
    assert_eq!(f.corpi[1].linee[0].prezzo_unitario, Some(d("100.00")));
    assert!(f.avvisi.iter().any(|a| a.contains("separatore decimale")));
    assert!(f.avvisi.iter().any(|a| a.contains("DataScadenzaPagamento")));
    assert_eq!(f.corpi[1].prima_scadenza(), None);
}

#[test]
fn semplificata() {
    let f = fattura("semplificata_fsm10.xml");
    assert_eq!(f.formato, Formato::Fsm10);
    let h = &f.intestazione;
    assert_eq!(
        h.cedente_prestatore.nome_visualizzato(),
        "Bar Esempio di Bianchi Anna"
    );
    assert_eq!(h.cessionario_committente.nome_visualizzato(), "Luigi Verdi");
    assert_eq!(
        h.cessionario_committente.chiave().as_deref(),
        Some("CF:VRDLGU85M01L219Y")
    );
    let corpo = &f.corpi[0];
    assert_eq!(corpo.linee.len(), 2);
    assert_eq!(corpo.linee[0].imposta, Some(d("4.09")));
    assert_eq!(corpo.linee[1].aliquota_iva, Some(d("22.00")));
    // Totale della semplificata: somma degli importi più le imposte indicate.
    assert_eq!(corpo.totale(), Some(d("52.09")));
}

#[test]
fn fattura_dentro_busta_p7m() {
    // Busta minima costruita a mano attorno al file di prova.
    let xml = std::fs::read(dati("ordinaria_fpr12.xml")).unwrap();
    let p7m = busta_minima(&xml);
    let doc = apri_byte(&p7m).unwrap();
    assert!(doc.firmato());
    assert_eq!(doc.busta.as_ref().unwrap().livelli, 1);
    let f = doc.fattura().unwrap();
    assert_eq!(
        f.corpi[0].dati_generali.documento.numero.as_deref(),
        Some("2026/1184")
    );
}

#[test]
fn serializzazione_json() {
    let doc = apri_file(dati("ordinaria_fpr12.xml")).unwrap();
    let json = serde_json::to_value(&doc).unwrap();
    assert_eq!(json["contenuto"]["tipo"], "Fattura");
    // Gli importi viaggiano come stringhe per non perdere precisione.
    assert_eq!(
        json["contenuto"]["dati"]["corpi"][0]["riepiloghi"][0]["imposta"],
        "272.80"
    );
}

/// Costruisce una busta SignedData senza firmatari, sufficiente per
/// verificare l'estrazione del contenuto.
fn busta_minima(contenuto: &[u8]) -> Vec<u8> {
    fn tlv(tag: u8, c: &[u8]) -> Vec<u8> {
        let mut v = vec![tag];
        let n = c.len();
        if n < 128 {
            v.push(n as u8);
        } else if n < 65536 {
            v.extend_from_slice(&[0x82, (n >> 8) as u8, n as u8]);
        } else {
            v.extend_from_slice(&[0x83, (n >> 16) as u8, (n >> 8) as u8, n as u8]);
        }
        v.extend_from_slice(c);
        v
    }
    let oid_data = tlv(
        0x06,
        &[0x2A, 0x86, 0x48, 0x86, 0xF7, 0x0D, 0x01, 0x07, 0x01],
    );
    let oid_signed = tlv(
        0x06,
        &[0x2A, 0x86, 0x48, 0x86, 0xF7, 0x0D, 0x01, 0x07, 0x02],
    );
    let encap = tlv(0x30, &[oid_data, tlv(0xA0, &tlv(0x04, contenuto))].concat());
    let signed = tlv(
        0x30,
        &[tlv(0x02, &[1]), tlv(0x31, &[]), encap, tlv(0x31, &[])].concat(),
    );
    tlv(0x30, &[oid_signed, tlv(0xA0, &signed)].concat())
}
