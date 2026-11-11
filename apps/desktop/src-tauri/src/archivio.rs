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
    /// Imponibile e imposta (stringhe decimali), per le statistiche IVA.
    #[serde(default)]
    pub imponibile: String,
    #[serde(default)]
    pub imposta: String,
    /// IVA in scissione dei pagamenti: non incassata dal cedente.
    #[serde(default)]
    pub scissione: bool,
    /// Regime fiscale del cedente (`RF01` ordinario, `RF19` forfettario...).
    #[serde(default)]
    pub regime: String,
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
    /// Nome del file fattura a cui si riferisce la notifica.
    pub riferito_a: String,
    #[serde(default)]
    pub identificativo_sdi: String,
    /// Data e ora della notifica (consegna, se presente, altrimenti ricezione).
    #[serde(default)]
    pub data: String,
    /// Esito del committente (`EC01` accettata, `EC02` rifiutata).
    #[serde(default)]
    pub esito: String,
    /// Note o motivazione indicate nella notifica.
    #[serde(default)]
    pub note: String,
    #[serde(default)]
    pub errori: Vec<ErroreNotifica>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErroreNotifica {
    pub codice: String,
    pub descrizione: String,
    pub suggerimento: String,
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
    elenca_con_impronta(percorsi)
        .into_iter()
        .map(|(p, _)| p)
        .collect()
}

/// Dimensione e data di modifica (in nanosecondi) di un file.
type Impronta = (u64, u128);

/// Elenca i file candidati con la loro impronta. Le cartelle vengono visitate
/// in parallelo: su dischi di rete e NAS il tempo è dominato dall'attesa delle
/// risposte, e più richieste contemporanee lo riducono molto. I dati del file
/// arrivano dalla stessa lettura della cartella (su Windows senza ulteriori
/// accessi al disco).
fn elenca_con_impronta(percorsi: &[String]) -> Vec<(PathBuf, Impronta)> {
    let mut file: Vec<(PathBuf, Impronta)> = percorsi
        .par_iter()
        .flat_map(|p| {
            let p = PathBuf::from(p);
            if p.is_dir() {
                visita(&p, 0)
            } else if p.is_file() {
                let i = impronta(&p).unwrap_or_default();
                vec![(p, i)]
            } else {
                Vec::new()
            }
        })
        .collect();
    file.par_sort_unstable_by(|a, b| a.0.cmp(&b.0));
    file.dedup_by(|a, b| a.0 == b.0);
    file
}

/// Profondità massima delle sottocartelle, contro cicli e strutture anomale.
const PROFONDITA_MASSIMA: usize = 64;

