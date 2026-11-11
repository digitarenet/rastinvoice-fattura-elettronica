// Copyright (c) 2026 Andrea Re - info@digitare.net
// Autore: Andrea Re @ 2026 - info@digitare.net
//
// Formattazione all'italiana di importi, numeri, date, e escape HTML.

use chrono::NaiveDate;
use rust_decimal::Decimal;

/// Escape dei caratteri speciali HTML.
pub fn esc(s: &str) -> String {
    let mut r = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '&' => r.push_str("&amp;"),
            '<' => r.push_str("&lt;"),
            '>' => r.push_str("&gt;"),
            '"' => r.push_str("&quot;"),
            '\'' => r.push_str("&#39;"),
            _ => r.push(c),
        }
    }
    r
}

/// Data nel formato gg/mm/aaaa, oppure stringa vuota.
pub fn data(d: Option<NaiveDate>) -> String {
    d.map(|x| x.format("%d/%m/%Y").to_string())
        .unwrap_or_default()
}

/// Raggruppa le migliaia con il punto e usa la virgola per i decimali.
fn all_italiana(valore: Decimal, decimali_min: u32) -> String {
    let mut v = valore.normalize();
    if v.scale() < decimali_min {
        v.rescale(decimali_min);
    }
    let testo = v.abs().to_string();
    let (intera, frazione) = match testo.split_once('.') {
        Some((i, f)) => (i.to_owned(), Some(f.to_owned())),
        None => (testo, None),
    };
    let mut gruppi = String::new();
    for (i, c) in intera.chars().enumerate() {
        if i > 0 && (intera.len() - i) % 3 == 0 {
            gruppi.push('.');
        }
        gruppi.push(c);
    }
    let segno = if valore.is_sign_negative() && !valore.is_zero() {
        "-"
    } else {
        ""
    };
    match frazione {
        Some(f) => format!("{segno}{gruppi},{f}"),
        None => format!("{segno}{gruppi}"),
    }
}

/// Importo con almeno due decimali (es. `1.234,50`); `-` se assente.
pub fn importo(v: Option<Decimal>) -> String {
    v.map(|x| all_italiana(x, 2))
        .unwrap_or_else(|| "-".to_owned())
}

/// Numero senza zeri superflui (quantità, riferimenti numerici).
pub fn numero(v: Option<Decimal>) -> String {
    v.map(|x| all_italiana(x, 0)).unwrap_or_default()
}

/// Percentuale (es. `22%`, `4,5%`).
pub fn percentuale(v: Option<Decimal>) -> String {
    v.map(|x| format!("{}%", all_italiana(x, 0)))
        .unwrap_or_default()
}

/// IBAN a gruppi di quattro caratteri, per una lettura più agevole.
pub fn iban(s: &str) -> String {
    s.chars()
        .filter(|c| !c.is_whitespace())
        .collect::<Vec<_>>()
        .chunks(4)
        .map(|g| g.iter().collect::<String>())
        .collect::<Vec<_>>()
        .join(" ")
}

#[cfg(test)]
mod test {
    use super::*;
    use std::str::FromStr;

    fn d(s: &str) -> Option<Decimal> {
        Some(Decimal::from_str(s).unwrap())
    }

    #[test]
    fn importi() {
        assert_eq!(importo(d("1234567.5")), "1.234.567,50");
        assert_eq!(importo(d("-45.9")), "-45,90");
        assert_eq!(importo(d("0.00")), "0,00");
        assert_eq!(importo(d("12.34567")), "12,34567");
        assert_eq!(numero(d("2.00")), "2");
        assert_eq!(percentuale(d("22.00")), "22%");
        assert_eq!(percentuale(d("4.50")), "4,5%");
    }

    #[test]
    fn varie() {
        assert_eq!(esc("<a&\"b\">"), "&lt;a&amp;&quot;b&quot;&gt;");
        assert_eq!(
            iban("IT60X0542811101000000123456"),
            "IT60 X054 2811 1010 0000 0123 456"
        );
    }
}
