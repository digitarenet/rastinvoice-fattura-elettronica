// Copyright (c) 2026 Andrea Re - info@digitare.net
// Autore: Andrea Re @ 2026 - info@digitare.net
//
// Modello dati della fattura elettronica, ricavato dalle specifiche
// tecniche pubbliche del formato FatturaPA (fattura ordinaria FPA12/FPR12
// e fattura semplificata FSM10). I nomi dei campi seguono quelli dei
// blocchi XML, in forma snake_case, per rendere immediato il confronto con
// la documentazione ufficiale.
//
// Il modello è volutamente "tollerante": quasi tutti i campi sono
// opzionali, perché un visualizzatore deve mostrare anche documenti
// incompleti o non perfettamente conformi. Le anomalie trovate durante la
// lettura finiscono in `FatturaElettronica::avvisi`.

use chrono::NaiveDate;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

/// Formato del documento, dedotto dall'attributo `versione` della radice.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Formato {
    /// Fattura ordinaria verso la Pubblica Amministrazione.
    Fpa12,
    /// Fattura ordinaria verso privati (B2B / B2C).
    Fpr12,
    /// Fattura semplificata.
    Fsm10,
    /// Versioni storiche (1.0 / 1.1) o valore non riconosciuto.
    Altro,
}

/// Identificativo fiscale composto da codice paese e codice.
#[derive(Debug, Clone, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct IdFiscale {
    pub paese: String,
    pub codice: String,
}

impl IdFiscale {
    /// Rappresentazione compatta, ad esempio `IT01234567890`.
    pub fn compatto(&self) -> String {
        format!("{}{}", self.paese, self.codice)
    }
}

/// Indirizzo (sede, stabile organizzazione, luogo di resa, ...).
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Indirizzo {
    pub indirizzo: Option<String>,
    pub numero_civico: Option<String>,
    pub cap: Option<String>,
    pub comune: Option<String>,
    pub provincia: Option<String>,
    pub nazione: Option<String>,
}

impl Indirizzo {
    /// Indirizzo su una riga, nel formato consueto italiano.
    pub fn su_una_riga(&self) -> String {
        let mut parti = Vec::new();
        let via = [self.indirizzo.as_deref(), self.numero_civico.as_deref()]
            .into_iter()
            .flatten()
            .collect::<Vec<_>>()
            .join(" ");
        if !via.is_empty() {
            parti.push(via);
        }
        let mut localita = [self.cap.as_deref(), self.comune.as_deref()]
            .into_iter()
            .flatten()
            .collect::<Vec<_>>()
            .join(" ");
        if let Some(p) = &self.provincia {
            localita.push_str(&format!(" ({p})"));
        }
        let localita = localita.trim().to_owned();
        if !localita.is_empty() {
            parti.push(localita);
        }
        if let Some(n) = &self.nazione {
            if n != "IT" {
                parti.push(n.clone());
            }
        }
        parti.join(", ")
    }
}

/// Recapiti di contatto.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Contatti {
    pub telefono: Option<String>,
    pub fax: Option<String>,
    pub email: Option<String>,
}

/// Dati di iscrizione al Registro delle Imprese.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct IscrizioneRea {
    pub ufficio: Option<String>,
    pub numero_rea: Option<String>,
    pub capitale_sociale: Option<Decimal>,
    /// `SU` socio unico, `SM` più soci.
    pub socio_unico: Option<String>,
    /// `LS` in liquidazione, `LN` non in liquidazione.
    pub stato_liquidazione: Option<String>,
}

/// Soggetto della fattura: cedente/prestatore, cessionario/committente,
/// rappresentante fiscale, terzo intermediario o vettore.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Soggetto {
    pub id_fiscale_iva: Option<IdFiscale>,
    pub codice_fiscale: Option<String>,
    pub denominazione: Option<String>,
    pub nome: Option<String>,
    pub cognome: Option<String>,
    pub titolo: Option<String>,
    pub cod_eori: Option<String>,
    pub albo_professionale: Option<String>,
    pub provincia_albo: Option<String>,
    pub numero_iscrizione_albo: Option<String>,
    pub data_iscrizione_albo: Option<NaiveDate>,
    /// Codice del regime fiscale (RF01 ... RF19), solo per il cedente.
    pub regime_fiscale: Option<String>,
    pub sede: Option<Indirizzo>,
    pub stabile_organizzazione: Option<Indirizzo>,
    pub iscrizione_rea: Option<IscrizioneRea>,
    pub contatti: Option<Contatti>,
    pub riferimento_amministrazione: Option<String>,
}

