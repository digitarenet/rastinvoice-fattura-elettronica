// Copyright (c) 2026 Andrea Re - info@digitare.net
// Autore: Andrea Re @ 2026 - info@digitare.net
//
// Riga di comando di RustinVoice.
//
// Comandi disponibili:
//   rustinvoice info <file>        riepilogo leggibile di una fattura
//   rustinvoice json <file>        intero modello dati in JSON
//   rustinvoice scansiona <cartella>  elenco dei documenti trovati (CSV)

use clap::{Parser, Subcommand};
use rustinvoice_core::{apri_byte, apri_file, codici, contenitori, Contenuto, Documento};
use std::path::{Path, PathBuf};
use std::process::ExitCode;

#[derive(Parser)]
#[command(
    name = "rustinvoice",
    version,
    about = "Lettura e analisi di fatture elettroniche FatturaPA"
)]
struct Argomenti {
    #[command(subcommand)]
    comando: Comando,
}

#[derive(Subcommand)]
enum Comando {
    /// Mostra un riepilogo leggibile del documento.
    Info { file: PathBuf },
    /// Stampa l'intero modello dati in formato JSON.
    Json { file: PathBuf },
    /// Scansiona una cartella (anche le sottocartelle) e stampa un elenco CSV.
    Scansiona { cartella: PathBuf },
}

fn main() -> ExitCode {
    let argomenti = Argomenti::parse();
    let esito = match argomenti.comando {
        Comando::Info { file } => info(&file),
        Comando::Json { file } => json(&file),
        Comando::Scansiona { cartella } => scansiona(&cartella),
    };
    match esito {
        Ok(()) => ExitCode::SUCCESS,
        Err(messaggio) => {
            eprintln!("Errore: {messaggio}");
            ExitCode::FAILURE
        }
    }
}

fn apri(file: &Path) -> Result<Documento, String> {
    apri_file(file).map_err(|e| format!("{}: {e}", file.display()))
}

fn info(file: &Path) -> Result<(), String> {
    let documento = apri(file)?;
    if let Some(b) = &documento.busta {
        println!(
            "Busta firmata: {} livello/i, {} certificato/i",
            b.livelli,
            b.certificati.len()
        );
    }
    match &documento.contenuto {
        Contenuto::MessaggioSdi(m) => {
            println!("{} ({})", m.tipo.descrizione(), m.tipo.sigla());
            if let Some(n) = &m.nome_file {
                println!("Riferito al file: {n}");
            }
            if let Some(id) = &m.identificativo_sdi {
                println!("Identificativo SdI: {id}");
            }
            for e in &m.errori {
                println!(
                    "  Errore {}: {}",
                    e.codice.as_deref().unwrap_or("?"),
                    e.descrizione.as_deref().unwrap_or("")
                );
            }
        }
        Contenuto::Fattura(f) => {
            let h = &f.intestazione;
            println!(
                "Formato: {:?} (versione {})",
                f.formato,
                f.versione.as_deref().unwrap_or("?")
            );
            println!(
                "Cedente/prestatore: {} [{}]",
                h.cedente_prestatore.nome_visualizzato(),
                h.cedente_prestatore.chiave().unwrap_or_default()
            );
            println!(
                "Cessionario/committente: {} [{}]",
                h.cessionario_committente.nome_visualizzato(),
                h.cessionario_committente.chiave().unwrap_or_default()
            );
            for (i, c) in f.corpi.iter().enumerate() {
                let d = &c.dati_generali.documento;
                let tipo = d.tipo_documento.as_deref().unwrap_or("?");
                println!(
                    "Documento {}: {} {} n. {} del {}",
                    i + 1,
                    tipo,
                    codici::tipo_documento(tipo).unwrap_or("codice non previsto"),
                    d.numero.as_deref().unwrap_or("?"),
                    d.data
                        .map(|x| x.format("%d/%m/%Y").to_string())
                        .unwrap_or_else(|| "?".into())
                );
                println!(
                    "  Righe: {}, totale: {} {}",
                    c.linee.len(),
                    c.totale()
                        .map(|t| t.to_string())
                        .unwrap_or_else(|| "?".into()),
                    d.divisa.as_deref().unwrap_or("")
                );
                if let Some(s) = c.prima_scadenza() {
                    println!("  Prima scadenza: {}", s.format("%d/%m/%Y"));
                }
                for a in &c.allegati {
                    println!(
                        "  Allegato: {} ({} byte)",
                        a.nome.as_deref().unwrap_or("senza nome"),
                        a.dimensione
                    );
                }
            }
            for a in &f.avvisi {
                println!("Avviso: {a}");
            }
        }
    }
    Ok(())
}

