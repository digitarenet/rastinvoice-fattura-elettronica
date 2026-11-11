// Copyright (c) 2026 Andrea Re - info@digitare.net
// Autore: Andrea Re @ 2026 - info@digitare.net
//
// Script di compilazione: prepara il contesto di Tauri e, quando
// l'interfaccia viene incorporata nell'eseguibile, la ricompila se i
// sorgenti sono più recenti di `../dist`. Così `cargo run` e il pulsante
// Run dell'IDE mostrano sempre l'interfaccia aggiornata.

use std::path::Path;
use std::process::Command;
use std::time::SystemTime;

/// File e cartelle da cui dipende l'interfaccia compilata.
const SORGENTI: &[&str] = &[
    "../src",
    "../index.html",
    "../package.json",
    "../vite.config.ts",
    "../tsconfig.json",
    "../svelte.config.js",
    "../../../CHANGELOG.md",
];

fn main() {
    for s in SORGENTI {
        println!("cargo:rerun-if-changed={s}");
    }
    println!("cargo:rerun-if-changed=../dist/index.html");
    println!("cargo:rerun-if-env-changed=RUSTINVOICE_SENZA_NPM");

    // Con la funzionalità "interfaccia-inclusa" (predefinita) l'interfaccia
    // viene incorporata: va compilata se manca o non è aggiornata.
    // RUSTINVOICE_SENZA_NPM=1 disattiva la compilazione automatica.
    if std::env::var_os("CARGO_FEATURE_INTERFACCIA_INCLUSA").is_some()
        && std::env::var_os("RUSTINVOICE_SENZA_NPM").is_none()
        && da_ricompilare()
    {
        compila_interfaccia();
    }
    tauri_build::build()
}

/// Vero se `dist` manca o qualche sorgente è stato modificato dopo.
fn da_ricompilare() -> bool {
    let Some(compilata) = data_modifica(Path::new("../dist/index.html")) else {
        return true;
    };
    SORGENTI
        .iter()
        .filter_map(|s| piu_recente(Path::new(s)))
        .any(|t| t > compilata)
}

fn data_modifica(p: &Path) -> Option<SystemTime> {
    std::fs::metadata(p).and_then(|m| m.modified()).ok()
}

/// Data di modifica più recente di un file o, ricorsivamente, di una cartella.
fn piu_recente(p: &Path) -> Option<SystemTime> {
    let mut massima = data_modifica(p);
    if p.is_dir() {
        for voce in std::fs::read_dir(p).into_iter().flatten().flatten() {
            if let Some(t) = piu_recente(&voce.path()) {
                massima = Some(massima.map_or(t, |m| m.max(t)));
            }
        }
    }
    massima
}

fn compila_interfaccia() {
    // Su Windows npm è uno script .cmd e va avviato con il suo nome completo.
    let npm = if cfg!(windows) { "npm.cmd" } else { "npm" };
    let esegui = |argomenti: &[&str]| {
        println!(
            "cargo:warning=Compilazione dell'interfaccia: npm {}",
            argomenti.join(" ")
        );
        match Command::new(npm).args(argomenti).current_dir("..").status() {
            Ok(s) if s.success() => {}
            Ok(s) => panic!(
                "\n\nnpm {} terminato con errore ({s}).\n\n",
                argomenti.join(" ")
            ),
            Err(e) => panic!(
                "\n\nImpossibile avviare npm ({e}).\n\
                 Installare Node.js oppure compilare l'interfaccia a mano,\n\
                 dalla cartella apps/desktop:\n\
                 \n    npm install\n    npm run build\n\n"
            ),
        }
    };
    if !Path::new("../node_modules").exists() {
        esegui(&["install"]);
    }
    esegui(&["run", "build"]);
}
