// Copyright (c) 2026 Andrea Re - info@digitare.net
// Autore: Andrea Re @ 2026 - info@digitare.net
//
// Contenitori di fatture: archivi .zip ed email .eml (comprese le PEC, con
// il messaggio originale annidato in postacert.eml).
//
// Un documento dentro un contenitore si indica con un percorso "virtuale"
// formato dal percorso del file seguito da `!/` e dal nome interno, anche su
// più livelli: `C:\Posta\pec.eml!/postacert.eml!/IT01234567890_00001.xml.p7m`.

use std::path::Path;

use base64::Engine;

/// Separatore tra il file su disco e il nome di un elemento interno.
pub const SEPARATORE: &str = "!/";

/// Livelli massimi di contenitori uno dentro l'altro.
const PROFONDITA_MASSIMA: usize = 6;
/// Dimensione massima di un singolo elemento estratto (protezione da archivi malevoli).
const MASSIMO_ELEMENTO: u64 = 64 * 1024 * 1024;
/// Dimensione massima complessiva estratta da un contenitore.
const MASSIMO_TOTALE: u64 = 512 * 1024 * 1024;

/// Tipo di contenitore riconosciuto dal nome.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TipoContenitore {
    Zip,
    Email,
}

/// Riconosce un contenitore dall'estensione del nome.
pub fn tipo_contenitore(nome: &str) -> Option<TipoContenitore> {
    let n = nome.to_lowercase();
    if n.ends_with(".zip") {
        Some(TipoContenitore::Zip)
    } else if n.ends_with(".eml") {
        Some(TipoContenitore::Email)
    } else {
        None
    }
}

/// Vero per i nomi che possono contenere una fattura o un messaggio SdI.
pub fn e_documento(nome: &str) -> bool {
    let n = nome.to_lowercase();
    // Nelle PEC i dati di certificazione non sono documenti fiscali.
    if n.ends_with("daticert.xml") {
        return false;
    }
    n.ends_with(".xml") || n.ends_with(".p7m")
}

/// Un documento trovato dentro un contenitore.
#[derive(Debug, Clone)]
pub struct Interno {
    /// Percorso relativo al contenitore, con `!/` tra i livelli.
    pub percorso: String,
    /// Nome del file interno (ultimo livello).
    pub nome: String,
    pub byte: Vec<u8>,
}

/// Elementi diretti di un contenitore, con nomi resi univoci.
pub fn elementi(tipo: TipoContenitore, byte: &[u8]) -> Result<Vec<(String, Vec<u8>)>, String> {
    let grezzi = match tipo {
        TipoContenitore::Zip => elementi_zip(byte)?,
        TipoContenitore::Email => {
            let mut v = Vec::new();
            allegati_email(byte, 0, &mut v);
            v
        }
    };
    // Due allegati con lo stesso nome diventano "nome" e "nome (2)".
    let mut visti = std::collections::HashMap::<String, usize>::new();
    Ok(grezzi
        .into_iter()
        .map(|(nome, b)| {
            let n = visti.entry(nome.to_lowercase()).or_insert(0);
            *n += 1;
            let nome = if *n == 1 {
                nome
            } else {
                aggiungi_suffisso(&nome, *n)
            };
            (nome, b)
        })
        .collect())
}

fn aggiungi_suffisso(nome: &str, n: usize) -> String {
    // Il suffisso va prima delle estensioni, così ".xml.p7m" resta riconoscibile.
    match nome.find('.') {
        Some(i) if i > 0 => format!("{} ({n}){}", &nome[..i], &nome[i..]),
        _ => format!("{nome} ({n})"),
    }
}

/// Tutti i documenti contenuti, anche in contenitori annidati.
pub fn espandi(tipo: TipoContenitore, byte: &[u8]) -> Result<Vec<Interno>, String> {
    let mut risultato = Vec::new();
    espandi_in(tipo, byte, "", 0, &mut risultato)?;
    Ok(risultato)
}

