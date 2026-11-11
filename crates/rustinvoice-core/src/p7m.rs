// Copyright (c) 2026 Andrea Re - info@digitare.net
// Autore: Andrea Re @ 2026 - info@digitare.net
//
// Estrazione del contenuto da una busta firmata CAdES (.p7m).
//
// La busta è una struttura CMS `SignedData` (RFC 5652) codificata in ASN.1.
// Nella pratica molti file usano la codifica BER con lunghezze indefinite e
// OCTET STRING "a pezzi", che un decodificatore DER rigoroso rifiuta: per
// questo qui c'è un piccolo lettore BER autonomo, limitato a ciò che serve
// per raggiungere il contenuto firmato e i certificati.
//
// Sono gestiti anche:
// - file .p7m codificati in base64 (con o senza intestazioni PEM);
// - buste annidate (file firmato più volte, es. `.xml.p7m.p7m`).
//
// La verifica crittografica della firma è nel modulo `firma`.

use crate::errore::{Errore, Risultato};

/// OID 1.2.840.113549.1.7.2 (signedData), già codificato.
const OID_SIGNED_DATA: &[u8] = &[0x2A, 0x86, 0x48, 0x86, 0xF7, 0x0D, 0x01, 0x07, 0x02];

/// Massimo numero di buste annidate che accettiamo di aprire.
const MAX_ANNIDAMENTO: usize = 5;

/// Profondità massima di ricorsione nel lettore BER (protezione da file ostili).
const MAX_PROFONDITA: usize = 64;

/// Risultato dell'apertura di una busta firmata.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BustaFirmata {
    /// Contenuto originale (tipicamente l'XML della fattura).
    pub contenuto: Vec<u8>,
    /// Certificati inclusi nella busta più esterna, in DER.
    pub certificati: Vec<Vec<u8>>,
    /// Numero di firme (buste) annidate rimosse.
    pub livelli: usize,
}

/// Indica se i byte sembrano una busta firmata (binaria o base64).
pub fn sembra_p7m(byte: &[u8]) -> bool {
    if byte.first() == Some(&0x30) {
        return contiene_oid_signed_data(byte);
    }
    match da_base64(byte) {
        Some(der) => der.first() == Some(&0x30) && contiene_oid_signed_data(&der),
        None => false,
    }
}

fn contiene_oid_signed_data(byte: &[u8]) -> bool {
    let finestra = &byte[..byte.len().min(64)];
    finestra
        .windows(OID_SIGNED_DATA.len())
        .any(|w| w == OID_SIGNED_DATA)
}

/// Apre una busta firmata e restituisce il contenuto.
pub fn estrai(byte: &[u8]) -> Risultato<BustaFirmata> {
    let mut corrente = if byte.first() == Some(&0x30) {
        byte.to_vec()
    } else {
        da_base64(byte).ok_or_else(|| {
            Errore::P7mNonValido("il file non è né DER né base64 valido".to_owned())
        })?
    };

    let (mut contenuto, certificati) = apri_signed_data(&corrente)?;
    let mut livelli = 1;

    // Buste annidate: il contenuto è a sua volta un SignedData.
    while livelli < MAX_ANNIDAMENTO && contenuto.first() == Some(&0x30) {
        match apri_signed_data(&contenuto) {
            Ok((interno, _)) => {
                corrente = contenuto;
                contenuto = interno;
                livelli += 1;
            }
            Err(_) => break,
        }
    }
    let _ = corrente;

    // Alcuni software firmano il file XML già codificato in base64.
    if !contenuto.trim_ascii_start().starts_with(b"<") {
        if let Some(decodificato) = da_base64(&contenuto) {
            if decodificato.trim_ascii_start().starts_with(b"<") {
                contenuto = decodificato;
            }
        }
    }

    Ok(BustaFirmata {
        contenuto,
        certificati,
        livelli,
    })
}

