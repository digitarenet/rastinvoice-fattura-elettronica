// Copyright (c) 2026 Andrea Re - info@digitare.net
// Autore: Andrea Re @ 2026 - info@digitare.net
//
// PDF della fattura nella vista "Moderna", generato direttamente dal modello
// dati: nessun browser e nessuna finestra di stampa. Formato A4, testo
// selezionabile, carattere Titillium Web incorporato.
//
// L'impaginazione avviene in due passi: prima si calcolano le operazioni di
// disegno pagina per pagina (così si conosce il numero totale di pagine per
// il piè di pagina), poi si scrive il PDF.

use crate::formato::{data, importo, numero, percentuale};
use printpdf::{
    path::PaintMode, Color, IndirectFontRef, Line, Mm, PdfDocument, PdfLayerReference, Point, Rect,
    Rgb,
};
use rust_decimal::Decimal;
use rustinvoice_core::codici;
use rustinvoice_core::modello::*;

const FONT_NORMALE: &[u8] = include_bytes!("../font/TitilliumWeb-Regular.ttf");
const FONT_GRASSETTO: &[u8] = include_bytes!("../font/TitilliumWeb-Bold.ttf");

// Pagina A4 e margini, in millimetri.
const LARGHEZZA: f32 = 210.0;
const ALTEZZA: f32 = 297.0;
const MARGINE: f32 = 15.0;
const UTILE: f32 = LARGHEZZA - 2.0 * MARGINE;
const FONDO: f32 = 18.0;

/// Conversione da punti tipografici a millimetri.
const PT: f32 = 0.352_778;

// Colori (palette di Bootstrap Italia).
const BLU: (f32, f32, f32) = (0.0, 0.4, 0.8);
const TESTO: (f32, f32, f32) = (0.1, 0.1, 0.1);
const TENUE: (f32, f32, f32) = (0.36, 0.44, 0.49);
const BORDO: (f32, f32, f32) = (0.85, 0.87, 0.89);
const SFONDO: (f32, f32, f32) = (0.94, 0.96, 0.98);

#[derive(Clone, Copy, PartialEq)]
enum Peso {
    Normale,
    Grassetto,
}

#[derive(Clone, Copy, PartialEq)]
enum Allinea {
    Sinistra,
    Destra,
}

/// Operazione di disegno, con coordinate dall'alto della pagina.
enum Op {
    Testo {
        x: f32,
        y: f32,
        corpo: f32,
        peso: Peso,
        colore: (f32, f32, f32),
        testo: String,
    },
    Linea {
        x1: f32,
        x2: f32,
        y: f32,
        colore: (f32, f32, f32),
        spessore: f32,
    },
    Riquadro {
        x: f32,
        y: f32,
        l: f32,
        h: f32,
        colore: (f32, f32, f32),
    },
}

/// Misura dei testi con le metriche del carattere.
struct Metriche {
    normale: ttf_parser::Face<'static>,
    grassetto: ttf_parser::Face<'static>,
}

impl Metriche {
    fn new() -> Self {
        Self {
            normale: ttf_parser::Face::parse(FONT_NORMALE, 0).expect("carattere incorporato"),
            grassetto: ttf_parser::Face::parse(FONT_GRASSETTO, 0).expect("carattere incorporato"),
        }
    }

    /// Larghezza del testo in millimetri.
    fn larghezza(&self, testo: &str, corpo: f32, peso: Peso) -> f32 {
        let f = if peso == Peso::Grassetto {
            &self.grassetto
        } else {
            &self.normale
        };
        let unita = f.units_per_em() as f32;
        let somma: f32 = testo
            .chars()
            .map(|c| {
                f.glyph_index(c)
                    .and_then(|g| f.glyph_hor_advance(g))
                    .unwrap_or((unita / 2.0) as u16) as f32
            })
            .sum();
        somma / unita * corpo * PT
    }

    /// Divide il testo in righe che stanno nella larghezza indicata.
    fn a_capo(&self, testo: &str, corpo: f32, peso: Peso, larghezza: f32) -> Vec<String> {
        let mut righe = Vec::new();
        for paragrafo in testo.split('\n') {
            let mut riga = String::new();
            for parola in paragrafo.split_whitespace() {
                let prova = if riga.is_empty() {
                    parola.to_owned()
                } else {
                    format!("{riga} {parola}")
                };
                if self.larghezza(&prova, corpo, peso) <= larghezza || riga.is_empty() {
                    riga = prova;
                } else {
                    righe.push(std::mem::take(&mut riga));
                    riga = parola.to_owned();
                }
                // Parola più lunga della colonna (IBAN, codici): spezzata a caratteri.
                while self.larghezza(&riga, corpo, peso) > larghezza && riga.chars().count() > 1 {
                    let mut taglio = String::new();
                    for c in riga.chars() {
                        let prova = format!("{taglio}{c}");
                        if self.larghezza(&prova, corpo, peso) > larghezza && !taglio.is_empty() {
                            break;
                        }
                        taglio = prova;
                    }
                    let resto: String = riga.chars().skip(taglio.chars().count()).collect();
                    righe.push(taglio);
                    riga = resto;
                }
            }
            righe.push(riga);
        }
        righe
    }
}