impl Soggetto {
    /// Nome da mostrare: la denominazione oppure "Nome Cognome".
    pub fn nome_visualizzato(&self) -> String {
        if let Some(d) = &self.denominazione {
            return normalizza_spazi(d);
        }
        let completo = [
            self.titolo.as_deref(),
            self.nome.as_deref(),
            self.cognome.as_deref(),
        ]
        .into_iter()
        .flatten()
        .collect::<Vec<_>>()
        .join(" ");
        normalizza_spazi(&completo)
    }

    /// Chiave stabile per riconoscere lo stesso soggetto in fatture diverse,
    /// anche quando la ragione sociale è scritta in modi differenti.
    /// Priorità: partita IVA (paese + codice), poi codice fiscale.
    pub fn chiave(&self) -> Option<String> {
        if let Some(id) = &self.id_fiscale_iva {
            let codice = id.codice.trim().to_uppercase();
            if !codice.is_empty() {
                return Some(format!("{}{}", id.paese.trim().to_uppercase(), codice));
            }
        }
        self.codice_fiscale
            .as_ref()
            .map(|cf| format!("CF:{}", cf.trim().to_uppercase()))
            .filter(|c| c.len() > 3)
    }
}

/// Riduce sequenze di spazi a uno solo e toglie quelli ai bordi.
fn normalizza_spazi(s: &str) -> String {
    s.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// Dati di trasmissione del file.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct DatiTrasmissione {
    pub id_trasmittente: Option<IdFiscale>,
    pub progressivo_invio: Option<String>,
    pub formato_trasmissione: Option<String>,
    pub codice_destinatario: Option<String>,
    pub pec_destinatario: Option<String>,
    pub contatti_trasmittente: Option<Contatti>,
}

/// Intestazione della fattura (`FatturaElettronicaHeader`).
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Intestazione {
    pub dati_trasmissione: DatiTrasmissione,
    pub cedente_prestatore: Soggetto,
    pub rappresentante_fiscale: Option<Soggetto>,
    pub cessionario_committente: Soggetto,
    pub terzo_intermediario: Option<Soggetto>,
    /// `CC` cessionario/committente, `TZ` terzo.
    pub soggetto_emittente: Option<String>,
}

/// Ritenuta d'acconto o previdenziale.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Ritenuta {
    pub tipo: Option<String>,
    pub importo: Option<Decimal>,
    pub aliquota: Option<Decimal>,
    pub causale_pagamento: Option<String>,
}

/// Imposta di bollo.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Bollo {
    pub virtuale: bool,
    pub importo: Option<Decimal>,
}

/// Contributo alla cassa previdenziale.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct CassaPrevidenziale {
    pub tipo: Option<String>,
    pub aliquota: Option<Decimal>,
    pub importo_contributo: Option<Decimal>,
    pub imponibile: Option<Decimal>,
    pub aliquota_iva: Option<Decimal>,
    pub ritenuta: bool,
    pub natura: Option<String>,
    pub riferimento_amministrazione: Option<String>,
}

/// Sconto (`SC`) o maggiorazione (`MG`).
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct ScontoMaggiorazione {
    pub tipo: Option<String>,
    pub percentuale: Option<Decimal>,
    pub importo: Option<Decimal>,
}

/// Dati generali del documento.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct DatiGeneraliDocumento {
    /// Tipo documento (TD01 ... TD29).
    pub tipo_documento: Option<String>,
    pub divisa: Option<String>,
    pub data: Option<NaiveDate>,
    pub numero: Option<String>,
    pub ritenute: Vec<Ritenuta>,
    pub bollo: Option<Bollo>,
    pub casse_previdenziali: Vec<CassaPrevidenziale>,
    pub sconti_maggiorazioni: Vec<ScontoMaggiorazione>,
    pub importo_totale_documento: Option<Decimal>,
    pub arrotondamento: Option<Decimal>,
    pub causale: Vec<String>,
    pub art73: bool,
}

/// Riferimento a un documento collegato (ordine, contratto, convenzione,
/// ricezione, fattura collegata).
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct DocumentoCorrelato {
    pub riferimento_numero_linea: Vec<u32>,
    pub id_documento: Option<String>,
    pub data: Option<NaiveDate>,
    pub num_item: Option<String>,
    pub codice_commessa_convenzione: Option<String>,
    pub codice_cup: Option<String>,
    pub codice_cig: Option<String>,
}