fn espandi_in(
    tipo: TipoContenitore,
    byte: &[u8],
    prefisso: &str,
    profondita: usize,
    out: &mut Vec<Interno>,
) -> Result<(), String> {
    if profondita >= PROFONDITA_MASSIMA {
        return Ok(());
    }
    for (nome, b) in elementi(tipo, byte)? {
        let percorso = if prefisso.is_empty() {
            nome.clone()
        } else {
            format!("{prefisso}{SEPARATORE}{nome}")
        };
        if let Some(t) = tipo_contenitore(&nome) {
            // Un contenitore interno rovinato non blocca gli altri documenti.
            let _ = espandi_in(t, &b, &percorso, profondita + 1, out);
        } else if e_documento(&nome) {
            out.push(Interno {
                percorso,
                nome,
                byte: b,
            });
        }
    }
    Ok(())
}

/// Divide un percorso virtuale nel file su disco e nei nomi interni.
pub fn dividi(percorso: &str) -> (&str, Vec<&str>) {
    let mut parti = percorso.split(SEPARATORE);
    let file = parti.next().unwrap_or_default();
    (file, parti.collect())
}

/// Legge i byte indicati da un percorso, anche virtuale.
pub fn leggi(percorso: impl AsRef<Path>) -> std::io::Result<Vec<u8>> {
    let testo = percorso.as_ref().to_string_lossy().to_string();
    let (file, interni) = dividi(&testo);
    let mut byte = std::fs::read(file)?;
    let mut nome_corrente = file.to_owned();
    for nome in interni {
        let tipo = tipo_contenitore(&nome_corrente).ok_or_else(|| {
            std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                format!("{nome_corrente} non è un contenitore"),
            )
        })?;
        let elementi = elementi(tipo, &byte)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        byte = elementi
            .into_iter()
            .find(|(n, _)| n == nome)
            .map(|(_, b)| b)
            .ok_or_else(|| {
                std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    format!("{nome} non trovato in {nome_corrente}"),
                )
            })?;
        nome_corrente = nome.to_owned();
    }
    Ok(byte)
}

// ---------------------------------------------------------------------------
// Archivi zip
// ---------------------------------------------------------------------------

fn elementi_zip(byte: &[u8]) -> Result<Vec<(String, Vec<u8>)>, String> {
    use std::io::Read;
    let mut archivio = zip::ZipArchive::new(std::io::Cursor::new(byte))
        .map_err(|e| format!("archivio zip non valido: {e}"))?;
    let mut out = Vec::new();
    let mut totale = 0u64;
    for i in 0..archivio.len() {
        let Ok(mut voce) = archivio.by_index(i) else {
            continue;
        };
        if voce.is_dir() {
            continue;
        }
        let nome = voce.name().replace('\\', "/");
        let nome = nome.rsplit('/').next().unwrap_or_default().to_owned();
        // Si estraggono solo documenti e contenitori: il resto non serve.
        if nome.is_empty() || !(e_documento(&nome) || tipo_contenitore(&nome).is_some()) {
            continue;
        }
        if voce.size() > MASSIMO_ELEMENTO || totale + voce.size() > MASSIMO_TOTALE {
            continue;
        }
        let mut b = Vec::with_capacity(voce.size() as usize);
        if (&mut voce)
            .take(MASSIMO_ELEMENTO + 1)
            .read_to_end(&mut b)
            .is_err()
            || b.len() as u64 > MASSIMO_ELEMENTO
        {
            continue;
        }
        totale += b.len() as u64;
        out.push((nome, b));
    }
    Ok(out)
}

// ---------------------------------------------------------------------------
// Email (MIME)
// ---------------------------------------------------------------------------

/// Una parte MIME: intestazioni e corpo grezzo.
struct Parte<'a> {
    intestazioni: Vec<(String, String)>,
    corpo: &'a [u8],
}