/// Riga di testo di una cella con peso e colore.
type RigaCella = (String, Peso, (f32, f32, f32));

/// Cella di una tabella: righe di testo con il loro stile.
struct Cella {
    righe: Vec<RigaCella>,
}

impl Cella {
    fn testo(t: impl Into<String>) -> Self {
        Self {
            righe: vec![(t.into(), Peso::Normale, TESTO)],
        }
    }
    fn con_nota(mut self, t: impl Into<String>) -> Self {
        let t = t.into();
        if !t.is_empty() {
            self.righe.push((t, Peso::Normale, TENUE));
        }
        self
    }
}

struct Colonna {
    titolo: &'static str,
    larghezza: f32,
    allinea: Allinea,
}

/// Impaginatore: accumula le operazioni e passa alla pagina successiva quando serve.
struct Impagina {
    m: Metriche,
    pagine: Vec<Vec<Op>>,
    y: f32,
    piede: String,
}

impl Impagina {
    fn new(piede: String) -> Self {
        Self {
            m: Metriche::new(),
            pagine: vec![Vec::new()],
            y: MARGINE,
            piede,
        }
    }

    fn ops(&mut self) -> &mut Vec<Op> {
        self.pagine.last_mut().expect("almeno una pagina")
    }

    fn nuova_pagina(&mut self) {
        self.pagine.push(Vec::new());
        self.y = MARGINE;
    }

    /// Assicura lo spazio verticale indicato, altrimenti cambia pagina.
    fn spazio(&mut self, h: f32) {
        if self.y + h > ALTEZZA - FONDO {
            self.nuova_pagina();
        }
    }

    fn testo(&mut self, x: f32, y: f32, corpo: f32, peso: Peso, colore: (f32, f32, f32), t: &str) {
        if t.is_empty() {
            return;
        }
        self.ops().push(Op::Testo {
            x,
            y,
            corpo,
            peso,
            colore,
            testo: t.to_owned(),
        });
    }

    fn linea(&mut self, x1: f32, x2: f32, y: f32, colore: (f32, f32, f32), spessore: f32) {
        self.ops().push(Op::Linea {
            x1,
            x2,
            y,
            colore,
            spessore,
        });
    }

    /// Paragrafo a capo automatico su tutta la larghezza utile.
    fn paragrafo(&mut self, t: &str, corpo: f32, peso: Peso, colore: (f32, f32, f32)) {
        let interlinea = corpo * PT * 1.35;
        for r in self.m.a_capo(t, corpo, peso, UTILE) {
            self.spazio(interlinea);
            self.testo(MARGINE, self.y + corpo * PT, corpo, peso, colore, &r);
            self.y += interlinea;
        }
    }

    /// Titolo di sezione in blu con una riga sotto.
    fn titolo(&mut self, t: &str) {
        self.spazio(14.0);
        self.y += 3.0;
        self.testo(MARGINE, self.y + 3.6, 10.5, Peso::Grassetto, BLU, t);
        self.y += 5.2;
        self.linea(MARGINE, MARGINE + UTILE, self.y, BLU, 0.4);
        self.y += 1.5;
    }

