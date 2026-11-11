// Copyright (c) 2026 Andrea Re - info@digitare.net
// Autore: Andrea Re @ 2026 - info@digitare.net
//
// Lettura dell'XML FatturaPA e costruzione del modello dati.
//
// La lettura è tollerante: un valore non interpretabile (data o importo
// scritti male) non blocca il documento, ma lascia il campo vuoto e
// registra un avviso leggibile dall'utente.

use crate::errore::{Errore, Risultato};
use crate::modello::*;
use crate::xml::{figli, figlio, testi, testo, testo_nodo};
use base64::Engine;
use chrono::NaiveDate;
use roxmltree::Node;
use rust_decimal::Decimal;
use std::str::FromStr;

/// Nome locale della radice della fattura ordinaria.
pub const RADICE_ORDINARIA: &str = "FatturaElettronica";
/// Nome locale della radice della fattura semplificata.
pub const RADICE_SEMPLIFICATA: &str = "FatturaElettronicaSemplificata";

/// Interpreta un documento XML già analizzato come fattura elettronica.
pub fn leggi_fattura(documento: &roxmltree::Document<'_>) -> Risultato<FatturaElettronica> {
    let radice = documento.root_element();
    let nome = radice.tag_name().name();
    let mut lettore = Lettore::default();
    let versione = radice.attribute("versione").map(|v| v.trim().to_owned());
    let fattura = match nome {
        RADICE_ORDINARIA => lettore.ordinaria(radice, versione),
        RADICE_SEMPLIFICATA => lettore.semplificata(radice, versione),
        altro => return Err(Errore::DocumentoNonRiconosciuto(altro.to_owned())),
    };
    Ok(fattura)
}

/// Deduce il formato dall'attributo `versione` o, in mancanza, dal
/// `FormatoTrasmissione` dell'intestazione.
fn formato_da(versione: Option<&str>, trasmissione: Option<&str>) -> Formato {
    let valore = versione.or(trasmissione).unwrap_or_default().to_uppercase();
    match valore.as_str() {
        "FPA12" => Formato::Fpa12,
        "FPR12" => Formato::Fpr12,
        "FSM10" => Formato::Fsm10,
        _ => Formato::Altro,
    }
}

/// Stato della lettura: raccoglie gli avvisi man mano che si procede.
#[derive(Default)]
struct Lettore {
    avvisi: Vec<String>,
}

impl Lettore {
    // ------------------------------------------------------------------
    // Conversioni elementari
    // ------------------------------------------------------------------

    /// Legge un importo o una quantità. Accetta anche la virgola come
    /// separatore decimale (non conforme, ma diffusa), segnalandolo.
    fn decimale(&mut self, nodo: Node<'_, '_>, nome: &str) -> Option<Decimal> {
        let valore = testo(nodo, nome)?;
        self.decimale_da_testo(&valore, nome)
    }

    fn decimale_da_testo(&mut self, valore: &str, nome: &str) -> Option<Decimal> {
        if let Ok(d) = Decimal::from_str(valore) {
            return Some(d);
        }
        let corretto = valore.replace(',', ".");
        match Decimal::from_str(&corretto) {
            Ok(d) => {
                self.avvisi.push(format!(
                    "{nome}: separatore decimale non conforme in \"{valore}\""
                ));
                Some(d)
            }
            Err(_) => {
                self.avvisi
                    .push(format!("{nome}: valore numerico non valido \"{valore}\""));
                None
            }
        }
    }

    /// Legge una data `AAAA-MM-GG`; tollera un orario in coda.
    fn data(&mut self, nodo: Node<'_, '_>, nome: &str) -> Option<NaiveDate> {
        let valore = testo(nodo, nome)?;
        let parte = valore.get(..10).unwrap_or(&valore);
        match NaiveDate::parse_from_str(parte, "%Y-%m-%d") {
            Ok(d) => Some(d),
            Err(_) => {
                self.avvisi
                    .push(format!("{nome}: data non valida \"{valore}\""));
                None
            }
        }
    }