impl Parte<'_> {
    fn intestazione(&self, nome: &str) -> Option<&str> {
        self.intestazioni
            .iter()
            .find(|(n, _)| n.eq_ignore_ascii_case(nome))
            .map(|(_, v)| v.as_str())
    }
}

/// Separa intestazioni e corpo, unendo le intestazioni su più righe.
fn analizza_parte(byte: &[u8]) -> Parte<'_> {
    let (testa, corpo) = match trova(byte, b"\r\n\r\n") {
        Some(i) => (&byte[..i], &byte[i + 4..]),
        None => match trova(byte, b"\n\n") {
            Some(i) => (&byte[..i], &byte[i + 2..]),
            None => (byte, &byte[byte.len()..]),
        },
    };
    let testa = String::from_utf8_lossy(testa);
    let mut intestazioni: Vec<(String, String)> = Vec::new();
    for riga in testa.lines() {
        if riga.starts_with(' ') || riga.starts_with('\t') {
            if let Some(ultima) = intestazioni.last_mut() {
                ultima.1.push(' ');
                ultima.1.push_str(riga.trim());
            }
        } else if let Some((n, v)) = riga.split_once(':') {
            intestazioni.push((n.trim().to_owned(), v.trim().to_owned()));
        }
    }
    Parte {
        intestazioni,
        corpo,
    }
}

fn trova(dove: &[u8], cosa: &[u8]) -> Option<usize> {
    dove.windows(cosa.len()).position(|w| w == cosa)
}

/// Valore principale e parametri di un'intestazione (`text/plain; charset=x`).
fn parametri(valore: &str) -> (String, Vec<(String, String)>) {
    let mut pezzi = dividi_parametri(valore).into_iter();
    let principale = pezzi.next().unwrap_or_default().trim().to_lowercase();
    let mut out = Vec::new();
    for p in pezzi {
        if let Some((n, v)) = p.split_once('=') {
            out.push((
                n.trim().to_lowercase(),
                v.trim().trim_matches('"').to_owned(),
            ));
        }
    }
    (principale, out)
}

/// Divide sui `;` che non sono tra virgolette.
fn dividi_parametri(s: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut corrente = String::new();
    let mut tra_virgolette = false;
    for c in s.chars() {
        match c {
            '"' => {
                tra_virgolette = !tra_virgolette;
                corrente.push(c);
            }
            ';' if !tra_virgolette => out.push(std::mem::take(&mut corrente)),
            _ => corrente.push(c),
        }
    }
    out.push(corrente);
    out
}

/// Nome del file di una parte, da Content-Disposition o Content-Type.
fn nome_allegato(parte: &Parte<'_>) -> Option<String> {
    for (intestazione, chiave) in [
        ("Content-Disposition", "filename"),
        ("Content-Type", "name"),
    ] {
        let Some(v) = parte.intestazione(intestazione) else {
            continue;
        };
        let (_, par) = parametri(v);
        // Forma RFC 2231 (filename*=utf-8''nome%20file.xml), anche spezzata in più parti.
        let estesi: Vec<&(String, String)> = par
            .iter()
            .filter(|(n, _)| n.starts_with(&format!("{chiave}*")))
            .collect();
        if !estesi.is_empty() {
            let mut unito = String::new();
            for (_, v) in &estesi {
                unito.push_str(v);
            }
            let valore = unito
                .split_once("''")
                .map(|(_, r)| r.to_owned())
                .unwrap_or(unito);
            return Some(pulisci_nome(&decodifica_percentuale(&valore)));
        }
        if let Some((_, v)) = par.iter().find(|(n, _)| n == chiave) {
            return Some(pulisci_nome(&decodifica_parole(v)));
        }
    }
    None
}

/// Toglie eventuali percorsi dal nome (un allegato non deve poter "uscire").
fn pulisci_nome(n: &str) -> String {
    n.rsplit(['/', '\\'])
        .next()
        .unwrap_or_default()
        .trim()
        .to_owned()
}