    /// Tabella con intestazione ripetuta a ogni cambio pagina.
    fn tabella(&mut self, colonne: &[Colonna], righe: Vec<Vec<Cella>>) {
        let corpo = 8.5;
        let interlinea = corpo * PT * 1.3;
        let pad = 1.2;
        // Le colonne a sinistra dopo la prima si staccano di più dai numeri allineati a destra.
        let rientro = |x: f32| if x > MARGINE { 4.0 } else { pad };
        let intestazione = |s: &mut Self| {
            s.spazio(interlinea + 2.0 * pad + 6.0);
            let mut x = MARGINE;
            for c in colonne {
                let tx = match c.allinea {
                    Allinea::Sinistra => x + rientro(x),
                    Allinea::Destra => {
                        x + c.larghezza - pad - s.m.larghezza(c.titolo, 7.5, Peso::Grassetto)
                    }
                };
                s.testo(
                    tx,
                    s.y + pad + 7.5 * PT,
                    7.5,
                    Peso::Grassetto,
                    TENUE,
                    c.titolo,
                );
                x += c.larghezza;
            }
            s.y += interlinea + 2.0 * pad;
            s.linea(MARGINE, MARGINE + UTILE, s.y, TESTO, 0.35);
        };
        intestazione(self);
        for riga in righe {
            // Righe di testo di ogni cella, già spezzate.
            let celle: Vec<Vec<RigaCella>> = riga
                .iter()
                .zip(colonne)
                .map(|(cella, col)| {
                    cella
                        .righe
                        .iter()
                        .flat_map(|(t, peso, colore)| {
                            let c = if *colore == TENUE { 7.5 } else { corpo };
                            self.m
                                .a_capo(t, c, *peso, col.larghezza - 2.0 * pad)
                                .into_iter()
                                .map(move |r| (r, *peso, *colore))
                        })
                        .collect()
                })
                .collect();
            let n = celle.iter().map(Vec::len).max().unwrap_or(1).max(1);
            let h = n as f32 * interlinea + 2.0 * pad;
            if self.y + h > ALTEZZA - FONDO {
                self.nuova_pagina();
                intestazione(self);
            }
            let mut x = MARGINE;
            for (righe_cella, col) in celle.iter().zip(colonne) {
                for (i, (t, peso, colore)) in righe_cella.iter().enumerate() {
                    let c = if *colore == TENUE { 7.5 } else { corpo };
                    let tx = match col.allinea {
                        Allinea::Sinistra => x + rientro(x),
                        Allinea::Destra => x + col.larghezza - pad - self.m.larghezza(t, c, *peso),
                    };
                    let ty = self.y + pad + corpo * PT + i as f32 * interlinea;
                    self.testo(tx, ty, c, *peso, *colore, t);
                }
                x += col.larghezza;
            }
            self.y += h;
            self.linea(MARGINE, MARGINE + UTILE, self.y, BORDO, 0.25);
        }
        self.y += 2.0;
    }

    /// Scrive il PDF con il piè di pagina numerato.
    fn pdf(mut self, titolo: &str) -> Result<Vec<u8>, String> {
        let totale = self.pagine.len();
        for (i, pagina) in self.pagine.iter_mut().enumerate() {
            let y = ALTEZZA - 10.0;
            pagina.push(Op::Linea {
                x1: MARGINE,
                x2: MARGINE + UTILE,
                y: y - 4.0,
                colore: BORDO,
                spessore: 0.25,
            });
            pagina.push(Op::Testo {
                x: MARGINE,
                y,
                corpo: 7.0,
                peso: Peso::Normale,
                colore: TENUE,
                testo: self.piede.clone(),
            });
            let numero = format!("Pagina {} di {}", i + 1, totale);
            let l = self.m.larghezza(&numero, 7.0, Peso::Normale);
            pagina.push(Op::Testo {
                x: MARGINE + UTILE - l,
                y,
                corpo: 7.0,
                peso: Peso::Normale,
                colore: TENUE,
                testo: numero,
            });
        }

        let (doc, p1, l1) = PdfDocument::new(titolo, Mm(LARGHEZZA), Mm(ALTEZZA), "Fattura");
        let doc = doc.with_creator("RastInvoice").with_producer("RastInvoice");
        let normale = doc
            .add_external_font(FONT_NORMALE)
            .map_err(|e| e.to_string())?;
        let grassetto = doc
            .add_external_font(FONT_GRASSETTO)
            .map_err(|e| e.to_string())?;
        for (i, pagina) in self.pagine.iter().enumerate() {
            let livello = if i == 0 {
                doc.get_page(p1).get_layer(l1)
            } else {
                let (p, l) = doc.add_page(Mm(LARGHEZZA), Mm(ALTEZZA), "Fattura");
                doc.get_page(p).get_layer(l)
            };
            disegna(&livello, pagina, &normale, &grassetto);
        }
        doc.save_to_bytes().map_err(|e| e.to_string())
    }
}

fn colore((r, g, b): (f32, f32, f32)) -> Color {
    Color::Rgb(Rgb::new(r, g, b, None))
}

fn disegna(
    livello: &PdfLayerReference,
    ops: &[Op],
    normale: &IndirectFontRef,
    grassetto: &IndirectFontRef,
) {
    for op in ops {
        match op {
            Op::Riquadro {
                x,
                y,
                l,
                h,
                colore: c,
            } => {
                livello.set_fill_color(colore(*c));
                livello.add_rect(
                    Rect::new(Mm(*x), Mm(ALTEZZA - y - h), Mm(x + l), Mm(ALTEZZA - y))
                        .with_mode(PaintMode::Fill),
                );
            }
            Op::Linea {
                x1,
                x2,
                y,
                colore: c,
                spessore,
            } => {
                livello.set_outline_color(colore(*c));
                livello.set_outline_thickness(*spessore / PT);
                livello.add_line(Line {
                    points: vec![
                        (Point::new(Mm(*x1), Mm(ALTEZZA - y)), false),
                        (Point::new(Mm(*x2), Mm(ALTEZZA - y)), false),
                    ],
                    is_closed: false,
                });
            }
            Op::Testo {
                x,
                y,
                corpo,
                peso,
                colore: c,
                testo,
            } => {
                livello.set_fill_color(colore(*c));
                let font = if *peso == Peso::Grassetto {
                    grassetto
                } else {
                    normale
                };
                livello.use_text(testo.as_str(), *corpo, Mm(*x), Mm(ALTEZZA - y), font);
            }
        }
    }
}

