// Copyright (c) 2026 Andrea Re - info@digitare.net
// Autore: Andrea Re @ 2026 - info@digitare.net
//
// Rappresentazione HTML "Moderna" della fattura elettronica.
//
// È generata direttamente dal modello dati, senza fogli di stile XSLT:
// compatta, leggibile a schermo e adatta alla stampa su A4. Mostra tutti i
// dati utili alla registrazione contabile e al pagamento, e collega DDT e
// ordini alle righe a cui si riferiscono.

pub mod formato;

use formato::{data, esc, importo, numero, percentuale};
use rust_decimal::Decimal;
use rustinvoice_core::codici;
use rustinvoice_core::modello::*;

/// Foglio di stile incorporato nella pagina generata.
const CSS: &str = include_str!("moderna.css");

/// Genera la pagina HTML completa per tutti i documenti del file.
pub fn html_moderna(f: &FatturaElettronica) -> String {
    let mut h = String::with_capacity(16 * 1024);
    h.push_str("<!doctype html><html lang=\"it\"><head><meta charset=\"utf-8\">");
    h.push_str("<meta name=\"viewport\" content=\"width=device-width, initial-scale=1\">");
    h.push_str("<title>Fattura</title><style>");
    h.push_str(CSS);
    h.push_str("</style></head><body><main>");
    for (i, corpo) in f.corpi.iter().enumerate() {
        if i > 0 {
            h.push_str("<div class=\"salto\"></div>");
        }
        documento(&mut h, f, corpo, i, f.corpi.len());
    }
    if !f.avvisi.is_empty() {
        h.push_str("<section class=\"avvisi\"><h2>Anomalie rilevate nel file</h2><ul>");
        for a in &f.avvisi {
            h.push_str(&format!("<li>{}</li>", esc(a)));
        }
        h.push_str("</ul></section>");
    }
    h.push_str("</main></body></html>");
    h
}

fn documento(
    h: &mut String,
    f: &FatturaElettronica,
    c: &Corpo,
    indice: usize,
    totale_corpi: usize,
) {
    let d = &c.dati_generali.documento;
    let tipo = d.tipo_documento.as_deref().unwrap_or("");
    let descr_tipo = codici::tipo_documento(tipo).unwrap_or("Documento");
    let divisa = d.divisa.as_deref().unwrap_or("EUR");

    // Intestazione del documento
    h.push_str("<article class=\"doc\"><header class=\"testa\"><div>");
    if totale_corpi > 1 {
        h.push_str(&format!(
            "<div class=\"lotto\">Documento {} di {} del lotto</div>",
            indice + 1,
            totale_corpi
        ));
    }
    h.push_str(&format!(
        "<h1>{} <span class=\"num\">n. {}</span></h1><div class=\"sotto\">{} · {} · {}</div>",
        esc(descr_tipo),
        esc(d.numero.as_deref().unwrap_or("-")),
        data(d.data),
        esc(tipo),
        esc(divisa)
    ));
    h.push_str("</div><div class=\"totale\"><span>Totale documento</span><strong>");
    h.push_str(&format!("{} {}", importo(c.totale()), esc(divisa)));
    h.push_str("</strong>");
    if let Some(s) = c.prima_scadenza() {
        h.push_str(&format!("<span>scadenza {}</span>", data(Some(s))));
    }
    h.push_str("</div></header>");

    // Soggetti
    let int = &f.intestazione;
    h.push_str("<section class=\"parti\">");
    soggetto(h, "Cedente / prestatore", &int.cedente_prestatore, true);
    soggetto(
        h,
        "Cessionario / committente",
        &int.cessionario_committente,
        false,
    );
    h.push_str("</section>");
    if let Some(r) = &int.rappresentante_fiscale {
        h.push_str("<section class=\"parti\">");
        soggetto(h, "Rappresentante fiscale", r, false);
        h.push_str("</section>");
    }
    if let Some(t) = &int.terzo_intermediario {
        h.push_str("<section class=\"parti\">");
        soggetto(h, "Terzo intermediario / soggetto emittente", t, false);
        h.push_str("</section>");
    }

    // Causale e riferimenti
    if !d.causale.is_empty() {
        h.push_str("<section class=\"blocco\"><h2>Causale</h2><p>");
        h.push_str(&esc(&d.causale.join(" ")));
        h.push_str("</p></section>");
    }
    riferimenti(h, &c.dati_generali);

    // Righe
    righe(h, c);

    // Riepilogo IVA e altri importi
    riepilogo(h, c);
    altri_importi(h, d);

    // Pagamenti
    pagamenti(h, c);

    // Allegati
    if !c.allegati.is_empty() {
        h.push_str("<section class=\"blocco\"><h2>Allegati</h2><ul class=\"allegati\">");
        for a in &c.allegati {
            h.push_str(&format!(
                "<li><strong>{}</strong> {} <span class=\"tenue\">{} · {} KB</span></li>",
                esc(a.nome.as_deref().unwrap_or("senza nome")),
                esc(a.descrizione.as_deref().unwrap_or("")),
                esc(a.formato.as_deref().unwrap_or("")),
                a.dimensione.div_ceil(1024)
            ));
        }
        h.push_str("</ul></section>");
    }

    // Trasmissione
    let t = &int.dati_trasmissione;
    h.push_str("<footer class=\"piede\">");
    h.push_str(&format!(
        "Formato {} · Trasmittente {} · Progressivo {}",
        esc(f.versione.as_deref().unwrap_or("-")),
        esc(&t
            .id_trasmittente
            .as_ref()
            .map(|i| i.compatto())
            .unwrap_or_default()),
        esc(t.progressivo_invio.as_deref().unwrap_or("-"))
    ));
    if let Some(cd) = &t.codice_destinatario {
        h.push_str(&format!(" · Codice destinatario {}", esc(cd)));
    }
    if let Some(pec) = &t.pec_destinatario {
        h.push_str(&format!(" · PEC {}", esc(pec)));
    }
    h.push_str("</footer></article>");
}