/// Decodifica un p7m testuale (base64, eventualmente in formato PEM).
fn da_base64(byte: &[u8]) -> Option<Vec<u8>> {
    let testo = std::str::from_utf8(byte).ok()?;
    let senza_pem: String = testo
        .lines()
        .filter(|riga| !riga.trim_start().starts_with("-----"))
        .collect();
    let risultato = crate::parser::decodifica_base64(&senza_pem)?;
    if risultato.is_empty() {
        None
    } else {
        Some(risultato)
    }
}

/// Un livello di busta: contenuto firmato, certificati e firmatari (in DER).
pub(crate) struct Livello {
    pub contenuto: Vec<u8>,
    pub certificati: Vec<Vec<u8>>,
    pub firmatari: Vec<Vec<u8>>,
}

/// Apre tutte le buste annidate, dalla più esterna alla più interna.
pub(crate) fn livelli(byte: &[u8]) -> Risultato<Vec<Livello>> {
    let iniziale = if byte.first() == Some(&0x30) {
        byte.to_vec()
    } else {
        da_base64(byte).ok_or_else(|| Errore::P7mNonValido("il file non è né DER né base64 valido".to_owned()))?
    };
    let mut out = vec![apri_livello(&iniziale)?];
    while out.len() < MAX_ANNIDAMENTO {
        let ultimo = &out[out.len() - 1].contenuto;
        if ultimo.first() != Some(&0x30) {
            break;
        }
        match apri_livello(ultimo) {
            Ok(l) => out.push(l),
            Err(_) => break,
        }
    }
    Ok(out)
}

/// Apre un singolo livello `ContentInfo` → `SignedData`.
fn apri_signed_data(byte: &[u8]) -> Risultato<(Vec<u8>, Vec<Vec<u8>>)> {
    let l = apri_livello(byte)?;
    Ok((l.contenuto, l.certificati))
}

fn apri_livello(byte: &[u8]) -> Risultato<Livello> {
    let errore = |m: &str| Errore::P7mNonValido(m.to_owned());

    // ContentInfo ::= SEQUENCE { contentType OID, content [0] EXPLICIT ANY }
    let content_info =
        leggi_tlv(byte, 0, 0).ok_or_else(|| errore("struttura ASN.1 illeggibile"))?;
    if !content_info.e_sequenza() {
        return Err(errore("ContentInfo non è una SEQUENCE"));
    }
    let figli_ci =
        figli_tlv(byte, &content_info).ok_or_else(|| errore("ContentInfo illeggibile"))?;
    let oid = figli_ci
        .first()
        .ok_or_else(|| errore("contentType mancante"))?;
    if oid.tag != 0x06 || &byte[oid.inizio_contenuto..oid.fine_contenuto] != OID_SIGNED_DATA {
        return Err(errore("il contenuto non è di tipo signedData"));
    }
    let esplicito = figli_ci
        .get(1)
        .filter(|t| t.tag == 0xA0)
        .ok_or_else(|| errore("contenuto [0] mancante"))?;
    let signed = figli_tlv(byte, esplicito)
        .and_then(|v| v.into_iter().next())
        .filter(|t| t.e_sequenza())
        .ok_or_else(|| errore("SignedData mancante"))?;

    // SignedData ::= SEQUENCE { version, digestAlgorithms, encapContentInfo,
    //                           [0] certificates OPZ, [1] crls OPZ, signerInfos }
    let campi = figli_tlv(byte, &signed).ok_or_else(|| errore("SignedData illeggibile"))?;
    let encap = campi
        .get(2)
        .filter(|t| t.e_sequenza())
        .ok_or_else(|| errore("encapContentInfo mancante"))?;

    let mut certificati = Vec::new();
    if let Some(cert) = campi.iter().skip(3).find(|t| t.tag == 0xA0) {
        if let Some(lista) = figli_tlv(byte, cert) {
            for c in lista {
                certificati.push(byte[c.inizio..c.fine].to_vec());
            }
        }
    }

    // EncapsulatedContentInfo ::= SEQUENCE { eContentType OID, eContent [0] EXPLICIT OCTET STRING OPZ }
    let campi_encap =
        figli_tlv(byte, encap).ok_or_else(|| errore("encapContentInfo illeggibile"))?;
    let econtent = campi_encap
        .get(1)
        .filter(|t| t.tag == 0xA0)
        .ok_or_else(|| errore("firma \"detached\": la busta non contiene il documento firmato"))?;
    let ottetti = figli_tlv(byte, econtent)
        .and_then(|v| v.into_iter().next())
        .ok_or_else(|| errore("eContent vuoto"))?;
    let mut contenuto = Vec::new();
    raccogli_ottetti(byte, &ottetti, &mut contenuto, 0)
        .ok_or_else(|| errore("OCTET STRING del contenuto malformato"))?;

    // signerInfos è l'ultimo campo di SignedData (un SET).
    let mut firmatari = Vec::new();
    if let Some(insieme) = campi.last().filter(|t| t.tag == 0x31) {
        for f in figli_tlv(byte, insieme).unwrap_or_default() {
            firmatari.push(byte[f.inizio..f.fine].to_vec());
        }
    }
    Ok(Livello {
        contenuto,
        certificati,
        firmatari,
    })
}