fn visita(cartella: &Path, profondita: usize) -> Vec<(PathBuf, Impronta)> {
    let Ok(voci) = std::fs::read_dir(cartella) else {
        return Vec::new();
    };
    let mut file = Vec::new();
    let mut sottocartelle = Vec::new();
    for voce in voci.flatten() {
        // Il tipo non segue i collegamenti simbolici: non si esce dalla cartella.
        let Ok(tipo) = voce.file_type() else { continue };
        if tipo.is_dir() {
            if profondita < PROFONDITA_MASSIMA {
                sottocartelle.push(voce.path());
            }
        } else if tipo.is_file() {
            let percorso = voce.path();
            if e_candidato(&percorso) {
                let i = voce
                    .metadata()
                    .ok()
                    .and_then(|m| impronta_da(&m))
                    .unwrap_or_default();
                file.push((percorso, i));
            }
        }
    }
    if !sottocartelle.is_empty() {
        file.extend(
            sottocartelle
                .par_iter()
                .flat_map(|c| visita(c, profondita + 1))
                .collect::<Vec<_>>(),
        );
    }
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
            identificativo_sdi: m.identificativo_sdi.unwrap_or_default(),
            data: m
                .data_ora_consegna
                .or(m.data_ora_ricezione)
                .unwrap_or_default(),
            esito: m.esito.unwrap_or_default(),
            note: m.descrizione.unwrap_or_default(),
            errori: m
                .errori
                .into_iter()
                .map(|e| ErroreNotifica {
                    codice: e.codice.unwrap_or_default(),
                    descrizione: e.descrizione.unwrap_or_default(),
                    suggerimento: e.suggerimento.unwrap_or_default(),
                })
                .collect(),
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
                        imponibile: c.imponibile_e_imposta().0.to_string(),
                        imposta: c.imponibile_e_imposta().1.to_string(),
                        scissione: c.scissione_pagamenti(),
                        regime: h
                            .cedente_prestatore
                            .regime_fiscale
                            .clone()
                            .unwrap_or_default(),
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
fn impronta(p: &Path) -> Option<Impronta> {
    impronta_da(&std::fs::metadata(p).ok()?)
}

fn impronta_da(m: &std::fs::Metadata) -> Option<Impronta> {
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
    let mut vecchio = carica_indice(dir_dati).file;
    let mut nuovo = Indice {
        versione: env!("CARGO_PKG_VERSION").to_owned(),
        file: HashMap::with_capacity(vecchio.len()),
    };
    let mut esito = Esito::default();
    // L'indice si riscrive solo se qualcosa è cambiato.
    let mut modificato = false;

    // Cartelle o file non raggiungibili: si conservano i dati già noti.
    let mut raggiungibili = Vec::new();
    for p in percorsi {
        if Path::new(p).exists() {
            raggiungibili.push(p.clone());
        } else {
            esito.non_raggiungibili.push(p.clone());
            let chiavi: Vec<String> = vecchio.keys().filter(|k| sotto(k, p)).cloned().collect();
            for k in chiavi {
                if let Some(v) = vecchio.remove(&k) {
                    nuovo.file.insert(k, v);
                }
            }
        }
    }

    // I file non modificati si spostano dal vecchio indice senza copiarli;
    // gli altri si leggono in parallelo.
    let mut da_leggere = Vec::new();
    for (p, (dimensione, data)) in elenca_con_impronta(&raggiungibili) {
        let chiave = p.to_string_lossy().to_string();
        match vecchio.remove(&chiave) {
            Some(v) if v.dimensione == dimensione && v.modificato == data && data != 0 => {
                nuovo.file.insert(chiave, v);
            }
            _ => da_leggere.push((chiave, p, dimensione, data)),
        }
    }
    // File rimossi dalle cartelle: l'indice va aggiornato.
    modificato |= !vecchio.is_empty() || !da_leggere.is_empty();
    drop(vecchio);
    let letti: Vec<(String, VoceIndice)> = da_leggere
        .into_par_iter()
        .map(|(chiave, p, dimensione, modificato)| {
            (
                chiave,
                VoceIndice {
                    dimensione,
                    modificato,
                    letti: leggi(&p),
                },
            )
        })
        .collect();
    nuovo.file.extend(letti);

    if modificato {
        salva_indice(dir_dati, &nuovo);
    }
    for v in nuovo.file.into_values() {
        for l in v.letti {
            match l {
                Letto::Voci(x) => esito.voci.extend(x),
                Letto::Messaggio(m) => esito.messaggi.push(m),
                Letto::Scartato(s) => esito.scartati.push(s),
            }
        }
    }

    // Più recenti per primi.
    esito.voci.par_sort_unstable_by(|a, b| {
        b.data
            .cmp(&a.data)
            .then_with(|| a.numero.cmp(&b.numero))
            .then_with(|| a.id.cmp(&b.id))
    });
    esito.messaggi.sort_by(|a, b| a.file.cmp(&b.file));
    esito.scartati.sort_by(|a, b| a.file.cmp(&b.file));
    esito
}

#[cfg(test)]
mod prove {
    use super::*;

    /// Misura dei tempi di scansione su una cartella indicata dalla variabile
    /// RUSTINVOICE_CARTELLA_MISURA: prima lettura completa e lettura dall'indice.
    /// Eseguire con `cargo test --release -p rustinvoice-desktop misura -- --ignored --nocapture`.
    #[test]
    #[ignore]
    fn misura_scansione() {
        let Some(cartella) = std::env::var_os("RUSTINVOICE_CARTELLA_MISURA") else {
            return;
        };
        let cartella = cartella.to_string_lossy().to_string();
        let dati = std::env::temp_dir().join("rustinvoice-misura");
        let _ = std::fs::remove_dir_all(&dati);
        std::fs::create_dir_all(&dati).unwrap();
        for giro in ["prima lettura", "da indice", "da indice"] {
            let inizio = std::time::Instant::now();
            let esito = scansiona(std::slice::from_ref(&cartella), &dati);
            println!(
                "{giro}: {} voci in {:?}",
                esito.voci.len(),
                inizio.elapsed()
            );
        }
        let t = std::time::Instant::now();
        let indice = carica_indice(&dati);
        println!("  carica indice: {:?}", t.elapsed());
        let t = std::time::Instant::now();
        salva_indice(&dati, &indice);
        println!("  salva indice: {:?}", t.elapsed());
        let t = std::time::Instant::now();
        let file = elenca_con_impronta(std::slice::from_ref(&cartella));
        println!(
            "  elenca {} file con impronta: {:?}",
            file.len(),
            t.elapsed()
        );
        let esito = scansiona(std::slice::from_ref(&cartella), &dati);
        let t = std::time::Instant::now();
        let json = serde_json::to_string(&esito).unwrap();
        println!(
            "  esito in JSON per l'interfaccia: {:?}, {} KB",
            t.elapsed(),
            json.len() / 1024
        );
        let dimensione = std::fs::metadata(dati.join("indice.json"))
            .map(|m| m.len())
            .unwrap_or(0);
        println!("indice: {} KB", dimensione / 1024);
    }
}