fn json(file: &Path) -> Result<(), String> {
    let documento = apri(file)?;
    let testo = serde_json::to_string_pretty(&documento).map_err(|e| e.to_string())?;
    println!("{testo}");
    Ok(())
}

/// Racchiude un campo CSV tra virgolette se necessario.
fn csv(campo: &str) -> String {
    if campo.contains([';', '"', '\n']) {
        format!("\"{}\"", campo.replace('"', "\"\""))
    } else {
        campo.to_owned()
    }
}

fn scansiona(cartella: &Path) -> Result<(), String> {
    println!("file;tipo;cedente;piva_cedente;cessionario;piva_cessionario;tipo_documento;numero;data;totale");
    let mut errori = 0usize;
    for voce in walkdir::WalkDir::new(cartella)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        if !voce.file_type().is_file() {
            continue;
        }
        let percorso = voce.path();
        let nome = percorso
            .file_name()
            .map(|n| n.to_string_lossy().to_lowercase())
            .unwrap_or_default();
        // Documenti singoli oppure contenuti in archivi .zip ed email .eml.
        let documenti: Vec<(String, Result<Documento, String>)> =
            if let Some(tipo) = contenitori::tipo_contenitore(&nome) {
                match std::fs::read(percorso)
                    .map_err(|e| e.to_string())
                    .and_then(|b| contenitori::espandi(tipo, &b))
                {
                    Ok(interni) => interni
                        .into_iter()
                        .map(|i| {
                            let virtuale = format!(
                                "{}{}{}",
                                percorso.display(),
                                contenitori::SEPARATORE,
                                i.percorso
                            );
                            (virtuale, apri_byte(&i.byte).map_err(|e| e.to_string()))
                        })
                        .collect(),
                    Err(e) => vec![(percorso.display().to_string(), Err(e))],
                }
            } else if contenitori::e_documento(&nome) {
                vec![(
                    percorso.display().to_string(),
                    apri_file(percorso).map_err(|e| e.to_string()),
                )]
            } else {
                continue;
            };
        for (percorso, letto) in documenti {
            match letto {
                Ok(doc) => match &doc.contenuto {
                    Contenuto::Fattura(f) => {
                        let h = &f.intestazione;
                        for c in &f.corpi {
                            let d = &c.dati_generali.documento;
                            let riga = [
                                percorso.clone(),
                                "fattura".into(),
                                h.cedente_prestatore.nome_visualizzato(),
                                h.cedente_prestatore.chiave().unwrap_or_default(),
                                h.cessionario_committente.nome_visualizzato(),
                                h.cessionario_committente.chiave().unwrap_or_default(),
                                d.tipo_documento.clone().unwrap_or_default(),
                                d.numero.clone().unwrap_or_default(),
                                d.data.map(|x| x.to_string()).unwrap_or_default(),
                                c.totale().map(|t| t.to_string()).unwrap_or_default(),
                            ];
                            println!(
                                "{}",
                                riga.iter().map(|c| csv(c)).collect::<Vec<_>>().join(";")
                            );
                        }
                    }
                    Contenuto::MessaggioSdi(m) => {
                        println!(
                            "{};{};;;;;;{};;",
                            csv(&percorso),
                            m.tipo.sigla(),
                            csv(m.nome_file.as_deref().unwrap_or(""))
                        );
                    }
                },
                Err(e) => {
                    errori += 1;
                    eprintln!("Ignorato {percorso}: {e}");
                }
            }
        }
    }
    if errori > 0 {
        eprintln!("{errori} file non riconosciuti");
    }
    Ok(())
}