/// Genera il PDF di tutti i documenti contenuti nel file.
pub fn pdf_moderna(f: &FatturaElettronica) -> Result<Vec<u8>, String> {
    let t = &f.intestazione.dati_trasmissione;
    let mut piede = format!(
        "Formato {} · Progressivo {}",
        f.versione.as_deref().unwrap_or("-"),
        t.progressivo_invio.as_deref().unwrap_or("-")
    );
    if let Some(cd) = &t.codice_destinatario {
        piede.push_str(&format!(" · Codice destinatario {cd}"));
    }
    piede.push_str(" · Copia di cortesia generata da RastInvoice");
    let mut p = Impagina::new(piede);
    for (i, corpo) in f.corpi.iter().enumerate() {
        if i > 0 {
            p.nuova_pagina();
        }
        documento(&mut p, f, corpo, i, f.corpi.len());
    }
    if !f.avvisi.is_empty() {
        p.titolo("Anomalie rilevate nel file");
        for a in &f.avvisi {
            p.paragrafo(&format!("• {a}"), 8.5, Peso::Normale, TESTO);
        }
    }
    let titolo = f
        .corpi
        .first()
        .map(|c| {
            let d = &c.dati_generali.documento;
            format!(
                "{} n. {} del {}",
                codici::tipo_documento(d.tipo_documento.as_deref().unwrap_or(""))
                    .unwrap_or("Documento"),
                d.numero.as_deref().unwrap_or("-"),
                data(d.data)
            )
        })
        .unwrap_or_else(|| "Fattura".into());
    p.pdf(&titolo)
}

fn documento(p: &mut Impagina, f: &FatturaElettronica, c: &Corpo, indice: usize, n: usize) {
    let d = &c.dati_generali.documento;
    let tipo = d.tipo_documento.as_deref().unwrap_or("");
    let divisa = d.divisa.as_deref().unwrap_or("EUR");

    // Intestazione: tipo, numero e data a sinistra, totale a destra.
    let x = MARGINE;
    if n > 1 {
        p.testo(
            x,
            p.y + 3.0,
            8.0,
            Peso::Normale,
            TENUE,
            &format!("Documento {} di {} del lotto", indice + 1, n),
        );
        p.y += 4.5;
    }
    let titolo = format!(
        "{} n. {}",
        codici::tipo_documento(tipo).unwrap_or("Documento"),
        d.numero.as_deref().unwrap_or("-")
    );
    let righe_titolo = p.m.a_capo(&titolo, 16.0, Peso::Grassetto, UTILE - 62.0);
    let y0 = p.y;
    for (i, r) in righe_titolo.iter().enumerate() {
        p.testo(
            x,
            y0 + 6.0 + i as f32 * 6.5,
            16.0,
            Peso::Grassetto,
            TESTO,
            r,
        );
    }
    let y_sotto = y0 + 6.0 + righe_titolo.len() as f32 * 6.5;
    p.testo(
        x,
        y_sotto,
        9.0,
        Peso::Normale,
        TENUE,
        &format!("del {} · {} · {}", data(d.data), tipo, divisa),
    );

    // Riquadro del totale.
    let lr = 58.0;
    let xr = MARGINE + UTILE - lr;
    let hr = if c.prima_scadenza().is_some() {
        20.0
    } else {
        15.0
    };
    p.ops().push(Op::Riquadro {
        x: xr,
        y: y0,
        l: lr,
        h: hr,
        colore: SFONDO,
    });
    p.testo(
        xr + 3.0,
        y0 + 5.0,
        7.5,
        Peso::Normale,
        TENUE,
        "TOTALE DOCUMENTO",
    );
    let tot = format!("{} {}", importo(c.totale()), divisa);
    p.testo(xr + 3.0, y0 + 11.5, 14.0, Peso::Grassetto, BLU, &tot);
    if let Some(s) = c.prima_scadenza() {
        p.testo(
            xr + 3.0,
            y0 + 16.5,
            8.0,
            Peso::Normale,
            TENUE,
            &format!("scadenza {}", data(Some(s))),
        );
    }
    p.y = (y_sotto + 3.0).max(y0 + hr) + 4.0;

    // Soggetti, su due colonne.
    let int = &f.intestazione;
    parti(
        p,
        ("Cedente / prestatore", &int.cedente_prestatore, true),
        (
            "Cessionario / committente",
            &int.cessionario_committente,
            false,
        ),
    );
    if let Some(r) = &int.rappresentante_fiscale {
        parte_singola(p, "Rappresentante fiscale", r);
    }
    if let Some(t) = &int.terzo_intermediario {
        parte_singola(p, "Terzo intermediario / soggetto emittente", t);
    }

    if !d.causale.is_empty() {
        p.titolo("Causale");
        p.paragrafo(&d.causale.join(" "), 9.0, Peso::Normale, TESTO);
    }
    riferimenti(p, &c.dati_generali);
    righe(p, c);
    riepilogo(p, c);
    altri_importi(p, d);
    pagamenti(p, c);

    if !c.allegati.is_empty() {
        p.titolo("Allegati");
        for a in &c.allegati {
            p.paragrafo(
                &format!(
                    "{} {} ({} · {} KB)",
                    a.nome.as_deref().unwrap_or("senza nome"),
                    a.descrizione.as_deref().unwrap_or(""),
                    a.formato.as_deref().unwrap_or(""),
                    a.dimensione.div_ceil(1024)
                ),
                8.5,
                Peso::Normale,
                TESTO,
            );
        }
    }
}

