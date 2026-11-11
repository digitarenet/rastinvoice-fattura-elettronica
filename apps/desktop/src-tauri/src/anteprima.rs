// Copyright (c) 2026 Andrea Re - info@digitare.net
// Autore: Andrea Re @ 2026 - info@digitare.net
//
// Anteprima degli allegati delle fatture. Le specifiche FatturaPA non
// limitano il formato (`FormatoAttachment` è testo libero): il tipo si
// riconosce quindi dal contenuto e non dal nome scelto dal mittente.
//
// Fogli di calcolo, CSV, testo e archivi vengono letti qui in Rust e passati
// all'interfaccia come dati; PDF, immagini e documenti Word vengono passati
// come byte e mostrati con visualizzatori che non eseguono script.

use calamine::{Data, Reader};
use serde::Serialize;
use std::io::{Cursor, Read};

/// Dimensione massima di un allegato da mostrare.
const MASSIMO_ANTEPRIMA: usize = 64 * 1024 * 1024;
/// Limiti delle tabelle mostrate, per non bloccare l'interfaccia.
const MASSIME_RIGHE: usize = 5000;
const MASSIME_COLONNE: usize = 200;
/// Testo mostrato al massimo (oltre si tronca).
const MASSIMO_TESTO: usize = 4 * 1024 * 1024;
/// Profondità massima di archivi e buste annidati.
const MASSIMA_PROFONDITA: usize = 4;

#[derive(Debug, Serialize)]
#[serde(tag = "tipo", rename_all = "snake_case")]
pub enum Anteprima {
    /// Immagine mostrata dall'interfaccia con un elemento `<img>`.
    Immagine { mime: String },
    /// PDF mostrato con il visualizzatore integrato.
    Pdf,
    /// Documento Word (.docx) convertito in pagina dall'interfaccia.
    Word,
    /// Testo semplice, XML o JSON.
    Testo {
        testo: String,
        formato: String,
        troncato: bool,
    },
    /// Pagina HTML, mostrata in una cornice senza script.
    Html { html: String },
    /// Fogli di calcolo e CSV.
    Tabella { fogli: Vec<Foglio> },
    /// Archivio zip: elenco dei file contenuti.
    Archivio { voci: Vec<VoceArchivio> },
    /// Formato non mostrabile: si può aprire con il programma predefinito o salvare.
    NonDisponibile { motivo: String },
}

#[derive(Debug, Serialize)]
pub struct Foglio {
    pub nome: String,
    pub righe: Vec<Vec<String>>,
    pub troncato: bool,
}

#[derive(Debug, Serialize)]
pub struct VoceArchivio {
    pub nome: String,
    pub dimensione: u64,
}

/// Tipo riconosciuto dal contenuto.
#[derive(Debug, Clone, Copy, PartialEq)]
enum Tipo {
    Pdf,
    Png,
    Jpeg,
    Gif,
    Webp,
    Bmp,
    Svg,
    Tiff,
    Zip,
    Docx,
    FoglioZip,
    TestoOdt,
    Ole,
    P7m,
    Rtf,
    Html,
    Testo,
    Sconosciuto,
}