fn decodifica_percentuale(s: &str) -> String {
    let b = s.as_bytes();
    let mut out = Vec::with_capacity(b.len());
    let mut i = 0;
    while i < b.len() {
        if b[i] == b'%' && i + 2 < b.len() {
            if let Ok(v) = u8::from_str_radix(&s[i + 1..i + 3], 16) {
                out.push(v);
                i += 3;
                continue;
            }
        }
        out.push(b[i]);
        i += 1;
    }
    String::from_utf8_lossy(&out).into_owned()
}

/// Decodifica le "encoded word" (`=?utf-8?B?...?=`, `=?iso-8859-1?Q?...?=`).
fn decodifica_parole(s: &str) -> String {
    let mut out = String::new();
    let mut resto = s;
    while let Some(inizio) = resto.find("=?") {
        // La parola ha la forma =?charset?codifica?testo?= e il testo può
        // contenere "?=" solo alla fine: si cercano i due '?' interni.
        let interno = &resto[inizio + 2..];
        let pezzi: Vec<&str> = interno.splitn(3, '?').collect();
        if pezzi.len() < 3 {
            break;
        }
        let Some(fine_testo) = pezzi[2].find("?=") else {
            break;
        };
        let (charset, codifica, testo) = (pezzi[0], pezzi[1], &pezzi[2][..fine_testo]);
        let byte = match codifica.to_ascii_uppercase().as_str() {
            "B" => base64::engine::general_purpose::STANDARD
                .decode(testo.trim())
                .unwrap_or_default(),
            "Q" => decodifica_qp(testo.replace('_', " ").as_bytes()),
            _ => testo.as_bytes().to_vec(),
        };
        let prima = &resto[..inizio];
        // Gli spazi tra due parole codificate consecutive vanno ignorati.
        if !(prima.trim().is_empty() && !out.is_empty()) {
            out.push_str(prima);
        }
        out.push_str(&in_testo(&byte, charset));
        let consumati = inizio + 2 + pezzi[0].len() + 1 + pezzi[1].len() + 1 + fine_testo + 2;
        resto = &resto[consumati.min(resto.len())..];
    }
    out.push_str(resto);
    out
}

fn in_testo(byte: &[u8], charset: &str) -> String {
    match encoding_rs::Encoding::for_label(charset.trim().as_bytes()) {
        Some(enc) => enc.decode(byte).0.into_owned(),
        None => String::from_utf8_lossy(byte).into_owned(),
    }
}

fn decodifica_qp(b: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(b.len());
    let mut i = 0;
    while i < b.len() {
        if b[i] == b'=' {
            // "=" a fine riga: interruzione morbida da eliminare.
            if b.get(i + 1) == Some(&b'\r') && b.get(i + 2) == Some(&b'\n') {
                i += 3;
                continue;
            }
            if b.get(i + 1) == Some(&b'\n') {
                i += 2;
                continue;
            }
            if i + 2 < b.len() {
                if let Ok(v) = u8::from_str_radix(&String::from_utf8_lossy(&b[i + 1..i + 3]), 16) {
                    out.push(v);
                    i += 3;
                    continue;
                }
            }
        }
        out.push(b[i]);
        i += 1;
    }
    out
}

fn decodifica_corpo(parte: &Parte<'_>) -> Vec<u8> {
    let codifica = parte
        .intestazione("Content-Transfer-Encoding")
        .unwrap_or("")
        .trim()
        .to_lowercase();
    match codifica.as_str() {
        "base64" => {
            let pulito: Vec<u8> = parte
                .corpo
                .iter()
                .copied()
                .filter(|c| !c.is_ascii_whitespace())
                .collect();
            base64::engine::general_purpose::STANDARD
                .decode(&pulito)
                .or_else(|_| {
                    base64::engine::general_purpose::STANDARD_NO_PAD
                        .decode(pulito.strip_suffix(b"=").unwrap_or(&pulito))
                })
                .unwrap_or_default()
        }
        "quoted-printable" => decodifica_qp(parte.corpo),
        _ => parte.corpo.to_vec(),
    }
}

