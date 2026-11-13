// Copyright (c) 2026 Andrea Re - info@digitare.net
// Autore: Andrea Re @ 2026 - info@digitare.net
//
// Nodi del documento sorgente e valori XPath con le relative conversioni.

use std::cmp::Ordering;

use roxmltree::{Node, NodeType};

use crate::uscita::Frammento;

/// Nodo del documento sorgente: un nodo vero e proprio oppure un
/// attributo (roxmltree non tratta gli attributi come nodi).
#[derive(Debug, Clone, Copy)]
pub enum Nodo<'a, 'i> {
    Normale(Node<'a, 'i>),
    Attributo(Node<'a, 'i>, usize),
}

impl PartialEq for Nodo<'_, '_> {
    fn eq(&self, altro: &Self) -> bool {
        self.ordine() == altro.ordine()
    }
}

impl Eq for Nodo<'_, '_> {}

impl<'a, 'i> Nodo<'a, 'i> {
    /// Chiave di ordinamento nel documento: gli attributi seguono il loro
    /// elemento e precedono i figli.
    pub fn ordine(&self) -> (usize, usize) {
        match self {
            Nodo::Normale(n) => (n.id().get_usize(), 0),
            Nodo::Attributo(n, i) => (n.id().get_usize(), i + 1),
        }
    }

    pub fn elemento(&self) -> Node<'a, 'i> {
        match self {
            Nodo::Normale(n) | Nodo::Attributo(n, _) => *n,
        }
    }

    pub fn genitore(&self) -> Option<Nodo<'a, 'i>> {
        match self {
            Nodo::Normale(n) => n.parent().map(Nodo::Normale),
            Nodo::Attributo(n, _) => Some(Nodo::Normale(*n)),
        }
    }

    pub fn radice(&self) -> Nodo<'a, 'i> {
        Nodo::Normale(self.elemento().document().root())
    }

    pub fn attributo(&self) -> Option<roxmltree::Attribute<'a, 'i>> {
        match self {
            Nodo::Attributo(n, i) => n.attributes().nth(*i),
            Nodo::Normale(_) => None,
        }
    }

    /// Spazio dei nomi e nome locale (solo per elementi e attributi).
    pub fn nome_espanso(&self) -> Option<(Option<&'a str>, &'a str)> {
        // Uno spazio dei nomi vuoto (`xmlns=""`, che annulla quello predefinito del padre) equivale
        // a nessuno spazio dei nomi: roxmltree lo restituisce come stringa vuota.
        let senza_vuoto = |u: Option<&'a str>| u.filter(|u| !u.is_empty());
        match self {
            Nodo::Normale(n) if n.is_element() => {
                Some((senza_vuoto(n.tag_name().namespace()), n.tag_name().name()))
            }
            Nodo::Normale(n) if n.is_pi() => n.pi().map(|p| (None, p.target)),
            Nodo::Attributo(..) => self
                .attributo()
                .map(|a| (senza_vuoto(a.namespace()), a.name())),
            _ => None,
        }
    }

    /// Nome qualificato con il prefisso usato nel documento.
    pub fn nome(&self) -> String {
        match self.nome_espanso() {
            Some((Some(ns), locale)) => match self.elemento().lookup_prefix(ns) {
                Some(p) if !p.is_empty() => format!("{p}:{locale}"),
                _ => locale.to_string(),
            },
            Some((None, locale)) => locale.to_string(),
            None => String::new(),
        }
    }

    /// Valore testuale secondo XPath.
    pub fn testo(&self) -> String {
        match self {
            Nodo::Attributo(..) => self
                .attributo()
                .map(|a| a.value().to_string())
                .unwrap_or_default(),
            Nodo::Normale(n) => match n.node_type() {
                NodeType::Root | NodeType::Element => {
                    let mut s = String::new();
                    for d in n.descendants().filter(|d| d.is_text()) {
                        s.push_str(d.text().unwrap_or(""));
                    }
                    s
                }
                NodeType::PI => n.pi().and_then(|p| p.value).unwrap_or("").to_string(),
                _ => n.text().unwrap_or("").to_string(),
            },
        }
    }
}

