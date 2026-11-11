// Copyright (c) 2026 Andrea Re - info@digitare.net
// Autore: Andrea Re @ 2026 - info@digitare.net
//
// Apertura di un file qualsiasi della cartella di lavoro e riconoscimento
// del suo contenuto: fattura (anche firmata), messaggio del Sistema di
// Interscambio (ricevute, notifiche, metadati) oppure file estraneo.
//
// Il riconoscimento si basa sul contenuto e non sul nome del file: così
// ricevute e metadati non vengono mai scambiati per fatture e non
// interrompono il caricamento dell'archivio.

use crate::errore::{Errore, Risultato};
use crate::modello::FatturaElettronica;
use crate::p7m;
use crate::parser::{leggi_fattura, RADICE_ORDINARIA, RADICE_SEMPLIFICATA};
use crate::xml::{analizza, decodifica, figli, figlio, testo, testo_percorso};
use serde::{Deserialize, Serialize};
use std::path::Path;

/// Tipo di messaggio del Sistema di Interscambio.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TipoMessaggioSdi {
    /// RC - ricevuta di consegna.
    RicevutaConsegna,
    /// NS - notifica di scarto.
    NotificaScarto,
    /// MC - notifica di mancata consegna.
    MancataConsegna,
    /// NE - notifica di esito (PA).
    NotificaEsito,
    /// DT - notifica di decorrenza termini (PA).
    DecorrenzaTermini,
    /// AT - attestazione di avvenuta trasmissione con impossibilità di recapito.
    AttestazioneTrasmissione,
    /// MT - file dei metadati.
    Metadati,
    /// EC - notifica di esito del committente.
    EsitoCommittente,
    /// SE - scarto dell'esito del committente.
    ScartoEsitoCommittente,
}

impl TipoMessaggioSdi {
    /// Sigla usata nei nomi file (`_RC_`, `_NS_`, ...).
    pub fn sigla(self) -> &'static str {
        match self {
            Self::RicevutaConsegna => "RC",
            Self::NotificaScarto => "NS",
            Self::MancataConsegna => "MC",
            Self::NotificaEsito => "NE",
            Self::DecorrenzaTermini => "DT",
            Self::AttestazioneTrasmissione => "AT",
            Self::Metadati => "MT",
            Self::EsitoCommittente => "EC",
            Self::ScartoEsitoCommittente => "SE",
        }
    }

    /// Descrizione per l'utente.
    pub fn descrizione(self) -> &'static str {
        match self {
            Self::RicevutaConsegna => "Ricevuta di consegna",
            Self::NotificaScarto => "Notifica di scarto",
            Self::MancataConsegna => "Notifica di mancata consegna",
            Self::NotificaEsito => "Notifica di esito",
            Self::DecorrenzaTermini => "Notifica di decorrenza termini",
            Self::AttestazioneTrasmissione => "Attestazione di avvenuta trasmissione",
            Self::Metadati => "File dei metadati",
            Self::EsitoCommittente => "Esito del committente",
            Self::ScartoEsitoCommittente => "Scarto dell'esito del committente",
        }
    }

    fn da_radice(nome: &str) -> Option<Self> {
        Some(match nome {
            "RicevutaConsegna" => Self::RicevutaConsegna,
            "NotificaScarto" => Self::NotificaScarto,
            "NotificaMancataConsegna" => Self::MancataConsegna,
            "NotificaEsito" => Self::NotificaEsito,
            "NotificaDecorrenzaTermini" => Self::DecorrenzaTermini,
            "AttestazioneTrasmissioneFattura" => Self::AttestazioneTrasmissione,
            "FileMetadati" => Self::Metadati,
            "NotificaEsitoCommittente" => Self::EsitoCommittente,
            "ScartoEsitoCommittente" => Self::ScartoEsitoCommittente,
            _ => return None,
        })
    }
}

/// Errore segnalato dal Sistema di Interscambio in una notifica di scarto.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ErroreSdi {
    pub codice: Option<String>,
    pub descrizione: Option<String>,
    pub suggerimento: Option<String>,
}

