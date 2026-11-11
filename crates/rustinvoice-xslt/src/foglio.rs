// Copyright (c) 2026 Andrea Re - info@digitare.net
// Autore: Andrea Re @ 2026 - info@digitare.net
//
// Compilazione del foglio di stile XSLT in una struttura di istruzioni.

use std::collections::HashMap;

use roxmltree::Node;

use crate::formato::FormatoDecimale;
use crate::xpath::{analizza, Asse, Base, Espr, Test};
use crate::Errore;

pub const NS_XSL: &str = "http://www.w3.org/1999/XSL/Transform";

/// Parte di un modello di valore di attributo (`testo {espressione}`).
#[derive(Debug, Clone)]
pub enum ParteAvt {
    Testo(String),
    Espr(Espr),
}

pub type Avt = Vec<ParteAvt>;

#[derive(Debug, Clone)]
pub struct Ordinamento {
    pub selezione: Espr,
    pub numerico: bool,
    pub discendente: bool,
}

/// Variabile, parametro o parametro passato.
#[derive(Debug, Clone)]
pub struct Variabile {
    pub nome: String,
    pub selezione: Option<Espr>,
    pub corpo: Vec<Istruzione>,
}

#[derive(Debug, Clone)]
pub enum Istruzione {
    Testo {
        testo: String,
        grezzo: bool,
    },
    Letterale {
        nome: String,
        attributi: Vec<(String, Avt)>,
        corpo: Vec<Istruzione>,
    },
    ValoreDi {
        selezione: Espr,
        grezzo: bool,
    },
    Se {
        test: Espr,
        corpo: Vec<Istruzione>,
    },
    Scegli {
        rami: Vec<(Espr, Vec<Istruzione>)>,
        altrimenti: Vec<Istruzione>,
    },
    PerOgni {
        selezione: Espr,
        ordinamento: Vec<Ordinamento>,
        corpo: Vec<Istruzione>,
    },
    ApplicaModelli {
        selezione: Option<Espr>,
        modo: Option<String>,
        parametri: Vec<Variabile>,
        ordinamento: Vec<Ordinamento>,
    },
    ChiamaModello {
        nome: String,
        parametri: Vec<Variabile>,
    },
    Variabile(Variabile),
    Attributo {
        nome: Avt,
        corpo: Vec<Istruzione>,
    },
    Elemento {
        nome: Avt,
        corpo: Vec<Istruzione>,
    },
    CopiaDi(Espr),
    Copia(Vec<Istruzione>),
    Commento(Vec<Istruzione>),
    Messaggio {
        corpo: Vec<Istruzione>,
        termina: bool,
    },
}

pub struct CorpoModello {
    pub parametri: Vec<Variabile>,
    pub istruzioni: Vec<Istruzione>,
}

/// Regola di corrispondenza: una alternativa del pattern di un modello.
pub struct Regola {
    pub pattern: Espr,
    pub priorita: f64,
    pub modo: Option<String>,
    pub corpo: usize,
    pub ordine: usize,
}

/// Foglio di stile compilato, riutilizzabile per più documenti.
pub struct Foglio {
    pub corpi: Vec<CorpoModello>,
    pub regole: Vec<Regola>,
    pub nominati: HashMap<String, usize>,
    pub globali: Vec<Variabile>,
    pub formati: HashMap<String, FormatoDecimale>,
    pub metodo: Option<String>,
}

fn xsl(n: &Node, locale: &str) -> bool {
    n.is_element() && n.tag_name().namespace() == Some(NS_XSL) && n.tag_name().name() == locale
}

fn obbligatorio<'a>(n: &Node<'a, '_>, attributo: &str) -> Result<&'a str, Errore> {
    n.attribute(attributo).ok_or_else(|| {
        Errore::Xslt(format!(
            "attributo '{attributo}' mancante in xsl:{}",
            n.tag_name().name()
        ))
    })
}

fn espressione(n: &Node, testo: &str) -> Result<Espr, Errore> {
    let risolvi = |p: &str| n.lookup_namespace_uri(Some(p)).map(str::to_string);
    analizza(testo, &risolvi)
}

