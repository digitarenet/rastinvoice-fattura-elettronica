// Copyright (c) 2026 Andrea Re - info@digitare.net
// Autore: Andrea Re @ 2026 - info@digitare.net
//
// Albero risultato della trasformazione e sua serializzazione in HTML o XML.

/// Nodo dell'albero risultato.
#[derive(Debug, Clone)]
pub enum NodoUscita {
    Elemento {
        nome: String,
        attributi: Vec<(String, String)>,
        figli: Vec<NodoUscita>,
    },
    /// Testo; `grezzo` disattiva l'escape (disable-output-escaping).
    Testo {
        testo: String,
        grezzo: bool,
    },
    Commento(String),
}

/// Frammento dell'albero risultato (contenuto di una variabile).
#[derive(Debug, Clone, Default)]
pub struct Frammento(pub Vec<NodoUscita>);

impl Frammento {
    pub fn testo(&self) -> String {
        let mut s = String::new();
        fn visita(n: &[NodoUscita], s: &mut String) {
            for x in n {
                match x {
                    NodoUscita::Elemento { figli, .. } => visita(figli, s),
                    NodoUscita::Testo { testo, .. } => s.push_str(testo),
                    NodoUscita::Commento(_) => {}
                }
            }
        }
        visita(&self.0, &mut s);
        s
    }
}

struct Cornice {
    nome: Option<String>,
    attributi: Vec<(String, String)>,
    figli: Vec<NodoUscita>,
}

/// Costruttore dell'albero risultato: una pila di elementi aperti.
pub struct Costruttore {
    pila: Vec<Cornice>,
    /// Numero di nodi prodotti, per porre un limite alle trasformazioni
    /// patologiche.
    pub nodi: usize,
}

impl Costruttore {
    pub fn nuovo() -> Self {
        Costruttore {
            pila: vec![Cornice {
                nome: None,
                attributi: Vec::new(),
                figli: Vec::new(),
            }],
            nodi: 0,
        }
    }

    fn cima(&mut self) -> &mut Cornice {
        self.pila.last_mut().expect("pila mai vuota")
    }

    pub fn testo(&mut self, t: &str, grezzo: bool) {
        if t.is_empty() {
            return;
        }
        self.nodi += 1;
        let figli = &mut self.cima().figli;
        if let Some(NodoUscita::Testo { testo, grezzo: g }) = figli.last_mut() {
            if *g == grezzo {
                testo.push_str(t);
                return;
            }
        }
        figli.push(NodoUscita::Testo {
            testo: t.to_string(),
            grezzo,
        });
    }

    pub fn commento(&mut self, t: String) {
        self.nodi += 1;
        self.cima().figli.push(NodoUscita::Commento(t));
    }

    pub fn apri(&mut self, nome: String) {
        self.nodi += 1;
        self.pila.push(Cornice {
            nome: Some(nome),
            attributi: Vec::new(),
            figli: Vec::new(),
        });
    }

    /// Aggiunge (o sostituisce) un attributo dell'elemento aperto. Viene
    /// ignorato se l'elemento ha già dei figli, come previsto dalla specifica.
    pub fn attributo(&mut self, nome: String, valore: String) {
        let c = self.cima();
        if c.nome.is_none() || !c.figli.is_empty() {
            return;
        }
        if let Some(a) = c.attributi.iter_mut().find(|(n, _)| *n == nome) {
            a.1 = valore;
        } else {
            c.attributi.push((nome, valore));
        }
    }

    pub fn chiudi(&mut self) {
        let c = self.pila.pop().expect("elemento aperto");
        let nodo = NodoUscita::Elemento {
            nome: c.nome.unwrap_or_default(),
            attributi: c.attributi,
            figli: c.figli,
        };
        self.cima().figli.push(nodo);
    }

    /// Inizia la cattura di un frammento separato.
    pub fn inizia_frammento(&mut self) {
        self.pila.push(Cornice {
            nome: None,
            attributi: Vec::new(),
            figli: Vec::new(),
        });
    }

    pub fn fine_frammento(&mut self) -> Frammento {
        Frammento(self.pila.pop().map(|c| c.figli).unwrap_or_default())
    }

    pub fn copia(&mut self, f: &Frammento) {
        for n in &f.0 {
            self.nodi += 1;
            match n {
                NodoUscita::Testo { testo, grezzo } => self.testo(testo, *grezzo),
                altro => self.cima().figli.push(altro.clone()),
            }
        }
    }

    pub fn risultato(mut self) -> Vec<NodoUscita> {
        while self.pila.len() > 1 {
            self.chiudi();
        }
        self.pila.pop().map(|c| c.figli).unwrap_or_default()
    }
}

/// Elementi HTML senza tag di chiusura.
const VUOTI: &[&str] = &[
    "area", "base", "basefont", "br", "col", "embed", "frame", "hr", "img", "input", "isindex",
    "link", "meta", "param", "source", "track", "wbr",
];

/// Elementi HTML il cui contenuto non va trasformato in entità.
const GREZZI: &[&str] = &["script", "style"];

fn escape_testo(t: &str, s: &mut String) {
    for c in t.chars() {
        match c {
            '&' => s.push_str("&amp;"),
            '<' => s.push_str("&lt;"),
            '>' => s.push_str("&gt;"),
            _ => s.push(c),
        }
    }
}

fn escape_attributo(t: &str, s: &mut String) {
    for c in t.chars() {
        match c {
            '&' => s.push_str("&amp;"),
            '<' => s.push_str("&lt;"),
            '"' => s.push_str("&quot;"),
            _ => s.push(c),
        }
    }
}

/// Serializza l'albero; con `html` si seguono le regole del metodo html.
pub fn serializza(nodi: &[NodoUscita], html: bool) -> String {
    let mut s = String::new();
    if html {
        s.push_str("<!DOCTYPE html>\n");
    }
    scrivi(nodi, html, false, &mut s);
    s
}

fn scrivi(nodi: &[NodoUscita], html: bool, grezzo_padre: bool, s: &mut String) {
    for n in nodi {
        match n {
            NodoUscita::Testo { testo, grezzo } => {
                if *grezzo || grezzo_padre {
                    s.push_str(testo);
                } else {
                    escape_testo(testo, s);
                }
            }
            NodoUscita::Commento(t) => {
                s.push_str("<!--");
                s.push_str(t);
                s.push_str("-->");
            }
            NodoUscita::Elemento {
                nome,
                attributi,
                figli,
            } => {
                s.push('<');
                s.push_str(nome);
                for (n, v) in attributi {
                    s.push(' ');
                    s.push_str(n);
                    s.push_str("=\"");
                    escape_attributo(v, s);
                    s.push('"');
                }
                let minuscolo = nome.to_ascii_lowercase();
                if html && VUOTI.contains(&minuscolo.as_str()) {
                    s.push('>');
                    continue;
                }
                if html && minuscolo == "head" {
                    // Come previsto dal metodo html, si dichiara la codifica.
                    s.push_str(
                        "><meta http-equiv=\"Content-Type\" content=\"text/html; charset=UTF-8\">",
                    );
                    scrivi(figli, html, false, s);
                    s.push_str("</");
                    s.push_str(nome);
                    s.push('>');
                    continue;
                }
                if !html && figli.is_empty() {
                    s.push_str("/>");
                    continue;
                }
                s.push('>');
                let grezzo = html && GREZZI.contains(&minuscolo.as_str());
                scrivi(figli, html, grezzo, s);
                s.push_str("</");
                s.push_str(nome);
                s.push('>');
            }
        }
    }
}
