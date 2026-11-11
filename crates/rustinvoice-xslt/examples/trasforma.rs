// Copyright (c) 2026 Andrea Re - info@digitare.net
// Autore: Andrea Re @ 2026 - info@digitare.net
//
// Applica un foglio di stile a un file XML e stampa il risultato:
//   cargo run -p rustinvoice-xslt --example trasforma -- foglio.xsl fattura.xml

fn main() {
    let argomenti: Vec<String> = std::env::args().skip(1).collect();
    let [foglio, xml] = argomenti.as_slice() else {
        eprintln!("uso: trasforma <foglio.xsl> <documento.xml>");
        std::process::exit(2);
    };
    // I file non UTF-8 vengono letti come ISO-8859-1.
    let leggi = |p: &str| {
        let byte = std::fs::read(p).unwrap_or_else(|e| panic!("{p}: {e}"));
        String::from_utf8(byte.clone())
            .unwrap_or_else(|_| byte.iter().map(|&b| b as char).collect())
    };
    match rustinvoice_xslt::trasforma(&leggi(foglio), &leggi(xml)) {
        Ok(html) => print!("{html}"),
        Err(e) => {
            eprintln!("{e}");
            std::process::exit(1);
        }
    }
}
