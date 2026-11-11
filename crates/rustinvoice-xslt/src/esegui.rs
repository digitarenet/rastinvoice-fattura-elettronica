// Copyright (c) 2026 Andrea Re - info@digitare.net
// Autore: Andrea Re @ 2026 - info@digitare.net
//
// Esecuzione del foglio compilato su un documento sorgente.

use roxmltree::NodeType;

use crate::foglio::{Avt, Foglio, Istruzione, Ordinamento, ParteAvt, Variabile};
use crate::uscita::{Costruttore, NodoUscita};
use crate::valore::{Nodo, Valore};
use crate::valuta::{insieme, valuta, Ambiente, Contesto};
use crate::xpath::{Base, Espr, Passo, Test};
use crate::Errore;

/// Annidamento massimo di modelli, contro le ricorsioni infinite.
const PROFONDITA_MASSIMA: usize = 1000;
/// Numero massimo di nodi prodotti.
const NODI_MASSIMI: usize = 5_000_000;

type Locali<'a, 'i> = Vec<(String, Valore<'a, 'i>)>;

pub struct Esecuzione<'f, 'a, 'i> {
    foglio: &'f Foglio,
    pub out: Costruttore,
    globali: Locali<'a, 'i>,
    profondita: usize,
}

impl<'f, 'a, 'i> Esecuzione<'f, 'a, 'i> {
    pub fn nuova(foglio: &'f Foglio) -> Self {
        Esecuzione {
            foglio,
            out: Costruttore::nuovo(),
            globali: Vec::new(),
            profondita: 0,
        }
    }

    pub fn avvia(&mut self, radice: Nodo<'a, 'i>) -> Result<(), Errore> {
        let c = Contesto {
            nodo: radice,
            posizione: 1,
            dimensione: 1,
            corrente: radice,
        };
        for g in &self.foglio.globali {
            let mut nessuna = Vec::new();
            let v = self.valore_variabile(g, &c, &mut nessuna)?;
            self.globali.push((g.nome.clone(), v));
        }
        self.applica(vec![radice], None, &[], &c, &mut Vec::new())
    }

    fn valuta(
        &self,
        e: &Espr,
        c: &Contesto<'a, 'i>,
        locali: &Locali<'a, 'i>,
    ) -> Result<Valore<'a, 'i>, Errore> {
        let amb = Ambiente {
            locali,
            globali: &self.globali,
            formati: &self.foglio.formati,
        };
        valuta(e, c, &amb)
    }

    fn avt(
        &self,
        a: &Avt,
        c: &Contesto<'a, 'i>,
        locali: &Locali<'a, 'i>,
    ) -> Result<String, Errore> {
        let mut s = String::new();
        for p in a {
            match p {
                ParteAvt::Testo(t) => s.push_str(t),
                ParteAvt::Espr(e) => s.push_str(&self.valuta(e, c, locali)?.stringa()),
            }
        }
        Ok(s)
    }

    fn valore_variabile(
        &mut self,
        v: &Variabile,
        c: &Contesto<'a, 'i>,
        locali: &mut Locali<'a, 'i>,
    ) -> Result<Valore<'a, 'i>, Errore> {
        if let Some(s) = &v.selezione {
            return self.valuta(s, c, locali);
        }
        if v.corpo.is_empty() {
            return Ok(Valore::Stringa(String::new()));
        }
        self.out.inizia_frammento();
        let esito = self.blocco(&v.corpo, c, locali);
        let f = self.out.fine_frammento();
        esito?;
        Ok(Valore::Frammento(f))
    }

    /// Esegue un blocco; le variabili dichiarate restano visibili fino alla fine.
    fn blocco(
        &mut self,
        istr: &[Istruzione],
        c: &Contesto<'a, 'i>,
        locali: &mut Locali<'a, 'i>,
    ) -> Result<(), Errore> {
        let base = locali.len();
        let mut esito = Ok(());
        for i in istr {
            esito = self.istruzione(i, c, locali);
            if esito.is_err() {
                break;
            }
        }
        locali.truncate(base);
        esito
    }

    fn testo_di(
        &mut self,
        corpo: &[Istruzione],
        c: &Contesto<'a, 'i>,
        locali: &mut Locali<'a, 'i>,
    ) -> Result<String, Errore> {
        self.out.inizia_frammento();
        let esito = self.blocco(corpo, c, locali);
        let f = self.out.fine_frammento();
        esito?;
        Ok(f.testo())
    }