/// Dati essenziali di un messaggio SdI, utili per collegarlo alla fattura.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MessaggioSdi {
    pub tipo: TipoMessaggioSdi,
    pub identificativo_sdi: Option<String>,
    /// Nome del file fattura a cui il messaggio si riferisce.
    pub nome_file: Option<String>,
    pub data_ora_ricezione: Option<String>,
    pub data_ora_consegna: Option<String>,
    /// Esito (es. `EC01` accettazione, `EC02` rifiuto).
    pub esito: Option<String>,
    pub descrizione: Option<String>,
    pub errori: Vec<ErroreSdi>,
}

/// Informazioni sulla busta firmata da cui proviene il documento.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InfoBusta {
    /// Numero di firme annidate.
    pub livelli: usize,
    /// Certificati inclusi, in DER (per la successiva verifica).
    #[serde(skip)]
    pub certificati: Vec<Vec<u8>>,
}

/// Contenuto riconosciuto di un file.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "tipo", content = "dati")]
pub enum Contenuto {
    Fattura(Box<FatturaElettronica>),
    MessaggioSdi(MessaggioSdi),
}

/// Documento aperto: contenuto interpretato più il testo XML originale,
/// che serve per applicare i fogli di stile.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Documento {
    pub contenuto: Contenuto,
    pub busta: Option<InfoBusta>,
    #[serde(skip)]
    pub xml: String,
}

impl Documento {
    /// La fattura contenuta, se il documento è una fattura.
    pub fn fattura(&self) -> Option<&FatturaElettronica> {
        match &self.contenuto {
            Contenuto::Fattura(f) => Some(f),
            Contenuto::MessaggioSdi(_) => None,
        }
    }

    /// Indica se il documento proveniva da una busta firmata.
    pub fn firmato(&self) -> bool {
        self.busta.is_some()
    }
}

/// Apre un documento dai suoi byte (XML in chiaro o busta .p7m).
pub fn apri_byte(byte: &[u8]) -> Risultato<Documento> {
    let (xml_byte, busta) = if p7m::sembra_p7m(byte) {
        let b = p7m::estrai(byte)?;
        (
            b.contenuto,
            Some(InfoBusta {
                livelli: b.livelli,
                certificati: b.certificati,
            }),
        )
    } else {
        (byte.to_vec(), None)
    };

    let xml = decodifica(&xml_byte)?;
    let doc = analizza(&xml)?;
    let radice = doc.root_element();
    let nome = radice.tag_name().name();

    let contenuto = if nome == RADICE_ORDINARIA || nome == RADICE_SEMPLIFICATA {
        Contenuto::Fattura(Box::new(leggi_fattura(&doc)?))
    } else if let Some(tipo) = TipoMessaggioSdi::da_radice(nome) {
        Contenuto::MessaggioSdi(leggi_messaggio(tipo, radice))
    } else {
        return Err(Errore::DocumentoNonRiconosciuto(nome.to_owned()));
    };
    drop(doc);
    Ok(Documento {
        contenuto,
        busta,
        xml,
    })
}

/// Apre un documento da file.
pub fn apri_file(percorso: impl AsRef<Path>) -> Risultato<Documento> {
    let byte = std::fs::read(percorso)?;
    apri_byte(&byte)
}

fn leggi_messaggio(tipo: TipoMessaggioSdi, radice: roxmltree::Node<'_, '_>) -> MessaggioSdi {
    let mut errori = Vec::new();
    if let Some(lista) = figlio(radice, "ListaErrori") {
        for e in figli(lista, "Errore") {
            errori.push(ErroreSdi {
                codice: testo(e, "Codice"),
                descrizione: testo(e, "Descrizione"),
                suggerimento: testo(e, "Suggerimento"),
            });
        }
    }
    MessaggioSdi {
        tipo,
        identificativo_sdi: testo(radice, "IdentificativoSdI"),
        nome_file: testo(radice, "NomeFile"),
        data_ora_ricezione: testo(radice, "DataOraRicezione"),
        data_ora_consegna: testo(radice, "DataOraConsegna"),
        esito: testo(radice, "Esito")
            .or_else(|| testo_percorso(radice, &["EsitoCommittente", "Esito"])),
        descrizione: testo(radice, "Descrizione").or_else(|| testo(radice, "Note")),
        errori,
    }
}