    /// Legge un intero positivo (numeri di linea, giorni, ...).
    fn intero(&mut self, nodo: Node<'_, '_>, nome: &str) -> Option<u32> {
        let valore = testo(nodo, nome)?;
        match valore.parse::<u32>() {
            Ok(n) => Some(n),
            Err(_) => {
                self.avvisi
                    .push(format!("{nome}: numero intero non valido \"{valore}\""));
                None
            }
        }
    }

    /// Tutti i valori interi dei figli con il nome dato.
    fn interi(&mut self, nodo: Node<'_, '_>, nome: &str) -> Vec<u32> {
        let mut risultato = Vec::new();
        for n in figli(nodo, nome) {
            if let Some(v) = testo_nodo(n) {
                match v.parse::<u32>() {
                    Ok(x) => risultato.push(x),
                    Err(_) => self
                        .avvisi
                        .push(format!("{nome}: numero intero non valido \"{v}\"")),
                }
            }
        }
        risultato
    }

    /// I flag del tracciato valgono "SI" quando presenti.
    fn flag(nodo: Node<'_, '_>, nome: &str) -> bool {
        testo(nodo, nome)
            .map(|v| v.eq_ignore_ascii_case("SI"))
            .unwrap_or(false)
    }

    // ------------------------------------------------------------------
    // Blocchi comuni
    // ------------------------------------------------------------------

    fn id_fiscale(nodo: Option<Node<'_, '_>>) -> Option<IdFiscale> {
        let n = nodo?;
        let paese = testo(n, "IdPaese").unwrap_or_default();
        let codice = testo(n, "IdCodice").unwrap_or_default();
        if paese.is_empty() && codice.is_empty() {
            None
        } else {
            Some(IdFiscale { paese, codice })
        }
    }

    fn indirizzo(nodo: Option<Node<'_, '_>>) -> Option<Indirizzo> {
        let n = nodo?;
        Some(Indirizzo {
            indirizzo: testo(n, "Indirizzo"),
            numero_civico: testo(n, "NumeroCivico"),
            cap: testo(n, "CAP"),
            comune: testo(n, "Comune"),
            provincia: testo(n, "Provincia"),
            nazione: testo(n, "Nazione"),
        })
    }

    fn contatti(nodo: Option<Node<'_, '_>>) -> Option<Contatti> {
        let n = nodo?;
        Some(Contatti {
            telefono: testo(n, "Telefono"),
            fax: testo(n, "Fax"),
            email: testo(n, "Email"),
        })
    }

    /// Compila i campi anagrafici partendo da un nodo che contiene
    /// `Denominazione` / `Nome` / `Cognome` / `Titolo` / `CodEORI`.
    fn anagrafica(soggetto: &mut Soggetto, nodo: Option<Node<'_, '_>>) {
        if let Some(a) = nodo {
            soggetto.denominazione = testo(a, "Denominazione");
            soggetto.nome = testo(a, "Nome");
            soggetto.cognome = testo(a, "Cognome");
            soggetto.titolo = testo(a, "Titolo");
            soggetto.cod_eori = testo(a, "CodEORI");
        }
    }

    /// Blocco `DatiAnagrafici` comune a tutti i soggetti della fattura ordinaria.
    fn dati_anagrafici(&mut self, nodo: Option<Node<'_, '_>>) -> Soggetto {
        let mut s = Soggetto::default();
        if let Some(d) = nodo {
            s.id_fiscale_iva = Self::id_fiscale(figlio(d, "IdFiscaleIVA"));
            s.codice_fiscale = testo(d, "CodiceFiscale");
            Self::anagrafica(&mut s, figlio(d, "Anagrafica"));
            s.albo_professionale = testo(d, "AlboProfessionale");
            s.provincia_albo = testo(d, "ProvinciaAlbo");
            s.numero_iscrizione_albo = testo(d, "NumeroIscrizioneAlbo");
            s.data_iscrizione_albo = self.data(d, "DataIscrizioneAlbo");
            s.regime_fiscale = testo(d, "RegimeFiscale");
        }
        s
    }

