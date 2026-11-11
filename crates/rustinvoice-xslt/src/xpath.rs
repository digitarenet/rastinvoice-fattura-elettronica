// Copyright (c) 2026 Andrea Re - info@digitare.net
// Autore: Andrea Re @ 2026 - info@digitare.net
//
// Analisi delle espressioni XPath 1.0: suddivisione in simboli e
// costruzione dell'albero sintattico.

use crate::Errore;

/// Assi di navigazione.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Asse {
    Antenato,
    AntenatoOSe,
    Attributo,
    Figlio,
    Discendente,
    DiscendenteOSe,
    Seguente,
    FratelloSeguente,
    Spazio,
    Genitore,
    Precedente,
    FratelloPrecedente,
    Se,
}

impl Asse {
    fn da_nome(nome: &str) -> Option<Asse> {
        Some(match nome {
            "ancestor" => Asse::Antenato,
            "ancestor-or-self" => Asse::AntenatoOSe,
            "attribute" => Asse::Attributo,
            "child" => Asse::Figlio,
            "descendant" => Asse::Discendente,
            "descendant-or-self" => Asse::DiscendenteOSe,
            "following" => Asse::Seguente,
            "following-sibling" => Asse::FratelloSeguente,
            "namespace" => Asse::Spazio,
            "parent" => Asse::Genitore,
            "preceding" => Asse::Precedente,
            "preceding-sibling" => Asse::FratelloPrecedente,
            "self" => Asse::Se,
            _ => return None,
        })
    }

    /// Gli assi inversi numerano le posizioni a partire dal nodo più vicino.
    pub fn inverso(self) -> bool {
        matches!(
            self,
            Asse::Antenato | Asse::AntenatoOSe | Asse::Precedente | Asse::FratelloPrecedente
        )
    }
}

/// Verifica sul nodo di un passo.
#[derive(Debug, Clone, PartialEq)]
pub enum Test {
    /// Nome con eventuale spazio dei nomi già risolto.
    Nome(Option<String>, String),
    /// `*`
    Qualsiasi,
    /// `prefisso:*`
    SpazioQualsiasi(String),
    Testo,
    Commento,
    Istruzione(Option<String>),
    Nodo,
}

#[derive(Debug, Clone)]
pub struct Passo {
    pub asse: Asse,
    pub test: Test,
    pub predicati: Vec<Espr>,
}