/// Righe descrittive di un soggetto.
fn righe_soggetto(s: &Soggetto, cedente: bool) -> Vec<String> {
    let mut r = Vec::new();
    if let Some(sede) = &s.sede {
        r.push(sede.su_una_riga());
    }
    if let Some(id) = &s.id_fiscale_iva {
        r.push(format!("P.IVA {}", id.compatto()));
    }
    if let Some(cf) = &s.codice_fiscale {
        r.push(format!("C.F. {cf}"));
    }
    if cedente {
        if let Some(rf) = &s.regime_fiscale {
            r.push(format!(
                "Regime {rf} {}",
                codici::regime_fiscale(rf).unwrap_or("codice non previsto")
            ));
        }
        if let Some(rea) = &s.iscrizione_rea {
            let mut t = format!(
                "REA {}-{}",
                rea.ufficio.as_deref().unwrap_or(""),
                rea.numero_rea.as_deref().unwrap_or("")
            );
            if let Some(cap) = rea.capitale_sociale {
                t.push_str(&format!(" · Cap. soc. {}", importo(Some(cap))));
            }
            if rea.stato_liquidazione.as_deref() == Some("LS") {
                t.push_str(" · in liquidazione");
            }
            r.push(t);
        }
        if let Some(albo) = &s.albo_professionale {
            r.push(format!(
                "Albo {albo} {} n. {}",
                s.provincia_albo.as_deref().unwrap_or(""),
                s.numero_iscrizione_albo.as_deref().unwrap_or("")
            ));
        }
    }
    if let Some(c) = &s.contatti {
        let v: Vec<&str> = [c.telefono.as_deref(), c.email.as_deref()]
            .into_iter()
            .flatten()
            .collect();
        if !v.is_empty() {
            r.push(v.join(" · "));
        }
    }
    r
}

/// Disegna un soggetto in una colonna e restituisce l'altezza occupata.
fn colonna_soggetto(
    p: &mut Impagina,
    x: f32,
    l: f32,
    titolo: &str,
    s: &Soggetto,
    ced: bool,
) -> f32 {
    let mut y = p.y;
    p.testo(
        x,
        y + 3.0,
        7.5,
        Peso::Grassetto,
        TENUE,
        &titolo.to_uppercase(),
    );
    y += 4.5;
    for r in p.m.a_capo(&s.nome_visualizzato(), 11.0, Peso::Grassetto, l) {
        p.testo(x, y + 4.0, 11.0, Peso::Grassetto, TESTO, &r);
        y += 5.0;
    }
    for riga in righe_soggetto(s, ced) {
        for r in p.m.a_capo(&riga, 8.5, Peso::Normale, l) {
            p.testo(x, y + 3.2, 8.5, Peso::Normale, TESTO, &r);
            y += 4.0;
        }
    }
    y - p.y
}

fn parti(p: &mut Impagina, a: (&str, &Soggetto, bool), b: (&str, &Soggetto, bool)) {
    p.spazio(40.0);
    let l = UTILE / 2.0 - 4.0;
    let ha = colonna_soggetto(p, MARGINE, l, a.0, a.1, a.2);
    let hb = colonna_soggetto(p, MARGINE + UTILE / 2.0 + 4.0, l, b.0, b.1, b.2);
    p.y += ha.max(hb) + 3.0;
}

fn parte_singola(p: &mut Impagina, titolo: &str, s: &Soggetto) {
    p.spazio(25.0);
    let h = colonna_soggetto(p, MARGINE, UTILE, titolo, s, false);
    p.y += h + 3.0;
}