    fn iscrizione_rea(&mut self, nodo: Option<Node<'_, '_>>) -> Option<IscrizioneRea> {
        let n = nodo?;
        Some(IscrizioneRea {
            ufficio: testo(n, "Ufficio"),
            numero_rea: testo(n, "NumeroREA"),
            capitale_sociale: self.decimale(n, "CapitaleSociale"),
            socio_unico: testo(n, "SocioUnico"),
            stato_liquidazione: testo(n, "StatoLiquidazione"),
        })
    }

    fn sconti(&mut self, nodo: Node<'_, '_>) -> Vec<ScontoMaggiorazione> {
        let mut v = Vec::new();
        for s in figli(nodo, "ScontoMaggiorazione") {
            v.push(ScontoMaggiorazione {
                tipo: testo(s, "Tipo"),
                percentuale: self.decimale(s, "Percentuale"),
                importo: self.decimale(s, "Importo"),
            });
        }
        v
    }

    fn allegati(&mut self, corpo: Node<'_, '_>) -> Vec<Allegato> {
        let mut v = Vec::new();
        for a in figli(corpo, "Allegati") {
            let nome = testo(a, "NomeAttachment");
            let grezzo = testo(a, "Attachment").unwrap_or_default();
            let dati = match decodifica_base64(&grezzo) {
                Some(d) => d,
                None => {
                    self.avvisi.push(format!(
                        "Allegato \"{}\": contenuto base64 non decodificabile",
                        nome.as_deref().unwrap_or("senza nome")
                    ));
                    Vec::new()
                }
            };
            v.push(Allegato {
                nome,
                algoritmo_compressione: testo(a, "AlgoritmoCompressione"),
                formato: testo(a, "FormatoAttachment"),
                descrizione: testo(a, "DescrizioneAttachment"),
                dimensione: dati.len(),
                dati,
            });
        }
        v
    }

    // ------------------------------------------------------------------
    // Fattura ordinaria (FPA12 / FPR12)
    // ------------------------------------------------------------------

    fn ordinaria(&mut self, radice: Node<'_, '_>, versione: Option<String>) -> FatturaElettronica {
        let intestazione = match figlio(radice, "FatturaElettronicaHeader") {
            Some(h) => self.intestazione_ordinaria(h),
            None => {
                self.avvisi
                    .push("Manca il blocco FatturaElettronicaHeader".to_owned());
                Intestazione::default()
            }
        };
        let corpi: Vec<Corpo> = figli(radice, "FatturaElettronicaBody")
            .map(|b| self.corpo_ordinario(b))
            .collect();
        if corpi.is_empty() {
            self.avvisi
                .push("Il documento non contiene alcun FatturaElettronicaBody".to_owned());
        }
        let formato = formato_da(
            versione.as_deref(),
            intestazione
                .dati_trasmissione
                .formato_trasmissione
                .as_deref(),
        );
        FatturaElettronica {
            formato,
            versione,
            intestazione,
            corpi,
            avvisi: std::mem::take(&mut self.avvisi),
        }
    }

    fn dati_trasmissione(&mut self, nodo: Option<Node<'_, '_>>) -> DatiTrasmissione {
        let Some(t) = nodo else {
            self.avvisi
                .push("Manca il blocco DatiTrasmissione".to_owned());
            return DatiTrasmissione::default();
        };
        DatiTrasmissione {
            id_trasmittente: Self::id_fiscale(figlio(t, "IdTrasmittente")),
            progressivo_invio: testo(t, "ProgressivoInvio"),
            formato_trasmissione: testo(t, "FormatoTrasmissione"),
            codice_destinatario: testo(t, "CodiceDestinatario"),
            pec_destinatario: testo(t, "PECDestinatario"),
            contatti_trasmittente: Self::contatti(figlio(t, "ContattiTrasmittente")),
        }
    }

