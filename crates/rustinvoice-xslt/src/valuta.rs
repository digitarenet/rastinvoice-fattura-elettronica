// Copyright (c) 2026 Andrea Re - info@digitare.net
// Autore: Andrea Re @ 2026 - info@digitare.net
//
// Valutazione delle espressioni XPath 1.0 sul documento sorgente.

use std::cmp::Ordering;
use std::collections::HashMap;

use roxmltree::NodeType;

use crate::formato::{formatta, FormatoDecimale};
use crate::valore::{confronta_numeri, ordina, Nodo, Valore};
use crate::xpath::{Asse, Base, Espr, Operatore, Passo, Test};
use crate::Errore;

/// Contesto di valutazione.
#[derive(Clone, Copy)]
pub struct Contesto<'a, 'i> {
    pub nodo: Nodo<'a, 'i>,
    pub posizione: usize,
    pub dimensione: usize,
    /// Nodo corrente XSLT, restituito da current().
    pub corrente: Nodo<'a, 'i>,
}

/// Variabili visibili e formati decimali.
pub struct Ambiente<'v, 'a, 'i> {
    pub locali: &'v [(String, Valore<'a, 'i>)],
    pub globali: &'v [(String, Valore<'a, 'i>)],
    pub formati: &'v HashMap<String, FormatoDecimale>,
}

impl<'a, 'i> Ambiente<'_, 'a, 'i> {
    fn variabile(&self, nome: &str) -> Option<&Valore<'a, 'i>> {
        self.locali
            .iter()
            .rev()
            .chain(self.globali.iter().rev())
            .find(|(n, _)| n == nome)
            .map(|(_, v)| v)
    }
}

type Esito<'a, 'i> = Result<Valore<'a, 'i>, Errore>;

pub fn valuta<'a, 'i>(e: &Espr, c: &Contesto<'a, 'i>, amb: &Ambiente<'_, 'a, 'i>) -> Esito<'a, 'i> {
    Ok(match e {
        Espr::Letterale(s) => Valore::Stringa(s.clone()),
        Espr::Numero(n) => Valore::Numero(*n),
        Espr::Variabile(n) => amb
            .variabile(n)
            .cloned()
            .ok_or_else(|| Errore::XPath(format!("variabile ${n} non definita")))?,
        Espr::Negazione(x) => Valore::Numero(-valuta(x, c, amb)?.numero()),
        Espr::Binaria(Operatore::O, a, b) => {
            Valore::Booleano(valuta(a, c, amb)?.booleano() || valuta(b, c, amb)?.booleano())
        }
        Espr::Binaria(Operatore::E, a, b) => {
            Valore::Booleano(valuta(a, c, amb)?.booleano() && valuta(b, c, amb)?.booleano())
        }
        Espr::Binaria(op, a, b) => {
            let x = valuta(a, c, amb)?;
            let y = valuta(b, c, amb)?;
            match op {
                Operatore::Piu => Valore::Numero(x.numero() + y.numero()),
                Operatore::Meno => Valore::Numero(x.numero() - y.numero()),
                Operatore::Per => Valore::Numero(x.numero() * y.numero()),
                Operatore::Diviso => Valore::Numero(x.numero() / y.numero()),
                Operatore::Modulo => Valore::Numero(x.numero() % y.numero()),
                _ => Valore::Booleano(confronta(*op, x, y)),
            }
        }
        Espr::Unione(a, b) => {
            let mut x = insieme(valuta(a, c, amb)?)?;
            x.extend(insieme(valuta(b, c, amb)?)?);
            ordina(&mut x);
            Valore::Nodi(x)
        }
        Espr::Filtro(base, predicati) => {
            let mut nodi = insieme(valuta(base, c, amb)?)?;
            for p in predicati {
                nodi = filtra(nodi, p, c, amb)?;
            }
            Valore::Nodi(nodi)
        }
        Espr::Percorso(base, passi) => {
            let inizio = match base {
                Base::Radice => vec![c.nodo.radice()],
                Base::Contesto => vec![c.nodo],
                Base::Filtro(e) => insieme(valuta(e, c, amb)?)?,
            };
            Valore::Nodi(percorri(inizio, passi, c, amb)?)
        }
        Espr::Funzione(nome, argomenti) => funzione(nome, argomenti, c, amb)?,
    })
}