#[derive(Debug, Clone)]
pub enum Base {
    Radice,
    Contesto,
    Filtro(Box<Espr>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Operatore {
    O,
    E,
    Uguale,
    Diverso,
    Minore,
    MinoreUguale,
    Maggiore,
    MaggioreUguale,
    Piu,
    Meno,
    Per,
    Diviso,
    Modulo,
}

#[derive(Debug, Clone)]
pub enum Espr {
    Binaria(Operatore, Box<Espr>, Box<Espr>),
    Negazione(Box<Espr>),
    Unione(Box<Espr>, Box<Espr>),
    Letterale(String),
    Numero(f64),
    Variabile(String),
    Funzione(String, Vec<Espr>),
    Filtro(Box<Espr>, Vec<Espr>),
    Percorso(Base, Vec<Passo>),
}

#[derive(Debug, Clone, PartialEq)]
enum Simbolo {
    ParAperta,
    ParChiusa,
    QuadraAperta,
    QuadraChiusa,
    Punto,
    DuePunti,
    Chiocciola,
    Virgola,
    DoppiDuePunti,
    Barra,
    DoppiaBarra,
    Pipe,
    Piu,
    Meno,
    Uguale,
    Diverso,
    Minore,
    MinoreUguale,
    Maggiore,
    MaggioreUguale,
    Asterisco,
    Per,
    E,
    O,
    Modulo,
    Diviso,
    Letterale(String),
    Numero(f64),
    Variabile(String),
    /// Nome di funzione (seguito da parentesi).
    Funzione(String),
    /// Tipo di nodo: text, node, comment, processing-instruction.
    TipoNodo(String),
    NomeAsse(String),
    /// Nome qualificato o `prefisso:*`.
    Nome(String),
}

impl Simbolo {
    /// Dopo questi simboli `*` e i nomi sono operandi, non operatori.
    fn apre_operando(&self) -> bool {
        matches!(
            self,
            Simbolo::Chiocciola
                | Simbolo::DoppiDuePunti
                | Simbolo::ParAperta
                | Simbolo::QuadraAperta
                | Simbolo::Virgola
                | Simbolo::E
                | Simbolo::O
                | Simbolo::Modulo
                | Simbolo::Diviso
                | Simbolo::Per
                | Simbolo::Barra
                | Simbolo::DoppiaBarra
                | Simbolo::Pipe
                | Simbolo::Piu
                | Simbolo::Meno
                | Simbolo::Uguale
                | Simbolo::Diverso
                | Simbolo::Minore
                | Simbolo::MinoreUguale
                | Simbolo::Maggiore
                | Simbolo::MaggioreUguale
        )
    }
}

fn inizio_nome(c: char) -> bool {
    c.is_alphabetic() || c == '_'
}

fn parte_nome(c: char) -> bool {
    c.is_alphanumeric() || matches!(c, '_' | '-' | '.' | '\u{B7}')
}

fn simboli(testo: &str) -> Result<Vec<Simbolo>, Errore> {
    let c: Vec<char> = testo.chars().collect();
    let mut i = 0;
    let mut out: Vec<Simbolo> = Vec::new();
    let errore = |m: &str| Errore::XPath(format!("{m} in \"{testo}\""));
    // Prossimo carattere non spazio a partire da j.
    let prossimo = |mut j: usize| {
        while j < c.len() && c[j].is_whitespace() {
            j += 1;
        }
        j
    };
    while i < c.len() {
        let ch = c[i];
        if ch.is_whitespace() {
            i += 1;
            continue;
        }
        let operando = out.last().map_or(true, |s| s.apre_operando());
        let s = match ch {
            '(' => Simbolo::ParAperta,
            ')' => Simbolo::ParChiusa,
            '[' => Simbolo::QuadraAperta,
            ']' => Simbolo::QuadraChiusa,
            '@' => Simbolo::Chiocciola,
            ',' => Simbolo::Virgola,
            '|' => Simbolo::Pipe,
            '+' => Simbolo::Piu,
            '-' => Simbolo::Meno,
            '=' => Simbolo::Uguale,
            '!' if c.get(i + 1) == Some(&'=') => {
                i += 1;
                Simbolo::Diverso
            }
            '<' | '>' => {
                let uguale = c.get(i + 1) == Some(&'=');
                if uguale {
                    i += 1;
                }
                match (ch, uguale) {
                    ('<', false) => Simbolo::Minore,
                    ('<', true) => Simbolo::MinoreUguale,
                    ('>', false) => Simbolo::Maggiore,
                    _ => Simbolo::MaggioreUguale,
                }
            }
            '/' => {
                if c.get(i + 1) == Some(&'/') {
                    i += 1;
                    Simbolo::DoppiaBarra
                } else {
                    Simbolo::Barra
                }
            }
            ':' if c.get(i + 1) == Some(&':') => {
                i += 1;
                Simbolo::DoppiDuePunti
            }
            '*' => {
                if operando {
                    Simbolo::Asterisco
                } else {
                    Simbolo::Per
                }
            }
            '"' | '\'' => {
                let fine = c[i + 1..]
                    .iter()
                    .position(|&x| x == ch)
                    .ok_or_else(|| errore("stringa non chiusa"))?;
                let s: String = c[i + 1..i + 1 + fine].iter().collect();
                i += fine + 1;
                Simbolo::Letterale(s)
            }
            '$' => {
                let inizio = i + 1;
                let mut j = inizio;
                while j < c.len() && (parte_nome(c[j]) || c[j] == ':') {
                    j += 1;
                }
                if j == inizio {
                    return Err(errore("nome di variabile mancante"));
                }
                let nome: String = c[inizio..j].iter().collect();
                i = j - 1;
                Simbolo::Variabile(nome)
            }
            '.' => {
                if c.get(i + 1) == Some(&'.') {
                    i += 1;
                    Simbolo::DuePunti
                } else if c.get(i + 1).is_some_and(|x| x.is_ascii_digit()) {
                    let inizio = i;
                    i += 1;
                    while i < c.len() && c[i].is_ascii_digit() {
                        i += 1;
                    }
                    let s: String = c[inizio..i].iter().collect();
                    i -= 1;
                    Simbolo::Numero(s.parse().map_err(|_| errore("numero non valido"))?)
                } else {
                    Simbolo::Punto
                }
            }
            d if d.is_ascii_digit() => {
                let inizio = i;
                while i < c.len() && c[i].is_ascii_digit() {
                    i += 1;
                }
                if i < c.len() && c[i] == '.' {
                    i += 1;
                    while i < c.len() && c[i].is_ascii_digit() {
                        i += 1;
                    }
                }
                let s: String = c[inizio..i].iter().collect();
                i -= 1;
                Simbolo::Numero(s.parse().map_err(|_| errore("numero non valido"))?)
            }
            n if inizio_nome(n) => {
                let inizio = i;
                while i < c.len() && parte_nome(c[i]) {
                    i += 1;
                }
                let mut nome: String = c[inizio..i].iter().collect();
                if !operando {
                    // Nomi di operatore.
                    let s = match nome.as_str() {
                        "and" => Simbolo::E,
                        "or" => Simbolo::O,
                        "mod" => Simbolo::Modulo,
                        "div" => Simbolo::Diviso,
                        _ => return Err(errore("operatore atteso")),
                    };
                    out.push(s);
                    continue;
                }
                // Nome qualificato `prefisso:locale` o `prefisso:*`.
                if i + 1 < c.len() && c[i] == ':' && c[i + 1] != ':' {
                    if c[i + 1] == '*' {
                        nome.push_str(":*");
                        i += 2;
                    } else if inizio_nome(c[i + 1]) {
                        let inizio2 = i + 1;
                        i += 1;
                        while i < c.len() && parte_nome(c[i]) {
                            i += 1;
                        }
                        nome.push(':');
                        nome.extend(&c[inizio2..i]);
                    }
                }
                let j = prossimo(i);
                let s = if c.get(j) == Some(&'(') {
                    match nome.as_str() {
                        "text" | "node" | "comment" | "processing-instruction" => {
                            Simbolo::TipoNodo(nome)
                        }
                        _ => Simbolo::Funzione(nome),
                    }
                } else if c.get(j) == Some(&':') && c.get(j + 1) == Some(&':') {
                    Simbolo::NomeAsse(nome)
                } else {
                    Simbolo::Nome(nome)
                };
                out.push(s);
                continue;
            }
            _ => return Err(errore(&format!("carattere inatteso '{ch}'"))),
        };
        out.push(s);
        i += 1;
    }
    Ok(out)
}

/// Risolve un prefisso nello spazio dei nomi corrispondente.
pub type Risolutore<'r> = &'r dyn Fn(&str) -> Option<String>;

struct Analizzatore<'r> {
    s: Vec<Simbolo>,
    i: usize,
    ns: Risolutore<'r>,
    testo: String,
}

/// Analizza un'espressione XPath.
pub fn analizza(testo: &str, ns: Risolutore<'_>) -> Result<Espr, Errore> {
    let mut a = Analizzatore {
        s: simboli(testo)?,
        i: 0,
        ns,
        testo: testo.to_string(),
    };
    let e = a.o()?;
    if a.i < a.s.len() {
        return Err(a.errore("simbolo inatteso"));
    }
    Ok(e)
}

impl Analizzatore<'_> {
    fn errore(&self, m: &str) -> Errore {
        Errore::XPath(format!("{m} in \"{}\"", self.testo))
    }

