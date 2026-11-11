// Copyright (c) 2026 Andrea Re - info@digitare.net
// Autore: Andrea Re @ 2026 - info@digitare.net
//
// Verifica crittografica delle firme CAdES (.p7m).
//
// Per ogni busta e ogni firmatario si controlla che:
// - l'impronta del documento coincida con quella firmata (messageDigest);
// - la firma sugli attributi firmati sia valida con la chiave pubblica del
//   certificato del firmatario (RSA PKCS#1 v1.5 o ECDSA P-256/P-384);
// - il certificato fosse nel periodo di validità al momento della firma.
//
// Non si verifica (ancora) la catena fino a un certificatore accreditato né
// lo stato di revoca: l'esito lo dichiara esplicitamente.

use serde::Serialize;
use sha1::Sha1;
use sha2::{Digest, Sha256, Sha384, Sha512};

use crate::p7m::{figli_tlv, leggi_tlv, livelli, Tlv};

/// Esito della verifica di un firmatario.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Esito {
    /// Firma integra: il documento non è stato modificato dopo la firma.
    Valida,
    /// Firma non corrispondente al documento o al certificato.
    NonValida,
    /// Algoritmo o struttura non supportati: impossibile dire.
    NonVerificabile,
}

/// Risultato della verifica di un firmatario.
#[derive(Debug, Clone, Serialize)]
pub struct Firmatario {
    /// Livello della busta (1 = la più esterna).
    pub livello: usize,
    pub esito: Esito,
    /// Spiegazione leggibile dell'esito.
    pub dettaglio: String,
    pub nome: String,
    pub codice_fiscale: String,
    pub organizzazione: String,
    pub emittente: String,
    pub valido_dal: String,
    pub valido_al: String,
    /// Data e ora della firma dichiarate nella busta (UTC), se presenti.
    pub data_firma: String,
    /// Certificato valido alla data della firma (o a oggi, se manca la data).
    pub certificato_in_corso_di_validita: bool,
    pub algoritmo: String,
}

/// Verifica tutte le firme di una busta .p7m.
pub fn verifica(byte: &[u8]) -> Vec<Firmatario> {
    let Ok(livelli) = livelli(byte) else {
        return Vec::new();
    };
    let mut out = Vec::new();
    for (i, l) in livelli.iter().enumerate() {
        for f in &l.firmatari {
            out.push(verifica_firmatario(i + 1, f, &l.contenuto, &l.certificati));
        }
    }
    out
}

// OID codificati (solo il contenuto, senza tag e lunghezza).
const OID_SHA1: &[u8] = &[0x2B, 0x0E, 0x03, 0x02, 0x1A];
const OID_SHA256: &[u8] = &[0x60, 0x86, 0x48, 0x01, 0x65, 0x03, 0x04, 0x02, 0x01];
const OID_SHA384: &[u8] = &[0x60, 0x86, 0x48, 0x01, 0x65, 0x03, 0x04, 0x02, 0x02];
const OID_SHA512: &[u8] = &[0x60, 0x86, 0x48, 0x01, 0x65, 0x03, 0x04, 0x02, 0x03];
const PREFISSO_PKCS1: &[u8] = &[0x2A, 0x86, 0x48, 0x86, 0xF7, 0x0D, 0x01, 0x01];
const PREFISSO_ECDSA: &[u8] = &[0x2A, 0x86, 0x48, 0xCE, 0x3D, 0x04, 0x03];
const OID_MESSAGE_DIGEST: &[u8] = &[0x2A, 0x86, 0x48, 0x86, 0xF7, 0x0D, 0x01, 0x09, 0x04];
const OID_SIGNING_TIME: &[u8] = &[0x2A, 0x86, 0x48, 0x86, 0xF7, 0x0D, 0x01, 0x09, 0x05];
const OID_CHIAVE_EC: &[u8] = &[0x2A, 0x86, 0x48, 0xCE, 0x3D, 0x02, 0x01];
const OID_P256: &[u8] = &[0x2A, 0x86, 0x48, 0xCE, 0x3D, 0x03, 0x01, 0x07];
const OID_P384: &[u8] = &[0x2B, 0x81, 0x04, 0x00, 0x22];
const OID_SKI: &[u8] = &[0x55, 0x1D, 0x0E];

#[derive(Clone, Copy, PartialEq, Eq)]
enum Hash {
    Sha1,
    Sha256,
    Sha384,
    Sha512,
}