/// Un elemento TLV letto dal buffer.
#[derive(Debug, Clone, Copy)]
pub(crate) struct Tlv {
    /// Primo byte dell'identificatore (classe + costruito + numero breve).
    pub tag: u8,
    /// Posizione del primo byte dell'elemento.
    pub inizio: usize,
    pub inizio_contenuto: usize,
    /// Fine del contenuto (esclusi gli ottetti di fine 00 00).
    pub fine_contenuto: usize,
    /// Posizione successiva all'elemento.
    pub fine: usize,
}

impl Tlv {
    pub fn costruito(&self) -> bool {
        self.tag & 0x20 != 0
    }
    fn e_sequenza(&self) -> bool {
        self.tag == 0x30
    }
}

/// Legge un elemento BER a partire da `pos`.
pub(crate) fn leggi_tlv(buf: &[u8], pos: usize, profondita: usize) -> Option<Tlv> {
    if profondita > MAX_PROFONDITA {
        return None;
    }
    let inizio = pos;
    let tag = *buf.get(pos)?;
    let mut p = pos + 1;
    // Numero di tag in forma lunga: lo saltiamo, non ci serve il valore.
    if tag & 0x1F == 0x1F {
        loop {
            let b = *buf.get(p)?;
            p += 1;
            if b & 0x80 == 0 {
                break;
            }
        }
    }
    let primo = *buf.get(p)?;
    p += 1;
    if primo == 0x80 {
        // Lunghezza indefinita: il contenuto prosegue fino agli ottetti 00 00.
        if tag & 0x20 == 0 {
            return None; // ammessa solo per tipi costruiti
        }
        let inizio_contenuto = p;
        let mut q = p;
        loop {
            if buf.get(q) == Some(&0) && buf.get(q + 1) == Some(&0) {
                return Some(Tlv {
                    tag,
                    inizio,
                    inizio_contenuto,
                    fine_contenuto: q,
                    fine: q + 2,
                });
            }
            let figlio = leggi_tlv(buf, q, profondita + 1)?;
            q = figlio.fine;
        }
    }
    let lunghezza = if primo & 0x80 == 0 {
        primo as usize
    } else {
        let n = (primo & 0x7F) as usize;
        if n == 0 || n > 8 {
            return None;
        }
        let mut l: usize = 0;
        for _ in 0..n {
            l = l.checked_mul(256)?.checked_add(*buf.get(p)? as usize)?;
            p += 1;
        }
        l
    };
    let fine = p.checked_add(lunghezza)?;
    if fine > buf.len() {
        return None;
    }
    Some(Tlv {
        tag,
        inizio,
        inizio_contenuto: p,
        fine_contenuto: fine,
        fine,
    })
}

/// Elenca i figli di un elemento costruito.
pub(crate) fn figli_tlv(buf: &[u8], padre: &Tlv) -> Option<Vec<Tlv>> {
    if !padre.costruito() {
        return None;
    }
    let mut v = Vec::new();
    let mut p = padre.inizio_contenuto;
    while p < padre.fine_contenuto {
        let t = leggi_tlv(buf, p, 1)?;
        p = t.fine;
        v.push(t);
    }
    Some(v)
}

