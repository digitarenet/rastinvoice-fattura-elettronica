// Copyright (c) 2026 Andrea Re - info@digitare.net
// Autore: Andrea Re @ 2026 - info@digitare.net
//
// Esempio: crea il PDF "Moderna" di una fattura XML.
// Uso: cargo run -p rustinvoice-render --example pdf -- fattura.xml uscita.pdf

fn main() {
    let argomenti: Vec<String> = std::env::args().collect();
    if argomenti.len() != 3 {
        eprintln!("uso: pdf <fattura.xml> <uscita.pdf>");
        std::process::exit(2);
    }
    let xml = std::fs::read_to_string(&argomenti[1]).expect("file non leggibile");
    let f = rustinvoice_core::parser::leggi_da_testo(&xml).expect("fattura non valida");
    let pdf = rustinvoice_render::pdf::pdf_moderna(&f).expect("PDF non generato");
    std::fs::write(&argomenti[2], pdf).expect("scrittura non riuscita");
}