impl Hash {
    fn da_oid(oid: &[u8]) -> Option<Self> {
        match oid {
            OID_SHA1 => Some(Hash::Sha1),
            OID_SHA256 => Some(Hash::Sha256),
            OID_SHA384 => Some(Hash::Sha384),
            OID_SHA512 => Some(Hash::Sha512),
            _ => None,
        }
    }
    fn calcola(self, dati: &[u8]) -> Vec<u8> {
        match self {
            Hash::Sha1 => Sha1::digest(dati).to_vec(),
            Hash::Sha256 => Sha256::digest(dati).to_vec(),
            Hash::Sha384 => Sha384::digest(dati).to_vec(),
            Hash::Sha512 => Sha512::digest(dati).to_vec(),
        }
    }
    fn nome(self) -> &'static str {
        match self {
            Hash::Sha1 => "SHA-1",
            Hash::Sha256 => "SHA-256",
            Hash::Sha384 => "SHA-384",
            Hash::Sha512 => "SHA-512",
        }
    }
}

/// Piccolo aiuto per navigare il DER.
struct Der<'a> {
    buf: &'a [u8],
}

impl<'a> Der<'a> {
    fn radice(&self) -> Option<Tlv> {
        leggi_tlv(self.buf, 0, 0)
    }
    fn figli(&self, t: &Tlv) -> Vec<Tlv> {
        figli_tlv(self.buf, t).unwrap_or_default()
    }
    fn contenuto(&self, t: &Tlv) -> &'a [u8] {
        &self.buf[t.inizio_contenuto..t.fine_contenuto]
    }
    fn intero(&self, t: &Tlv) -> &'a [u8] {
        &self.buf[t.inizio..t.fine]
    }
}

/// Dati utili estratti da un certificato X.509.
#[derive(Default)]
struct Certificato<'a> {
    emittente_der: &'a [u8],
    seriale: &'a [u8],
    soggetto: Vec<(Vec<u8>, String)>,
    emittente: Vec<(Vec<u8>, String)>,
    dal: String,
    al: String,
    spki: &'a [u8],
    ski: Option<&'a [u8]>,
}

fn leggi_certificato(der: &[u8]) -> Option<Certificato<'_>> {
    let d = Der { buf: der };
    let cert = d.radice()?;
    let tbs = *d.figli(&cert).first()?;
    let mut campi = d.figli(&tbs).into_iter();
    let mut primo = campi.next()?;
    if primo.tag == 0xA0 {
        // Versione esplicita: il seriale è il campo successivo.
        primo = campi.next()?;
    }
    let seriale = d.contenuto(&primo);
    let _algoritmo = campi.next()?;
    let emittente = campi.next()?;
    let validita = campi.next()?;
    let soggetto = campi.next()?;
    let spki = campi.next()?;
    let date = d.figli(&validita);
    let mut c = Certificato {
        emittente_der: d.intero(&emittente),
        seriale,
        soggetto: nomi(&d, &soggetto),
        emittente: nomi(&d, &emittente),
        dal: date
            .first()
            .map(|t| data_asn1(t.tag, d.contenuto(t)))
            .unwrap_or_default(),
        al: date
            .get(1)
            .map(|t| data_asn1(t.tag, d.contenuto(t)))
            .unwrap_or_default(),
        spki: d.intero(&spki),
        ski: None,
    };
    // Estensioni [3]: cerca la Subject Key Identifier.
    for resto in campi {
        if resto.tag != 0xA3 {
            continue;
        }
        for estensioni in d.figli(&resto) {
            for e in d.figli(&estensioni) {
                let parti = d.figli(&e);
                if parti.first().map(|o| d.contenuto(o)) == Some(OID_SKI) {
                    if let Some(valore) = parti.last() {
                        let interno = Der {
                            buf: d.contenuto(valore),
                        };
                        if let Some(t) = interno.radice() {
                            c.ski =
                                Some(&d.contenuto(valore)[t.inizio_contenuto..t.fine_contenuto]);
                        }
                    }
                }
            }
        }
    }
    Some(c)
}

/// Attributi di un Name X.509 come coppie (OID, valore testuale).
fn nomi(d: &Der<'_>, nome: &Tlv) -> Vec<(Vec<u8>, String)> {
    let mut out = Vec::new();
    for rdn in d.figli(nome) {
        for atv in d.figli(&rdn) {
            let parti = d.figli(&atv);
            if let (Some(oid), Some(valore)) = (parti.first(), parti.get(1)) {
                let testo = match valore.tag {
                    // BMPString: UTF-16 big endian.
                    0x1E => {
                        let u: Vec<u16> = d
                            .contenuto(valore)
                            .chunks_exact(2)
                            .map(|c| u16::from_be_bytes([c[0], c[1]]))
                            .collect();
                        String::from_utf16_lossy(&u)
                    }
                    _ => String::from_utf8_lossy(d.contenuto(valore)).into_owned(),
                };
                out.push((d.contenuto(oid).to_vec(), testo));
            }
        }
    }
    out
}