    fn intestazione_ordinaria(&mut self, h: Node<'_, '_>) -> Intestazione {
        let dati_trasmissione = self.dati_trasmissione(figlio(h, "DatiTrasmissione"));

        // Cedente / prestatore
        let mut cedente = Soggetto::default();
        if let Some(c) = figlio(h, "CedentePrestatore") {
            cedente = self.dati_anagrafici(figlio(c, "DatiAnagrafici"));
            cedente.sede = Self::indirizzo(figlio(c, "Sede"));
            cedente.stabile_organizzazione = Self::indirizzo(figlio(c, "StabileOrganizzazione"));
            cedente.iscrizione_rea = self.iscrizione_rea(figlio(c, "IscrizioneREA"));
            cedente.contatti = Self::contatti(figlio(c, "Contatti"));
            cedente.riferimento_amministrazione = testo(c, "RiferimentoAmministrazione");
        } else {
            self.avvisi
                .push("Manca il blocco CedentePrestatore".to_owned());
        }

        // Rappresentante fiscale del cedente
        let rappresentante_fiscale = figlio(h, "RappresentanteFiscale")
            .map(|r| self.dati_anagrafici(figlio(r, "DatiAnagrafici")));

        // Cessionario / committente
        let mut cessionario = Soggetto::default();
        if let Some(c) = figlio(h, "CessionarioCommittente") {
            cessionario = self.dati_anagrafici(figlio(c, "DatiAnagrafici"));
            cessionario.sede = Self::indirizzo(figlio(c, "Sede"));
            cessionario.stabile_organizzazione =
                Self::indirizzo(figlio(c, "StabileOrganizzazione"));
        } else {
            self.avvisi
                .push("Manca il blocco CessionarioCommittente".to_owned());
        }

        let terzo_intermediario = figlio(h, "TerzoIntermediarioOSoggettoEmittente")
            .map(|t| self.dati_anagrafici(figlio(t, "DatiAnagrafici")));

        Intestazione {
            dati_trasmissione,
            cedente_prestatore: cedente,
            rappresentante_fiscale,
            cessionario_committente: cessionario,
            terzo_intermediario,
            soggetto_emittente: testo(h, "SoggettoEmittente"),
        }
    }

    fn documenti_correlati(&mut self, dg: Node<'_, '_>, nome: &str) -> Vec<DocumentoCorrelato> {
        let mut v = Vec::new();
        for d in figli(dg, nome) {
            v.push(DocumentoCorrelato {
                riferimento_numero_linea: self.interi(d, "RiferimentoNumeroLinea"),
                id_documento: testo(d, "IdDocumento"),
                data: self.data(d, "Data"),
                num_item: testo(d, "NumItem"),
                codice_commessa_convenzione: testo(d, "CodiceCommessaConvenzione"),
                codice_cup: testo(d, "CodiceCUP"),
                codice_cig: testo(d, "CodiceCIG"),
            });
        }
        v
    }

    fn dati_generali_documento(&mut self, d: Node<'_, '_>) -> DatiGeneraliDocumento {
        let mut ritenute = Vec::new();
        for r in figli(d, "DatiRitenuta") {
            ritenute.push(Ritenuta {
                tipo: testo(r, "TipoRitenuta"),
                importo: self.decimale(r, "ImportoRitenuta"),
                aliquota: self.decimale(r, "AliquotaRitenuta"),
                causale_pagamento: testo(r, "CausalePagamento"),
            });
        }
        let bollo = figlio(d, "DatiBollo").map(|b| Bollo {
            virtuale: Self::flag(b, "BolloVirtuale"),
            importo: self.decimale(b, "ImportoBollo"),
        });
        let mut casse = Vec::new();
        for c in figli(d, "DatiCassaPrevidenziale") {
            casse.push(CassaPrevidenziale {
                tipo: testo(c, "TipoCassa"),
                aliquota: self.decimale(c, "AlCassa"),
                importo_contributo: self.decimale(c, "ImportoContributoCassa"),
                imponibile: self.decimale(c, "ImponibileCassa"),
                aliquota_iva: self.decimale(c, "AliquotaIVA"),
                ritenuta: Self::flag(c, "Ritenuta"),
                natura: testo(c, "Natura"),
                riferimento_amministrazione: testo(c, "RiferimentoAmministrazione"),
            });
        }
        DatiGeneraliDocumento {
            tipo_documento: testo(d, "TipoDocumento"),
            divisa: testo(d, "Divisa"),
            data: self.data(d, "Data"),
            numero: testo(d, "Numero"),
            ritenute,
            bollo,
            casse_previdenziali: casse,
            sconti_maggiorazioni: self.sconti(d),
            importo_totale_documento: self.decimale(d, "ImportoTotaleDocumento"),
            arrotondamento: self.decimale(d, "Arrotondamento"),
            causale: testi(d, "Causale"),
            art73: Self::flag(d, "Art73"),
        }
    }

