// Copyright (c) 2026 Andrea Re - info@digitare.net
// Autore: Andrea Re @ 2026 - info@digitare.net
//
// Funzione format-number() e dichiarazioni xsl:decimal-format.

/// Simboli di un formato decimale.
#[derive(Debug, Clone)]
pub struct FormatoDecimale {
    pub decimale: char,
    pub migliaia: char,
    pub infinito: String,
    pub meno: char,
    pub nan: String,
    pub percento: char,
    pub permille: char,
    pub zero: char,
    pub cifra: char,
    pub separatore: char,
}

impl Default for FormatoDecimale {
    fn default() -> Self {
        FormatoDecimale {
            decimale: '.',
            migliaia: ',',
            infinito: "Infinity".into(),
            meno: '-',
            nan: "NaN".into(),
            percento: '%',
            permille: '\u{2030}',
            zero: '0',
            cifra: '#',
            separatore: ';',
        }
    }
}

struct Modello {
    prefisso: String,
    suffisso: String,
    minimo_interi: usize,
    minimo_decimali: usize,
    massimo_decimali: usize,
    gruppo: usize,
    moltiplicatore: f64,
}

fn analizza_modello(m: &str, f: &FormatoDecimale) -> Modello {
    let numerico = |c: char| c == f.cifra || c == f.zero || c == f.decimale || c == f.migliaia;
    let caratteri: Vec<char> = m.chars().collect();
    let inizio = caratteri
        .iter()
        .position(|&c| numerico(c))
        .unwrap_or(caratteri.len());
    let mut fine = inizio;
    while fine < caratteri.len() && numerico(caratteri[fine]) {
        fine += 1;
    }
    let prefisso: String = caratteri[..inizio].iter().collect();
    let suffisso: String = caratteri[fine..].iter().collect();
    let corpo = &caratteri[inizio..fine];
    let (interi, decimali) = match corpo.iter().position(|&c| c == f.decimale) {
        Some(p) => (&corpo[..p], &corpo[p + 1..]),
        None => (corpo, &corpo[corpo.len()..]),
    };
    let gruppo = match interi.iter().rposition(|&c| c == f.migliaia) {
        Some(p) => interi[p + 1..]
            .iter()
            .filter(|&&c| c == f.cifra || c == f.zero)
            .count(),
        None => 0,
    };
    let mut moltiplicatore = 1.0;
    if prefisso.contains(f.percento) || suffisso.contains(f.percento) {
        moltiplicatore = 100.0;
    } else if prefisso.contains(f.permille) || suffisso.contains(f.permille) {
        moltiplicatore = 1000.0;
    }
    Modello {
        prefisso,
        suffisso,
        minimo_interi: interi.iter().filter(|&&c| c == f.zero).count(),
        minimo_decimali: decimali.iter().filter(|&&c| c == f.zero).count(),
        massimo_decimali: decimali
            .iter()
            .filter(|&&c| c == f.zero || c == f.cifra)
            .count(),
        gruppo,
        moltiplicatore,
    }
}

/// Formatta un numero secondo un modello in stile DecimalFormat.
pub fn formatta(n: f64, modello: &str, f: &FormatoDecimale) -> String {
    if n.is_nan() {
        return f.nan.clone();
    }
    let (positivo, negativo) = match modello.split_once(f.separatore) {
        Some((p, q)) => (p, Some(q)),
        None => (modello, None),
    };
    let mp = analizza_modello(positivo, f);
    let valore = n.abs() * mp.moltiplicatore;
    let corpo = if valore.is_infinite() {
        f.infinito.clone()
    } else {
        cifre(valore, &mp, f)
    };
    // Lo zero arrotondato non porta il segno.
    let zero = f.zero as u32;
    let negativo_reale = n < 0.0
        && (valore.is_infinite()
            || corpo
                .chars()
                .any(|c| (c as u32) > zero && (c as u32) <= zero + 9));
    if !negativo_reale {
        return format!("{}{}{}", mp.prefisso, corpo, mp.suffisso);
    }
    match negativo {
        Some(q) => {
            let mq = analizza_modello(q, f);
            format!("{}{}{}", mq.prefisso, corpo, mq.suffisso)
        }
        None => format!("{}{}{}{}", f.meno, mp.prefisso, corpo, mp.suffisso),
    }
}

fn cifre(valore: f64, m: &Modello, f: &FormatoDecimale) -> String {
    let scala = 10f64.powi(m.massimo_decimali as i32);
    // Arrotondamento al più vicino, a metà verso l'alto.
    let arrotondato = (valore * scala + 0.5).floor();
    let mut tutte = format!("{arrotondato:.0}");
    while tutte.len() <= m.massimo_decimali {
        tutte.insert(0, '0');
    }
    let (interi, decimali) = tutte.split_at(tutte.len() - m.massimo_decimali);
    let mut decimali = decimali.to_string();
    while decimali.len() > m.minimo_decimali && decimali.ends_with('0') {
        decimali.pop();
    }
    let mut interi = interi.trim_start_matches('0').to_string();
    while interi.len() < m.minimo_interi {
        interi.insert(0, '0');
    }
    let mut s = String::new();
    let lunghezza = interi.len();
    for (i, c) in interi.chars().enumerate() {
        if m.gruppo > 0 && i > 0 && (lunghezza - i) % m.gruppo == 0 {
            s.push(f.migliaia);
        }
        s.push(cifra(c, f));
    }
    if !decimali.is_empty() {
        s.push(f.decimale);
        s.extend(decimali.chars().map(|c| cifra(c, f)));
    }
    s
}

fn cifra(c: char, f: &FormatoDecimale) -> char {
    let d = c.to_digit(10).unwrap_or(0);
    char::from_u32(f.zero as u32 + d).unwrap_or(c)
}

#[cfg(test)]
mod test {
    use super::*;

    fn euro() -> FormatoDecimale {
        FormatoDecimale {
            decimale: ',',
            migliaia: '.',
            ..Default::default()
        }
    }

    #[test]
    fn formato_euro() {
        let f = euro();
        assert_eq!(formatta(1234567.891, "###.###.##0,00", &f), "1.234.567,89");
        assert_eq!(formatta(0.5, "###.###.##0,00", &f), "0,50");
        assert_eq!(formatta(-22.0, "###.###.##0,00", &f), "-22,00");
        assert_eq!(formatta(12.345, "###.###.##0,00", &f), "12,35");
        assert_eq!(
            formatta(1.23456789, "###.###.##0,00######", &f),
            "1,23456789"
        );
        assert_eq!(formatta(1.5, "###.###.##0,00######", &f), "1,50");
        assert_eq!(formatta(-0.001, "###.###.##0,00", &f), "0,00");
        assert_eq!(formatta(f64::NAN, "###.###.##0,00", &f), "NaN");
    }

    #[test]
    fn formato_standard() {
        let f = FormatoDecimale::default();
        assert_eq!(formatta(1234.5, "#,##0.00", &f), "1,234.50");
        assert_eq!(formatta(0.25, "0%", &f), "25%");
        assert_eq!(formatta(-3.0, "0;(0)", &f), "(3)");
        assert_eq!(formatta(7.0, "000", &f), "007");
    }
}