fn riferimenti(p: &mut Impagina, dg: &DatiGenerali) {
    let mut voci: Vec<String> = Vec::new();
    let mut aggiungi = |etichetta: &str, lista: &[DocumentoCorrelato]| {
        for x in lista {
            let mut s = format!("{etichetta} {}", x.id_documento.as_deref().unwrap_or("-"));
            if x.data.is_some() {
                s.push_str(&format!(" del {}", data(x.data)));
            }
            if let Some(cig) = &x.codice_cig {
                s.push_str(&format!(" · CIG {cig}"));
            }
            if let Some(cup) = &x.codice_cup {
                s.push_str(&format!(" · CUP {cup}"));
            }
            if let Some(cc) = &x.codice_commessa_convenzione {
                s.push_str(&format!(" · Commessa {cc}"));
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
            ddt.numero.as_deref().unwrap_or("-"),
            data(ddt.data)
        ));
    }
    if let Some(fp) = &dg.fattura_principale {
        voci.push(format!(
            "Fattura principale {} del {}",
            fp.numero.as_deref().unwrap_or("-"),
            data(fp.data)
        ));
    }
    if let Some(fr) = &dg.fattura_rettificata {
        voci.push(format!(
            "Rettifica della fattura {} del {}",
            fr.numero.as_deref().unwrap_or("-"),
            data(fr.data)
        ));
    }
    if !dg.sal.is_empty() {
        voci.push(format!("SAL fase {}", dg.sal.join(", ")));
    }
    if voci.is_empty() {
        return;
    }
    p.titolo("Riferimenti");
    for v in voci {
        p.paragrafo(&format!("• {v}"), 8.5, Peso::Normale, TESTO);
    }
}

fn iva(aliquota: Option<Decimal>, natura: Option<&str>) -> String {
    match natura {
        Some(n) => n.to_owned(),
        None => percentuale(aliquota),
    }
}

fn facoltativo(v: Option<Decimal>) -> String {
    if v.is_some() {
        importo(v)
    } else {
        String::new()
    }
}

fn righe(p: &mut Impagina, c: &Corpo) {
    if c.linee.is_empty() {
        return;
    }
    p.titolo("Dettaglio");
    let colonne = [
        Colonna {
            titolo: "Descrizione",
            larghezza: UTILE - 104.0,
            allinea: Allinea::Sinistra,
        },
        Colonna {
            titolo: "Q.tà",
            larghezza: 20.0,
            allinea: Allinea::Destra,
        },
        Colonna {
            titolo: "Prezzo",
            larghezza: 22.0,
            allinea: Allinea::Destra,
        },
        Colonna {
            titolo: "Sconti",
            larghezza: 18.0,
            allinea: Allinea::Destra,
        },
        Colonna {
            titolo: "Importo",
            larghezza: 26.0,
            allinea: Allinea::Destra,
        },
        Colonna {
            titolo: "IVA",
            larghezza: 18.0,
            allinea: Allinea::Destra,
        },
    ];
    let mut righe = Vec::new();
    for l in &c.linee {
        let codici_art: Vec<&str> = l
            .codici_articolo
            .iter()
            .filter_map(|ca| ca.valore.as_deref())
            .collect();
        let mut descr = String::new();
        if !codici_art.is_empty() {
            descr.push_str(&format!("[{}] ", codici_art.join(" / ")));
        }
        descr.push_str(l.descrizione.as_deref().unwrap_or(""));
        let mut cella = Cella::testo(descr);
        if l.data_inizio_periodo.is_some() || l.data_fine_periodo.is_some() {
            cella = cella.con_nota(format!(
                "Periodo {} - {}",
                data(l.data_inizio_periodo),
                data(l.data_fine_periodo)
            ));
        }
        if let Some(nl) = l.numero_linea {
            for ddt in &c.dati_generali.ddt {
                if ddt.riferimento_numero_linea.contains(&nl) {
                    cella = cella.con_nota(format!(
                        "DDT {} del {}",
                        ddt.numero.as_deref().unwrap_or("-"),
                        data(ddt.data)
                    ));
                }
            }
            for o in &c.dati_generali.ordini_acquisto {
                if o.riferimento_numero_linea.contains(&nl) {
                    cella = cella.con_nota(format!(
                        "Ordine {}",
                        o.id_documento.as_deref().unwrap_or("-")
                    ));
                }
            }
        }
        for a in &l.altri_dati_gestionali {
            let tipo = a.tipo_dato.as_deref().unwrap_or("");
            let mut parti = vec![match codici::tipo_dato_normativo(tipo) {
                Some(descr) => format!("{descr} ({tipo})"),
                None => tipo.to_owned(),
            }];
            if let Some(t) = &a.riferimento_testo {
                parti.push(t.clone());
            }
            if let Some(n) = a.riferimento_numero {
                parti.push(numero(Some(n)));
            }
            if a.riferimento_data.is_some() {
                parti.push(data(a.riferimento_data));
            }
            cella = cella.con_nota(parti.join(" · "));
        }
        let qta = match (&l.quantita, &l.unita_misura) {
            (Some(q), Some(u)) => format!("{} {u}", numero(Some(*q))),
            (Some(q), None) => numero(Some(*q)),
            _ => String::new(),
        };
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
                    (Some(pc), _) => format!("{segno}{}", percentuale(Some(pc))),
                    (None, Some(i)) => format!("{segno}{}", importo(Some(i))),
                    _ => String::new(),
                }
            })
            .collect();
        righe.push(vec![
            cella,
            Cella::testo(qta),
            Cella::testo(facoltativo(l.prezzo_unitario)),
            Cella::testo(sconti.join(" ")),
            Cella::testo(facoltativo(l.prezzo_totale)),
            Cella::testo(iva(l.aliquota_iva, l.natura.as_deref())),
        ]);
    }
    p.tabella(&colonne, righe);
}

