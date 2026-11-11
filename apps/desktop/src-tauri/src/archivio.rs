// Copyright (c) 2026 Andrea Re - info@digitare.net
// Autore: Andrea Re @ 2026 - info@digitare.net
//
// Scansione di file e cartelle e costruzione dell'elenco sintetico dei
// documenti mostrato nell'interfaccia. La lettura avviene in parallelo su
// tutti i core disponibili.

use rayon::prelude::*;
use rustinvoice_core::{apri_byte, apri_file, codici, contenitori, Contenuto, Documento};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Soggetto in forma sintetica.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SoggettoBreve {
    pub nome: String,
    /// Chiave stabile: partita IVA (es. `IT01234567890`) o `CF:...`.
    pub chiave: String,
    pub comune: String,
}

/// Una riga dell'elenco: un singolo documento (un file con un lotto
/// produce più voci, una per corpo).
#[derive(Debug, Clone, Serialize, Deserialize)]
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
    /// Nome del file .zip o .eml da cui proviene il documento (vuoto se è un file singolo).
    pub contenitore: String,
    pub allegati: usize,
    pub avvisi: usize,
    /// Controlli formali e di calcolo non superati (solo errori).
    pub controlli: usize,
    /// Testo normalizzato su cui eseguire la ricerca libera.
    pub testo: String,
}

/// Messaggio SdI trovato durante la scansione.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoceMessaggio {
    pub file: String,
    pub sigla: String,
    pub descrizione: String,
    pub riferito_a: String,
}

/// File che non è stato possibile interpretare.
#[derive(Debug, Clone, Serialize, Deserialize)]
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
    /// Cartelle non raggiungibili (es. NAS spento): si mostrano i dati dell'indice.
    pub non_raggiungibili: Vec<String>,
}

/// Estensioni prese in considerazione.
fn e_candidato(p: &Path) -> bool {
    let nome = p
        .file_name()
        .map(|n| n.to_string_lossy().to_lowercase())
        .unwrap_or_default();
    contenitori::e_documento(&nome) || contenitori::tipo_contenitore(&nome).is_some()
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
    s.to_lowercase()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

#[derive(Debug, Clone, Serialize, Deserialize)]
enum Letto {
    Voci(Vec<Voce>),
    Messaggio(VoceMessaggio),
    Scartato(FileScartato),
}

/// Legge un file: un documento singolo oppure tutti quelli di un contenitore.
fn leggi(percorso: &Path) -> Vec<Letto> {
    let file = percorso.to_string_lossy().to_string();
    let nome_file = percorso
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_default();
    let Some(tipo) = contenitori::tipo_contenitore(&nome_file) else {
        return vec![leggi_documento(file, nome_file, apri_file(percorso))];
    };
    let interni = std::fs::read(percorso)
        .map_err(|e| e.to_string())
        .and_then(|b| contenitori::espandi(tipo, &b));
    match interni {
        Ok(interni) => interni
            .into_iter()
            .map(|i| {
                let virtuale = format!("{file}{}{}", contenitori::SEPARATORE, i.percorso);
                leggi_documento(virtuale, i.nome, apri_byte(&i.byte))
            })
            .collect(),
        Err(motivo) => vec![Letto::Scartato(FileScartato { file, motivo })],
    }
}

fn leggi_documento(
    file: String,
    nome_file: String,
    letto: rustinvoice_core::Risultato<Documento>,
) -> Letto {
    let doc = match letto {
        Ok(d) => d,
        Err(e) => {
            return Letto::Scartato(FileScartato {
                file,
                motivo: e.to_string(),
            })
        }
    };
    let firmato = doc.firmato();
    // Documento dentro uno zip o un'email: nome del contenitore e catena dei livelli.
    let (disco, interni) = contenitori::dividi(&file);
    let (contenitore, catena) = if interni.is_empty() {
        (String::new(), String::new())
    } else {
        let nome_disco = Path::new(disco)
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_default();
        let catena = std::iter::once(nome_disco.as_str())
            .chain(interni.iter().copied())
            .collect::<Vec<_>>()
            .join(" ");
        (nome_disco, catena)
    };
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
            let esiti_controlli = rustinvoice_core::controlli::controlla(&f);
            let errori_controlli = |corpo: usize| {
                esiti_controlli
                    .iter()
                    .filter(|c| c.gravita == rustinvoice_core::controlli::Gravita::Errore)
                    .filter(|c| c.corpo.is_none() || c.corpo == Some(corpo))
                    .count()
            };
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
                        catena.clone(),
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
                        contenitore: contenitore.clone(),
                        allegati: c.allegati.len(),
                        avvisi: f.avvisi.len(),
                        controlli: errori_controlli(i),
                        testo: normalizza(&testo.join(" ")),
                    }
                })
                .collect();
            Letto::Voci(voci)
        }
    }
}