fn attributo<'a>(nomi: &'a [(Vec<u8>, String)], oid: &[u8]) -> Option<&'a str> {
    nomi.iter().find(|(o, _)| o == oid).map(|(_, v)| v.as_str())
}

const OID_CN: &[u8] = &[0x55, 0x04, 0x03];
const OID_SERIAL: &[u8] = &[0x55, 0x04, 0x05];
const OID_O: &[u8] = &[0x55, 0x04, 0x0A];
const OID_NOME: &[u8] = &[0x55, 0x04, 0x2A];
const OID_COGNOME: &[u8] = &[0x55, 0x04, 0x04];

/// UTCTime / GeneralizedTime in "AAAA-MM-GG hh:mm:ss".
fn data_asn1(tag: u8, b: &[u8]) -> String {
    let s = String::from_utf8_lossy(b);
    let (anno, resto) = if tag == 0x17 {
        let aa: i32 = s.get(0..2).and_then(|x| x.parse().ok()).unwrap_or(0);
        (
            if aa >= 50 { 1900 + aa } else { 2000 + aa },
            s.get(2..).unwrap_or(""),
        )
    } else {
        (
            s.get(0..4).and_then(|x| x.parse().ok()).unwrap_or(0),
            s.get(4..).unwrap_or(""),
        )
    };
    let p = |i: usize| resto.get(i..i + 2).unwrap_or("00");
    format!("{anno:04}-{}-{} {}:{}:{}", p(0), p(2), p(4), p(6), p(8))
}