    fn dati_trasporto(&mut self, t: Node<'_, '_>) -> DatiTrasporto {
        let vettore_nodo = figlio(t, "DatiAnagraficiVettore");
        let vettore = vettore_nodo.map(|v| self.dati_anagrafici(Some(v)));
        DatiTrasporto {
            vettore,
            numero_licenza_guida: vettore_nodo.and_then(|v| testo(v, "NumeroLicenzaGuida")),
            mezzo_trasporto: testo(t, "MezzoTrasporto"),
            causale_trasporto: testo(t, "CausaleTrasporto"),
            numero_colli: testo(t, "NumeroColli"),
            descrizione: testo(t, "Descrizione"),
            unita_misura_peso: testo(t, "UnitaMisuraPeso"),
            peso_lordo: self.decimale(t, "PesoLordo"),
            peso_netto: self.decimale(t, "PesoNetto"),
            data_ora_ritiro: testo(t, "DataOraRitiro"),
            data_inizio_trasporto: self.data(t, "DataInizioTrasporto"),
            tipo_resa: testo(t, "TipoResa"),
            indirizzo_resa: Self::indirizzo(figlio(t, "IndirizzoResa")),
            data_ora_consegna: testo(t, "DataOraConsegna"),
        }
    }

    fn dati_generali(&mut self, dg: Node<'_, '_>) -> DatiGenerali {
        let documento = match figlio(dg, "DatiGeneraliDocumento") {
            Some(d) => self.dati_generali_documento(d),
            None => {
                self.avvisi
                    .push("Manca il blocco DatiGeneraliDocumento".to_owned());
                DatiGeneraliDocumento::default()
            }
        };
        let mut ddt = Vec::new();
        for d in figli(dg, "DatiDDT") {
            ddt.push(DatiDdt {
                numero: testo(d, "NumeroDDT"),
                data: self.data(d, "DataDDT"),
                riferimento_numero_linea: self.interi(d, "RiferimentoNumeroLinea"),
            });
        }
        let sal = figli(dg, "DatiSAL")
            .filter_map(|s| testo(s, "RiferimentoFase"))
            .collect();
        let trasporto = figlio(dg, "DatiTrasporto").map(|t| self.dati_trasporto(t));
        let fattura_principale = figlio(dg, "FatturaPrincipale").map(|f| FatturaPrincipale {
            numero: testo(f, "NumeroFatturaPrincipale"),
            data: self.data(f, "DataFatturaPrincipale"),
        });
        DatiGenerali {
            documento,
            ordini_acquisto: self.documenti_correlati(dg, "DatiOrdineAcquisto"),
            contratti: self.documenti_correlati(dg, "DatiContratto"),
            convenzioni: self.documenti_correlati(dg, "DatiConvenzione"),
            ricezioni: self.documenti_correlati(dg, "DatiRicezione"),
            fatture_collegate: self.documenti_correlati(dg, "DatiFattureCollegate"),
            sal,
            ddt,
            trasporto,
            fattura_principale,
            fattura_rettificata: None,
        }
    }