    fn istruzione(
        &mut self,
        i: &Istruzione,
        c: &Contesto<'a, 'i>,
        locali: &mut Locali<'a, 'i>,
    ) -> Result<(), Errore> {
        if self.out.nodi > NODI_MASSIMI {
            return Err(Errore::Limite("risultato troppo grande".into()));
        }
        match i {
            Istruzione::Testo { testo, grezzo } => self.out.testo(testo, *grezzo),
            Istruzione::ValoreDi { selezione, grezzo } => {
                let s = self.valuta(selezione, c, locali)?.stringa();
                self.out.testo(&s, *grezzo);
            }
            Istruzione::Letterale {
                nome,
                attributi,
                corpo,
            } => {
                let mut valori = Vec::with_capacity(attributi.len());
                for (n, a) in attributi {
                    valori.push((n.clone(), self.avt(a, c, locali)?));
                }
                self.out.apri(nome.clone());
                for (n, v) in valori {
                    self.out.attributo(n, v);
                }
                let esito = self.blocco(corpo, c, locali);
                self.out.chiudi();
                esito?;
            }
            Istruzione::Se { test, corpo } => {
                if self.valuta(test, c, locali)?.booleano() {
                    self.blocco(corpo, c, locali)?;
                }
            }
            Istruzione::Scegli { rami, altrimenti } => {
                let mut scelto = None;
                for (test, corpo) in rami {
                    if self.valuta(test, c, locali)?.booleano() {
                        scelto = Some(corpo);
                        break;
                    }
                }
                self.blocco(scelto.unwrap_or(altrimenti), c, locali)?;
            }
            Istruzione::PerOgni {
                selezione,
                ordinamento,
                corpo,
            } => {
                let nodi = insieme(self.valuta(selezione, c, locali)?)?;
                let nodi = self.ordina(nodi, ordinamento, c, locali)?;
                let dimensione = nodi.len();
                for (k, n) in nodi.into_iter().enumerate() {
                    let interno = Contesto {
                        nodo: n,
                        posizione: k + 1,
                        dimensione,
                        corrente: n,
                    };
                    self.blocco(corpo, &interno, locali)?;
                }
            }
            Istruzione::ApplicaModelli {
                selezione,
                modo,
                parametri,
                ordinamento,
            } => {
                let nodi = match selezione {
                    Some(s) => insieme(self.valuta(s, c, locali)?)?,
                    None => figli(c.nodo),
                };
                let nodi = self.ordina(nodi, ordinamento, c, locali)?;
                let passati = self.parametri(parametri, c, locali)?;
                self.applica(nodi, modo.as_deref(), &passati, c, locali)?;
            }
            Istruzione::ChiamaModello { nome, parametri } => {
                let indice = *self
                    .foglio
                    .nominati
                    .get(nome)
                    .ok_or_else(|| Errore::Xslt(format!("modello '{nome}' non definito")))?;
                let passati = self.parametri(parametri, c, locali)?;
                self.invoca(indice, c, passati)?;
            }
            Istruzione::Variabile(v) => {
                let valore = self.valore_variabile(v, c, locali)?;
                locali.push((v.nome.clone(), valore));
            }
            Istruzione::Attributo { nome, corpo } => {
                let n = self.avt(nome, c, locali)?;
                let v = self.testo_di(corpo, c, locali)?;
                self.out.attributo(n, v);
            }
            Istruzione::Elemento { nome, corpo } => {
                let n = self.avt(nome, c, locali)?;
                self.out.apri(n);
                let esito = self.blocco(corpo, c, locali);
                self.out.chiudi();
                esito?;
            }
            Istruzione::CopiaDi(e) => match self.valuta(e, c, locali)? {
                Valore::Nodi(nodi) => {
                    for n in nodi {
                        self.copia_profonda(n);
                    }
                }
                Valore::Frammento(f) => self.out.copia(&f),
                altro => self.out.testo(&altro.stringa(), false),
            },
            Istruzione::Copia(corpo) => match c.nodo {
                Nodo::Normale(n) if n.is_element() => {
                    self.out.apri(c.nodo.nome());
                    let esito = self.blocco(corpo, c, locali);
                    self.out.chiudi();
                    esito?;
                }
                Nodo::Normale(n) if n.node_type() == NodeType::Root => {
                    self.blocco(corpo, c, locali)?
                }
                Nodo::Normale(n) if n.is_comment() => self.out.commento(c.nodo.testo()),
                Nodo::Normale(n) if n.is_text() => self.out.testo(&c.nodo.testo(), false),
                Nodo::Attributo(..) => self.out.attributo(c.nodo.nome(), c.nodo.testo()),
                Nodo::Normale(_) => {}
            },
            Istruzione::Commento(corpo) => {
                let t = self.testo_di(corpo, c, locali)?;
                self.out.commento(t);
            }
            Istruzione::Messaggio { corpo, termina } => {
                let t = self.testo_di(corpo, c, locali)?;
                if *termina {
                    return Err(Errore::Terminata(t));
                }
            }
        }
        Ok(())
    }