pub fn insieme<'a, 'i>(v: Valore<'a, 'i>) -> Result<Vec<Nodo<'a, 'i>>, Errore> {
    match v {
        Valore::Nodi(n) => Ok(n),
        _ => Err(Errore::XPath("era atteso un insieme di nodi".into())),
    }
}

pub fn percorri<'a, 'i>(
    mut nodi: Vec<Nodo<'a, 'i>>,
    passi: &[Passo],
    c: &Contesto<'a, 'i>,
    amb: &Ambiente<'_, 'a, 'i>,
) -> Result<Vec<Nodo<'a, 'i>>, Errore> {
    for passo in passi {
        let mut risultato = Vec::new();
        for n in &nodi {
            let mut candidati: Vec<Nodo> = lungo(*n, passo.asse)
                .into_iter()
                .filter(|x| soddisfa(x, passo.asse, &passo.test))
                .collect();
            for p in &passo.predicati {
                candidati = filtra(candidati, p, c, amb)?;
            }
            risultato.extend(candidati);
        }
        ordina(&mut risultato);
        nodi = risultato;
    }
    Ok(nodi)
}

/// Applica un predicato a un elenco di nodi nell'ordine dato.
fn filtra<'a, 'i>(
    nodi: Vec<Nodo<'a, 'i>>,
    p: &Espr,
    c: &Contesto<'a, 'i>,
    amb: &Ambiente<'_, 'a, 'i>,
) -> Result<Vec<Nodo<'a, 'i>>, Errore> {
    let dimensione = nodi.len();
    let mut tenuti = Vec::new();
    for (i, n) in nodi.into_iter().enumerate() {
        let interno = Contesto {
            nodo: n,
            posizione: i + 1,
            dimensione,
            corrente: c.corrente,
        };
        let tieni = match valuta(p, &interno, amb)? {
            Valore::Numero(x) => x == (i + 1) as f64,
            v => v.booleano(),
        };
        if tieni {
            tenuti.push(n);
        }
    }
    Ok(tenuti)
}

/// Nodi raggiunti lungo un asse, nell'ordine dell'asse.
fn lungo<'a, 'i>(n: Nodo<'a, 'i>, asse: Asse) -> Vec<Nodo<'a, 'i>> {
    use Nodo::Normale as N;
    match (n, asse) {
        (_, Asse::Se) => vec![n],
        (_, Asse::Genitore) => n.genitore().into_iter().collect(),
        (_, Asse::Antenato | Asse::AntenatoOSe) => {
            let mut v = Vec::new();
            if asse == Asse::AntenatoOSe {
                v.push(n);
            }
            let mut g = n.genitore();
            while let Some(x) = g {
                v.push(x);
                g = x.genitore();
            }
            v
        }
        (_, Asse::Spazio) => Vec::new(),
        (N(e), Asse::Attributo) => (0..e.attributes().len())
            .map(|i| Nodo::Attributo(e, i))
            .collect(),
        (N(e), Asse::Figlio) => e.children().map(N).collect(),
        (N(e), Asse::Discendente) => e.descendants().skip(1).map(N).collect(),
        (N(e), Asse::DiscendenteOSe) => e.descendants().map(N).collect(),
        (N(e), Asse::FratelloSeguente) => e.next_siblings().skip(1).map(N).collect(),
        (N(e), Asse::FratelloPrecedente) => e.prev_siblings().skip(1).map(N).collect(),
        (N(e), Asse::Seguente) => {
            let mut v = Vec::new();
            for a in e.ancestors() {
                for f in a.next_siblings().skip(1) {
                    v.extend(f.descendants().map(N));
                }
            }
            v
        }
        (N(e), Asse::Precedente) => {
            let antenati: Vec<_> = e.ancestors().map(|a| a.id()).collect();
            let radice = e.document().root();
            let mut v: Vec<Nodo> = radice
                .descendants()
                .take_while(|d| d.id() != e.id())
                .filter(|d| !antenati.contains(&d.id()))
                .map(N)
                .collect();
            v.reverse();
            v
        }
        (Nodo::Attributo(e, _), Asse::Seguente) => {
            let mut v: Vec<Nodo> = e.descendants().skip(1).map(N).collect();
            v.extend(lungo(N(e), Asse::Seguente));
            v
        }
        (Nodo::Attributo(e, _), Asse::Precedente) => lungo(N(e), Asse::Precedente),
        (Nodo::Attributo(..), _) => Vec::new(),
    }
}