/// Riconosce il tipo dai primi byte e, per gli archivi zip, dal contenuto.
fn riconosci(nome: &str, b: &[u8]) -> Tipo {
    let est = estensione(nome);
    if b.starts_with(b"%PDF-") {
        return Tipo::Pdf;
    }
    if b.starts_with(b"\x89PNG\r\n\x1a\n") {
        return Tipo::Png;
    }
    if b.starts_with(&[0xFF, 0xD8, 0xFF]) {
        return Tipo::Jpeg;
    }
    if b.starts_with(b"GIF87a") || b.starts_with(b"GIF89a") {
        return Tipo::Gif;
    }
    if b.len() > 12 && &b[..4] == b"RIFF" && &b[8..12] == b"WEBP" {
        return Tipo::Webp;
    }
    if b.starts_with(b"BM") && matches!(est.as_str(), "bmp" | "dib") {
        return Tipo::Bmp;
    }
    if b.starts_with(b"II*\0") || b.starts_with(b"MM\0*") {
        return Tipo::Tiff;
    }
    if b.starts_with(b"PK\x03\x04") || b.starts_with(b"PK\x05\x06") {
        return riconosci_zip(b);
    }
    if b.starts_with(&[0xD0, 0xCF, 0x11, 0xE0, 0xA1, 0xB1, 0x1A, 0xE1]) {
        return Tipo::Ole;
    }
    if b.starts_with(b"{\\rtf") {
        return Tipo::Rtf;
    }
    if b.first() == Some(&0x30) && est == "p7m" || rustinvoice_core::p7m::sembra_p7m(b) {
        return Tipo::P7m;
    }
    // Formati di testo: si guarda l'inizio dopo eventuali spazi e BOM.
    let inizio = testo_iniziale(b);
    let minuscolo = inizio.to_lowercase();
    if minuscolo.starts_with("<svg")
        || (minuscolo.starts_with("<?xml") && minuscolo.contains("<svg"))
    {
        return Tipo::Svg;
    }
    if minuscolo.starts_with("<!doctype html")
        || minuscolo.starts_with("<html")
        || matches!(est.as_str(), "htm" | "html") && sembra_testo(b)
    {
        return Tipo::Html;
    }
    if sembra_testo(b) {
        return Tipo::Testo;
    }
    Tipo::Sconosciuto
}

fn estensione(nome: &str) -> String {
    std::path::Path::new(nome)
        .extension()
        .map(|e| e.to_string_lossy().to_lowercase())
        .unwrap_or_default()
}

fn testo_iniziale(b: &[u8]) -> String {
    let pezzo = &b[..b.len().min(512)];
    let (t, _, _) = encoding_rs::UTF_8.decode(pezzo);
    t.trim_start_matches('\u{feff}').trim_start().to_owned()
}

/// Vero se il contenuto sembra testo: niente byte nulli (salvo UTF-16 con BOM)
/// e pochi caratteri di controllo.
fn sembra_testo(b: &[u8]) -> bool {
    if b.starts_with(&[0xFF, 0xFE]) || b.starts_with(&[0xFE, 0xFF]) {
        return true;
    }
    let campione = &b[..b.len().min(8192)];
    if campione.contains(&0) {
        return false;
    }
    let controllo = campione
        .iter()
        .filter(|&&c| c < 0x20 && !matches!(c, b'\n' | b'\r' | b'\t' | 0x0C))
        .count();
    controllo * 100 <= campione.len()
}

/// I formati Office moderni e OpenDocument sono archivi zip: si distinguono
/// dai file che contengono.
fn riconosci_zip(b: &[u8]) -> Tipo {
    let Ok(mut archivio) = zip::ZipArchive::new(Cursor::new(b)) else {
        return Tipo::Sconosciuto;
    };
    let nomi: Vec<String> = archivio
        .file_names()
        .filter_map(|n| n.ok().map(|n| n.into_owned()))
        .collect();
    if nomi.iter().any(|n| n == "word/document.xml") {
        return Tipo::Docx;
    }
    if nomi
        .iter()
        .any(|n| n == "xl/workbook.xml" || n == "xl/workbook.bin")
    {
        return Tipo::FoglioZip;
    }
    if let Ok(mut m) = archivio.by_name("mimetype") {
        let mut tipo = String::new();
        let _ = (&mut m).take(200).read_to_string(&mut tipo);
        if tipo.contains("opendocument.spreadsheet") {
            return Tipo::FoglioZip;
        }
        if tipo.contains("opendocument.text") {
            return Tipo::TestoOdt;
        }
    }
    Tipo::Zip
}