fn riepilogo(p: &mut Impagina, c: &Corpo) {
    if c.riepiloghi.is_empty() {
        return;
    }
    p.titolo("Riepilogo IVA");
    let colonne = [
        Colonna {
            titolo: "Aliquota / natura",
            larghezza: UTILE - 112.0,
            allinea: Allinea::Sinistra,
        },
        Colonna {
            titolo: "Imponibile",
            larghezza: 30.0,
            allinea: Allinea::Destra,
        },
        Colonna {
            titolo: "Imposta",
            larghezza: 30.0,
            allinea: Allinea::Destra,
        },
        Colonna {
            titolo: "Esigibilità",
            larghezza: 52.0,
            allinea: Allinea::Sinistra,
        },
    ];
    let righe = c
        .riepiloghi
        .iter()
        .map(|r| {
            let mut prima = Cella::testo(match r.natura.as_deref() {
                Some(n) => format!("{n} {}", codici::natura(n).unwrap_or("codice non previsto")),
                None => percentuale(r.aliquota_iva),
            });
            if let Some(rn) = &r.riferimento_normativo {
                prima = prima.con_nota(rn.clone());
            }
            let esig = r
                .esigibilita_iva
                .as_deref()
                .map(|e| codici::esigibilita_iva(e).unwrap_or(e).to_owned())
                .unwrap_or_default();
            vec![
                prima,
                Cella::testo(importo(r.imponibile_importo)),
                Cella::testo(importo(r.imposta)),
                Cella::testo(esig),
            ]
        })
        .collect();
    p.tabella(&colonne, righe);
}