fn soddisfa(n: &Nodo<'_, '_>, asse: Asse, t: &Test) -> bool {
    let principale = |n: &Nodo| match n {
        Nodo::Attributo(..) => asse == Asse::Attributo,
        Nodo::Normale(x) => x.is_element() && asse != Asse::Attributo,
    };
    match t {
        Test::Nodo => true,
        Test::Testo => matches!(n, Nodo::Normale(x) if x.is_text()),
        Test::Commento => matches!(n, Nodo::Normale(x) if x.is_comment()),
        Test::Istruzione(obiettivo) => match n {
            Nodo::Normale(x) if x.node_type() == NodeType::PI => obiettivo
                .as_ref()
                .map_or(true, |o| x.pi().is_some_and(|p| p.target == o)),
            _ => false,
        },
        Test::Qualsiasi => principale(n),
        Test::SpazioQualsiasi(ns) => {
            principale(n)
                && n.nome_espanso()
                    .is_some_and(|(u, _)| u == Some(ns.as_str()))
        }
        Test::Nome(ns, locale) => {
            principale(n)
                && n.nome_espanso()
                    .is_some_and(|(u, l)| l == locale && u == ns.as_deref())
        }
    }
}

fn senza_frammento<'a, 'i>(v: Valore<'a, 'i>) -> Valore<'a, 'i> {
    match v {
        Valore::Frammento(f) => Valore::Stringa(f.testo()),
        altro => altro,
    }
}

fn inverti(op: Operatore) -> Operatore {
    match op {
        Operatore::Minore => Operatore::Maggiore,
        Operatore::MinoreUguale => Operatore::MaggioreUguale,
        Operatore::Maggiore => Operatore::Minore,
        Operatore::MaggioreUguale => Operatore::MinoreUguale,
        altro => altro,
    }
}

fn confronto_numerico(op: Operatore, a: f64, b: f64) -> bool {
    let o = confronta_numeri(a, b);
    match op {
        Operatore::Uguale => o == Some(Ordering::Equal),
        Operatore::Diverso => a != b,
        Operatore::Minore => o == Some(Ordering::Less),
        Operatore::MinoreUguale => matches!(o, Some(Ordering::Less | Ordering::Equal)),
        Operatore::Maggiore => o == Some(Ordering::Greater),
        Operatore::MaggioreUguale => matches!(o, Some(Ordering::Greater | Ordering::Equal)),
        _ => false,
    }
}

fn uguaglianza(op: Operatore) -> bool {
    matches!(op, Operatore::Uguale | Operatore::Diverso)
}

fn confronto_stringhe(op: Operatore, a: &str, b: &str) -> bool {
    match op {
        Operatore::Uguale => a == b,
        Operatore::Diverso => a != b,
        _ => confronto_numerico(
            op,
            Valore::Stringa(a.into()).numero(),
            Valore::Stringa(b.into()).numero(),
        ),
    }
}

fn confronta(op: Operatore, a: Valore<'_, '_>, b: Valore<'_, '_>) -> bool {
    let a = senza_frammento(a);
    let b = senza_frammento(b);
    match (&a, &b) {
        (Valore::Nodi(x), Valore::Nodi(y)) => {
            let sy: Vec<String> = y.iter().map(|n| n.testo()).collect();
            x.iter().any(|n| {
                let s = n.testo();
                sy.iter().any(|t| confronto_stringhe(op, &s, t))
            })
        }
        (Valore::Nodi(_), Valore::Booleano(y)) => {
            if uguaglianza(op) {
                (a.booleano() == *y) == (op == Operatore::Uguale)
            } else {
                confronto_numerico(op, a.booleano() as u8 as f64, *y as u8 as f64)
            }
        }
        (Valore::Nodi(x), Valore::Numero(y)) => x
            .iter()
            .any(|n| confronto_numerico(op, Valore::Stringa(n.testo()).numero(), *y)),
        (Valore::Nodi(x), Valore::Stringa(y)) => {
            x.iter().any(|n| confronto_stringhe(op, &n.testo(), y))
        }
        (_, Valore::Nodi(_)) => confronta(inverti(op), b, a),
        _ => {
            if uguaglianza(op) {
                let uguali = if matches!(a, Valore::Booleano(_)) || matches!(b, Valore::Booleano(_))
                {
                    a.booleano() == b.booleano()
                } else if matches!(a, Valore::Numero(_)) || matches!(b, Valore::Numero(_)) {
                    a.numero() == b.numero()
                } else {
                    a.stringa() == b.stringa()
                };
                uguali == (op == Operatore::Uguale)
            } else {
                confronto_numerico(op, a.numero(), b.numero())
            }
        }
    }
}