fn soggetto(h: &mut String, titolo: &str, s: &Soggetto, cedente: bool) {
    h.push_str(&format!(
        "<div class=\"parte\"><div class=\"etichetta\">{}</div><div class=\"nome\">{}</div>",
        esc(titolo),
        esc(&s.nome_visualizzato())
    ));
    let mut righe: Vec<String> = Vec::new();
    if let Some(sede) = &s.sede {
        righe.push(esc(&sede.su_una_riga()));
    }
    if let Some(id) = &s.id_fiscale_iva {
        righe.push(format!("P.IVA {}", esc(&id.compatto())));
    }
    if let Some(cf) = &s.codice_fiscale {
        righe.push(format!("C.F. {}", esc(cf)));
    }
    if cedente {
        if let Some(rf) = &s.regime_fiscale {
            righe.push(format!(
                "Regime {} {}",
                esc(rf),
                esc(codici::regime_fiscale(rf).unwrap_or("codice non previsto"))
            ));
        }
        if let Some(rea) = &s.iscrizione_rea {
            let mut r = format!(
                "REA {}-{}",
                esc(rea.ufficio.as_deref().unwrap_or("")),
                esc(rea.numero_rea.as_deref().unwrap_or(""))
            );
            if let Some(cap) = rea.capitale_sociale {
                r.push_str(&format!(" · Cap. soc. {}", importo(Some(cap))));
            }
            if rea.stato_liquidazione.as_deref() == Some("LS") {
                r.push_str(" · in liquidazione");
            }
            righe.push(r);
        }
        if let Some(albo) = &s.albo_professionale {
            righe.push(format!(
                "Albo {} {} n. {}",
                esc(albo),
                esc(s.provincia_albo.as_deref().unwrap_or("")),
                esc(s.numero_iscrizione_albo.as_deref().unwrap_or(""))
            ));
        }
    }
    if let Some(c) = &s.contatti {
        let v: Vec<String> = [c.telefono.as_deref(), c.email.as_deref()]
            .into_iter()
            .flatten()
            .map(esc)
            .collect();
        if !v.is_empty() {
            righe.push(v.join(" · "));
        }
    }
    for r in righe {
        h.push_str(&format!("<div>{r}</div>"));
    }
    h.push_str("</div>");
}