/// Parti del nome di un file secondo la convenzione del Sistema di
/// Interscambio: `IT01234567890_0A1B2.xml.p7m` per la fattura,
/// `IT01234567890_0A1B2_RC_001.xml` per i messaggi.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NomeFileSdi {
    pub paese: String,
    pub codice: String,
    pub progressivo: String,
    /// Sigla del messaggio (`RC`, `NS`, `MT`, ...) se non è una fattura.
    pub sigla_messaggio: Option<String>,
    pub firmato: bool,
}

/// Analizza un nome file; restituisce `None` se non segue la convenzione.
pub fn analizza_nome_file(nome: &str) -> Option<NomeFileSdi> {
    // Toglie un suffisso ignorando maiuscole/minuscole.
    fn togli<'a>(s: &'a str, suffisso: &str) -> (&'a str, bool) {
        let n = suffisso.len();
        if s.len() >= n
            && s.is_char_boundary(s.len() - n)
            && s[s.len() - n..].eq_ignore_ascii_case(suffisso)
        {
            (&s[..s.len() - n], true)
        } else {
            (s, false)
        }
    }
    let (mut base, mut firmato) = togli(nome, ".p7m");
    // Buste annidate: `.xml.p7m.p7m`.
    while let (b, true) = togli(base, ".p7m") {
        base = b;
        firmato = true;
    }
    let (base, _) = togli(base, ".xml");
    let parti: Vec<&str> = base.split('_').collect();
    if parti.len() < 2 || parti[0].len() < 3 {
        return None;
    }
    let paese = parti[0].get(..2)?;
    if !paese.chars().all(|c| c.is_ascii_alphabetic()) {
        return None;
    }
    let codice = parti[0].get(2..)?;
    if !codice.chars().all(|c| c.is_ascii_alphanumeric()) {
        return None;
    }
    let sigla_messaggio = if parti.len() >= 4 {
        Some(parti[2].to_uppercase())
    } else {
        None
    };
    Some(NomeFileSdi {
        paese: paese.to_uppercase(),
        codice: codice.to_owned(),
        progressivo: parti[1].to_owned(),
        sigla_messaggio,
        firmato,
    })
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn nome_file_fattura_e_ricevuta() {
        let f = analizza_nome_file("IT01234567890_0A1B2.xml.p7m").unwrap();
        assert_eq!(f.codice, "01234567890");
        assert!(f.firmato);
        assert!(f.sigla_messaggio.is_none());
        let r = analizza_nome_file("IT01234567890_0A1B2_RC_001.xml").unwrap();
        assert_eq!(r.sigla_messaggio.as_deref(), Some("RC"));
        assert!(analizza_nome_file("fattura.xml").is_none());
    }

    #[test]
    fn riconosce_ricevuta_di_consegna() {
        let xml = r#"<?xml version="1.0"?>
<ns3:RicevutaConsegna xmlns:ns3="http://ivaservizi.agenziaentrate.gov.it/docs/xsd/fattura/messaggi/v1.0" versione="1.0">
  <IdentificativoSdI>111</IdentificativoSdI>
  <NomeFile>IT01234567890_00001.xml</NomeFile>
  <DataOraRicezione>2026-11-03T10:00:00.000+01:00</DataOraRicezione>
  <DataOraConsegna>2026-11-03T10:05:00.000+01:00</DataOraConsegna>
</ns3:RicevutaConsegna>"#;
        let d = apri_byte(xml.as_bytes()).unwrap();
        match d.contenuto {
            Contenuto::MessaggioSdi(m) => {
                assert_eq!(m.tipo, TipoMessaggioSdi::RicevutaConsegna);
                assert_eq!(m.nome_file.as_deref(), Some("IT01234567890_00001.xml"));
            }
            _ => panic!("atteso un messaggio SdI"),
        }
    }

    #[test]
    fn rifiuta_xml_estraneo() {
        assert!(matches!(
            apri_byte(b"<Ordine/>"),
            Err(Errore::DocumentoNonRiconosciuto(_))
        ));
    }
}