/// Ordina in ordine di documento ed elimina i duplicati.
pub fn ordina(nodi: &mut Vec<Nodo<'_, '_>>) {
    nodi.sort_unstable_by_key(|n| n.ordine());
    nodi.dedup();
}

#[derive(Debug, Clone)]
pub enum Valore<'a, 'i> {
    Nodi(Vec<Nodo<'a, 'i>>),
    Stringa(String),
    Numero(f64),
    Booleano(bool),
    /// Frammento dell'albero risultato (variabile con contenuto).
    Frammento(Frammento),
}

impl Valore<'_, '_> {
    pub fn stringa(&self) -> String {
        match self {
            Valore::Nodi(n) => n.first().map(|n| n.testo()).unwrap_or_default(),
            Valore::Stringa(s) => s.clone(),
            Valore::Numero(n) => numero_in_stringa(*n),
            Valore::Booleano(b) => if *b { "true" } else { "false" }.to_string(),
            Valore::Frammento(f) => f.testo(),
        }
    }

    pub fn numero(&self) -> f64 {
        match self {
            Valore::Numero(n) => *n,
            Valore::Booleano(b) => f64::from(u8::from(*b)),
            _ => stringa_in_numero(&self.stringa()),
        }
    }

    pub fn booleano(&self) -> bool {
        match self {
            Valore::Nodi(n) => !n.is_empty(),
            Valore::Stringa(s) => !s.is_empty(),
            Valore::Numero(n) => *n != 0.0 && !n.is_nan(),
            Valore::Booleano(b) => *b,
            // Un frammento contiene sempre il nodo radice.
            Valore::Frammento(_) => true,
        }
    }
}

/// Conversione da numero a stringa secondo XPath 1.0 (mai in notazione
/// esponenziale).
pub fn numero_in_stringa(n: f64) -> String {
    if n.is_nan() {
        "NaN".into()
    } else if n.is_infinite() {
        if n > 0.0 { "Infinity" } else { "-Infinity" }.into()
    } else if n == 0.0 {
        "0".into()
    } else if n.fract() == 0.0 && n.abs() < 1e17 {
        format!("{}", n as i64)
    } else {
        // Quindici cifre significative, come i motori XSLT diffusi:
        // 0.1 + 0.2 diventa "0.3".
        let interi = n.abs().log10().floor() as i32 + 1;
        let decimali = (15 - interi).clamp(0, 340) as usize;
        let s = format!("{n:.decimali$}");
        if s.contains('.') {
            s.trim_end_matches('0').trim_end_matches('.').to_string()
        } else {
            s
        }
    }
}

/// Conversione da stringa a numero: solo cifre, punto e segno meno iniziale.
pub fn stringa_in_numero(s: &str) -> f64 {
    let t = s.trim_matches(|c: char| matches!(c, ' ' | '\t' | '\r' | '\n'));
    let corpo = t.strip_prefix('-').unwrap_or(t);
    let valido = !corpo.is_empty()
        && corpo != "."
        && corpo.chars().all(|c| c.is_ascii_digit() || c == '.')
        && corpo.matches('.').count() <= 1;
    if valido {
        t.parse().unwrap_or(f64::NAN)
    } else {
        f64::NAN
    }
}

/// Confronto fra numeri compatibile con NaN (sempre falso).
pub fn confronta_numeri(a: f64, b: f64) -> Option<Ordering> {
    a.partial_cmp(&b)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn conversioni_numeriche() {
        assert_eq!(numero_in_stringa(3.0), "3");
        assert_eq!(numero_in_stringa(-0.5), "-0.5");
        assert_eq!(numero_in_stringa(1e21), "1000000000000000000000");
        assert_eq!(numero_in_stringa(f64::NAN), "NaN");
        assert_eq!(numero_in_stringa(0.1 + 0.2), "0.3");
        assert_eq!(numero_in_stringa(1234.5678), "1234.5678");
        assert_eq!(numero_in_stringa(2.0 / 3.0), "0.666666666666667");
        assert_eq!(stringa_in_numero(" 12.50 "), 12.5);
        assert!(stringa_in_numero("1e3").is_nan());
        assert!(stringa_in_numero("+1").is_nan());
        assert!(stringa_in_numero("").is_nan());
    }
}