fn verifica_firmatario(
    livello: usize,
    firmatario: &[u8],
    contenuto: &[u8],
    certificati: &[Vec<u8>],
) -> Firmatario {
    let mut r = Firmatario {
        livello,
        esito: Esito::NonVerificabile,
        dettaglio: String::new(),
        nome: String::new(),
        codice_fiscale: String::new(),
        organizzazione: String::new(),
        emittente: String::new(),
        valido_dal: String::new(),
        valido_al: String::new(),
        data_firma: String::new(),
        certificato_in_corso_di_validita: false,
        algoritmo: String::new(),
    };
    let d = Der { buf: firmatario };
    let Some(si) = d.radice() else {
        r.dettaglio = "struttura del firmatario illeggibile".into();
        return r;
    };
    let campi = d.figli(&si);
    // SignerInfo ::= SEQUENCE { version, sid, digestAlgorithm, [0] signedAttrs OPZ,
    //                           signatureAlgorithm, signature, [1] unsignedAttrs OPZ }
    let (Some(sid), Some(alg_hash)) = (campi.get(1), campi.get(2)) else {
        r.dettaglio = "firmatario incompleto".into();
        return r;
    };
    let mut i = 3;
    let attributi = campi.get(i).filter(|t| t.tag == 0xA0).copied();
    if attributi.is_some() {
        i += 1;
    }
    let (Some(alg_firma), Some(firma)) = (campi.get(i), campi.get(i + 1)) else {
        r.dettaglio = "firma mancante".into();
        return r;
    };

    // Certificato del firmatario: per emittente e seriale o per identificativo di chiave.
    let certificati: Vec<Certificato<'_>> = certificati
        .iter()
        .filter_map(|c| leggi_certificato(c))
        .collect();
    let cert = if sid.tag == 0x30 {
        let parti = d.figli(sid);
        match (parti.first(), parti.get(1)) {
            (Some(em), Some(ser)) => certificati
                .iter()
                .find(|c| c.emittente_der == d.intero(em) && c.seriale == d.contenuto(ser)),
            _ => None,
        }
    } else {
        let ski = d.contenuto(sid);
        certificati.iter().find(|c| c.ski == Some(ski))
    }
    .or(if certificati.len() == 1 {
        certificati.first()
    } else {
        None
    });

    let Some(cert) = cert else {
        r.dettaglio = "il certificato del firmatario non è incluso nella busta".into();
        return r;
    };
    let cn = attributo(&cert.soggetto, OID_CN).unwrap_or_default();
    r.nome = match (
        attributo(&cert.soggetto, OID_NOME),
        attributo(&cert.soggetto, OID_COGNOME),
    ) {
        (Some(n), Some(c)) => format!("{n} {c}"),
        _ => cn.to_owned(),
    };
    r.codice_fiscale = attributo(&cert.soggetto, OID_SERIAL)
        .map(|s| {
            s.trim_start_matches("TINIT-")
                .trim_start_matches("IT:")
                .to_owned()
        })
        .unwrap_or_default();
    r.organizzazione = attributo(&cert.soggetto, OID_O)
        .unwrap_or_default()
        .to_owned();
    r.emittente = attributo(&cert.emittente, OID_O)
        .or(attributo(&cert.emittente, OID_CN))
        .unwrap_or_default()
        .to_owned();
    r.valido_dal = cert.dal.clone();
    r.valido_al = cert.al.clone();

    // Algoritmo di impronta del documento.
    let Some(hash) = d
        .figli(alg_hash)
        .first()
        .and_then(|o| Hash::da_oid(d.contenuto(o)))
    else {
        r.dettaglio = "algoritmo di impronta non supportato".into();
        return r;
    };

    // Dati effettivamente firmati.
    let impronta_doc = hash.calcola(contenuto);
    let firmati: Vec<u8> = match attributi {
        Some(a) => {
            let mut digest_firmato = None;
            for attr in d.figli(&a) {
                let parti = d.figli(&attr);
                let (Some(oid), Some(valori)) = (parti.first(), parti.get(1)) else {
                    continue;
                };
                let valore = d.figli(valori).first().copied();
                match (d.contenuto(oid), valore) {
                    (OID_MESSAGE_DIGEST, Some(v)) => {
                        digest_firmato = Some(d.contenuto(&v).to_vec())
                    }
                    (OID_SIGNING_TIME, Some(v)) => r.data_firma = data_asn1(v.tag, d.contenuto(&v)),
                    _ => {}
                }
            }
            if digest_firmato.as_deref() != Some(impronta_doc.as_slice()) {
                r.esito = Esito::NonValida;
                r.dettaglio =
                    "il documento è stato modificato dopo la firma (impronta diversa)".into();
                return r;
            }
            // Per la verifica gli attributi si codificano come SET (tag 0x31).
            if a.fine_contenuto != a.fine {
                r.dettaglio = "attributi firmati in codifica non DER".into();
                return r;
            }
            let mut v = d.intero(&a).to_vec();
            v[0] = 0x31;
            v
        }
        None => contenuto.to_vec(),
    };

    // Algoritmo di firma.
    let oid_firma = d
        .figli(alg_firma)
        .first()
        .map(|o| d.contenuto(o).to_vec())
        .unwrap_or_default();
    let byte_firma = d.contenuto(firma);
    let esito = if oid_firma.starts_with(PREFISSO_PKCS1) {
        // rsaEncryption usa l'impronta indicata nel firmatario; gli altri la dichiarano nell'OID.
        let h = match oid_firma.last() {
            Some(0x01) => Some(hash),
            Some(0x05) => Some(Hash::Sha1),
            Some(0x0B) => Some(Hash::Sha256),
            Some(0x0C) => Some(Hash::Sha384),
            Some(0x0D) => Some(Hash::Sha512),
            _ => None,
        };
        match h {
            Some(h) => {
                r.algoritmo = format!("RSA con {}", h.nome());
                verifica_rsa(cert.spki, h, &firmati, byte_firma)
            }
            None => {
                r.dettaglio = "algoritmo RSA non supportato (es. RSA-PSS)".into();
                return r;
            }
        }
    } else if oid_firma.starts_with(PREFISSO_ECDSA) {
        let h = match oid_firma.last() {
            Some(0x02) => Hash::Sha256,
            Some(0x03) => Hash::Sha384,
            Some(0x04) => Hash::Sha512,
            _ => hash,
        };
        r.algoritmo = format!("ECDSA con {}", h.nome());
        verifica_ecdsa(cert.spki, h, &firmati, byte_firma)
    } else {
        r.dettaglio = "algoritmo di firma non supportato".into();
        return r;
    };

    match esito {
        Ok(true) => {
            r.esito = Esito::Valida;
            if hash == Hash::Sha1 {
                r.dettaglio =
                    "firma integra, ma con algoritmo SHA-1 non più ammesso per le nuove firme"
                        .into();
            } else {
                r.dettaglio =
                    "firma integra: il documento non è stato modificato dopo la firma".into();
            }
        }
        Ok(false) => {
            r.esito = Esito::NonValida;
            r.dettaglio = "la firma non corrisponde al certificato del firmatario".into();
        }
        Err(m) => r.dettaglio = m,
    }

    // Validità del certificato alla data della firma (o a oggi).
    let riferimento = if r.data_firma.is_empty() {
        chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string()
    } else {
        r.data_firma.clone()
    };
    r.certificato_in_corso_di_validita = cert.dal <= riferimento && riferimento <= cert.al;
    r
}