fn riferimenti(h: &mut String, dg: &DatiGenerali) {
    let mut voci: Vec<String> = Vec::new();
    let mut aggiungi = |etichetta: &str, lista: &[DocumentoCorrelato]| {
        for x in lista {
            let mut s = format!(
                "{} {}",
                etichetta,
                esc(x.id_documento.as_deref().unwrap_or("-"))
            );
            if x.data.is_some() {
                s.push_str(&format!(" del {}", data(x.data)));
            }
            if let Some(cig) = &x.codice_cig {
                s.push_str(&format!(" · CIG {}", esc(cig)));
            }
            if let Some(cup) = &x.codice_cup {
                s.push_str(&format!(" · CUP {}", esc(cup)));
            }
            if let Some(cc) = &x.codice_commessa_convenzione {
                s.push_str(&format!(" · Commessa {}", esc(cc)));
            }
            voci.push(s);
        }
    };
    aggiungi("Ordine", &dg.ordini_acquisto);
    aggiungi("Contratto", &dg.contratti);
    aggiungi("Convenzione", &dg.convenzioni);
    aggiungi("Ricezione", &dg.ricezioni);
    aggiungi("Fattura collegata", &dg.fatture_collegate);
    for ddt in &dg.ddt {
        voci.push(format!(
            "DDT {} del {}",
            esc(ddt.numero.as_deref().unwrap_or("-")),
            data(ddt.data)
        ));
    }
    if let Some(fp) = &dg.fattura_principale {
        voci.push(format!(
            "Fattura principale {} del {}",
            esc(fp.numero.as_deref().unwrap_or("-")),
            data(fp.data)
        ));
    }
    if let Some(fr) = &dg.fattura_rettificata {
        voci.push(format!(
            "Rettifica della fattura {} del {}",
            esc(fr.numero.as_deref().unwrap_or("-")),
            data(fr.data)
        ));
    }
    if !dg.sal.is_empty() {
        voci.push(format!("SAL fase {}", esc(&dg.sal.join(", "))));
    }
    if voci.is_empty() {
        return;
    }
    h.push_str("<section class=\"blocco\"><h2>Riferimenti</h2><ul class=\"rif\">");
    for v in voci {
        h.push_str(&format!("<li>{v}</li>"));
    }
    h.push_str("</ul></section>");
}

/// Riferimenti a DDT e ordini legati a una specifica riga.
fn riferimenti_riga(dg: &DatiGenerali, numero_linea: Option<u32>) -> Vec<String> {
    let Some(n) = numero_linea else {
        return Vec::new();
    };
    let mut v = Vec::new();
    for ddt in &dg.ddt {
        if ddt.riferimento_numero_linea.contains(&n) {
            v.push(format!(
                "DDT {} del {}",
                esc(ddt.numero.as_deref().unwrap_or("-")),
                data(ddt.data)
            ));
        }
    }
    for o in &dg.ordini_acquisto {
        if o.riferimento_numero_linea.contains(&n) {
            v.push(format!(
                "Ordine {}",
                esc(o.id_documento.as_deref().unwrap_or("-"))
            ));
        }
    }
    v
}

fn righe(h: &mut String, c: &Corpo) {
    if c.linee.is_empty() {
        return;
    }
    h.push_str("<section class=\"blocco\"><table class=\"righe\"><thead><tr>");
    h.push_str(
        "<th class=\"desc\">Descrizione</th><th class=\"n\">Q.tà</th><th class=\"n\">Prezzo</th>",
    );
    h.push_str("<th class=\"n\">Sconti</th><th class=\"n\">Importo</th><th class=\"n\">IVA</th></tr></thead><tbody>");
    for l in &c.linee {
        h.push_str("<tr><td class=\"desc\">");
        let codici_art: Vec<String> = l
            .codici_articolo
            .iter()
            .filter_map(|ca| ca.valore.as_deref())
            .map(esc)
            .collect();
        if !codici_art.is_empty() {
            h.push_str(&format!(
                "<span class=\"codice\">{}</span> ",
                codici_art.join(" / ")
            ));
        }
        h.push_str(&esc(l.descrizione.as_deref().unwrap_or("")));
        if l.data_inizio_periodo.is_some() || l.data_fine_periodo.is_some() {
            h.push_str(&format!(
                "<div class=\"tenue\">Periodo {} - {}</div>",
                data(l.data_inizio_periodo),
                data(l.data_fine_periodo)
            ));
        }
        for r in riferimenti_riga(&c.dati_generali, l.numero_linea) {
            h.push_str(&format!("<div class=\"tenue\">{r}</div>"));
        }
        for a in &l.altri_dati_gestionali {
            let tipo = a.tipo_dato.as_deref().unwrap_or("");
            let mut parti = vec![match codici::tipo_dato_normativo(tipo) {
                Some(descr) => format!("{} ({})", esc(descr), esc(tipo)),
                None => esc(tipo),
            }];
            if let Some(t) = &a.riferimento_testo {
                parti.push(esc(t));
            }
            if let Some(n) = a.riferimento_numero {
                parti.push(numero(Some(n)));
            }
            if a.riferimento_data.is_some() {
                parti.push(data(a.riferimento_data));
            }
            h.push_str(&format!("<div class=\"tenue\">{}</div>", parti.join(" · ")));
        }
        h.push_str("</td>");
        let qta = match (&l.quantita, &l.unita_misura) {
            (Some(q), Some(u)) => format!("{} {}", numero(Some(*q)), esc(u)),
            (Some(q), None) => numero(Some(*q)),
            _ => String::new(),
        };
        h.push_str(&format!("<td class=\"n\">{qta}</td>"));
        h.push_str(&format!(
            "<td class=\"n\">{}</td>",
            importo_facoltativo(l.prezzo_unitario)
        ));
        let sconti: Vec<String> = l
            .sconti_maggiorazioni
            .iter()
            .map(|s| {
                let segno = if s.tipo.as_deref() == Some("MG") {
                    "+"
                } else {
                    "-"
                };
                match (s.percentuale, s.importo) {
                    (Some(p), _) => format!("{segno}{}", percentuale(Some(p))),
                    (None, Some(i)) => format!("{segno}{}", importo(Some(i))),
                    _ => String::new(),
                }
            })
            .collect();
        h.push_str(&format!("<td class=\"n\">{}</td>", sconti.join(" ")));
        h.push_str(&format!(
            "<td class=\"n\">{}</td>",
            importo_facoltativo(l.prezzo_totale)
        ));
        h.push_str(&format!(
            "<td class=\"n\">{}</td></tr>",
            iva(l.aliquota_iva, l.natura.as_deref())
        ));
    }
    h.push_str("</tbody></table></section>");
}