/// Documento di trasporto collegato.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct DatiDdt {
    pub numero: Option<String>,
    pub data: Option<NaiveDate>,
    pub riferimento_numero_linea: Vec<u32>,
}

/// Dati del trasporto (fattura accompagnatoria).
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct DatiTrasporto {
    pub vettore: Option<Soggetto>,
    pub numero_licenza_guida: Option<String>,
    pub mezzo_trasporto: Option<String>,
    pub causale_trasporto: Option<String>,
    pub numero_colli: Option<String>,
    pub descrizione: Option<String>,
    pub unita_misura_peso: Option<String>,
    pub peso_lordo: Option<Decimal>,
    pub peso_netto: Option<Decimal>,
    pub data_ora_ritiro: Option<String>,
    pub data_inizio_trasporto: Option<NaiveDate>,
    pub tipo_resa: Option<String>,
    pub indirizzo_resa: Option<Indirizzo>,
    pub data_ora_consegna: Option<String>,
}

/// Fattura principale, per le fatture accessorie degli autotrasportatori.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct FatturaPrincipale {
    pub numero: Option<String>,
    pub data: Option<NaiveDate>,
}

/// Blocco `DatiGenerali` del corpo.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct DatiGenerali {
    pub documento: DatiGeneraliDocumento,
    pub ordini_acquisto: Vec<DocumentoCorrelato>,
    pub contratti: Vec<DocumentoCorrelato>,
    pub convenzioni: Vec<DocumentoCorrelato>,
    pub ricezioni: Vec<DocumentoCorrelato>,
    pub fatture_collegate: Vec<DocumentoCorrelato>,
    /// Numeri di fase dello stato avanzamento lavori.
    pub sal: Vec<String>,
    pub ddt: Vec<DatiDdt>,
    pub trasporto: Option<DatiTrasporto>,
    pub fattura_principale: Option<FatturaPrincipale>,
    /// Solo fattura semplificata: documento rettificato.
    pub fattura_rettificata: Option<FatturaPrincipale>,
}

/// Codice articolo (EAN, SSC, codice del fornitore, ...).
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct CodiceArticolo {
    pub tipo: Option<String>,
    pub valore: Option<String>,
}

/// Dato gestionale aggiuntivo di riga. Le codifiche standard concordate
/// tra produttori di software usano `TipoDato` con prefisso `Asw`.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct AltroDatoGestionale {
    pub tipo_dato: Option<String>,
    pub riferimento_testo: Option<String>,
    pub riferimento_numero: Option<Decimal>,
    pub riferimento_data: Option<NaiveDate>,
}

/// Riga di dettaglio (`DettaglioLinee`) o, per la semplificata, voce di
/// `DatiBeniServizi`.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Linea {
    pub numero_linea: Option<u32>,
    /// `SC` sconto, `PR` premio, `AB` abbuono, `AC` spesa accessoria.
    pub tipo_cessione_prestazione: Option<String>,
    pub codici_articolo: Vec<CodiceArticolo>,
    pub descrizione: Option<String>,
    pub quantita: Option<Decimal>,
    pub unita_misura: Option<String>,
    pub data_inizio_periodo: Option<NaiveDate>,
    pub data_fine_periodo: Option<NaiveDate>,
    pub prezzo_unitario: Option<Decimal>,
    pub sconti_maggiorazioni: Vec<ScontoMaggiorazione>,
    pub prezzo_totale: Option<Decimal>,
    pub aliquota_iva: Option<Decimal>,
    /// Solo fattura semplificata: imposta indicata sulla voce.
    pub imposta: Option<Decimal>,
    pub ritenuta: bool,
    pub natura: Option<String>,
    pub riferimento_amministrazione: Option<String>,
    pub riferimento_normativo: Option<String>,
    pub altri_dati_gestionali: Vec<AltroDatoGestionale>,
}

/// Riepilogo per aliquota IVA o natura.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Riepilogo {
    pub aliquota_iva: Option<Decimal>,
    pub natura: Option<String>,
    pub spese_accessorie: Option<Decimal>,
    pub arrotondamento: Option<Decimal>,
    pub imponibile_importo: Option<Decimal>,
    pub imposta: Option<Decimal>,
    /// `I` immediata, `D` differita, `S` scissione dei pagamenti.
    pub esigibilita_iva: Option<String>,
    pub riferimento_normativo: Option<String>,
}