fn altri_importi(p: &mut Impagina, d: &DatiGeneraliDocumento) {
    let mut voci: Vec<(String, String)> = Vec::new();
    for c in &d.casse_previdenziali {
        let tipo = c.tipo.as_deref().unwrap_or("");
        voci.push((
            format!(
                "{tipo} {} ({})",
                codici::tipo_cassa(tipo).unwrap_or("Cassa previdenziale"),
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
                codici::tipo_ritenuta(tipo).unwrap_or("Ritenuta"),
                percentuale(r.aliquota),
                r.causale_pagamento.as_deref().unwrap_or("-")
            ),
            format!("-{}", importo(r.importo)),
        ));
    }
    if let Some(b) = &d.bollo {
        voci.push((
            if b.virtuale {
                "Bollo virtuale"
            } else {
                "Bollo"
            }
            .to_owned(),
            facoltativo(b.importo),
        ));
    }
    for s in &d.sconti_maggiorazioni {
        let nome = if s.tipo.as_deref() == Some("MG") {
            "Maggiorazione"
        } else {
            "Sconto"
        };
        let valore = match (s.percentuale, s.importo) {
            (Some(pc), _) => percentuale(Some(pc)),
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
    p.titolo("Altri importi");
    let colonne = [
        Colonna {
            titolo: "Voce",
            larghezza: UTILE - 40.0,
            allinea: Allinea::Sinistra,
        },
        Colonna {
            titolo: "Importo",
            larghezza: 40.0,
            allinea: Allinea::Destra,
        },
    ];
    let righe = voci
        .into_iter()
        .map(|(k, v)| vec![Cella::testo(k), Cella::testo(v)])
        .collect();
    p.tabella(&colonne, righe);
}

fn pagamenti(p: &mut Impagina, c: &Corpo) {
    let dettagli: Vec<(&DatiPagamento, &DettaglioPagamento)> = c
        .pagamenti
        .iter()
        .flat_map(|pg| pg.dettagli.iter().map(move |d| (pg, d)))
        .collect();
    if dettagli.is_empty() {
        return;
    }
    p.titolo("Pagamento");
    let colonne = [
        Colonna {
            titolo: "Modalità",
            larghezza: 50.0,
            allinea: Allinea::Sinistra,
        },
        Colonna {
            titolo: "Scadenza",
            larghezza: 22.0,
            allinea: Allinea::Sinistra,
        },
        Colonna {
            titolo: "Coordinate",
            larghezza: UTILE - 102.0,
            allinea: Allinea::Sinistra,
        },
        Colonna {
            titolo: "Importo",
            larghezza: 30.0,
            allinea: Allinea::Destra,
        },
    ];
    let righe = dettagli
        .into_iter()
        .map(|(pg, d)| {
            let mp = d.modalita_pagamento.as_deref().unwrap_or("");
            let mut modalita = Cella::testo(format!(
                "{mp} {}",
                codici::modalita_pagamento(mp).unwrap_or("")
            ));
            if let Some(dc) = pg
                .condizioni_pagamento
                .as_deref()
                .and_then(codici::condizioni_pagamento)
            {
                modalita = modalita.con_nota(dc);
            }
            let mut coord = Cella { righe: Vec::new() };
            let mut aggiungi = |t: String| coord.righe.push((t, Peso::Normale, TESTO));
            if let Some(b) = &d.beneficiario {
                aggiungi(format!("Beneficiario {b}"));
            }
            if let Some(iban) = &d.iban {
                aggiungi(crate::formato::iban(iban));
            }
            if let Some(ist) = &d.istituto_finanziario {
                aggiungi(ist.clone());
            }
            if let Some(bic) = &d.bic {
                aggiungi(format!("BIC {bic}"));
            }
            if let Some(cp) = &d.codice_pagamento {
                aggiungi(format!("Codice pagamento {cp}"));
            }
            let scadenza = if d.data_scadenza_pagamento.is_some() {
                data(d.data_scadenza_pagamento)
            } else if let Some(g) = d.giorni_termini_pagamento {
                format!("{g} gg")
            } else {
                String::new()
            };
            vec![
                modalita,
                Cella::testo(scadenza),
                coord,
                Cella::testo(importo(d.importo_pagamento)),
            ]
        })
        .collect();
    p.tabella(&colonne, righe);
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn genera_pdf_valido() {
        let xml = r#"<FatturaElettronica versione="FPR12"><FatturaElettronicaHeader>
<CedentePrestatore><DatiAnagrafici><IdFiscaleIVA><IdPaese>IT</IdPaese><IdCodice>01234567890</IdCodice></IdFiscaleIVA><Anagrafica><Denominazione>Società Più &amp; Co. S.r.l.</Denominazione></Anagrafica><RegimeFiscale>RF01</RegimeFiscale></DatiAnagrafici></CedentePrestatore>
<CessionarioCommittente><DatiAnagrafici><Anagrafica><Denominazione>Cliente</Denominazione></Anagrafica></DatiAnagrafici></CessionarioCommittente>
</FatturaElettronicaHeader><FatturaElettronicaBody><DatiGenerali><DatiGeneraliDocumento>
<TipoDocumento>TD01</TipoDocumento><Divisa>EUR</Divisa><Data>2026-01-02</Data><Numero>7/A</Numero>
<ImportoTotaleDocumento>1220.00</ImportoTotaleDocumento></DatiGeneraliDocumento></DatiGenerali>
<DatiBeniServizi><DettaglioLinee><NumeroLinea>1</NumeroLinea><Descrizione>Consulenza tecnica con una descrizione molto lunga che deve andare a capo più volte nella colonna</Descrizione><PrezzoUnitario>1000.00</PrezzoUnitario><PrezzoTotale>1000.00</PrezzoTotale><AliquotaIVA>22.00</AliquotaIVA></DettaglioLinee>
<DatiRiepilogo><AliquotaIVA>22.00</AliquotaIVA><ImponibileImporto>1000.00</ImponibileImporto><Imposta>220.00</Imposta></DatiRiepilogo></DatiBeniServizi></FatturaElettronicaBody></FatturaElettronica>"#;
        let f = rustinvoice_core::parser::leggi_da_testo(xml).unwrap();
        let pdf = pdf_moderna(&f).unwrap();
        assert!(pdf.starts_with(b"%PDF-"));
        assert!(pdf.len() > 1000);
    }

    #[test]
    fn a_capo_rispetta_la_larghezza() {
        let m = Metriche::new();
        let righe = m.a_capo(
            "uno due tre quattro cinque sei sette",
            10.0,
            Peso::Normale,
            20.0,
        );
        assert!(righe.len() > 1);
        for r in &righe {
            assert!(m.larghezza(r, 10.0, Peso::Normale) <= 20.0 || !r.contains(' '));
        }
        // Una parola lunghissima viene spezzata.
        let lunga = m.a_capo("IT60X0542811101000000123456", 10.0, Peso::Normale, 15.0);
        assert!(lunga.len() > 1);
    }
}