fn avt(n: &Node, testo: &str) -> Result<Avt, Errore> {
    let mut parti = Vec::new();
    let mut corrente = String::new();
    let mut c = testo.chars().peekable();
    while let Some(ch) = c.next() {
        match ch {
            '{' if c.peek() == Some(&'{') => {
                c.next();
                corrente.push('{');
            }
            '}' if c.peek() == Some(&'}') => {
                c.next();
                corrente.push('}');
            }
            '{' => {
                if !corrente.is_empty() {
                    parti.push(ParteAvt::Testo(std::mem::take(&mut corrente)));
                }
                let mut e = String::new();
                let mut chiuso = false;
                let mut virgolette: Option<char> = None;
                for x in c.by_ref() {
                    match virgolette {
                        Some(q) if x == q => virgolette = None,
                        Some(_) => {}
                        None if x == '"' || x == '\'' => virgolette = Some(x),
                        None if x == '}' => {
                            chiuso = true;
                            break;
                        }
                        None => {}
                    }
                    e.push(x);
                }
                if !chiuso {
                    return Err(Errore::Xslt(format!(
                        "parentesi graffa non chiusa in \"{testo}\""
                    )));
                }
                parti.push(ParteAvt::Espr(espressione(n, &e)?));
            }
            altro => corrente.push(altro),
        }
    }
    if !corrente.is_empty() {
        parti.push(ParteAvt::Testo(corrente));
    }
    Ok(parti)
}

/// Nome qualificato di un elemento o attributo del foglio, con prefisso.
fn nome_qualificato(n: &Node, ns: Option<&str>, locale: &str) -> String {
    match ns.and_then(|u| n.lookup_prefix(u)) {
        Some(p) if !p.is_empty() => format!("{p}:{locale}"),
        _ => locale.to_string(),
    }
}

impl Foglio {
    pub fn compila(testo: &str) -> Result<Foglio, Errore> {
        let opzioni = roxmltree::ParsingOptions {
            allow_dtd: true,
            ..Default::default()
        };
        let doc = roxmltree::Document::parse_with_options(testo, opzioni)
            .map_err(|e| Errore::Xml(format!("foglio di stile: {e}")))?;
        let radice = doc.root_element();
        if !(xsl(&radice, "stylesheet") || xsl(&radice, "transform")) {
            return Err(Errore::NonSupportato("fogli di stile semplificati".into()));
        }
        let mut f = Foglio {
            corpi: Vec::new(),
            regole: Vec::new(),
            nominati: HashMap::new(),
            globali: Vec::new(),
            formati: HashMap::new(),
            metodo: None,
        };
        for n in radice.children().filter(|n| n.is_element()) {
            if n.tag_name().namespace() != Some(NS_XSL) {
                continue;
            }
            match n.tag_name().name() {
                "template" => f.modello(&n)?,
                "variable" | "param" => f.globali.push(variabile(&n)?),
                "output" => f.metodo = n.attribute("method").map(str::to_string),
                "decimal-format" => {
                    let nome = n.attribute("name").unwrap_or("").to_string();
                    f.formati.insert(nome, formato_decimale(&n));
                }
                "preserve-space" | "namespace-alias" => {}
                altro => return Err(Errore::NonSupportato(format!("xsl:{altro}"))),
            }
        }
        // Prima le regole con priorità più alta; a parità, l'ultima definita.
        f.regole.sort_by(|a, b| {
            b.priorita
                .partial_cmp(&a.priorita)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then(b.ordine.cmp(&a.ordine))
        });
        Ok(f)
    }

