// Copyright (c) 2026 Andrea Re - info@digitare.net
// Autore: Andrea Re @ 2026 - info@digitare.net
//
// Scansione di file e cartelle e costruzione dell'elenco sintetico dei
// documenti mostrato nell'interfaccia. La lettura avviene in parallelo su
// tutti i core disponibili.

use rayon::prelude::*;
use rustinvoice_core::{apri_file, codici, Contenuto};
use serde::Serialize;
use std::path::{Path, PathBuf};

/// Soggetto in forma sintetica.
#[derive(Debug, Clone, Serialize)]
pub struct SoggettoBreve {
    pub nome: String,
    /// Chiave stabile: partita IVA (es. `IT01234567890`) o `CF:...`.
    pub chiave: String,
    pub comune: String,
}

/// Una riga dell'elenco: un singolo documento (un file con un lotto
/// produce più voci, una per corpo).
#[derive(Debug, Clone, Serialize)]
pub struct Voce {
    pub id: String,
    pub file: String,
    pub nome_file: String,
    pub corpo: usize,
    pub corpi_nel_file: usize,
    pub tipo_documento: String,
    pub descrizione_tipo: String,
    pub nota_credito: bool,
    pub numero: String,
    /// Data in formato ISO `AAAA-MM-GG` (vuota se mancante).
    pub data: String,
    pub anno: i32,
    pub cedente: SoggettoBreve,
    pub cessionario: SoggettoBreve,
    /// Totale come stringa decimale (es. `1512.80`), per non perdere precisione.
    pub totale: String,
    pub divisa: String,
    pub prima_scadenza: String,
    pub firmato: bool,
    pub allegati: usize,
    pub avvisi: usize,
    /// Testo normalizzato su cui eseguire la ricerca libera.
    pub testo: String,
}

/// Messaggio SdI trovato durante la scansione.
#[derive(Debug, Clone, Serialize)]
pub struct VoceMessaggio {
    pub file: String,
    pub sigla: String,
    pub descrizione: String,
    pub riferito_a: String,
}

/// File che non è stato possibile interpretare.
#[derive(Debug, Clone, Serialize)]
pub struct FileScartato {
    pub file: String,
    pub motivo: String,
}

/// Esito di una scansione.
#[derive(Debug, Clone, Default, Serialize)]
pub struct Esito {
    pub voci: Vec<Voce>,
    pub messaggi: Vec<VoceMessaggio>,
    pub scartati: Vec<FileScartato>,
}

/// Estensioni prese in considerazione.
fn e_candidato(p: &Path) -> bool {
    let nome = p
        .file_name()
        .map(|n| n.to_string_lossy().to_lowercase())
        .unwrap_or_default();
    nome.ends_with(".xml") || nome.ends_with(".p7m")
}

/// Elenca i file candidati partendo da file e cartelle (ricorsivamente).
pub fn elenca(percorsi: &[String]) -> Vec<PathBuf> {
    let mut file = Vec::new();
    for p in percorsi {
        let p = PathBuf::from(p);
        if p.is_dir() {
            for voce in walkdir::WalkDir::new(&p)
                .follow_links(false)
                .into_iter()
                .filter_map(|e| e.ok())
            {
                if voce.file_type().is_file() && e_candidato(voce.path()) {
                    file.push(voce.into_path());
                }
            }
        } else if p.is_file() {
            file.push(p);
        }
    }
    file.sort();
    file.dedup();
    file
}

/// Normalizza il testo per la ricerca: minuscole e spazi singoli.
fn normalizza(s: &str) -> String {
    s.to_lowercase().split_whitespace().collect::<Vec<_>>().join(" ")
}

enum Letto {
    Voci(Vec<Voce>),
    Messaggio(VoceMessaggio),
    Scartato(FileScartato),
}