    fn linea(&mut self, l: Node<'_, '_>) -> Linea {
        let codici_articolo = figli(l, "CodiceArticolo")
            .map(|c| CodiceArticolo {
                tipo: testo(c, "CodiceTipo"),
                valore: testo(c, "CodiceValore"),
            })
            .collect();
        let mut altri = Vec::new();
        for a in figli(l, "AltriDatiGestionali") {
            altri.push(AltroDatoGestionale {
                tipo_dato: testo(a, "TipoDato"),
                riferimento_testo: testo(a, "RiferimentoTesto"),
                riferimento_numero: self.decimale(a, "RiferimentoNumero"),
                riferimento_data: self.data(a, "RiferimentoData"),
            });
        }
        Linea {
            numero_linea: self.intero(l, "NumeroLinea"),
            tipo_cessione_prestazione: testo(l, "TipoCessionePrestazione"),
            codici_articolo,
            descrizione: testo(l, "Descrizione"),
            quantita: self.decimale(l, "Quantita"),
            unita_misura: testo(l, "UnitaMisura"),
            data_inizio_periodo: self.data(l, "DataInizioPeriodo"),
            data_fine_periodo: self.data(l, "DataFinePeriodo"),
            prezzo_unitario: self.decimale(l, "PrezzoUnitario"),
            sconti_maggiorazioni: self.sconti(l),
            prezzo_totale: self.decimale(l, "PrezzoTotale"),
            aliquota_iva: self.decimale(l, "AliquotaIVA"),
            imposta: None,
            ritenuta: Self::flag(l, "Ritenuta"),
            natura: testo(l, "Natura"),
            riferimento_amministrazione: testo(l, "RiferimentoAmministrazione"),
            riferimento_normativo: None,
            altri_dati_gestionali: altri,
        }
    }

    fn riepilogo(&mut self, r: Node<'_, '_>) -> Riepilogo {
        Riepilogo {
            aliquota_iva: self.decimale(r, "AliquotaIVA"),
            natura: testo(r, "Natura"),
            spese_accessorie: self.decimale(r, "SpeseAccessorie"),
            arrotondamento: self.decimale(r, "Arrotondamento"),
            imponibile_importo: self.decimale(r, "ImponibileImporto"),
            imposta: self.decimale(r, "Imposta"),
            esigibilita_iva: testo(r, "EsigibilitaIVA"),
            riferimento_normativo: testo(r, "RiferimentoNormativo"),
        }
    }

    fn pagamenti(&mut self, corpo: Node<'_, '_>) -> Vec<DatiPagamento> {
        let mut v = Vec::new();
        for p in figli(corpo, "DatiPagamento") {
            let mut dettagli = Vec::new();
            for d in figli(p, "DettaglioPagamento") {
                dettagli.push(DettaglioPagamento {
                    beneficiario: testo(d, "Beneficiario"),
                    modalita_pagamento: testo(d, "ModalitaPagamento"),
                    data_riferimento_termini: self.data(d, "DataRiferimentoTerminiPagamento"),
                    giorni_termini_pagamento: self.intero(d, "GiorniTerminiPagamento"),
                    data_scadenza_pagamento: self.data(d, "DataScadenzaPagamento"),
                    importo_pagamento: self.decimale(d, "ImportoPagamento"),
                    cod_ufficio_postale: testo(d, "CodUfficioPostale"),
                    cognome_quietanzante: testo(d, "CognomeQuietanzante"),
                    nome_quietanzante: testo(d, "NomeQuietanzante"),
                    cf_quietanzante: testo(d, "CFQuietanzante"),
                    titolo_quietanzante: testo(d, "TitoloQuietanzante"),
                    istituto_finanziario: testo(d, "IstitutoFinanziario"),
                    iban: testo(d, "IBAN").map(|i| i.replace(' ', "")),
                    abi: testo(d, "ABI"),
                    cab: testo(d, "CAB"),
                    bic: testo(d, "BIC"),
                    sconto_pagamento_anticipato: self.decimale(d, "ScontoPagamentoAnticipato"),
                    data_limite_pagamento_anticipato: self.data(d, "DataLimitePagamentoAnticipato"),
                    penalita_pagamenti_ritardati: self.decimale(d, "PenalitaPagamentiRitardati"),
                    data_decorrenza_penale: self.data(d, "DataDecorrenzaPenale"),
                    codice_pagamento: testo(d, "CodicePagamento"),
                });
            }
            v.push(DatiPagamento {
                condizioni_pagamento: testo(p, "CondizioniPagamento"),
                dettagli,
            });
        }
        v
    }