    fn modello(&mut self, n: &Node) -> Result<(), Errore> {
        let mut parametri = Vec::new();
        let mut figli = Vec::new();
        for x in n.children() {
            if xsl(&x, "param") {
                parametri.push(variabile(&x)?);
            } else {
                figli.push(x);
            }
        }
        let istruzioni = istruzioni(figli.into_iter(), false)?;
        let indice = self.corpi.len();
        self.corpi.push(CorpoModello {
            parametri,
            istruzioni,
        });
        if let Some(nome) = n.attribute("name") {
            self.nominati.insert(nome.to_string(), indice);
        }
        if let Some(m) = n.attribute("match") {
            let priorita_esplicita = n
                .attribute("priority")
                .and_then(|p| p.trim().parse::<f64>().ok());
            let mut alternative = Vec::new();
            separa_unione(espressione(n, m)?, &mut alternative);
            for pattern in alternative {
                let ordine = self.regole.len();
                self.regole.push(Regola {
                    priorita: priorita_esplicita.unwrap_or_else(|| priorita_predefinita(&pattern)),
                    pattern,
                    modo: n.attribute("mode").map(str::to_string),
                    corpo: indice,
                    ordine,
                });
            }
        }
        Ok(())
    }
}

fn separa_unione(e: Espr, out: &mut Vec<Espr>) {
    match e {
        Espr::Unione(a, b) => {
            separa_unione(*a, out);
            separa_unione(*b, out);
        }
        altro => out.push(altro),
    }
}

fn priorita_predefinita(p: &Espr) -> f64 {
    match p {
        Espr::Percorso(Base::Contesto, passi)
            if passi.len() == 1
                && passi[0].predicati.is_empty()
                && matches!(passi[0].asse, Asse::Figlio | Asse::Attributo) =>
        {
            match passi[0].test {
                Test::Nome(..) | Test::Istruzione(Some(_)) => 0.0,
                Test::SpazioQualsiasi(_) => -0.25,
                _ => -0.5,
            }
        }
        _ => 0.5,
    }
}

fn formato_decimale(n: &Node) -> FormatoDecimale {
    let mut f = FormatoDecimale::default();
    let carattere = |a: &str, predefinito: char| {
        n.attribute(a)
            .and_then(|v| v.chars().next())
            .unwrap_or(predefinito)
    };
    f.decimale = carattere("decimal-separator", f.decimale);
    f.migliaia = carattere("grouping-separator", f.migliaia);
    f.meno = carattere("minus-sign", f.meno);
    f.percento = carattere("percent", f.percento);
    f.permille = carattere("per-mille", f.permille);
    f.zero = carattere("zero-digit", f.zero);
    f.cifra = carattere("digit", f.cifra);
    f.separatore = carattere("pattern-separator", f.separatore);
    if let Some(v) = n.attribute("infinity") {
        f.infinito = v.to_string();
    }
    if let Some(v) = n.attribute("NaN") {
        f.nan = v.to_string();
    }
    f
}

fn variabile(n: &Node) -> Result<Variabile, Errore> {
    Ok(Variabile {
        nome: obbligatorio(n, "name")?.to_string(),
        selezione: n
            .attribute("select")
            .map(|s| espressione(n, s))
            .transpose()?,
        corpo: istruzioni(n.children(), false)?,
    })
}

fn ordinamenti(n: &Node) -> Result<Vec<Ordinamento>, Errore> {
    n.children()
        .filter(|x| xsl(x, "sort"))
        .map(|x| {
            Ok(Ordinamento {
                selezione: espressione(&x, x.attribute("select").unwrap_or("."))?,
                numerico: x.attribute("data-type") == Some("number"),
                discendente: x.attribute("order") == Some("descending"),
            })
        })
        .collect()
}

fn parametri(n: &Node) -> Result<Vec<Variabile>, Errore> {
    n.children()
        .filter(|x| xsl(x, "with-param"))
        .map(|x| variabile(&x))
        .collect()
}

fn grezzo(n: &Node) -> bool {
    n.attribute("disable-output-escaping") == Some("yes")
}

/// Compila una sequenza di nodi del foglio. Gli spazi bianchi isolati
/// vengono scartati, tranne dentro xsl:text.
fn istruzioni<'a, 'i: 'a>(
    nodi: impl Iterator<Item = Node<'a, 'i>>,
    conserva_spazi: bool,
) -> Result<Vec<Istruzione>, Errore> {
    let nodi: Vec<Node> = nodi.collect();
    compila_nodi(&nodi, conserva_spazi)
}