    fn guarda(&self) -> Option<&Simbolo> {
        self.s.get(self.i)
    }

    fn accetta(&mut self, s: &Simbolo) -> bool {
        if self.guarda() == Some(s) {
            self.i += 1;
            true
        } else {
            false
        }
    }

    fn atteso(&mut self, s: &Simbolo) -> Result<(), Errore> {
        if self.accetta(s) {
            Ok(())
        } else {
            Err(self.errore(&format!("atteso {s:?}")))
        }
    }

    fn binaria(
        &mut self,
        prossimo: fn(&mut Self) -> Result<Espr, Errore>,
        ops: &[(Simbolo, Operatore)],
    ) -> Result<Espr, Errore> {
        let mut sx = prossimo(self)?;
        'esterno: loop {
            for (s, op) in ops {
                if self.accetta(s) {
                    let dx = prossimo(self)?;
                    sx = Espr::Binaria(*op, Box::new(sx), Box::new(dx));
                    continue 'esterno;
                }
            }
            return Ok(sx);
        }
    }

    fn o(&mut self) -> Result<Espr, Errore> {
        self.binaria(Self::e, &[(Simbolo::O, Operatore::O)])
    }

    fn e(&mut self) -> Result<Espr, Errore> {
        self.binaria(Self::uguaglianza, &[(Simbolo::E, Operatore::E)])
    }

    fn uguaglianza(&mut self) -> Result<Espr, Errore> {
        self.binaria(
            Self::relazionale,
            &[
                (Simbolo::Uguale, Operatore::Uguale),
                (Simbolo::Diverso, Operatore::Diverso),
            ],
        )
    }

    fn relazionale(&mut self) -> Result<Espr, Errore> {
        self.binaria(
            Self::additiva,
            &[
                (Simbolo::MinoreUguale, Operatore::MinoreUguale),
                (Simbolo::Minore, Operatore::Minore),
                (Simbolo::MaggioreUguale, Operatore::MaggioreUguale),
                (Simbolo::Maggiore, Operatore::Maggiore),
            ],
        )
    }

    fn additiva(&mut self) -> Result<Espr, Errore> {
        self.binaria(
            Self::moltiplicativa,
            &[
                (Simbolo::Piu, Operatore::Piu),
                (Simbolo::Meno, Operatore::Meno),
            ],
        )
    }

    fn moltiplicativa(&mut self) -> Result<Espr, Errore> {
        self.binaria(
            Self::unaria,
            &[
                (Simbolo::Per, Operatore::Per),
                (Simbolo::Diviso, Operatore::Diviso),
                (Simbolo::Modulo, Operatore::Modulo),
            ],
        )
    }

    fn unaria(&mut self) -> Result<Espr, Errore> {
        if self.accetta(&Simbolo::Meno) {
            return Ok(Espr::Negazione(Box::new(self.unaria()?)));
        }
        let mut sx = self.percorso()?;
        while self.accetta(&Simbolo::Pipe) {
            let dx = self.percorso()?;
            sx = Espr::Unione(Box::new(sx), Box::new(dx));
        }
        Ok(sx)
    }

    fn percorso(&mut self) -> Result<Espr, Errore> {
        match self.guarda() {
            Some(
                Simbolo::Variabile(_)
                | Simbolo::ParAperta
                | Simbolo::Letterale(_)
                | Simbolo::Numero(_)
                | Simbolo::Funzione(_),
            ) => {
                let primaria = self.primaria()?;
                let predicati = self.predicati()?;
                let filtro = if predicati.is_empty() {
                    primaria
                } else {
                    Espr::Filtro(Box::new(primaria), predicati)
                };
                let mut passi = Vec::new();
                if self.continua_percorso(&mut passi)? {
                    Ok(Espr::Percorso(Base::Filtro(Box::new(filtro)), passi))
                } else {
                    Ok(filtro)
                }
            }
            Some(Simbolo::Barra) => {
                self.i += 1;
                let mut passi = Vec::new();
                if self.inizia_passo() {
                    passi.push(self.passo()?);
                    self.continua_percorso(&mut passi)?;
                }
                Ok(Espr::Percorso(Base::Radice, passi))
            }
            Some(Simbolo::DoppiaBarra) => {
                self.i += 1;
                let mut passi = vec![discendenti()];
                passi.push(self.passo()?);
                self.continua_percorso(&mut passi)?;
                Ok(Espr::Percorso(Base::Radice, passi))
            }
            _ => {
                let mut passi = vec![self.passo()?];
                self.continua_percorso(&mut passi)?;
                Ok(Espr::Percorso(Base::Contesto, passi))
            }
        }
    }

    fn inizia_passo(&self) -> bool {
        matches!(
            self.guarda(),
            Some(
                Simbolo::Nome(_)
                    | Simbolo::Asterisco
                    | Simbolo::Chiocciola
                    | Simbolo::Punto
                    | Simbolo::DuePunti
                    | Simbolo::NomeAsse(_)
                    | Simbolo::TipoNodo(_)
            )
        )
    }

    /// Legge i passi successivi separati da `/` o `//`.
    fn continua_percorso(&mut self, passi: &mut Vec<Passo>) -> Result<bool, Errore> {
        let mut trovato = false;
        loop {
            if self.accetta(&Simbolo::Barra) {
                passi.push(self.passo()?);
            } else if self.accetta(&Simbolo::DoppiaBarra) {
                passi.push(discendenti());
                passi.push(self.passo()?);
            } else {
                return Ok(trovato);
            }
            trovato = true;
        }
    }

    fn passo(&mut self) -> Result<Passo, Errore> {
        if self.accetta(&Simbolo::Punto) {
            return Ok(Passo {
                asse: Asse::Se,
                test: Test::Nodo,
                predicati: Vec::new(),
            });
        }
        if self.accetta(&Simbolo::DuePunti) {
            return Ok(Passo {
                asse: Asse::Genitore,
                test: Test::Nodo,
                predicati: Vec::new(),
            });
        }
        let asse = if self.accetta(&Simbolo::Chiocciola) {
            Asse::Attributo
        } else if let Some(Simbolo::NomeAsse(n)) = self.guarda() {
            let a = Asse::da_nome(n).ok_or_else(|| self.errore("asse sconosciuto"))?;
            self.i += 1;
            self.atteso(&Simbolo::DoppiDuePunti)?;
            a
        } else {
            Asse::Figlio
        };
        let test = match self.guarda().cloned() {
            Some(Simbolo::Asterisco) => {
                self.i += 1;
                Test::Qualsiasi
            }
            Some(Simbolo::Nome(n)) => {
                self.i += 1;
                match n.split_once(':') {
                    Some((p, "*")) => Test::SpazioQualsiasi(self.risolvi(p)?),
                    Some((p, l)) => Test::Nome(Some(self.risolvi(p)?), l.to_string()),
                    None => Test::Nome(None, n),
                }
            }
            Some(Simbolo::TipoNodo(t)) => {
                self.i += 1;
                self.atteso(&Simbolo::ParAperta)?;
                let test = match t.as_str() {
                    "text" => Test::Testo,
                    "comment" => Test::Commento,
                    "node" => Test::Nodo,
                    _ => {
                        let obiettivo = match self.guarda() {
                            Some(Simbolo::Letterale(s)) => Some(s.clone()),
                            _ => None,
                        };
                        if obiettivo.is_some() {
                            self.i += 1;
                        }
                        Test::Istruzione(obiettivo)
                    }
                };
                self.atteso(&Simbolo::ParChiusa)?;
                test
            }
            _ => return Err(self.errore("passo atteso")),
        };
        let predicati = self.predicati()?;
        Ok(Passo {
            asse,
            test,
            predicati,
        })
    }

    fn risolvi(&self, prefisso: &str) -> Result<String, Errore> {
        (self.ns)(prefisso)
            .ok_or_else(|| self.errore(&format!("prefisso '{prefisso}' non dichiarato")))
    }

    fn predicati(&mut self) -> Result<Vec<Espr>, Errore> {
        let mut v = Vec::new();
        while self.accetta(&Simbolo::QuadraAperta) {
            v.push(self.o()?);
            self.atteso(&Simbolo::QuadraChiusa)?;
        }
        Ok(v)
    }

    fn primaria(&mut self) -> Result<Espr, Errore> {
        let s = self
            .guarda()
            .cloned()
            .ok_or_else(|| self.errore("espressione incompleta"))?;
        self.i += 1;
        match s {
            Simbolo::Variabile(n) => Ok(Espr::Variabile(n)),
            Simbolo::Letterale(t) => Ok(Espr::Letterale(t)),
            Simbolo::Numero(n) => Ok(Espr::Numero(n)),
            Simbolo::ParAperta => {
                let e = self.o()?;
                self.atteso(&Simbolo::ParChiusa)?;
                Ok(e)
            }
            Simbolo::Funzione(nome) => {
                self.atteso(&Simbolo::ParAperta)?;
                let mut argomenti = Vec::new();
                if !self.accetta(&Simbolo::ParChiusa) {
                    loop {
                        argomenti.push(self.o()?);
                        if self.accetta(&Simbolo::ParChiusa) {
                            break;
                        }
                        self.atteso(&Simbolo::Virgola)?;
                    }
                }
                Ok(Espr::Funzione(nome, argomenti))
            }
            _ => Err(self.errore("espressione primaria attesa")),
        }
    }
}