fn iva(aliquota: Option<Decimal>, natura: Option<&str>) -> String {
    match natura {
        Some(n) => format!(
            "<abbr title=\"{}\">{}</abbr>",
            esc(codici::natura(n).unwrap_or("codice non previsto")),
            esc(n)
        ),
        None => percentuale(aliquota),
    }
}

fn importo_facoltativo(v: Option<Decimal>) -> String {
    if v.is_some() {
        importo(v)
    } else {
        String::new()
    }
}

fn riepilogo(h: &mut String, c: &Corpo) {
    if c.riepiloghi.is_empty() {
        return;
    }
    h.push_str("<section class=\"blocco\"><h2>Riepilogo IVA</h2><table class=\"riep\"><thead><tr>");
    h.push_str("<th>Aliquota / natura</th><th class=\"n\">Imponibile</th><th class=\"n\">Imposta</th><th>Esigibilità</th></tr></thead><tbody>");
    for r in &c.riepiloghi {
        let mut prima = iva(r.aliquota_iva, r.natura.as_deref());
        if let Some(rn) = &r.riferimento_normativo {
            prima.push_str(&format!("<div class=\"tenue\">{}</div>", esc(rn)));
        }
        let esig = r
            .esigibilita_iva
            .as_deref()
            .map(|e| codici::esigibilita_iva(e).unwrap_or(e).to_owned())
            .unwrap_or_default();
        h.push_str(&format!(
            "<tr><td>{prima}</td><td class=\"n\">{}</td><td class=\"n\">{}</td><td>{}</td></tr>",
            importo(r.imponibile_importo),
            importo(r.imposta),
            esc(&esig)
        ));
    }
    h.push_str("</tbody></table></section>");
}

fn altri_importi(h: &mut String, d: &DatiGeneraliDocumento) {
    let mut voci: Vec<(String, String)> = Vec::new();
    for c in &d.casse_previdenziali {
        let tipo = c.tipo.as_deref().unwrap_or("");
        voci.push((
            format!(
                "{} {} ({})",
                esc(tipo),
                esc(codici::tipo_cassa(tipo).unwrap_or("Cassa previdenziale")),
                percentuale(c.aliquota)
            ),
            importo(c.importo_contributo),
        ));
    }
    for r in &d.ritenute {
        let tipo = r.tipo.as_deref().unwrap_or("");
        voci.push((
            format!(
                "{} ({}) causale {}",
                esc(codici::tipo_ritenuta(tipo).unwrap_or("Ritenuta")),
                percentuale(r.aliquota),
                esc(r.causale_pagamento.as_deref().unwrap_or("-"))
            ),
            format!("-{}", importo(r.importo)),
        ));
    }
    if let Some(b) = &d.bollo {
        voci.push((
            if b.virtuale {
                "Bollo virtuale".to_owned()
            } else {
                "Bollo".to_owned()
            },
            importo_facoltativo(b.importo),
        ));
    }
    for s in &d.sconti_maggiorazioni {
        let nome = if s.tipo.as_deref() == Some("MG") {
            "Maggiorazione"
        } else {
            "Sconto"
        };
        let valore = match (s.percentuale, s.importo) {
            (Some(p), _) => percentuale(Some(p)),
            (None, Some(i)) => importo(Some(i)),
            _ => String::new(),
        };
        voci.push((format!("{nome} sul documento"), valore));
    }
    if let Some(a) = d.arrotondamento {
        voci.push(("Arrotondamento".to_owned(), importo(Some(a))));
    }
    if d.art73 {
        voci.push((
            "Emesso ai sensi dell'art. 73 DPR 633/72".to_owned(),
            String::new(),
        ));
    }
    if voci.is_empty() {
        return;
    }
    h.push_str("<section class=\"blocco\"><table class=\"riep\"><tbody>");
    for (k, v) in voci {
        h.push_str(&format!("<tr><td>{k}</td><td class=\"n\">{v}</td></tr>"));
    }
    h.push_str("</tbody></table></section>");
}