    fn parametri(
        &mut self,
        parametri: &[Variabile],
        c: &Contesto<'a, 'i>,
        locali: &mut Locali<'a, 'i>,
    ) -> Result<Locali<'a, 'i>, Errore> {
        let mut v = Vec::with_capacity(parametri.len());
        for p in parametri {
            let valore = self.valore_variabile(p, c, locali)?;
            v.push((p.nome.clone(), valore));
        }
        Ok(v)
    }

    /// Esegue il modello `indice` con i parametri passati.
    fn invoca(
        &mut self,
        indice: usize,
        c: &Contesto<'a, 'i>,
        passati: Locali<'a, 'i>,
    ) -> Result<(), Errore> {
        if self.profondita >= PROFONDITA_MASSIMA {
            return Err(Errore::Limite("troppi modelli annidati".into()));
        }
        self.profondita += 1;
        let foglio = self.foglio;
        let corpo = &foglio.corpi[indice];
        let mut locali: Locali = Vec::new();
        let mut passati = passati;
        let mut esito = Ok(());
        for p in &corpo.parametri {
            if let Some(pos) = passati.iter().position(|(n, _)| *n == p.nome) {
                let (n, v) = passati.swap_remove(pos);
                locali.push((n, v));
            } else {
                match self.valore_variabile(p, c, &mut locali) {
                    Ok(v) => locali.push((p.nome.clone(), v)),
                    Err(e) => {
                        esito = Err(e);
                        break;
                    }
                }
            }
        }
        if esito.is_ok() {
            esito = self.blocco(&corpo.istruzioni, c, &mut locali);
        }
        self.profondita -= 1;
        esito
    }

    fn applica(
        &mut self,
        nodi: Vec<Nodo<'a, 'i>>,
        modo: Option<&str>,
        passati: &[(String, Valore<'a, 'i>)],
        _c: &Contesto<'a, 'i>,
        _locali: &mut Locali<'a, 'i>,
    ) -> Result<(), Errore> {
        let dimensione = nodi.len();
        for (k, n) in nodi.into_iter().enumerate() {
            let c = Contesto {
                nodo: n,
                posizione: k + 1,
                dimensione,
                corrente: n,
            };
            match self.trova(n, modo)? {
                Some(indice) => self.invoca(indice, &c, passati.to_vec())?,
                None => self.predefinito(n, modo, &c)?,
            }
        }
        Ok(())
    }

    /// Modelli predefiniti: discesa nei figli e copia del testo.
    fn predefinito(
        &mut self,
        n: Nodo<'a, 'i>,
        modo: Option<&str>,
        c: &Contesto<'a, 'i>,
    ) -> Result<(), Errore> {
        match n {
            Nodo::Normale(x) if x.is_element() || x.node_type() == NodeType::Root => {
                if self.profondita >= PROFONDITA_MASSIMA {
                    return Err(Errore::Limite("documento troppo annidato".into()));
                }
                self.profondita += 1;
                let esito = self.applica(figli(n), modo, &[], c, &mut Vec::new());
                self.profondita -= 1;
                esito
            }
            Nodo::Normale(x) if x.is_text() => {
                self.out.testo(&n.testo(), false);
                Ok(())
            }
            Nodo::Attributo(..) => {
                self.out.testo(&n.testo(), false);
                Ok(())
            }
            _ => Ok(()),
        }
    }

    fn trova(&self, n: Nodo<'a, 'i>, modo: Option<&str>) -> Result<Option<usize>, Errore> {
        for r in &self.foglio.regole {
            if r.modo.as_deref() == modo && self.corrisponde(&r.pattern, n)? {
                return Ok(Some(r.corpo));
            }
        }
        Ok(None)
    }

    fn corrisponde(&self, p: &Espr, n: Nodo<'a, 'i>) -> Result<bool, Errore> {
        let Espr::Percorso(base, passi) = p else {
            return Err(Errore::NonSupportato(
                "pattern diverso da un percorso".into(),
            ));
        };
        if !ultimo_passo_compatibile(passi, n) {
            return Ok(false);
        }
        let vuote: Locali = Vec::new();
        let prova = |contesto: Nodo<'a, 'i>| -> Result<bool, Errore> {
            let c = Contesto {
                nodo: contesto,
                posizione: 1,
                dimensione: 1,
                corrente: n,
            };
            let e = Espr::Percorso(Base::Contesto, passi.clone());
            let trovati = insieme(self.valuta(&e, &c, &vuote)?)?;
            Ok(trovati.contains(&n))
        };
        match base {
            Base::Radice => {
                if passi.is_empty() {
                    return Ok(matches!(n, Nodo::Normale(x) if x.node_type() == NodeType::Root));
                }
                prova(n.radice())
            }
            Base::Contesto => {
                let mut g = n.genitore();
                while let Some(a) = g {
                    if prova(a)? {
                        return Ok(true);
                    }
                    g = a.genitore();
                }
                Ok(false)
            }
            Base::Filtro(_) => Err(Errore::NonSupportato("pattern con id() o key()".into())),
        }
    }