/// Dati relativi ai veicoli (cessioni intracomunitarie di mezzi nuovi).
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct DatiVeicoli {
    pub data: Option<NaiveDate>,
    pub totale_percorso: Option<String>,
}

/// Singola rata o modalità di pagamento.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct DettaglioPagamento {
    pub beneficiario: Option<String>,
    /// Modalità di pagamento (MP01 ... MP23).
    pub modalita_pagamento: Option<String>,
    pub data_riferimento_termini: Option<NaiveDate>,
    pub giorni_termini_pagamento: Option<u32>,
    pub data_scadenza_pagamento: Option<NaiveDate>,
    pub importo_pagamento: Option<Decimal>,
    pub cod_ufficio_postale: Option<String>,
    pub cognome_quietanzante: Option<String>,
    pub nome_quietanzante: Option<String>,
    pub cf_quietanzante: Option<String>,
    pub titolo_quietanzante: Option<String>,
    pub istituto_finanziario: Option<String>,
    pub iban: Option<String>,
    pub abi: Option<String>,
    pub cab: Option<String>,
    pub bic: Option<String>,
    pub sconto_pagamento_anticipato: Option<Decimal>,
    pub data_limite_pagamento_anticipato: Option<NaiveDate>,
    pub penalita_pagamenti_ritardati: Option<Decimal>,
    pub data_decorrenza_penale: Option<NaiveDate>,
    pub codice_pagamento: Option<String>,
}

/// Blocco `DatiPagamento`.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct DatiPagamento {
    /// `TP01` a rate, `TP02` completo, `TP03` anticipo.
    pub condizioni_pagamento: Option<String>,
    pub dettagli: Vec<DettaglioPagamento>,
}

/// Allegato incorporato nella fattura.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Allegato {
    pub nome: Option<String>,
    pub algoritmo_compressione: Option<String>,
    pub formato: Option<String>,
    pub descrizione: Option<String>,
    /// Contenuto già decodificato dal base64.
    #[serde(skip)]
    pub dati: Vec<u8>,
    /// Dimensione in byte del contenuto decodificato.
    pub dimensione: usize,
}

/// Corpo della fattura (`FatturaElettronicaBody`). Un file può contenerne
/// più di uno: è il caso del "lotto di fatture".
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Corpo {
    pub dati_generali: DatiGenerali,
    pub linee: Vec<Linea>,
    pub riepiloghi: Vec<Riepilogo>,
    pub dati_veicoli: Option<DatiVeicoli>,
    pub pagamenti: Vec<DatiPagamento>,
    pub allegati: Vec<Allegato>,
}

impl Corpo {
    /// Totale del documento: quello dichiarato, oppure, se assente, la somma
    /// di imponibili e imposte del riepilogo (o delle voci, per la
    /// semplificata). Il campo `ImportoTotaleDocumento` è facoltativo e
    /// spesso viene omesso.
    pub fn totale(&self) -> Option<Decimal> {
        if let Some(t) = self.dati_generali.documento.importo_totale_documento {
            return Some(t);
        }
        if !self.riepiloghi.is_empty() {
            let somma = self
                .riepiloghi
                .iter()
                .map(|r| r.imponibile_importo.unwrap_or_default() + r.imposta.unwrap_or_default())
                .sum();
            return Some(somma);
        }
        if self.linee.iter().any(|l| l.prezzo_totale.is_some()) {
            let somma = self
                .linee
                .iter()
                .map(|l| l.prezzo_totale.unwrap_or_default() + l.imposta.unwrap_or_default())
                .sum();
            return Some(somma);
        }
        None
    }

    /// Prima data di scadenza tra le rate di pagamento.
    pub fn prima_scadenza(&self) -> Option<NaiveDate> {
        self.pagamenti
            .iter()
            .flat_map(|p| p.dettagli.iter())
            .filter_map(|d| d.data_scadenza_pagamento)
            .min()
    }
}

/// Fattura elettronica completa, già interpretata.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FatturaElettronica {
    pub formato: Formato,
    /// Valore grezzo dell'attributo `versione` della radice.
    pub versione: Option<String>,
    pub intestazione: Intestazione,
    pub corpi: Vec<Corpo>,
    /// Anomalie non bloccanti rilevate durante la lettura.
    pub avvisi: Vec<String>,
}