fn verifica_rsa(spki: &[u8], h: Hash, dati: &[u8], firma: &[u8]) -> Result<bool, String> {
    use rsa::pkcs8::DecodePublicKey;
    use rsa::{Pkcs1v15Sign, RsaPublicKey};
    let chiave = RsaPublicKey::from_public_key_der(spki)
        .map_err(|_| "chiave pubblica RSA illeggibile".to_owned())?;
    let schema = match h {
        Hash::Sha1 => Pkcs1v15Sign::new::<Sha1>(),
        Hash::Sha256 => Pkcs1v15Sign::new::<Sha256>(),
        Hash::Sha384 => Pkcs1v15Sign::new::<Sha384>(),
        Hash::Sha512 => Pkcs1v15Sign::new::<Sha512>(),
    };
    Ok(chiave.verify(schema, &h.calcola(dati), firma).is_ok())
}

fn verifica_ecdsa(spki: &[u8], h: Hash, dati: &[u8], firma: &[u8]) -> Result<bool, String> {
    use p256::ecdsa::signature::hazmat::PrehashVerifier;
    use p256::pkcs8::DecodePublicKey;
    let impronta = h.calcola(dati);
    // La curva è indicata nei parametri dell'algoritmo della chiave pubblica.
    let d = Der { buf: spki };
    let curva = d
        .radice()
        .and_then(|t| d.figli(&t).first().copied())
        .map(|alg| d.figli(&alg))
        .filter(|p| p.first().map(|o| d.contenuto(o)) == Some(OID_CHIAVE_EC))
        .and_then(|p| p.get(1).map(|c| d.contenuto(c).to_vec()))
        .unwrap_or_default();
    match curva.as_slice() {
        OID_P256 => {
            let k = p256::ecdsa::VerifyingKey::from_public_key_der(spki)
                .map_err(|_| "chiave ECDSA illeggibile".to_owned())?;
            let s = p256::ecdsa::Signature::from_der(firma)
                .map_err(|_| "firma ECDSA illeggibile".to_owned())?;
            Ok(k.verify_prehash(&impronta, &s).is_ok())
        }
        OID_P384 => {
            let k = p384::ecdsa::VerifyingKey::from_public_key_der(spki)
                .map_err(|_| "chiave ECDSA illeggibile".to_owned())?;
            let s = p384::ecdsa::Signature::from_der(firma)
                .map_err(|_| "firma ECDSA illeggibile".to_owned())?;
            Ok(k.verify_prehash(&impronta, &s).is_ok())
        }
        _ => Err("curva ellittica non supportata".to_owned()),
    }
}

#[cfg(test)]
mod test {
    use super::*;

    // Buste generate con OpenSSL (`openssl cms -sign -nodetach -binary -outform DER`)
    // a partire da certificati di prova autofirmati.
    const RSA_P7M: &[u8] = include_bytes!("../tests/dati/firma_rsa.xml.p7m");
    const EC_P7M: &[u8] = include_bytes!("../tests/dati/firma_ec.xml.p7m");

    #[test]
    fn firma_rsa_valida() {
        let v = verifica(RSA_P7M);
        assert_eq!(v.len(), 1);
        assert_eq!(v[0].esito, Esito::Valida, "{}", v[0].dettaglio);
        assert_eq!(v[0].nome, "Mario Rossi");
        assert_eq!(v[0].codice_fiscale, "RSSMRA80A01L219X");
        assert!(v[0].certificato_in_corso_di_validita);
        assert!(!v[0].data_firma.is_empty());
    }

    #[test]
    fn firma_ecdsa_valida() {
        let v = verifica(EC_P7M);
        assert_eq!(v.len(), 1);
        assert_eq!(v[0].esito, Esito::Valida, "{}", v[0].dettaglio);
        assert!(v[0].algoritmo.starts_with("ECDSA"));
    }

    #[test]
    fn documento_alterato() {
        // Si cambia un carattere del contenuto: l'impronta non torna più.
        let mut b = RSA_P7M.to_vec();
        let pos = b.windows(6).position(|w| w == b"Numero").unwrap();
        b[pos] = b'n';
        let v = verifica(&b);
        assert_eq!(v[0].esito, Esito::NonValida);
    }
}
