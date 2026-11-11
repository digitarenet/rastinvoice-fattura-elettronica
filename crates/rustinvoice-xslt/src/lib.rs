// Copyright (c) 2026 Andrea Re - info@digitare.net
// Autore: Andrea Re @ 2026 - info@digitare.net
//
// Motore XSLT 1.0 / XPath 1.0 essenziale, scritto da zero, per applicare
// fogli di stile alle fatture senza dipendere dal motore della webview.
//
// Copre le istruzioni e le funzioni usate nei fogli di stile per la
// FatturaPA. Le parti non supportate (xsl:key, xsl:import, xsl:number,
// document() e simili) producono un errore `NonSupportato`, così chi
// chiama può ripiegare su un altro motore.

mod esegui;
mod foglio;
mod formato;
mod uscita;
mod valore;
mod valuta;
mod xpath;

use std::fmt;

pub use foglio::Foglio;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Errore {
    /// Documento o foglio di stile non ben formato.
    Xml(String),
    XPath(String),
    Xslt(String),
    /// Funzionalità XSLT non implementata.
    NonSupportato(String),
    /// Superato un limite di sicurezza (ricorsione, dimensione).
    Limite(String),
    /// xsl:message con terminate="yes".
    Terminata(String),
}

impl fmt::Display for Errore {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Errore::Xml(m) => write!(f, "XML non valido: {m}"),
            Errore::XPath(m) => write!(f, "espressione XPath non valida: {m}"),
            Errore::Xslt(m) => write!(f, "foglio di stile non valido: {m}"),
            Errore::NonSupportato(m) => write!(f, "funzionalità XSLT non supportata: {m}"),
            Errore::Limite(m) => write!(f, "trasformazione interrotta: {m}"),
            Errore::Terminata(m) => write!(f, "trasformazione terminata dal foglio di stile: {m}"),
        }
    }
}

impl std::error::Error for Errore {}

impl Foglio {
    /// Applica il foglio a un documento XML e restituisce il risultato
    /// serializzato.
    pub fn applica(&self, xml: &str) -> Result<String, Errore> {
        let opzioni = roxmltree::ParsingOptions {
            allow_dtd: true,
            ..Default::default()
        };
        let doc = roxmltree::Document::parse_with_options(xml, opzioni)
            .map_err(|e| Errore::Xml(e.to_string()))?;
        let mut es = esegui::Esecuzione::nuova(self);
        es.avvia(valore::Nodo::Normale(doc.root()))?;
        let nodi = es.risultato();
        let html = match self.metodo.as_deref() {
            Some("html") => true,
            Some(_) => false,
            None => matches!(
                nodi.iter().find(|n| !matches!(n, uscita::NodoUscita::Testo { testo, .. } if testo.trim().is_empty())),
                Some(uscita::NodoUscita::Elemento { nome, .. }) if nome.eq_ignore_ascii_case("html")
            ),
        };
        if self.metodo.as_deref() == Some("text") {
            return Ok(uscita::Frammento(nodi).testo());
        }
        Ok(uscita::serializza(&nodi, html))
    }
}

/// Compila il foglio e lo applica al documento, in un thread con una pila
/// ampia per reggere i modelli ricorsivi.
pub fn trasforma(foglio: &str, xml: &str) -> Result<String, Errore> {
    let foglio = foglio.to_string();
    let xml = xml.to_string();
    std::thread::Builder::new()
        .name("xslt".into())
        .stack_size(256 * 1024 * 1024)
        .spawn(move || Foglio::compila(&foglio)?.applica(&xml))
        .map_err(|e| Errore::Limite(e.to_string()))?
        .join()
        .map_err(|_| Errore::Limite("errore interno del motore XSLT".into()))?
}