/// Concatena il contenuto di un OCTET STRING, anche se spezzato in parti.
pub(crate) fn raccogli_ottetti(buf: &[u8], t: &Tlv, uscita: &mut Vec<u8>, profondita: usize) -> Option<()> {
    if profondita > MAX_PROFONDITA {
        return None;
    }
    match t.tag {
        0x04 => {
            uscita.extend_from_slice(&buf[t.inizio_contenuto..t.fine_contenuto]);
            Some(())
        }
        0x24 => {
            for f in figli_tlv(buf, t)? {
                raccogli_ottetti(buf, &f, uscita, profondita + 1)?;
            }
            Some(())
        }
        _ => None,
    }
}

#[cfg(test)]
mod test {
    use super::*;

    /// Costruisce un TLV con lunghezza definita.
    fn tlv(tag: u8, contenuto: &[u8]) -> Vec<u8> {
        let mut v = vec![tag];
        let n = contenuto.len();
        if n < 128 {
            v.push(n as u8);
        } else if n < 256 {
            v.extend_from_slice(&[0x81, n as u8]);
        } else {
            v.extend_from_slice(&[0x82, (n >> 8) as u8, n as u8]);
        }
        v.extend_from_slice(contenuto);
        v
    }

    /// Costruisce un TLV a lunghezza indefinita.
    fn tlv_indefinito(tag: u8, contenuto: &[u8]) -> Vec<u8> {
        let mut v = vec![tag, 0x80];
        v.extend_from_slice(contenuto);
        v.extend_from_slice(&[0, 0]);
        v
    }

    /// Busta minima: SignedData con contenuto e nessun firmatario.
    fn busta(contenuto_ottetti: Vec<u8>, indefinita: bool) -> Vec<u8> {
        let costruisci = if indefinita { tlv_indefinito } else { tlv };
        let oid_data = tlv(
            0x06,
            &[0x2A, 0x86, 0x48, 0x86, 0xF7, 0x0D, 0x01, 0x07, 0x01],
        );
        let econtent = costruisci(0xA0, &contenuto_ottetti);
        let encap = costruisci(0x30, &[oid_data, econtent].concat());
        let versione = tlv(0x02, &[1]);
        let digest = tlv(0x31, &[]);
        let firmatari = tlv(0x31, &[]);
        let signed = costruisci(0x30, &[versione, digest, encap, firmatari].concat());
        let oid = tlv(0x06, OID_SIGNED_DATA);
        let esplicito = costruisci(0xA0, &signed);
        costruisci(0x30, &[oid, esplicito].concat())
    }

    #[test]
    fn estrae_contenuto_der() {
        let xml = b"<a>ciao</a>";
        let p7m = busta(tlv(0x04, xml), false);
        assert!(sembra_p7m(&p7m));
        let b = estrai(&p7m).unwrap();
        assert_eq!(b.contenuto, xml);
        assert_eq!(b.livelli, 1);
    }

    #[test]
    fn estrae_contenuto_ber_indefinito_a_pezzi() {
        let pezzi = [tlv(0x04, b"<a>ci"), tlv(0x04, b"ao</a>")].concat();
        let p7m = busta(tlv_indefinito(0x24, &pezzi), true);
        let b = estrai(&p7m).unwrap();
        assert_eq!(b.contenuto, b"<a>ciao</a>");
    }

    #[test]
    fn estrae_da_base64_e_buste_annidate() {
        let interna = busta(tlv(0x04, b"<a/>"), false);
        let esterna = busta(tlv(0x04, &interna), false);
        let testo = base64::Engine::encode(&base64::engine::general_purpose::STANDARD, &esterna);
        let b = estrai(testo.as_bytes()).unwrap();
        assert_eq!(b.contenuto, b"<a/>");
        assert_eq!(b.livelli, 2);
    }

    #[test]
    fn rifiuta_dati_casuali() {
        assert!(estrai(&[0x30, 0x03, 0x01, 0x02]).is_err());
        assert!(!sembra_p7m(b"<xml/>"));
    }
}