/// Byte da mostrare: per buste p7m e file dentro un archivio, il contenuto
/// interno indicato da `interno` (nomi separati da `/` per più livelli).
pub fn risolvi(nome: &str, byte: Vec<u8>, interno: &[String]) -> Result<(String, Vec<u8>), String> {
    let mut nome = nome.to_owned();
    let mut byte = byte;
    let mut profondita = 0;
    // Le buste firmate si aprono sempre: si mostra il documento firmato.
    loop {
        if profondita > MASSIMA_PROFONDITA {
            return Err("troppi livelli annidati".into());
        }
        if riconosci(&nome, &byte) == Tipo::P7m {
            let busta = rustinvoice_core::p7m::estrai(&byte).map_err(|e| e.to_string())?;
            byte = busta.contenuto;
            nome = nome
                .strip_suffix(".p7m")
                .or_else(|| nome.strip_suffix(".P7M"))
                .map(str::to_owned)
                .unwrap_or(nome);
            profondita += 1;
            continue;
        }
        break;
    }
    if let Some((primo, resto)) = interno.split_first() {
        if !matches!(riconosci(&nome, &byte), Tipo::Zip) {
            return Err("il file non è un archivio".into());
        }
        let dentro = leggi_da_zip(&byte, primo)?;
        return risolvi(primo, dentro, resto);
    }
    Ok((nome, byte))
}

fn leggi_da_zip(byte: &[u8], nome: &str) -> Result<Vec<u8>, String> {
    let mut archivio = zip::ZipArchive::new(Cursor::new(byte)).map_err(|e| e.to_string())?;
    let mut voce = archivio
        .by_name(nome)
        .map_err(|_| "file non trovato nell'archivio")?;
    if voce.size() as usize > MASSIMO_ANTEPRIMA {
        return Err("file troppo grande per l'anteprima".into());
    }
    let mut b = Vec::new();
    (&mut voce)
        .take(MASSIMO_ANTEPRIMA as u64 + 1)
        .read_to_end(&mut b)
        .map_err(|e| e.to_string())?;
    if b.len() > MASSIMO_ANTEPRIMA {
        return Err("file troppo grande per l'anteprima".into());
    }
    Ok(b)
}

/// Prepara l'anteprima di un allegato.
pub fn prepara(nome: &str, byte: &[u8]) -> Anteprima {
    if byte.len() > MASSIMO_ANTEPRIMA {
        return Anteprima::NonDisponibile {
            motivo: "L'allegato è troppo grande per l'anteprima.".into(),
        };
    }
    match riconosci(nome, byte) {
        Tipo::Pdf => Anteprima::Pdf,
        Tipo::Png => immagine("image/png"),
        Tipo::Jpeg => immagine("image/jpeg"),
        Tipo::Gif => immagine("image/gif"),
        Tipo::Webp => immagine("image/webp"),
        Tipo::Bmp => immagine("image/bmp"),
        // Un SVG mostrato come immagine non esegue script né carica risorse.
        Tipo::Svg => immagine("image/svg+xml"),
        Tipo::Docx => Anteprima::Word,
        Tipo::FoglioZip => tabella_office(byte),
        Tipo::Ole => {
            // I vecchi .xls si leggono; i .doc binari no.
            match tabella_office(byte) {
                t @ Anteprima::Tabella { .. } => t,
                _ => {
                    non_disponibile("I documenti Word nel vecchio formato .doc non hanno anteprima")
                }
            }
        }
        Tipo::Zip => archivio(byte),
        Tipo::Html => Anteprima::Html {
            html: decodifica(byte).0,
        },
        Tipo::Testo => testo(nome, byte),
        Tipo::Tiff => non_disponibile("Le immagini TIFF non hanno anteprima"),
        Tipo::TestoOdt => non_disponibile("I documenti OpenDocument di testo non hanno anteprima"),
        Tipo::Rtf => non_disponibile("I documenti RTF non hanno anteprima"),
        Tipo::P7m => non_disponibile("Busta firmata non leggibile"),
        Tipo::Sconosciuto => non_disponibile("Formato non riconosciuto"),
    }
}