/// Raccoglie ricorsivamente gli allegati di un messaggio.
fn allegati_email(byte: &[u8], profondita: usize, out: &mut Vec<(String, Vec<u8>)>) {
    if profondita > PROFONDITA_MASSIMA {
        return;
    }
    let parte = analizza_parte(byte);
    let (tipo, par) = parametri(parte.intestazione("Content-Type").unwrap_or("text/plain"));

    if tipo.starts_with("multipart/") {
        let Some((_, confine)) = par.iter().find(|(n, _)| n == "boundary") else {
            return;
        };
        for sotto in dividi_multipart(parte.corpo, confine) {
            allegati_email(sotto, profondita + 1, out);
        }
        return;
    }

    let nome = nome_allegato(&parte);
    let contenuto = decodifica_corpo(&parte);
    if contenuto.len() as u64 > MASSIMO_ELEMENTO {
        return;
    }
    match nome {
        // Messaggio allegato con nome (postacert.eml nelle PEC): resta un
        // contenitore, così il percorso virtuale ne mostra la struttura.
        Some(n) if tipo_contenitore(&n).is_some() || e_documento(&n) => out.push((n, contenuto)),
        Some(_) => {}
        // Messaggio allegato senza nome: si guarda direttamente dentro.
        None if tipo == "message/rfc822" => allegati_email(&contenuto, profondita + 1, out),
        None => {}
    }
}

/// Divide il corpo di una parte multipart sulle righe `--confine`.
fn dividi_multipart<'a>(corpo: &'a [u8], confine: &str) -> Vec<&'a [u8]> {
    let segno = format!("--{confine}");
    let segno = segno.as_bytes();
    let mut posizioni = Vec::new();
    let mut i = 0;
    while let Some(p) = trova(&corpo[i..], segno) {
        let assoluta = i + p;
        // Il confine deve stare a inizio riga.
        if assoluta == 0 || corpo[assoluta - 1] == b'\n' {
            posizioni.push(assoluta);
        }
        i = assoluta + segno.len();
    }
    let mut parti = Vec::new();
    for w in posizioni.windows(2) {
        let inizio = w[0] + segno.len();
        // Salta il fine riga dopo il confine.
        let mut s = inizio;
        while s < w[1] && (corpo[s] == b'\r' || corpo[s] == b'\n') {
            s += 1;
        }
        // Toglie il fine riga prima del confine successivo.
        let mut e = w[1];
        if e > s && corpo[e - 1] == b'\n' {
            e -= 1;
        }
        if e > s && corpo[e - 1] == b'\r' {
            e -= 1;
        }
        if s < e && !corpo[inizio..].starts_with(b"--") {
            parti.push(&corpo[s..e]);
        }
    }
    parti
}

#[cfg(test)]
mod test {
    use super::*;

    const FATTURA: &str = r#"<?xml version="1.0"?><p:FatturaElettronica versione="FPR12" xmlns:p="http://ivaservizi.agenziaentrate.gov.it/docs/xsd/fatture/v1.2"/>"#;