fn compila_nodi(nodi: &[Node], conserva_spazi: bool) -> Result<Vec<Istruzione>, Errore> {
    let mut v = Vec::new();
    for &n in nodi {
        if n.is_text() {
            let t = n.text().unwrap_or("");
            if conserva_spazi || !t.chars().all(char::is_whitespace) {
                v.push(Istruzione::Testo {
                    testo: t.to_string(),
                    grezzo: false,
                });
            }
            continue;
        }
        if !n.is_element() {
            continue;
        }
        if n.tag_name().namespace() != Some(NS_XSL) {
            let mut attributi = Vec::new();
            for a in n.attributes() {
                if a.namespace() == Some(NS_XSL) {
                    if a.name() == "use-attribute-sets" {
                        return Err(Errore::NonSupportato("xsl:use-attribute-sets".into()));
                    }
                    continue;
                }
                attributi.push((
                    nome_qualificato(&n, a.namespace(), a.name()),
                    avt(&n, a.value())?,
                ));
            }
            v.push(Istruzione::Letterale {
                nome: nome_qualificato(&n, n.tag_name().namespace(), n.tag_name().name()),
                attributi,
                corpo: istruzioni(n.children(), false)?,
            });
            continue;
        }
        let i = match n.tag_name().name() {
            "text" => Istruzione::Testo {
                testo: n.text().unwrap_or("").to_string(),
                grezzo: grezzo(&n),
            },
            "value-of" => Istruzione::ValoreDi {
                selezione: espressione(&n, obbligatorio(&n, "select")?)?,
                grezzo: grezzo(&n),
            },
            "if" => Istruzione::Se {
                test: espressione(&n, obbligatorio(&n, "test")?)?,
                corpo: istruzioni(n.children(), false)?,
            },
            "choose" => {
                let mut rami = Vec::new();
                let mut altrimenti = Vec::new();
                for x in n.children().filter(|x| x.is_element()) {
                    if xsl(&x, "when") {
                        rami.push((
                            espressione(&x, obbligatorio(&x, "test")?)?,
                            istruzioni(x.children(), false)?,
                        ));
                    } else if xsl(&x, "otherwise") {
                        altrimenti = istruzioni(x.children(), false)?;
                    } else {
                        return Err(Errore::Xslt("elemento inatteso in xsl:choose".into()));
                    }
                }
                Istruzione::Scegli { rami, altrimenti }
            }
            "for-each" => Istruzione::PerOgni {
                selezione: espressione(&n, obbligatorio(&n, "select")?)?,
                ordinamento: ordinamenti(&n)?,
                corpo: compila_nodi(
                    &n.children().filter(|x| !xsl(x, "sort")).collect::<Vec<_>>(),
                    false,
                )?,
            },
            "apply-templates" => Istruzione::ApplicaModelli {
                selezione: n
                    .attribute("select")
                    .map(|s| espressione(&n, s))
                    .transpose()?,
                modo: n.attribute("mode").map(str::to_string),
                parametri: parametri(&n)?,
                ordinamento: ordinamenti(&n)?,
            },
            "call-template" => Istruzione::ChiamaModello {
                nome: obbligatorio(&n, "name")?.to_string(),
                parametri: parametri(&n)?,
            },
            "variable" | "param" => Istruzione::Variabile(variabile(&n)?),
            "attribute" => Istruzione::Attributo {
                nome: avt(&n, obbligatorio(&n, "name")?)?,
                corpo: istruzioni(n.children(), false)?,
            },
            "element" => Istruzione::Elemento {
                nome: avt(&n, obbligatorio(&n, "name")?)?,
                corpo: istruzioni(n.children(), false)?,
            },
            "copy-of" => Istruzione::CopiaDi(espressione(&n, obbligatorio(&n, "select")?)?),
            "copy" => Istruzione::Copia(istruzioni(n.children(), false)?),
            "comment" => Istruzione::Commento(istruzioni(n.children(), false)?),
            "message" => Istruzione::Messaggio {
                corpo: istruzioni(n.children(), false)?,
                termina: n.attribute("terminate") == Some("yes"),
            },
            "fallback" => continue,
            altro => return Err(Errore::NonSupportato(format!("xsl:{altro}"))),
        };
        v.push(i);
    }
    Ok(v)
}
