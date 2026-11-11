// Copyright (c) 2026 Andrea Re - info@digitare.net
// Autore: Andrea Re @ 2026 - info@digitare.net
//
// Utilità per la decodifica dei byte in testo XML e per la navigazione
// dell'albero. I nomi degli elementi vengono confrontati sul solo nome
// locale: nei file FatturaPA soltanto la radice è qualificata da uno
// spazio dei nomi, ma alcuni gestionali producono prefissi anche sui figli.

use crate::errore::{Errore, Risultato};
use encoding_rs::{Encoding, UTF_16BE, UTF_16LE, UTF_8};
use roxmltree::Node;

/// Converte i byte di un file in una stringa UTF-8 pronta per il parser.
///
/// Ordine di priorità per scegliere la codifica:
/// 1. BOM (UTF-8, UTF-16 LE/BE);
/// 2. attributo `encoding` della dichiarazione XML;
/// 3. UTF-8, con ripiego su Windows-1252 se i byte non sono UTF-8 valido
///    (caso frequente con file prodotti da software datati).
pub fn decodifica(byte: &[u8]) -> Risultato<String> {
    let testo = if let Some((codifica, lunghezza_bom)) = Encoding::for_bom(byte) {
        let (s, _, _) = codifica.decode(&byte[lunghezza_bom..]);
        s.into_owned()
    } else if let Some(codifica) = codifica_dichiarata(byte) {
        if codifica == UTF_16LE || codifica == UTF_16BE {
            // Una dichiarazione UTF-16 senza BOM leggibile in ASCII è contraddittoria:
            // il file è in realtà a 8 bit, quindi lo trattiamo come UTF-8.
            decodifica_utf8_o_1252(byte)
        } else if codifica == UTF_8 {
            decodifica_utf8_o_1252(byte)
        } else {
            let (s, _, _) = codifica.decode(byte);
            s.into_owned()
        }
    } else {
        decodifica_utf8_o_1252(byte)
    };
    Ok(ripulisci(&testo))
}

/// Tenta UTF-8; se fallisce usa Windows-1252, superinsieme di ISO-8859-1.
fn decodifica_utf8_o_1252(byte: &[u8]) -> String {
    match std::str::from_utf8(byte) {
        Ok(s) => s.to_owned(),
        Err(_) => {
            let (s, _, _) = encoding_rs::WINDOWS_1252.decode(byte);
            s.into_owned()
        }
    }
}

/// Legge l'attributo `encoding` dalla dichiarazione `<?xml ... ?>`, se presente.
fn codifica_dichiarata(byte: &[u8]) -> Option<&'static Encoding> {
    let inizio = &byte[..byte.len().min(200)];
    let testa = String::from_utf8_lossy(inizio);
    let testa = testa.trim_start();
    if !testa.starts_with("<?xml") {
        return None;
    }
    let fine = testa.find("?>")?;
    let dichiarazione = &testa[..fine];
    let pos = dichiarazione.find("encoding")?;
    let resto = &dichiarazione[pos + "encoding".len()..];
    let resto = resto.trim_start().strip_prefix('=')?.trim_start();
    let apice = resto.chars().next()?;
    if apice != '"' && apice != '\'' {
        return None;
    }
    let valore = &resto[1..];
    let chiusura = valore.find(apice)?;
    Encoding::for_label(valore[..chiusura].trim().as_bytes())
}

/// Rimuove ciò che precede il primo `<` e ciò che segue l'ultimo `>`.
///
/// Alcune buste firmate lasciano byte nulli o spazzatura in coda al
/// contenuto, che renderebbero il documento non ben formato.
fn ripulisci(testo: &str) -> String {
    let inizio = testo.find('<').unwrap_or(0);
    let fine = testo.rfind('>').map(|p| p + 1).unwrap_or(testo.len());
    if inizio >= fine {
        return testo.to_owned();
    }
    testo[inizio..fine].to_owned()
}

/// Analizza il testo come documento XML.
pub fn analizza(testo: &str) -> Risultato<roxmltree::Document<'_>> {
    let opzioni = roxmltree::ParsingOptions {
        allow_dtd: false,
        ..Default::default()
    };
    roxmltree::Document::parse_with_options(testo, opzioni)
        .map_err(|e| Errore::XmlNonValido(e.to_string()))
}

/// Restituisce il primo figlio elemento con il nome locale indicato.
pub fn figlio<'a, 'i>(nodo: Node<'a, 'i>, nome: &str) -> Option<Node<'a, 'i>> {
    nodo.children()
        .find(|n| n.is_element() && n.tag_name().name() == nome)
}

/// Restituisce tutti i figli elemento con il nome locale indicato.
pub fn figli<'a, 'i: 'a>(
    nodo: Node<'a, 'i>,
    nome: &'a str,
) -> impl Iterator<Item = Node<'a, 'i>> + 'a {
    nodo.children()
        .filter(move |n| n.is_element() && n.tag_name().name() == nome)
}

/// Segue un percorso di nomi a partire dal nodo dato.
pub fn percorso<'a, 'i>(nodo: Node<'a, 'i>, nomi: &[&str]) -> Option<Node<'a, 'i>> {
    let mut corrente = nodo;
    for nome in nomi {
        corrente = figlio(corrente, nome)?;
    }
    Some(corrente)
}

/// Testo di un nodo, ripulito dagli spazi e `None` se vuoto.
pub fn testo_nodo(nodo: Node<'_, '_>) -> Option<String> {
    let mut valore = String::new();
    for n in nodo.children() {
        if n.is_text() {
            valore.push_str(n.text().unwrap_or_default());
        }
    }
    let valore = valore.trim();
    if valore.is_empty() {
        None
    } else {
        Some(valore.to_owned())
    }
}

/// Testo del primo figlio con il nome indicato.
pub fn testo(nodo: Node<'_, '_>, nome: &str) -> Option<String> {
    figlio(nodo, nome).and_then(testo_nodo)
}

/// Testo dell'elemento raggiunto seguendo il percorso.
pub fn testo_percorso(nodo: Node<'_, '_>, nomi: &[&str]) -> Option<String> {
    percorso(nodo, nomi).and_then(testo_nodo)
}

/// Testi di tutti i figli con il nome indicato (es. più `Causale`).
pub fn testi(nodo: Node<'_, '_>, nome: &str) -> Vec<String> {
    figli(nodo, nome).filter_map(testo_nodo).collect()
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn decodifica_utf8_con_bom() {
        let byte = b"\xEF\xBB\xBF<a>\xC3\xA8</a>";
        assert_eq!(decodifica(byte).unwrap(), "<a>è</a>");
    }

    #[test]
    fn decodifica_iso_8859_1_dichiarato() {
        let byte = b"<?xml version=\"1.0\" encoding=\"ISO-8859-1\"?><a>\xE8</a>";
        assert!(decodifica(byte).unwrap().ends_with("<a>è</a>"));
    }

    #[test]
    fn ripiego_su_windows_1252() {
        let byte = b"<?xml version=\"1.0\" encoding=\"UTF-8\"?><a>\xE8</a>";
        assert!(decodifica(byte).unwrap().ends_with("<a>è</a>"));
    }

    #[test]
    fn rimuove_spazzatura_in_coda() {
        let byte = b"<a>x</a>\0\0\0";
        assert_eq!(decodifica(byte).unwrap(), "<a>x</a>");
    }
}