    fn ordina(
        &mut self,
        nodi: Vec<Nodo<'a, 'i>>,
        criteri: &[Ordinamento],
        c: &Contesto<'a, 'i>,
        locali: &mut Locali<'a, 'i>,
    ) -> Result<Vec<Nodo<'a, 'i>>, Errore> {
        if criteri.is_empty() {
            return Ok(nodi);
        }
        let dimensione = nodi.len();
        let mut chiavi = Vec::with_capacity(dimensione);
        for (k, n) in nodi.iter().enumerate() {
            let interno = Contesto {
                nodo: *n,
                posizione: k + 1,
                dimensione,
                corrente: *n,
            };
            let mut riga = Vec::with_capacity(criteri.len());
            for o in criteri {
                let v = self.valuta(&o.selezione, &interno, locali)?;
                riga.push(if o.numerico {
                    Chiave::Numero(v.numero())
                } else {
                    Chiave::Testo(v.stringa())
                });
            }
            chiavi.push(riga);
        }
        let mut indici: Vec<usize> = (0..dimensione).collect();
        indici.sort_by(|&a, &b| {
            for (k, o) in criteri.iter().enumerate() {
                let ord = chiavi[a][k].confronta(&chiavi[b][k]);
                let ord = if o.discendente { ord.reverse() } else { ord };
                if ord != std::cmp::Ordering::Equal {
                    return ord;
                }
            }
            std::cmp::Ordering::Equal
        });
        let _ = c;
        Ok(indici.into_iter().map(|k| nodi[k]).collect())
    }

    fn copia_profonda(&mut self, n: Nodo<'a, 'i>) {
        match n {
            Nodo::Attributo(..) => self.out.attributo(n.nome(), n.testo()),
            Nodo::Normale(x) => match x.node_type() {
                NodeType::Root => {
                    for f in x.children() {
                        self.copia_profonda(Nodo::Normale(f));
                    }
                }
                NodeType::Element => {
                    self.out.apri(n.nome());
                    for k in 0..x.attributes().len() {
                        let a = Nodo::Attributo(x, k);
                        self.out.attributo(a.nome(), a.testo());
                    }
                    for f in x.children() {
                        self.copia_profonda(Nodo::Normale(f));
                    }
                    self.out.chiudi();
                }
                NodeType::Text => self.out.testo(x.text().unwrap_or(""), false),
                NodeType::Comment => self.out.commento(x.text().unwrap_or("").to_string()),
                NodeType::PI => {}
            },
        }
    }

    pub fn risultato(self) -> Vec<NodoUscita> {
        self.out.risultato()
    }
}

enum Chiave {
    Numero(f64),
    Testo(String),
}

impl Chiave {
    fn confronta(&self, altra: &Chiave) -> std::cmp::Ordering {
        match (self, altra) {
            // I valori non numerici precedono gli altri.
            (Chiave::Numero(a), Chiave::Numero(b)) => match (a.is_nan(), b.is_nan()) {
                (true, true) => std::cmp::Ordering::Equal,
                (true, false) => std::cmp::Ordering::Less,
                (false, true) => std::cmp::Ordering::Greater,
                _ => a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal),
            },
            (Chiave::Testo(a), Chiave::Testo(b)) => a.cmp(b),
            _ => std::cmp::Ordering::Equal,
        }
    }
}

fn figli<'a, 'i>(n: Nodo<'a, 'i>) -> Vec<Nodo<'a, 'i>> {
    match n {
        Nodo::Normale(x) => x.children().map(Nodo::Normale).collect(),
        Nodo::Attributo(..) => Vec::new(),
    }
}

/// Scarta subito i nodi il cui nome non può corrispondere all'ultimo passo.
fn ultimo_passo_compatibile(passi: &[Passo], n: Nodo<'_, '_>) -> bool {
    match passi.last().map(|p| &p.test) {
        Some(Test::Nome(ns, locale)) => n
            .nome_espanso()
            .is_some_and(|(u, l)| l == locale && u == ns.as_deref()),
        Some(Test::Testo) => matches!(n, Nodo::Normale(x) if x.is_text()),
        _ => true,
    }
}