    fn corpo_ordinario(&mut self, b: Node<'_, '_>) -> Corpo {
        let dati_generali = match figlio(b, "DatiGenerali") {
            Some(dg) => self.dati_generali(dg),
            None => {
                self.avvisi.push("Manca il blocco DatiGenerali".to_owned());
                DatiGenerali::default()
            }
        };
        let (linee, riepiloghi) = match figlio(b, "DatiBeniServizi") {
            Some(bs) => {
                let linee = figli(bs, "DettaglioLinee").map(|l| self.linea(l)).collect();
                let riepiloghi = figli(bs, "DatiRiepilogo")
                    .map(|r| self.riepilogo(r))
                    .collect();
                (linee, riepiloghi)
            }
            None => {
                self.avvisi
                    .push("Manca il blocco DatiBeniServizi".to_owned());
                (Vec::new(), Vec::new())
            }
        };
        let dati_veicoli = figlio(b, "DatiVeicoli").map(|v| DatiVeicoli {
            data: self.data(v, "Data"),
            totale_percorso: testo(v, "TotalePercorso"),
        });
        Corpo {
            dati_generali,
            linee,
            riepiloghi,
            dati_veicoli,
            pagamenti: self.pagamenti(b),
            allegati: self.allegati(b),
        }
    }

    // ------------------------------------------------------------------
    // Fattura semplificata (FSM10)
    // ------------------------------------------------------------------

    fn semplificata(
        &mut self,
        radice: Node<'_, '_>,
        versione: Option<String>,
    ) -> FatturaElettronica {
        let mut intestazione = Intestazione::default();
        if let Some(h) = figlio(radice, "FatturaElettronicaHeader") {
            intestazione.dati_trasmissione = self.dati_trasmissione(figlio(h, "DatiTrasmissione"));

            if let Some(c) = figlio(h, "CedentePrestatore") {
                let mut s = Soggetto {
                    id_fiscale_iva: Self::id_fiscale(figlio(c, "IdFiscaleIVA")),
                    codice_fiscale: testo(c, "CodiceFiscale"),
                    regime_fiscale: testo(c, "RegimeFiscale"),
                    sede: Self::indirizzo(figlio(c, "Sede")),
                    stabile_organizzazione: Self::indirizzo(figlio(c, "StabileOrganizzazione")),
                    ..Default::default()
                };
                Self::anagrafica(&mut s, Some(c));
                s.iscrizione_rea = self.iscrizione_rea(figlio(c, "IscrizioneREA"));
                intestazione.rappresentante_fiscale = figlio(c, "RappresentanteFiscale").map(|r| {
                    let mut rf = Soggetto {
                        id_fiscale_iva: Self::id_fiscale(figlio(r, "IdFiscaleIVA")),
                        ..Default::default()
                    };
                    Self::anagrafica(&mut rf, Some(r));
                    rf
                });
                intestazione.cedente_prestatore = s;
            } else {
                self.avvisi
                    .push("Manca il blocco CedentePrestatore".to_owned());
            }

            if let Some(c) = figlio(h, "CessionarioCommittente") {
                let mut s = Soggetto::default();
                if let Some(idf) = figlio(c, "IdentificativiFiscali") {
                    s.id_fiscale_iva = Self::id_fiscale(figlio(idf, "IdFiscaleIVA"));
                    s.codice_fiscale = testo(idf, "CodiceFiscale");
                }
                if let Some(altri) = figlio(c, "AltriDatiIdentificativi") {
                    Self::anagrafica(&mut s, Some(altri));
                    s.sede = Self::indirizzo(figlio(altri, "Sede"));
                    s.stabile_organizzazione =
                        Self::indirizzo(figlio(altri, "StabileOrganizzazione"));
                }
                intestazione.cessionario_committente = s;
            } else {
                self.avvisi
                    .push("Manca il blocco CessionarioCommittente".to_owned());
            }
            intestazione.soggetto_emittente = testo(h, "SoggettoEmittente");
        } else {
            self.avvisi
                .push("Manca il blocco FatturaElettronicaHeader".to_owned());
        }

        let corpi: Vec<Corpo> = figli(radice, "FatturaElettronicaBody")
            .map(|b| self.corpo_semplificato(b))
            .collect();
        if corpi.is_empty() {
            self.avvisi
                .push("Il documento non contiene alcun FatturaElettronicaBody".to_owned());
        }
        let formato = formato_da(
            versione.as_deref(),
            intestazione
                .dati_trasmissione
                .formato_trasmissione
                .as_deref(),
        );
        FatturaElettronica {
            formato,
            versione,
            intestazione,
            corpi,
            avvisi: std::mem::take(&mut self.avvisi),
        }
    }