/// Funzioni XPath e XSLT note, per function-available().
const FUNZIONI: &[&str] = &[
    "last",
    "position",
    "count",
    "local-name",
    "namespace-uri",
    "name",
    "string",
    "concat",
    "starts-with",
    "contains",
    "substring-before",
    "substring-after",
    "substring",
    "string-length",
    "normalize-space",
    "translate",
    "boolean",
    "not",
    "true",
    "false",
    "number",
    "sum",
    "floor",
    "ceiling",
    "round",
    "format-number",
    "current",
    "generate-id",
    "system-property",
    "function-available",
    "element-available",
];

fn funzione<'a, 'i>(
    nome: &str,
    arg: &[Espr],
    c: &Contesto<'a, 'i>,
    amb: &Ambiente<'_, 'a, 'i>,
) -> Esito<'a, 'i> {
    let n_arg = arg.len();
    let controlla = |min: usize, max: usize| -> Result<(), Errore> {
        if n_arg < min || n_arg > max {
            Err(Errore::XPath(format!(
                "numero di argomenti errato per {nome}()"
            )))
        } else {
            Ok(())
        }
    };
    let v = |i: usize| valuta(&arg[i], c, amb);
    let s = |i: usize| -> Result<String, Errore> { Ok(v(i)?.stringa()) };
    // Stringa dell'argomento o del nodo di contesto se assente.
    let s_o_contesto = || -> Result<String, Errore> {
        if n_arg == 0 {
            Ok(c.nodo.testo())
        } else {
            s(0)
        }
    };
    // Primo nodo dell'argomento o nodo di contesto.
    let nodo_arg = || -> Result<Option<Nodo<'a, 'i>>, Errore> {
        if n_arg == 0 {
            Ok(Some(c.nodo))
        } else {
            Ok(insieme(v(0)?)?.into_iter().next())
        }
    };
    Ok(match nome {
        "last" => Valore::Numero(c.dimensione as f64),
        "position" => Valore::Numero(c.posizione as f64),
        "count" => {
            controlla(1, 1)?;
            Valore::Numero(insieme(v(0)?)?.len() as f64)
        }
        "local-name" => {
            controlla(0, 1)?;
            Valore::Stringa(
                nodo_arg()?
                    .and_then(|n| n.nome_espanso())
                    .map(|(_, l)| l.to_string())
                    .unwrap_or_default(),
            )
        }
        "namespace-uri" => {
            controlla(0, 1)?;
            Valore::Stringa(
                nodo_arg()?
                    .and_then(|n| n.nome_espanso())
                    .and_then(|(u, _)| u)
                    .unwrap_or("")
                    .to_string(),
            )
        }
        "name" => {
            controlla(0, 1)?;
            Valore::Stringa(nodo_arg()?.map(|n| n.nome()).unwrap_or_default())
        }
        "string" => {
            controlla(0, 1)?;
            Valore::Stringa(s_o_contesto()?)
        }
        "concat" => {
            if n_arg < 2 {
                return Err(Errore::XPath(
                    "concat() richiede almeno due argomenti".into(),
                ));
            }
            let mut r = String::new();
            for i in 0..n_arg {
                r.push_str(&s(i)?);
            }
            Valore::Stringa(r)
        }
        "starts-with" => {
            controlla(2, 2)?;
            Valore::Booleano(s(0)?.starts_with(&s(1)?))
        }
        "contains" => {
            controlla(2, 2)?;
            Valore::Booleano(s(0)?.contains(&s(1)?))
        }
        "substring-before" => {
            controlla(2, 2)?;
            let (a, b) = (s(0)?, s(1)?);
            Valore::Stringa(a.find(&b).map(|p| a[..p].to_string()).unwrap_or_default())
        }
        "substring-after" => {
            controlla(2, 2)?;
            let (a, b) = (s(0)?, s(1)?);
            Valore::Stringa(
                a.find(&b)
                    .map(|p| a[p + b.len()..].to_string())
                    .unwrap_or_default(),
            )
        }
        "substring" => {
            controlla(2, 3)?;
            let testo = s(0)?;
            let inizio = rotonda(v(1)?.numero());
            let fine = if n_arg == 3 {
                inizio + rotonda(v(2)?.numero())
            } else {
                f64::INFINITY
            };
            // Posizioni da 1; i caratteri tenuti sono quelli con inizio <= p < fine.
            let r: String = testo
                .chars()
                .enumerate()
                .filter(|(i, _)| {
                    let p = (*i + 1) as f64;
                    p >= inizio && p < fine
                })
                .map(|(_, ch)| ch)
                .collect();
            Valore::Stringa(r)
        }
        "string-length" => {
            controlla(0, 1)?;
            Valore::Numero(s_o_contesto()?.chars().count() as f64)
        }
        "normalize-space" => {
            controlla(0, 1)?;
            let t = s_o_contesto()?;
            Valore::Stringa(
                t.split([' ', '\t', '\r', '\n'])
                    .filter(|x| !x.is_empty())
                    .collect::<Vec<_>>()
                    .join(" "),
            )
        }
        "translate" => {
            controlla(3, 3)?;
            let (t, da, a) = (s(0)?, s(1)?, s(2)?);
            let da: Vec<char> = da.chars().collect();
            let a: Vec<char> = a.chars().collect();
            let r: String = t
                .chars()
                .filter_map(|ch| match da.iter().position(|&x| x == ch) {
                    Some(p) => a.get(p).copied(),
                    None => Some(ch),
                })
                .collect();
            Valore::Stringa(r)
        }
        "boolean" => {
            controlla(1, 1)?;
            Valore::Booleano(v(0)?.booleano())
        }
        "not" => {
            controlla(1, 1)?;
            Valore::Booleano(!v(0)?.booleano())
        }
        "true" => Valore::Booleano(true),
        "false" => Valore::Booleano(false),
        "lang" => Valore::Booleano(false),
        "number" => {
            controlla(0, 1)?;
            if n_arg == 0 {
                Valore::Numero(Valore::Stringa(c.nodo.testo()).numero())
            } else {
                Valore::Numero(v(0)?.numero())
            }
        }
        "sum" => {
            controlla(1, 1)?;
            Valore::Numero(
                insieme(v(0)?)?
                    .iter()
                    .map(|n| Valore::Stringa(n.testo()).numero())
                    .sum(),
            )
        }
        "floor" => {
            controlla(1, 1)?;
            Valore::Numero(v(0)?.numero().floor())
        }
        "ceiling" => {
            controlla(1, 1)?;
            Valore::Numero(v(0)?.numero().ceil())
        }
        "round" => {
            controlla(1, 1)?;
            Valore::Numero(rotonda(v(0)?.numero()))
        }
        "format-number" => {
            controlla(2, 3)?;
            let n = v(0)?.numero();
            let modello = s(1)?;
            let nome_formato = if n_arg == 3 { s(2)? } else { String::new() };
            let predefinito = FormatoDecimale::default();
            let f = match amb.formati.get(&nome_formato) {
                Some(f) => f,
                None if nome_formato.is_empty() => &predefinito,
                None => {
                    return Err(Errore::XPath(format!(
                        "formato decimale '{nome_formato}' non dichiarato"
                    )))
                }
            };
            Valore::Stringa(formatta(n, &modello, f))
        }
        "current" => Valore::Nodi(vec![c.corrente]),
        "generate-id" => {
            controlla(0, 1)?;
            Valore::Stringa(
                nodo_arg()?
                    .map(|n| {
                        let (a, b) = n.ordine();
                        format!("id{a}x{b}")
                    })
                    .unwrap_or_default(),
            )
        }
        "system-property" => {
            controlla(1, 1)?;
            match s(0)?.rsplit(':').next().unwrap_or("") {
                "version" => Valore::Numero(1.0),
                "vendor" => Valore::Stringa("RustinVoice".into()),
                _ => Valore::Stringa(String::new()),
            }
        }
        "function-available" => {
            controlla(1, 1)?;
            Valore::Booleano(FUNZIONI.contains(&s(0)?.as_str()))
        }
        "element-available" => Valore::Booleano(false),
        _ => return Err(Errore::NonSupportato(format!("funzione {nome}()"))),
    })
}

/// Arrotondamento XPath: a metà verso +infinito.
fn rotonda(x: f64) -> f64 {
    if x.is_nan() || x.is_infinite() {
        x
    } else {
        (x + 0.5).floor()
    }
}