fn leggi(percorso: &Path) -> Letto {
    let file = percorso.to_string_lossy().to_string();
    let nome_file = percorso
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_default();
    let doc = match apri_file(percorso) {
        Ok(d) => d,
        Err(e) => {
            return Letto::Scartato(FileScartato {
                file,
                motivo: e.to_string(),
            })
        }
    };
    let firmato = doc.firmato();
    match doc.contenuto {
        Contenuto::MessaggioSdi(m) => Letto::Messaggio(VoceMessaggio {
            file,
            sigla: m.tipo.sigla().to_owned(),
            descrizione: m.tipo.descrizione().to_owned(),
            riferito_a: m.nome_file.unwrap_or_default(),
        }),
        Contenuto::Fattura(f) => {
            let h = &f.intestazione;
            let breve = |s: &rustinvoice_core::modello::Soggetto| SoggettoBreve {
                nome: s.nome_visualizzato(),
                chiave: s.chiave().unwrap_or_default(),
                comune: s
                    .sede
                    .as_ref()
                    .and_then(|x| x.comune.clone())
                    .unwrap_or_default(),
            };
            let cedente = breve(&h.cedente_prestatore);
            let cessionario = breve(&h.cessionario_committente);
            let n = f.corpi.len();
            let voci = f
                .corpi
                .iter()
                .enumerate()
                .map(|(i, c)| {
                    let d = &c.dati_generali.documento;
                    let tipo = d.tipo_documento.clone().unwrap_or_default();
                    let mut testo = vec![
                        d.numero.clone().unwrap_or_default(),
                        cedente.nome.clone(),
                        cedente.chiave.clone(),
                        cessionario.nome.clone(),
                        cessionario.chiave.clone(),
                        nome_file.clone(),
                        d.causale.join(" "),
                    ];
                    for l in &c.linee {
                        if let Some(x) = &l.descrizione {
                            testo.push(x.clone());
                        }
                        for ca in &l.codici_articolo {
                            if let Some(v) = &ca.valore {
                                testo.push(v.clone());
                            }
                        }
                    }
                    for a in &c.allegati {
                        if let Some(x) = &a.nome {
                            testo.push(x.clone());
                        }
                    }
                    let totale = c.totale();
                    if let Some(t) = totale {
                        testo.push(t.to_string());
                    }
                    Voce {
                        id: format!("{file}#{i}"),
                        file: file.clone(),
                        nome_file: nome_file.clone(),
                        corpo: i,
                        corpi_nel_file: n,
                        descrizione_tipo: codici::tipo_documento(&tipo)
                            .unwrap_or("Documento")
                            .to_owned(),
                        nota_credito: codici::e_nota_di_credito(&tipo),
                        tipo_documento: tipo,
                        numero: d.numero.clone().unwrap_or_default(),
                        data: d.data.map(|x| x.to_string()).unwrap_or_default(),
                        anno: d.data.map(|x| chrono::Datelike::year(&x)).unwrap_or(0),
                        cedente: cedente.clone(),
                        cessionario: cessionario.clone(),
                        totale: totale.map(|t| t.to_string()).unwrap_or_default(),
                        divisa: d.divisa.clone().unwrap_or_else(|| "EUR".into()),
                        prima_scadenza: c
                            .prima_scadenza()
                            .map(|x| x.to_string())
                            .unwrap_or_default(),
                        firmato,
                        allegati: c.allegati.len(),
                        avvisi: f.avvisi.len(),
                        testo: normalizza(&testo.join(" ")),
                    }
                })
                .collect();
            Letto::Voci(voci)
        }
    }
}

/// Legge in parallelo tutti i file indicati (o contenuti nelle cartelle).
pub fn scansiona(percorsi: &[String]) -> Esito {
    let file = elenca(percorsi);
    let letti: Vec<Letto> = file.par_iter().map(|p| leggi(p)).collect();
    let mut esito = Esito::default();
    for l in letti {
        match l {
            Letto::Voci(v) => esito.voci.extend(v),
            Letto::Messaggio(m) => esito.messaggi.push(m),
            Letto::Scartato(s) => esito.scartati.push(s),
        }
    }
    // Più recenti per primi.
    esito.voci.sort_by(|a, b| b.data.cmp(&a.data).then(a.numero.cmp(&b.numero)));
    esito
}