fn pagamenti(h: &mut String, c: &Corpo) {
    let dettagli: Vec<(&DatiPagamento, &DettaglioPagamento)> = c
        .pagamenti
        .iter()
        .flat_map(|p| p.dettagli.iter().map(move |d| (p, d)))
        .collect();
    if dettagli.is_empty() {
        return;
    }
    h.push_str("<section class=\"blocco\"><h2>Pagamento</h2><table class=\"riep\"><thead><tr>");
    h.push_str("<th>Modalità</th><th>Scadenza</th><th>Coordinate</th><th class=\"n\">Importo</th></tr></thead><tbody>");
    for (p, d) in dettagli {
        let mp = d.modalita_pagamento.as_deref().unwrap_or("");
        let mut modalita = format!(
            "{} {}",
            esc(mp),
            esc(codici::modalita_pagamento(mp).unwrap_or(""))
        );
        if let Some(cond) = p.condizioni_pagamento.as_deref() {
            if let Some(dc) = codici::condizioni_pagamento(cond) {
                modalita.push_str(&format!("<div class=\"tenue\">{}</div>", esc(dc)));
            }
        }
        let mut coord: Vec<String> = Vec::new();
        if let Some(b) = &d.beneficiario {
            coord.push(format!("Beneficiario {}", esc(b)));
        }
        if let Some(iban) = &d.iban {
            coord.push(format!(
                "<span class=\"iban\">{}</span>",
                esc(&formato::iban(iban))
            ));
        }
        if let Some(ist) = &d.istituto_finanziario {
            coord.push(esc(ist));
        }
        if let Some(bic) = &d.bic {
            coord.push(format!("BIC {}", esc(bic)));
        }
        if let Some(cp) = &d.codice_pagamento {
            coord.push(format!("Codice pagamento {}", esc(cp)));
        }
        let scadenza = if d.data_scadenza_pagamento.is_some() {
            data(d.data_scadenza_pagamento)
        } else if let Some(g) = d.giorni_termini_pagamento {
            format!("{g} gg")
        } else {
            String::new()
        };
        h.push_str(&format!(
            "<tr><td>{modalita}</td><td>{scadenza}</td><td>{}</td><td class=\"n\">{}</td></tr>",
            coord.join("<br>"),
            importo(d.importo_pagamento)
        ));
    }
    h.push_str("</tbody></table></section>");
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn genera_html_con_escape() {
        let xml = r#"<FatturaElettronica versione="FPR12"><FatturaElettronicaHeader>
<CedentePrestatore><DatiAnagrafici><Anagrafica><Denominazione>A &amp; B &lt;srl&gt;</Denominazione></Anagrafica></DatiAnagrafici></CedentePrestatore>
<CessionarioCommittente><DatiAnagrafici><Anagrafica><Denominazione>C</Denominazione></Anagrafica></DatiAnagrafici></CessionarioCommittente>
</FatturaElettronicaHeader><FatturaElettronicaBody><DatiGenerali><DatiGeneraliDocumento>
<TipoDocumento>TD01</TipoDocumento><Divisa>EUR</Divisa><Data>2026-01-02</Data><Numero>7</Numero>
<ImportoTotaleDocumento>1234.5</ImportoTotaleDocumento></DatiGeneraliDocumento></DatiGenerali>
<DatiBeniServizi/></FatturaElettronicaBody></FatturaElettronica>"#;
        let f = rustinvoice_core::parser::leggi_da_testo(xml).unwrap();
        let html = html_moderna(&f);
        assert!(html.contains("A &amp; B &lt;srl&gt;"));
        assert!(html.contains("1.234,50 EUR"));
        assert!(html.contains("02/01/2026"));
    }
}