fn immagine(mime: &str) -> Anteprima {
    Anteprima::Immagine { mime: mime.into() }
}

fn non_disponibile(motivo: &str) -> Anteprima {
    Anteprima::NonDisponibile {
        motivo: format!("{motivo}: si può aprire con il programma predefinito oppure salvare."),
    }
}

/// Decodifica un testo: BOM, UTF-8 oppure Windows-1252 (tipico dei file italiani).
fn decodifica(byte: &[u8]) -> (String, bool) {
    let (testo, _) = match encoding_rs::Encoding::for_bom(byte) {
        Some((codifica, lunghezza)) => {
            let (t, _) = codifica.decode_without_bom_handling(&byte[lunghezza..]);
            (t.into_owned(), ())
        }
        None => match std::str::from_utf8(byte) {
            Ok(t) => (t.to_owned(), ()),
            Err(_) => (encoding_rs::WINDOWS_1252.decode(byte).0.into_owned(), ()),
        },
    };
    if testo.len() > MASSIMO_TESTO {
        let mut fine = MASSIMO_TESTO;
        while !testo.is_char_boundary(fine) {
            fine -= 1;
        }
        (testo[..fine].to_owned(), true)
    } else {
        (testo, false)
    }
}

fn testo(nome: &str, byte: &[u8]) -> Anteprima {
    let (testo, troncato) = decodifica(byte);
    let est = estensione(nome);
    let inizio = testo.trim_start_matches('\u{feff}').trim_start();
    if est == "csv" || est == "tsv" {
        if let Some(t) = tabella_csv(&testo) {
            return t;
        }
    }
    let formato = if inizio.starts_with('<') {
        "xml"
    } else if est == "json" {
        "json"
    } else {
        "testo"
    };
    Anteprima::Testo {
        testo,
        formato: formato.into(),
        troncato,
    }
}

/// Legge un CSV riconoscendo il separatore (`;`, `,` o tabulazione) e i
/// campi tra virgolette.
fn tabella_csv(testo: &str) -> Option<Anteprima> {
    let prima = testo.lines().next().unwrap_or_default();
    let separatore = [';', '\t', ',']
        .into_iter()
        .max_by_key(|s| prima.matches(*s).count())
        .filter(|s| prima.contains(*s))?;
    let mut righe = Vec::new();
    let mut riga = Vec::new();
    let mut campo = String::new();
    let mut virgolette = false;
    let mut caratteri = testo.trim_start_matches('\u{feff}').chars().peekable();
    let mut troncato = false;
    while let Some(c) = caratteri.next() {
        match c {
            '"' if virgolette && caratteri.peek() == Some(&'"') => {
                campo.push('"');
                caratteri.next();
            }
            '"' => virgolette = !virgolette,
            c if c == separatore && !virgolette => riga.push(std::mem::take(&mut campo)),
            '\r' if !virgolette => {}
            '\n' if !virgolette => {
                riga.push(std::mem::take(&mut campo));
                righe.push(std::mem::take(&mut riga));
                if righe.len() >= MASSIME_RIGHE {
                    troncato = true;
                    break;
                }
            }
            c => campo.push(c),
        }
    }
    if !campo.is_empty() || !riga.is_empty() {
        riga.push(campo);
        righe.push(riga);
    }
    for r in &mut righe {
        r.truncate(MASSIME_COLONNE);
    }
    Some(Anteprima::Tabella {
        fogli: vec![Foglio {
            nome: "CSV".into(),
            righe,
            troncato,
        }],
    })
}