    fn email_pec() -> Vec<u8> {
        let b64 = base64::engine::general_purpose::STANDARD.encode(FATTURA);
        let interna = format!(
            "From: sdi01@pec.fatturapa.it\r\nContent-Type: multipart/mixed; boundary=\"INT\"\r\n\r\n--INT\r\nContent-Type: text/plain\r\n\r\nInvio file\r\n--INT\r\nContent-Type: application/octet-stream; name=\"IT01234567890_00001.xml\"\r\nContent-Transfer-Encoding: base64\r\nContent-Disposition: attachment; filename=\"IT01234567890_00001.xml\"\r\n\r\n{b64}\r\n--INT--\r\n"
        );
        format!(
            "From: posta-certificata@pec.it\r\nSubject: =?UTF-8?B?UE9TVEEgQ0VSVElGSUNBVEE6IEludmlv?=\r\nContent-Type: multipart/signed; protocol=\"application/pkcs7-signature\"; boundary=\"EST\"\r\n\r\n--EST\r\nContent-Type: multipart/mixed; boundary=\"MIX\"\r\n\r\n--MIX\r\nContent-Type: text/plain\r\n\r\nMessaggio di posta certificata\r\n--MIX\r\nContent-Type: application/xml; name=\"daticert.xml\"\r\nContent-Disposition: attachment; filename=\"daticert.xml\"\r\n\r\n<postacert/>\r\n--MIX\r\nContent-Type: message/rfc822; name=\"postacert.eml\"\r\nContent-Disposition: attachment; filename=\"postacert.eml\"\r\n\r\n{interna}\r\n--MIX--\r\n--EST\r\nContent-Type: application/pkcs7-signature; name=\"smime.p7s\"\r\nContent-Transfer-Encoding: base64\r\n\r\nAAAA\r\n--EST--\r\n"
        )
        .into_bytes()
    }

    #[test]
    fn pec_con_fattura_annidata() {
        let trovati = espandi(TipoContenitore::Email, &email_pec()).unwrap();
        assert_eq!(
            trovati.len(),
            1,
            "{:?}",
            trovati.iter().map(|t| &t.percorso).collect::<Vec<_>>()
        );
        assert_eq!(
            trovati[0].percorso,
            "postacert.eml!/IT01234567890_00001.xml"
        );
        assert_eq!(trovati[0].byte, FATTURA.as_bytes());
    }

    #[test]
    fn zip_con_eml_e_percorso_virtuale() {
        use std::io::Write;
        let mut buf = std::io::Cursor::new(Vec::new());
        {
            let mut z = zip::ZipWriter::new(&mut buf);
            let opz = zip::write::SimpleFileOptions::default();
            z.start_file("cartella/IT01234567890_00002.xml", opz)
                .unwrap();
            z.write_all(FATTURA.as_bytes()).unwrap();
            z.start_file("pec.eml", opz).unwrap();
            z.write_all(&email_pec()).unwrap();
            z.start_file("leggimi.txt", opz).unwrap();
            z.write_all(b"ciao").unwrap();
            z.finish().unwrap();
        }
        let dati = buf.into_inner();
        let trovati = espandi(TipoContenitore::Zip, &dati).unwrap();
        let percorsi: Vec<_> = trovati.iter().map(|t| t.percorso.as_str()).collect();
        assert_eq!(
            percorsi,
            [
                "IT01234567890_00002.xml",
                "pec.eml!/postacert.eml!/IT01234567890_00001.xml"
            ]
        );

        let dir = std::env::temp_dir().join(format!("ri_test_{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let file = dir.join("archivio.zip");
        std::fs::write(&file, &dati).unwrap();
        let virtuale = format!(
            "{}!/pec.eml!/postacert.eml!/IT01234567890_00001.xml",
            file.display()
        );
        assert_eq!(leggi(&virtuale).unwrap(), FATTURA.as_bytes());
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn nomi_codificati_e_duplicati() {
        assert_eq!(
            decodifica_parole("=?iso-8859-1?Q?fattura_n=B0_1.xml?="),
            "fattura n° 1.xml"
        );
        assert_eq!(
            decodifica_parole("=?UTF-8?B?Y2lhbw==?= =?UTF-8?B?IG1vbmRv?="),
            "ciao mondo"
        );
        assert_eq!(aggiungi_suffisso("IT0_1.xml.p7m", 2), "IT0_1 (2).xml.p7m");
        assert_eq!(pulisci_nome("..\\..\\x.xml"), "x.xml");
    }
}