/// Passo implicito di `//`.
fn discendenti() -> Passo {
    Passo {
        asse: Asse::DiscendenteOSe,
        test: Test::Nodo,
        predicati: Vec::new(),
    }
}

#[cfg(test)]
mod test {
    use super::*;

    fn nessuno(_: &str) -> Option<String> {
        None
    }

    #[test]
    fn distingue_operatori_e_nomi() {
        let e = analizza("div div div", &nessuno).unwrap();
        assert!(matches!(e, Espr::Binaria(Operatore::Diviso, _, _)));
        let e = analizza("* * 2", &nessuno).unwrap();
        assert!(matches!(e, Espr::Binaria(Operatore::Per, _, _)));
        let e = analizza("position()-1", &nessuno).unwrap();
        assert!(matches!(e, Espr::Binaria(Operatore::Meno, _, _)));
    }

    #[test]
    fn risolve_prefissi() {
        let ns = |p: &str| (p == "a").then(|| "urn:a".to_string());
        let e = analizza("a:Radice/Figlio[@x='1']", &ns).unwrap();
        let Espr::Percorso(Base::Contesto, passi) = e else {
            panic!()
        };
        assert_eq!(
            passi[0].test,
            Test::Nome(Some("urn:a".into()), "Radice".into())
        );
        assert!(analizza("z:Radice", &ns).is_err());
    }
}