/// Fogli di calcolo Excel (xls, xlsx, xlsb) e OpenDocument (ods).
fn tabella_office(byte: &[u8]) -> Anteprima {
    let Ok(mut cartella) = calamine::open_workbook_auto_from_rs(Cursor::new(byte)) else {
        return non_disponibile("Foglio di calcolo non leggibile");
    };
    let mut fogli = Vec::new();
    for nome in cartella.sheet_names() {
        let Ok(intervallo) = cartella.worksheet_range(&nome) else {
            continue;
        };
        let mut righe = Vec::new();
        let mut troncato = false;
        for (i, r) in intervallo.rows().enumerate() {
            if i >= MASSIME_RIGHE {
                troncato = true;
                break;
            }
            righe.push(r.iter().take(MASSIME_COLONNE).map(cella).collect());
        }
        fogli.push(Foglio {
            nome,
            righe,
            troncato,
        });
    }
    if fogli.is_empty() {
        return non_disponibile("Foglio di calcolo vuoto o non leggibile");
    }
    Anteprima::Tabella { fogli }
}

fn cella(d: &Data) -> String {
    match d {
        Data::Empty => String::new(),
        Data::Float(f) => {
            if f.fract() == 0.0 && f.abs() < 1e15 {
                format!("{}", *f as i64)
            } else {
                format!("{f}").replace('.', ",")
            }
        }
        Data::DateTime(dt) => dt
            .as_datetime()
            .map(|t| {
                if t.time() == chrono::NaiveTime::MIN {
                    t.format("%d/%m/%Y").to_string()
                } else {
                    t.format("%d/%m/%Y %H:%M").to_string()
                }
            })
            .unwrap_or_else(|| dt.to_string()),
        Data::Error(e) => format!("#{e:?}"),
        altro => altro.to_string(),
    }
}

fn archivio(byte: &[u8]) -> Anteprima {
    let Ok(mut archivio) = zip::ZipArchive::new(Cursor::new(byte)) else {
        return non_disponibile("Archivio non leggibile");
    };
    let mut voci = Vec::new();
    for i in 0..archivio.len().min(2000) {
        if let Ok(v) = archivio.by_index(i) {
            if !v.is_dir() {
                voci.push(VoceArchivio {
                    nome: v.name().map(|n| n.into_owned()).unwrap_or_default(),
                    dimensione: v.size(),
                });
            }
        }
    }
    Anteprima::Archivio { voci }
}

#[cfg(test)]
mod prove {
    use super::*;

    #[test]
    fn riconosce_dai_byte_non_dal_nome() {
        assert_eq!(riconosci("fattura.txt", b"%PDF-1.7 ..."), Tipo::Pdf);
        assert_eq!(riconosci("foto.pdf", b"\x89PNG\r\n\x1a\nxxxx"), Tipo::Png);
        assert_eq!(riconosci("x.exe", b"MZ\x90\0\x03\0\0\0"), Tipo::Sconosciuto);
        assert_eq!(
            riconosci("nota", "Testo semplice àèì\r\n".as_bytes()),
            Tipo::Testo
        );
        assert_eq!(
            riconosci("p.htm", b"<!DOCTYPE html><html></html>"),
            Tipo::Html
        );
        assert_eq!(
            riconosci("i.svg", b"<svg xmlns='http://www.w3.org/2000/svg'/>"),
            Tipo::Svg
        );
    }

    #[test]
    fn legge_csv_con_virgolette() {
        let Some(Anteprima::Tabella { fogli }) =
            tabella_csv("a;b;c\n1;\"x;y\";\"di \"\"lui\"\"\"\n")
        else {
            panic!("tabella attesa");
        };
        assert_eq!(fogli[0].righe[1], vec!["1", "x;y", "di \"lui\""]);
    }

    #[test]
    fn testo_latin1() {
        let Anteprima::Testo { testo, .. } = prepara("n.txt", b"Citt\xe0 di Torino") else {
            panic!("testo atteso");
        };
        assert_eq!(testo, "Città di Torino");
    }
}