/// Indice locale dei file già letti, per non rileggere a ogni avvio quelli
/// non modificati (utile soprattutto con cartelle di rete).
#[derive(Default, Serialize, Deserialize)]
struct Indice {
    versione: String,
    file: HashMap<String, VoceIndice>,
}

#[derive(Clone, Serialize, Deserialize)]
struct VoceIndice {
    dimensione: u64,
    modificato: u128,
    letti: Vec<Letto>,
}

const NOME_INDICE: &str = "indice.json";

fn carica_indice(dir: &Path) -> Indice {
    let indice: Indice = std::fs::read(dir.join(NOME_INDICE))
        .ok()
        .and_then(|b| serde_json::from_slice(&b).ok())
        .unwrap_or_default();
    // Con una nuova versione del programma la lettura può essere cambiata: si riparte da zero.
    if indice.versione != env!("CARGO_PKG_VERSION") {
        return Indice::default();
    }
    indice
}

fn salva_indice(dir: &Path, indice: &Indice) {
    let Ok(b) = serde_json::to_vec(indice) else {
        return;
    };
    let tmp = dir.join(format!("{NOME_INDICE}.tmp"));
    if std::fs::write(&tmp, b).is_ok() {
        let _ = std::fs::rename(&tmp, dir.join(NOME_INDICE));
    }
}

/// Firma del file: dimensione e data di modifica.
fn impronta(p: &Path) -> Option<(u64, u128)> {
    let m = std::fs::metadata(p).ok()?;
    let t = m
        .modified()
        .ok()?
        .duration_since(std::time::UNIX_EPOCH)
        .ok()?
        .as_nanos();
    Some((m.len(), t))
}

/// Vero se `file` si trova sotto la cartella o coincide con il file indicato.
fn sotto(file: &str, radice: &str) -> bool {
    let r = radice.trim_end_matches(['/', '\\']);
    file == r
        || file
            .strip_prefix(r)
            .is_some_and(|resto| resto.starts_with('/') || resto.starts_with('\\'))
}

/// Legge in parallelo tutti i file indicati (o contenuti nelle cartelle),
/// riutilizzando l'indice per quelli non modificati.
pub fn scansiona(percorsi: &[String], dir_dati: &Path) -> Esito {
    let vecchio = carica_indice(dir_dati);
    let mut nuovo = Indice {
        versione: env!("CARGO_PKG_VERSION").to_owned(),
        file: HashMap::new(),
    };
    let mut esito = Esito::default();

    // Cartelle o file non raggiungibili: si conservano i dati già noti.
    let mut raggiungibili = Vec::new();
    for p in percorsi {
        if Path::new(p).exists() {
            raggiungibili.push(p.clone());
        } else {
            esito.non_raggiungibili.push(p.clone());
            for (k, v) in &vecchio.file {
                if sotto(k, p) {
                    nuovo.file.insert(k.clone(), v.clone());
                }
            }
        }
    }

    let file = elenca(&raggiungibili);
    let letti: Vec<(String, VoceIndice)> = file
        .par_iter()
        .map(|p| {
            let chiave = p.to_string_lossy().to_string();
            let (dimensione, modificato) = impronta(p).unwrap_or_default();
            if let Some(v) = vecchio.file.get(&chiave) {
                if v.dimensione == dimensione && v.modificato == modificato && modificato != 0 {
                    return (chiave, v.clone());
                }
            }
            (
                chiave,
                VoceIndice {
                    dimensione,
                    modificato,
                    letti: leggi(p),
                },
            )
        })
        .collect();
    nuovo.file.extend(letti);

    for v in nuovo.file.values() {
        for l in &v.letti {
            match l {
                Letto::Voci(x) => esito.voci.extend(x.iter().cloned()),
                Letto::Messaggio(m) => esito.messaggi.push(m.clone()),
                Letto::Scartato(s) => esito.scartati.push(s.clone()),
            }
        }
    }
    salva_indice(dir_dati, &nuovo);

    // Più recenti per primi.
    esito
        .voci
        .sort_by(|a, b| b.data.cmp(&a.data).then(a.numero.cmp(&b.numero)));
    esito.messaggi.sort_by(|a, b| a.file.cmp(&b.file));
    esito.scartati.sort_by(|a, b| a.file.cmp(&b.file));
    esito
}