    fn corpo_semplificato(&mut self, b: Node<'_, '_>) -> Corpo {
        let mut dati_generali = DatiGenerali::default();
        if let Some(dg) = figlio(b, "DatiGenerali") {
            if let Some(d) = figlio(dg, "DatiGeneraliDocumento") {
                dati_generali.documento = DatiGeneraliDocumento {
                    tipo_documento: testo(d, "TipoDocumento"),
                    divisa: testo(d, "Divisa"),
                    data: self.data(d, "Data"),
                    numero: testo(d, "Numero"),
                    bollo: if Self::flag(d, "BolloVirtuale") {
                        Some(Bollo {
                            virtuale: true,
                            importo: None,
                        })
                    } else {
                        None
                    },
                    ..Default::default()
                };
            }
            dati_generali.fattura_rettificata =
                figlio(dg, "DatiFatturaRettificata").map(|r| FatturaPrincipale {
                    numero: testo(r, "NumeroFR"),
                    data: self.data(r, "DataFR"),
                });
        } else {
            self.avvisi.push("Manca il blocco DatiGenerali".to_owned());
        }

        let mut linee = Vec::new();
        for (indice, v) in figli(b, "DatiBeniServizi").enumerate() {
            let iva = figlio(v, "DatiIVA");
            linee.push(Linea {
                numero_linea: Some(indice as u32 + 1),
                descrizione: testo(v, "Descrizione"),
                prezzo_totale: self.decimale(v, "Importo"),
                aliquota_iva: iva.and_then(|i| self.decimale(i, "Aliquota")),
                imposta: iva.and_then(|i| self.decimale(i, "Imposta")),
                natura: testo(v, "Natura"),
                riferimento_normativo: testo(v, "RiferimentoNormativo"),
                ..Default::default()
            });
        }
        Corpo {
            dati_generali,
            linee,
            riepiloghi: Vec::new(),
            dati_veicoli: None,
            pagamenti: Vec::new(),
            allegati: self.allegati(b),
        }
    }
}

/// Decodifica base64 ignorando spazi e a capo; tollera il padding mancante.
pub(crate) fn decodifica_base64(testo: &str) -> Option<Vec<u8>> {
    let pulito: String = testo.chars().filter(|c| !c.is_whitespace()).collect();
    if pulito.is_empty() {
        return Some(Vec::new());
    }
    let standard = base64::engine::general_purpose::STANDARD;
    if let Ok(d) = standard.decode(&pulito) {
        return Some(d);
    }
    let senza_padding = base64::engine::general_purpose::STANDARD_NO_PAD;
    senza_padding.decode(pulito.trim_end_matches('=')).ok()
}

/// Comodità per i test: legge una fattura direttamente da testo XML.
#[doc(hidden)]
pub fn leggi_da_testo(xml: &str) -> Risultato<FatturaElettronica> {
    let doc = crate::xml::analizza(xml)?;
    leggi_fattura(&doc)
}
