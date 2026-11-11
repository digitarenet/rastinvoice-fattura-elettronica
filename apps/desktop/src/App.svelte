<!--
  Copyright (c) 2026 Andrea Re - info@digitare.net
  Autore: Andrea Re @ 2026 - info@digitare.net

  Finestra principale: intestazione, menu laterale, elenco documenti e
  anteprima; rubrica, messaggi SdI e impostazioni.
-->
<script lang="ts">
  import { onMount, tick } from "svelte";
  import { getCurrentWebview } from "@tauri-apps/api/webview";
  import { open } from "@tauri-apps/plugin-dialog";
  import { api, type Avvio, type Esito, type Impostazioni as Imp, type StatoFoglio, type Voce } from "./lib/api";
  import {
    controparte,
    csv,
    direzione,
    filtra,
    indovinaPropri,
    type Ordinamento,
    type Sezione,
  } from "./lib/archivio";
  import { data, dataOra, importo, iniziali } from "./lib/formato";
  import { SIGLE_NOTE, chiaveFattura, indiceFatture, indiceNotifiche, infoSigla, notificheDi } from "./lib/sdi";
  import { modale } from "./lib/modale";
  import Icona from "./componenti/Icona.svelte";
  import Anteprima from "./componenti/Anteprima.svelte";
  import Rubrica from "./componenti/Rubrica.svelte";
  import Impostazioni from "./componenti/Impostazioni.svelte";
  import Credits from "./componenti/Credits.svelte";
  import SceltaDati from "./componenti/SceltaDati.svelte";
  import Statistiche from "./componenti/Statistiche.svelte";

  type Vista = "documenti" | "statistiche" | "rubrica" | "messaggi" | "impostazioni" | "credits";

  let info = $state<Avvio | null>(null);
  let imp = $state<Imp | null>(null);
  // Stato non profondo: l'elenco viene sostituito per intero a ogni lettura e
  // con migliaia di documenti evita il costo del tracciamento di ogni campo.
  let esito = $state.raw<Esito>({ voci: [], messaggi: [], scartati: [], non_raggiungibili: [] });
  let foglio = $state<StatoFoglio | null>(null);
  let scansione = $state(false);
  let vista = $state<Vista>("documenti");
  let avviso = $state<{ testo: string; tipo: "info" | "errore" } | null>(null);

  // Filtri dell'elenco.
  let sezione = $state<Sezione>("tutte");
  let anno = $state<number | null>(null);
  let ricerca = $state("");
  let contatto = $state<string | null>(null);
  let dal = $state("");
  let al = $state("");
  let ordinamento = $state<Ordinamento>("data_desc");
  let filtriAperti = $state(false);
  // Menu a scomparsa per finestre strette o interfaccia ingrandita.
  let menuAperto = $state(false);
  $effect(() => {
    void vista;
    void sezione;
    void anno;
    menuAperto = false;
  });

  let selezionata = $state<Voce | null>(null);
  let trascinamento = $state(false);
  let daConfermare = $state<string[] | null>(null);
  let ricordaScelta = $state(false);
  let campoRicerca = $state<HTMLInputElement | null>(null);
  let elencoEl = $state<HTMLElement | null>(null);

  const propri = $derived(new Set(imp?.identificativi_propri ?? []));
  const lette = $derived(new Set(imp?.lette ?? []));
  const contrassegnate = $derived(new Set(imp?.contrassegnate ?? []));

  const visibili = $derived(
    filtra(esito.voci, { sezione, anno, ricerca, contatto, dal, al, ordinamento }, propri, lette, contrassegnate),
  );

  // Elenco virtuale: si disegnano solo le righe visibili (più un margine),
  // così lo scorrimento resta fluido anche con decine di migliaia di documenti.
  const ALTEZZA_RIGA = 74;
  const RIGHE_MARGINE = 8;
  let scorrimento = $state(0);
  let altezzaElenco = $state(800);
  const primaRiga = $derived(Math.max(0, Math.floor(scorrimento / ALTEZZA_RIGA) - RIGHE_MARGINE));
  const ultimaRiga = $derived(
    Math.min(visibili.length, Math.ceil((scorrimento + altezzaElenco) / ALTEZZA_RIGA) + RIGHE_MARGINE),
  );
  const righeDisegnate = $derived(visibili.slice(primaRiga, ultimaRiga));

  // Notifiche SdI collegate alle fatture.
  const notifichePerFattura = $derived(indiceNotifiche(esito.messaggi));
  const fatturePerNome = $derived(indiceFatture(esito.voci));
  const notificheOrdinate = $derived([...esito.messaggi].sort((a, b) => b.data.localeCompare(a.data)));
  let legendaAperta = $state(false);
  let sceltaDatiFatta = $state(false);

  const anni = $derived([...new Set(esito.voci.map((v) => v.anno).filter((a) => a > 0))].sort((a, b) => b - a));
  const conteggi = $derived({
    tutte: esito.voci.length,
    ricevute: esito.voci.filter((v) => direzione(v, propri) === "ricevuta").length,
    emesse: esito.voci.filter((v) => direzione(v, propri) === "emessa").length,
    da_leggere: esito.voci.filter((v) => !lette.has(v.id)).length,
    contrassegnate: esito.voci.filter((v) => contrassegnate.has(v.id)).length,
    anomalie: esito.voci.filter((v) => v.avvisi > 0 || v.controlli > 0).length,
  });
  const totaleVisibili = $derived(
    visibili.reduce((s, v) => s + (v.nota_credito ? -1 : 1) * Math.abs(Number(v.totale) || 0), 0),
  );
  const suggerito = $derived(indovinaPropri(esito.voci));
  const nomeContatto = $derived.by(() => {
    if (!contatto) return "";
    const v = esito.voci.find((x) => x.cedente.chiave === contatto || x.cessionario.chiave === contatto);
    if (!v) return contatto;
    return v.cedente.chiave === contatto ? v.cedente.nome : v.cessionario.nome;
  });
  const vuoto = $derived(imp !== null && imp.cartelle.length === 0 && imp.file_singoli.length === 0);

  // Ingrandimento dell'interfaccia scelto nelle impostazioni.
  $effect(() => {
    const z = imp?.zoom ?? 1;
    getCurrentWebview()
      .setZoom(z)
      .catch(() => {});
  });

  onMount(() => {
    let smetti: (() => void) | undefined;
    (async () => {
      info = await api.avvio();
      imp = info.impostazioni;
      foglio = await api.foglioStato();
      if (imp.aggiornamenti) {
        // Controllo facoltativo, in silenzio se non c'è rete.
        api.controllaAggiornamenti()
          .then((e) => e.disponibile && mostra(`È disponibile la versione ${e.ultima}: scaricala da Impostazioni.`))
          .catch(() => {});
      }
      await scansiona();
      smetti = await getCurrentWebview().onDragDropEvent((e) => {
        const p = e.payload;
        if (p.type === "enter" || p.type === "over") trascinamento = true;
        else if (p.type === "leave") trascinamento = false;
        else if (p.type === "drop") {
          trascinamento = false;
          gestisciRilascio(p.paths);
        }
      });
    })().catch((e) => mostra(String(e), "errore"));
    return () => smetti?.();
  });

  function mostra(testo: string, tipo: "info" | "errore" = "info") {
    avviso = { testo, tipo };
    const corrente = avviso;
    setTimeout(() => {
      if (avviso === corrente) avviso = null;
    }, tipo === "errore" ? 8000 : 4000);
  }

  async function scansiona() {
    if (!imp) return;
    scansione = true;
    try {
      esito = await api.scansiona([...imp.cartelle, ...imp.file_singoli]);
      if (selezionata) selezionata = esito.voci.find((v) => v.id === selezionata!.id) ?? null;
    } catch (e) {
      mostra(String(e), "errore");
    } finally {
      scansione = false;
    }
  }

  async function salva(nuove: Imp, riscansiona = false) {
    imp = nuove;
    try {
      await api.salvaImpostazioni(nuove);
    } catch (e) {
      mostra(`Impossibile salvare le impostazioni: ${e}`, "errore");
    }
    if (riscansiona) await scansiona();
  }

  async function aggiungiCartella() {
    const scelta = await open({ directory: true, multiple: true });
    if (!scelta || !imp) return;
    const elenco = (Array.isArray(scelta) ? scelta : [scelta]).filter((c) => !imp!.cartelle.includes(c));
    if (elenco.length === 0) return;
    await salva({ ...imp, cartelle: [...imp.cartelle, ...elenco], benvenuto_visto: true }, true);
  }

  async function apriFile() {
    const scelta = await open({
      multiple: true,
      filters: [{ name: "Fatture elettroniche, archivi zip, email", extensions: ["xml", "p7m", "zip", "eml", "XML", "P7M", "ZIP", "EML"] }],
    });
    if (!scelta) return;
    gestisciRilascio(Array.isArray(scelta) ? scelta : [scelta]);
  }

  const eFattura = (p: string) => /\.(xml|p7m|zip|eml)$/i.test(p);

  function gestisciRilascio(percorsi: string[]) {
    if (!imp || percorsi.length === 0) return;
    const cartelle = percorsi.filter((p) => !eFattura(p));
    const file = percorsi.filter(eFattura);
    if (file.length > 0 && imp.file_trascinati === "chiedi") {
      daConfermare = file;
      if (cartelle.length > 0) aggiungiPercorsi(cartelle, [], false);
      return;
    }
    aggiungiPercorsi(cartelle, file, imp.file_trascinati === "copia");
  }

  async function aggiungiPercorsi(cartelle: string[], file: string[], copia: boolean) {
    if (!imp) return;
    let nuove = { ...imp, benvenuto_visto: true };
    nuove.cartelle = [...nuove.cartelle, ...cartelle.filter((c) => !nuove.cartelle.includes(c))];
    if (file.length > 0) {
      if (copia) {
        try {
          const copiati = await api.copiaInArchivio(file);
          const archivio = await api.cartellaArchivio();
          if (!nuove.cartelle.includes(archivio)) nuove.cartelle = [...nuove.cartelle, archivio];
          mostra(`${copiati.length} file copiati nell'archivio.`);
        } catch (e) {
          mostra(`Copia non riuscita: ${e}`, "errore");
        }
      } else {
        nuove.file_singoli = [...nuove.file_singoli, ...file.filter((f) => !nuove.file_singoli.includes(f))];
      }
    }
    await salva(nuove, true);
    vista = "documenti";
    // Mostra subito il primo file aggiunto.
    if (file.length > 0) {
      const nome = file[0].split(/[\\/]/).pop();
      const v = esito.voci.find((x) => x.nome_file === nome);
      if (v) seleziona(v);
    }
  }

  async function confermaRilascio(copia: boolean) {
    const file = daConfermare ?? [];
    daConfermare = null;
    if (ricordaScelta && imp) imp = { ...imp, file_trascinati: copia ? "copia" : "lascia" };
    await aggiungiPercorsi([], file, copia);
  }

  function seleziona(v: Voce) {
    selezionata = v;
    if (imp && !lette.has(v.id)) {
      salva({ ...imp, lette: [...imp.lette, v.id] });
    }
  }

  function contrassegna(v: Voce) {
    if (!imp) return;
    const insieme = new Set(imp.contrassegnate);
    if (insieme.has(v.id)) insieme.delete(v.id);
    else insieme.add(v.id);
    salva({ ...imp, contrassegnate: [...insieme] });
  }

  function segnaComeNonLetta(v: Voce) {
    if (!imp) return;
    salva({ ...imp, lette: imp.lette.filter((x) => x !== v.id) });
  }

  async function esportaCsv() {
    try {
      if (await api.salvaTesto("fatture.csv", "csv", csv(visibili, propri)))
        mostra(`Esportati ${visibili.length} documenti.`);
    } catch (e) {
      mostra(String(e), "errore");
    }
  }

  function azzeraFiltri() {
    sezione = "tutte";
    anno = null;
    ricerca = "";
    contatto = null;
    dal = "";
    al = "";
  }

  function vaiA(s: Sezione, a: number | null = null) {
    vista = "documenti";
    sezione = s;
    anno = a;
  }

  // Navigazione da tastiera.
  function tastiera(e: KeyboardEvent) {
    // Con una finestra in primo piano (visualizzatore allegati) i tasti sono suoi.
    if (document.querySelector('[aria-modal="true"]')) return;
    const ctrl = e.ctrlKey || e.metaKey;
    if (ctrl && e.key.toLowerCase() === "f") {
      e.preventDefault();
      vista = "documenti";
      campoRicerca?.focus();
      campoRicerca?.select();
      return;
    }
    if (ctrl && e.key.toLowerCase() === "o") {
      e.preventDefault();
      apriFile();
      return;
    }
    if (vista !== "documenti" || visibili.length === 0) return;
    const bersaglio = e.target as HTMLElement;
    if (bersaglio.tagName === "INPUT" || bersaglio.tagName === "SELECT") return;
    if (e.key === "ArrowDown" || e.key === "ArrowUp") {
      e.preventDefault();
      const i = selezionata ? visibili.findIndex((v) => v.id === selezionata!.id) : -1;
      const j = Math.max(0, Math.min(visibili.length - 1, i + (e.key === "ArrowDown" ? 1 : -1)));
      seleziona(visibili[j]);
      mostraRiga(j);
      // Il fuoco segue la selezione, così i lettori di schermo la annunciano.
      requestAnimationFrame(() =>
        tick().then(() => elencoEl?.querySelector<HTMLElement>('.voce[aria-current="true"]')?.focus()),
      );
    }
  }

  /** Mostra una fattura nell'elenco completo, selezionata. */
  async function apriFattura(v: Voce) {
    vista = "documenti";
    sezione = "tutte";
    anno = null;
    ricerca = "";
    contatto = null;
    dal = "";
    al = "";
    seleziona(v);
    await tick();
    const j = visibili.findIndex((x) => x.id === v.id);
    if (j >= 0 && elencoEl) elencoEl.scrollTop = Math.max(0, j * ALTEZZA_RIGA - elencoEl.clientHeight / 2);
  }

  /** Scorre l'elenco quanto basta per rendere visibile la riga indicata. */
  function mostraRiga(j: number) {
    if (!elencoEl) return;
    const sopra = j * ALTEZZA_RIGA;
    if (sopra < elencoEl.scrollTop) elencoEl.scrollTop = sopra;
    else if (sopra + ALTEZZA_RIGA > elencoEl.scrollTop + elencoEl.clientHeight)
      elencoEl.scrollTop = sopra + ALTEZZA_RIGA - elencoEl.clientHeight;
  }

  const etichette: Record<Sezione, string> = {
    tutte: "Tutti i documenti",
    ricevute: "Ricevute",
    emesse: "Emesse",
    da_leggere: "Da leggere",
    contrassegnate: "Contrassegnate",
    anomalie: "Con anomalie",
  };
  const icone: Record<Sezione, string> = {
    tutte: "it-files",
    ricevute: "it-inbox",
    emesse: "it-upload",
    da_leggere: "it-mail",
    contrassegnate: "it-star-outline",
    anomalie: "it-warning-circle",
  };
</script>

<svelte:window onkeydown={tastiera} />

{#if info && imp}
  <div class="app">
    <a class="salta visually-hidden-focusable" href="#contenuto" onclick={(e) => { e.preventDefault(); document.getElementById("contenuto")?.focus(); }}>Vai al contenuto principale</a>
    <header class="testata">
      <button class="btn btn-xs btn-testata menu-stretto" aria-expanded={menuAperto} aria-controls="menu-laterale"
        onclick={() => (menuAperto = !menuAperto)}><Icona nome="it-burger" /> Menu</button>
      <div class="marchio">
        <span class="logo"><Icona nome="it-file" /></span>
        <span class="nome">RustinVoice</span>
      </div>
      <div class="cerca">
        <Icona nome="it-search" />
        <input
          bind:this={campoRicerca}
          type="search"
          placeholder="Cerca per numero, soggetto, P.IVA, descrizione, importo... (Ctrl+F)"
          bind:value={ricerca}
          oninput={() => (vista = "documenti")}
          aria-label="Ricerca"
        />
      </div>
      <div class="comandi">
        <button class="btn btn-xs btn-testata" onclick={apriFile} title="Apri file (Ctrl+O)"><Icona nome="it-file" /> Apri file</button>
        <button class="btn btn-xs btn-testata" onclick={aggiungiCartella}><Icona nome="it-folder" /> Aggiungi cartella</button>
        <button class="btn btn-xs btn-testata" onclick={scansiona} title="Rileggi le cartelle" aria-label="Rileggi le cartelle" disabled={scansione}>
          <Icona nome="it-refresh" classe={scansione ? "gira" : ""} />
        </button>
      </div>
    </header>

    <div class="corpo">
      <nav class="laterale ri-pannello ri-scroll" id="menu-laterale" class:aperto={menuAperto} aria-label="Sezioni">
        <ul>
          <li>
            <button class:attivo={vista === "statistiche"} aria-current={vista === "statistiche" ? "page" : undefined} onclick={() => (vista = "statistiche")}>
              <Icona nome="it-chart-line" /> <span>Statistiche</span>
            </button>
          </li>
          <li>
            <button class:attivo={vista === "rubrica"} aria-current={vista === "rubrica" ? "page" : undefined} onclick={() => (vista = "rubrica")}>
              <Icona nome="it-user" /> <span>Clienti e fornitori</span>
            </button>
          </li>
        </ul>

        <h2 class="titoletto">Documenti</h2>
        <ul>
          {#each ["tutte", "ricevute", "emesse", "da_leggere", "contrassegnate", "anomalie"] as const as s (s)}
            {#if !((s === "ricevute" || s === "emesse") && propri.size === 0) && !(s === "anomalie" && conteggi.anomalie === 0)}
              <li>
                <button class:attivo={vista === "documenti" && sezione === s && anno === null} aria-current={vista === "documenti" && sezione === s && anno === null ? "page" : undefined} onclick={() => vaiA(s)}>
                  <Icona nome={icone[s]} /> <span>{etichette[s]}</span> <small>{conteggi[s]}</small>
                </button>
              </li>
            {/if}
          {/each}
        </ul>

        {#if anni.length > 0}
          <h2 class="titoletto">Anni</h2>
          <ul>
            {#each anni as a (a)}
              <li>
                <button class:attivo={vista === "documenti" && anno === a} aria-current={vista === "documenti" && anno === a ? "page" : undefined} onclick={() => vaiA(sezione, a)}>
                  <Icona nome="it-calendar" /> <span>{a}</span>
                  <small>{esito.voci.filter((v) => v.anno === a).length}</small>
                </button>
              </li>
            {/each}
          </ul>
        {/if}

        <h2 class="titoletto">Strumenti</h2>
        <ul>
          <li>
            <button class:attivo={vista === "messaggi"} aria-current={vista === "messaggi" ? "page" : undefined} onclick={() => (vista = "messaggi")}>
              <Icona nome="it-mail" /> <span>Notifiche SdI</span>
              <small>{esito.messaggi.filter((m) => m.sigla !== "MT").length || ""}</small>
            </button>
          </li>
          <li>
            <button class:attivo={vista === "impostazioni"} aria-current={vista === "impostazioni" ? "page" : undefined} onclick={() => (vista = "impostazioni")}>
              <Icona nome="it-settings" /> <span>Impostazioni</span>
            </button>
          </li>
          <li>
            <button class:attivo={vista === "credits"} aria-current={vista === "credits" ? "page" : undefined} onclick={() => (vista = "credits")}>
              <Icona nome="it-info-circle" /> <span>Credits</span>
            </button>
          </li>
        </ul>

        {#if propri.size === 0 && suggerito}
          <div class="suggerimento">
            I documenti sembrano di <strong>{suggerito.nome}</strong>. Vuoi separare ricevute ed emesse?
            <button class="btn btn-primary btn-xs mt-2" onclick={() => salva({ ...imp!, identificativi_propri: [suggerito.chiave] })}>Sì, sono io</button>
          </div>
        {/if}
      </nav>

      <main class="principale" id="contenuto" tabindex="-1">
        {#if vista === "documenti"}
          {#if vuoto}
            <div class="benvenuto">
              <div class="scheda-benvenuto ri-pannello">
                <h1 class="h3">Benvenuto in RustinVoice</h1>
                <p class="lead">Il visualizzatore di fatture elettroniche che funziona senza installazione e senza connessione.</p>
                <ol>
                  <li><strong>Scegli una cartella</strong> che contiene fatture .xml o .p7m, anche dentro archivi .zip o email PEC .eml: verrà letta con tutte le sottocartelle.</li>
                  <li><strong>Oppure trascina i file</strong> direttamente in questa finestra: potrai copiarli nell'archivio dell'app o lasciarli dove sono.</li>
                  <li><strong>Indica la tua partita IVA</strong> nelle impostazioni per separare le fatture ricevute da quelle emesse.</li>
                </ol>
                <p class="text-secondary-ri">I file originali non vengono mai modificati e nessun dato lascia il tuo computer.</p>
                <div class="d-flex gap-2 mt-3">
                  <button class="btn btn-primary" onclick={aggiungiCartella}><Icona nome="it-folder" classe="bianca" /> Scegli una cartella</button>
                  <button class="btn btn-outline-primary" onclick={apriFile}><Icona nome="it-file" /> Apri dei file</button>
                </div>
              </div>
            </div>
          {:else}
            <div class="documenti">
              <section class="elenco ri-pannello">
                <div class="intestazione-elenco">
                  <div class="d-flex align-items-center justify-content-between">
                    <h1 class="h6 m-0">
                      {etichette[sezione]}{anno ? ` · ${anno}` : ""}
                      <small class="text-secondary-ri fw-normal"> · {visibili.length}</small>
                    </h1>
                    <div class="d-flex">
                      <button class="btn btn-link btn-xs" class:attivo={filtriAperti} title="Filtri" aria-label="Filtri" aria-expanded={filtriAperti} onclick={() => (filtriAperti = !filtriAperti)}><Icona nome="it-funnel" /></button>
                      <button class="btn btn-link btn-xs" title="Esporta l'elenco in CSV" aria-label="Esporta l'elenco in CSV" onclick={esportaCsv}><Icona nome="it-download" /></button>
                    </div>
                  </div>
                  {#if contatto || ricerca || dal || al}
                    <div class="filtri-attivi">
                      {#if contatto}<span class="filtro">{nomeContatto}<button aria-label="Togli il filtro {nomeContatto}" onclick={() => (contatto = null)}><Icona nome="it-close" dimensione="xs" /></button></span>{/if}
                      {#if ricerca}<span class="filtro">"{ricerca}"<button aria-label="Togli la ricerca {ricerca}" onclick={() => (ricerca = "")}><Icona nome="it-close" dimensione="xs" /></button></span>{/if}
                      {#if dal || al}<span class="filtro">{dal ? data(dal) : "…"} – {al ? data(al) : "…"}<button aria-label="Togli il filtro per date" onclick={() => { dal = ""; al = ""; }}><Icona nome="it-close" dimensione="xs" /></button></span>{/if}
                      <button class="btn btn-link btn-xs p-0" onclick={azzeraFiltri}>Azzera</button>
                    </div>
                  {/if}
                  {#if filtriAperti}
                    <div class="pannello-filtri">
                      <label>Dal <input class="form-control form-control-sm" type="date" bind:value={dal} /></label>
                      <label>Al <input class="form-control form-control-sm" type="date" bind:value={al} /></label>
                      <label>Ordina
                        <select class="form-select form-select-sm" bind:value={ordinamento}>
                          <option value="data_desc">Data, più recenti</option>
                          <option value="data_asc">Data, meno recenti</option>
                          <option value="importo_desc">Importo, decrescente</option>
                          <option value="importo_asc">Importo, crescente</option>
                          <option value="controparte">Controparte A-Z</option>
                        </select>
                      </label>
                    </div>
                  {/if}
                </div>

                {#if esito.non_raggiungibili.length > 0}
                  <div class="irraggiungibili" role="status">
                    <Icona nome="it-warning-circle" />
                    <span>
                      {esito.non_raggiungibili.length === 1 ? "Cartella non raggiungibile" : "Cartelle non raggiungibili"}:
                      <strong class="selezionabile">{esito.non_raggiungibili.join(", ")}</strong>.
                      Sono mostrati i documenti letti l'ultima volta.
                    </span>
                    <button class="btn btn-link btn-xs p-0" onclick={scansiona}>Riprova</button>
                  </div>
                {/if}
                <ul class="voci ri-scroll" aria-label="Documenti" bind:this={elencoEl} bind:clientHeight={altezzaElenco}
                  onscroll={() => (scorrimento = elencoEl?.scrollTop ?? 0)}>
                  {#if primaRiga > 0}<li class="spazio" style="height: {primaRiga * ALTEZZA_RIGA}px" aria-hidden="true"></li>{/if}
                  {#each righeDisegnate as v, k (v.id)}
                    {@const altro = controparte(v, propri)}
                    {@const dir = direzione(v, propri)}
                    <li class="riga-elenco" aria-setsize={visibili.length} aria-posinset={primaRiga + k + 1}>
                      <button
                        class="voce"
                        aria-current={selezionata?.id === v.id ? "true" : undefined}
                        class:selezionata={selezionata?.id === v.id}
                        class:non-letta={!lette.has(v.id)}
                        onclick={() => seleziona(v)}
                        ondblclick={() => segnaComeNonLetta(v)}
                      >
                        <span class="avatar-ri" class:emessa={dir === "emessa"} aria-hidden="true">{iniziali(altro.nome)}</span>
                        {#if !lette.has(v.id)}<span class="visually-hidden">Da leggere.</span>{/if}
                        <span class="dati">
                          <span class="riga">
                            <span class="soggetto">{altro.nome || altro.chiave}</span>
                            <span class="totale" class:negativo={v.nota_credito}>{importo(v.totale, v.divisa)}</span>
                          </span>
                          <span class="riga secondaria">
                            <span>
                              {#if dir === "emessa"}<span class="dir">a</span>{:else if dir === "ricevuta"}<span class="dir">da</span>{/if}
                              <span title="{v.tipo_documento} · {v.descrizione_tipo}">{v.descrizione_tipo || v.tipo_documento}</span> n. {v.numero}
                              {#if v.corpi_nel_file > 1}<span title="Documento {v.corpo + 1} di {v.corpi_nel_file} nello stesso file"> · lotto {v.corpo + 1}/{v.corpi_nel_file}</span>{/if}
                            </span>
                            <span>{data(v.data)}</span>
                          </span>
                          <span class="segni">
                            {#if contrassegnate.has(v.id)}<Icona nome="it-star-full" dimensione="xs" classe="stella" titolo="Contrassegnata" />{/if}
                            {#if v.firmato}<Icona nome="it-file-signed" dimensione="xs" titolo="Firmata (.p7m)" />{/if}
                            {#if v.contenitore}<Icona nome={/\.eml$/i.test(v.contenitore) ? "it-mail" : "it-box"} dimensione="xs" titolo="Contenuta in {v.contenitore}" />{/if}
                            {#if v.allegati > 0}<Icona nome="it-clip" dimensione="xs" titolo="{v.allegati} allegati" />{/if}
                            {#if v.controlli > 0}<Icona nome="it-error" dimensione="xs" classe="errore-controllo" titolo="{v.controlli} controlli non superati" />{/if}
                            {#if v.avvisi > 0}<Icona nome="it-warning-circle" dimensione="xs" classe="anomalia" titolo="{v.avvisi} anomalie di lettura" />{/if}
                            {#if v.prima_scadenza}<small class="scadenza">scade {data(v.prima_scadenza)}</small>{/if}
                          </span>
                        </span>
                      </button>
                    </li>
                  {:else}
                    <li class="nessuno text-secondary-ri">
                      {scansione ? "Lettura in corso..." : "Nessun documento con questi filtri."}
                    </li>
                  {/each}
                  {#if ultimaRiga < visibili.length}<li class="spazio" style="height: {(visibili.length - ultimaRiga) * ALTEZZA_RIGA}px" aria-hidden="true"></li>{/if}
                </ul>
                <footer class="piede" aria-live="polite">
                  <span>{visibili.length} documenti</span>
                  <strong>{importo(totaleVisibili.toFixed(2))}</strong>
                </footer>
              </section>

              <section class="dettaglio">
                {#if selezionata}
                  <Anteprima
                    voce={selezionata}
                    contrassegnata={contrassegnate.has(selezionata.id)}
                    letta={lette.has(selezionata.id)}
                    onNonLetta={() => segnaComeNonLetta(selezionata!)}
                    schedaIniziale={imp.vista}
                    foglioPresente={foglio?.presente ?? false}
                    onContrassegna={() => contrassegna(selezionata!)}
                    onVistaCambiata={(v) => imp && imp.vista !== v && salva({ ...imp, vista: v })}
                    onApriImpostazioni={() => (vista = "impostazioni")}
                    onErrore={(m) => mostra(m, "errore")}
                    onAvviso={(m) => mostra(m)}
                    notifiche={notificheDi(selezionata, notifichePerFattura)}
                  />
                {:else}
                  <div class="nessuna-selezione text-secondary-ri">
                    <Icona nome="it-file" dimensione="lg" />
                    <p class="mt-3">Seleziona un documento per visualizzarlo.<br /><small>Frecce ↑ ↓ per scorrere l'elenco.</small></p>
                  </div>
                {/if}
              </section>
            </div>
          {/if}
        {:else if vista === "statistiche"}
          <Statistiche
            voci={esito.voci}
            {propri}
            onApriImpostazioni={() => (vista = "impostazioni")}
            onMostraContatto={(c) => {
              vaiA("tutte");
              contatto = c;
            }}
          />
        {:else if vista === "rubrica"}
          <Rubrica
            voci={esito.voci}
            {propri}
            onApriDocumenti={(c) => {
              azzeraFiltri();
              contatto = c;
              vista = "documenti";
            }}
            onApriDocumento={(v) => {
              azzeraFiltri();
              contatto = v.cedente.chiave && !propri.has(v.cedente.chiave) ? v.cedente.chiave : v.cessionario.chiave;
              vista = "documenti";
              seleziona(v);
            }}
          />
        {:else if vista === "messaggi"}
          <div class="messaggi ri-scroll">
            <h1 class="h4">Notifiche del Sistema di Interscambio</h1>
            <p class="text-secondary-ri mb-2">
              Dopo l'invio di una fattura, lo SdI risponde con una notifica: consegnata, scartata, non consegnata e così via.
              Qui sono raccolte quelle trovate nelle tue cartelle (file come <code>IT01234567890_00005_RC_001.xml</code>), collegate
              alla fattura a cui si riferiscono. Lo stato compare anche nell'anteprima della fattura.
            </p>
            <button class="btn btn-link btn-xs p-0 mb-3" onclick={() => (legendaAperta = !legendaAperta)} aria-expanded={legendaAperta}>
              {legendaAperta ? "Nascondi" : "Cosa significano"} le sigle
            </button>
            {#if legendaAperta}
              <dl class="legenda">
                {#each SIGLE_NOTE as l (l.sigla)}
                  <dt><span class="sigla sigla-{l.gravita}">{l.sigla}</span> {l.nome}</dt>
                  <dd>{l.spiegazione}</dd>
                {/each}
              </dl>
            {/if}
            {#if notificheOrdinate.length === 0}
              <p>Nessuna notifica trovata nelle cartelle.</p>
            {:else}
              <table class="table table-sm">
                <thead><tr><th>Data</th><th>Stato</th><th>Fattura</th><th>Dettagli</th></tr></thead>
                <tbody>
                  {#each notificheOrdinate as m (m.file)}
                    {@const i = infoSigla(m)}
                    {@const f = fatturePerNome.get(chiaveFattura(m))}
                    <tr>
                      <td class="text-nowrap">{dataOra(m.data)}</td>
                      <td><span class="sigla sigla-{i.gravita}" title={i.nome}>{m.sigla}</span> {i.stato}</td>
                      <td>
                        {#if f}
                          <button class="btn btn-link btn-xs p-0 text-start" onclick={() => apriFattura(f)}>
                            {f.descrizione_tipo || f.tipo_documento} n. {f.numero} del {data(f.data)}
                          </button>
                          <div class="text-secondary-ri piccolo">{f.cessionario.nome}</div>
                        {:else}
                          <span class="selezionabile">{m.riferito_a || "—"}</span>
                          <div class="text-secondary-ri piccolo">fattura non presente nelle cartelle</div>
                        {/if}
                      </td>
                      <td>
                        {#if m.note}<div>{m.note}</div>{/if}
                        {#each m.errori as e, j (j)}<div>Errore {e.codice}: {e.descrizione}</div>{/each}
                        <div class="text-secondary-ri piccolo percorso" title={m.file}>{m.file.split(/[\\/]/).pop()}</div>
                      </td>
                    </tr>
                  {/each}
                </tbody>
              </table>
            {/if}
            {#if esito.scartati.length > 0}
              <h2 class="h5 mt-4">File non riconosciuti</h2>
              <p class="text-secondary-ri">
                File trovati nelle cartelle con estensione da fattura (.xml, .p7m) ma che non sono né fatture né notifiche SdI
                leggibili. Di solito si possono ignorare.
              </p>
              <table class="table table-sm">
                <thead><tr><th>File</th><th>Motivo</th></tr></thead>
                <tbody>
                  {#each esito.scartati as s (s.file)}
                    <tr><td class="selezionabile percorso">{s.file}</td><td>{s.motivo}</td></tr>
                  {/each}
                </tbody>
              </table>
            {/if}
          </div>
        {:else if vista === "impostazioni"}
          <Impostazioni
            {imp}
            {info}
            {foglio}
            {suggerito}
            onSalva={salva}
            onFoglio={(s) => (foglio = s)}
            onAggiungiCartella={aggiungiCartella}
          />
        {:else if vista === "credits"}
          <Credits versione={info?.versione ?? ""} onErrore={(m) => mostra(m, "errore")} />
        {/if}
      </main>
    </div>
  </div>

  {#if trascinamento}
    <div class="rilascio" aria-hidden="true">
      <div>
        <Icona nome="it-upload" dimensione="lg" />
        <p class="h4 mt-3">Rilascia qui fatture .xml, .p7m, archivi .zip, email .eml o cartelle</p>
      </div>
    </div>
  {/if}

  {#if info && info.primo_avvio && !sceltaDatiFatta}
    <div class="velo" role="dialog" aria-modal="true" aria-labelledby="titolo-dati" use:modale>
      <div class="finestra ri-pannello">
        <h2 id="titolo-dati" class="h5">Dove vuoi salvare i dati di RustinVoice?</h2>
        <SceltaDati {info} etichettaConferma="Inizia" onFatto={() => (sceltaDatiFatta = true)} />
        <p class="text-secondary-ri mt-3 mb-0"><small>Si può cambiare in seguito dalle Impostazioni.</small></p>
      </div>
    </div>
  {/if}

  {#if daConfermare}
    <div class="velo" role="dialog" aria-modal="true" aria-labelledby="titolo-rilascio" use:modale={() => (daConfermare = null)}>
      <div class="finestra ri-pannello">
        <h2 id="titolo-rilascio" class="h5">Dove vuoi tenere {daConfermare.length === 1 ? "il file" : `i ${daConfermare.length} file`}?</h2>
        <p>
          <strong>Copia nell'archivio</strong>: i file vengono copiati nella cartella dati dell'app e restano disponibili anche
          se sposti o cancelli gli originali.<br />
          <strong>Lascia dove sono</strong>: l'app ricorda solo il percorso dei file originali.
        </p>
        <div class="form-check">
          <input id="ricorda" type="checkbox" bind:checked={ricordaScelta} />
          <label for="ricorda">Ricorda la scelta</label>
        </div>
        <div class="d-flex gap-2 justify-content-end mt-3">
          <button class="btn btn-outline-secondary btn-sm" onclick={() => (daConfermare = null)}>Annulla</button>
          <button class="btn btn-outline-primary btn-sm" onclick={() => confermaRilascio(false)}>Lascia dove sono</button>
          <button class="btn btn-primary btn-sm" onclick={() => confermaRilascio(true)}>Copia nell'archivio</button>
        </div>
      </div>
    </div>
  {/if}

  {#if avviso}
    <div class="notifica" class:errore={avviso.tipo === "errore"} role={avviso.tipo === "errore" ? "alert" : "status"}>
      {avviso.testo}
      <button aria-label="Chiudi l'avviso" onclick={() => (avviso = null)}><Icona nome="it-close" dimensione="xs" /></button>
    </div>
  {/if}
{:else}
  <div class="avvio" role="status"><div class="progress-spinner progress-spinner-active"></div><span class="visually-hidden">Caricamento in corso</span></div>
{/if}

<style>
  .app {
    display: flex;
    flex-direction: column;
    height: 100vh;
  }

  /* Intestazione blu in stile Designers Italia. */
  .testata {
    display: flex;
    align-items: center;
    gap: 1.25rem;
    background: var(--ri-primario);
    color: #fff;
    padding: 0 1rem;
    height: 60px;
    flex: none;
  }
  .btn-testata.menu-stretto {
    display: none;
  }
  .marchio {
    display: flex;
    align-items: center;
    gap: 0.6rem;
    width: 216px;
  }
  .logo {
    width: 34px;
    height: 34px;
    border-radius: 8px;
    background: #fff;
    display: flex;
    align-items: center;
    justify-content: center;
  }
  .logo :global(.icon) {
    fill: var(--ri-primario);
  }
  .nome {
    font-weight: 700;
    font-size: 1.25rem;
    letter-spacing: 0.01em;
  }
  .cerca {
    flex: 1;
    max-width: 680px;
    display: flex;
    align-items: center;
    gap: 0.5rem;
    background: rgba(0, 0, 0, 0.22);
    border-radius: 6px;
    padding: 0 0.75rem;
    height: 40px;
  }
  .cerca:focus-within {
    background: #fff;
  }
  .cerca :global(.icon) {
    fill: #fff;
  }
  .cerca:focus-within :global(.icon) {
    fill: var(--ri-primario);
  }
  .cerca input {
    flex: 1;
    border: 0;
    background: transparent;
    color: #fff;
    outline: none;
    font-size: 0.95rem;
    height: 100%;
    box-shadow: none;
  }
  .cerca:focus-within input {
    color: #1a1a1a;
  }
  .cerca input::placeholder {
    color: #fff;
    opacity: 1;
  }
  .cerca:focus-within input::placeholder {
    color: var(--ri-testo-2);
  }
  .comandi {
    display: flex;
    gap: 0.4rem;
    margin-left: auto;
  }
  .btn-testata {
    color: #fff;
    box-shadow: inset 0 0 0 1px rgba(255, 255, 255, 0.6);
    display: inline-flex;
    align-items: center;
    gap: 0.35rem;
  }
  .btn-testata:hover,
  .btn-testata[aria-expanded="true"] {
    background: rgba(0, 0, 0, 0.18);
    color: #fff;
  }
  .btn-testata :global(.icon) {
    fill: #fff;
  }
  :global(.gira) {
    animation: gira 1s linear infinite;
  }
  @keyframes gira {
    to {
      transform: rotate(360deg);
    }
  }

  .corpo {
    flex: 1;
    display: flex;
    min-height: 0;
  }

  /* Menu laterale. */
  .laterale {
    width: 232px;
    flex: none;
    border-right: 1px solid var(--ri-bordo);
    padding: 0.75rem 0;
  }
  .laterale ul {
    list-style: none;
    margin: 0;
    padding: 0;
  }
  .laterale ul button {
    display: flex;
    align-items: center;
    gap: 0.6rem;
    width: 100%;
    border: 0;
    background: none;
    color: var(--ri-testo);
    padding: 0.45rem 1rem 0.45rem 1.25rem;
    font-size: 0.95rem;
    text-align: left;
    /* Nessun bordo: l'indicatore della voce attiva è un'ombra interna, così
       le regole di focus di Bootstrap Italia non colorano bordi trasparenti. */
    border: 0 !important;
  }
  /* Dopo un clic del mouse nessun anello di focus; con la tastiera resta. */
  .laterale ul button:focus:not(:focus-visible) {
    box-shadow: none !important;
    outline: none !important;
  }
  .laterale ul button.attivo:focus:not(:focus-visible) {
    box-shadow: inset 4px 0 0 var(--ri-primario) !important;
  }
  .laterale ul button:hover {
    background: var(--ri-sfondo);
  }
  .laterale ul button.attivo {
    background: var(--ri-selezione);
    box-shadow: inset 4px 0 0 var(--ri-primario);
    color: var(--ri-primario);
    font-weight: 600;
  }
  .laterale ul button span {
    flex: 1;
  }
  .laterale ul button small {
    color: var(--ri-testo-2);
    font-size: 0.8rem;
  }
  .laterale :global(.icon) {
    fill: var(--ri-primario);
  }
  .titoletto {
    font-size: 0.75rem;
    text-transform: uppercase;
    letter-spacing: 0.06em;
    color: var(--ri-testo-2);
    margin: 1.25rem 1rem 0.4rem;
    font-weight: 600;
  }
  .suggerimento {
    margin: 1.25rem 0.75rem 0;
    padding: 0.75rem;
    border-radius: 8px;
    background: var(--ri-selezione);
    font-size: 0.85rem;
    line-height: 1.35;
  }

  .principale {
    flex: 1;
    min-width: 0;
    min-height: 0;
  }

  /* Elenco documenti. */
  .documenti {
    display: grid;
    grid-template-columns: minmax(340px, 420px) 1fr;
    height: 100%;
  }
  .elenco {
    display: flex;
    flex-direction: column;
    border-right: 1px solid var(--ri-bordo);
    min-height: 0;
  }
  .intestazione-elenco {
    padding: 0.6rem 0.75rem;
    border-bottom: 1px solid var(--ri-bordo);
  }
  .intestazione-elenco :global(.icon) {
    fill: var(--ri-primario);
  }
  .intestazione-elenco .attivo {
    background: var(--ri-selezione);
  }
  .filtri-attivi {
    display: flex;
    flex-wrap: wrap;
    gap: 0.35rem;
    margin-top: 0.4rem;
    align-items: center;
  }
  .filtro {
    display: inline-flex;
    align-items: center;
    gap: 0.2rem;
    background: var(--ri-selezione);
    border-radius: 999px;
    padding: 0.05rem 0.25rem 0.05rem 0.6rem;
    font-size: 0.8rem;
    max-width: 100%;
  }
  .filtro button {
    border: 0;
    background: none;
    padding: 0 0.15rem;
  }
  .pannello-filtri {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 0.5rem;
    margin-top: 0.5rem;
  }
  .pannello-filtri label {
    font-size: 0.8rem;
    color: var(--ri-testo-2);
    position: static;
    transform: none;
    width: auto;
  }
  .pannello-filtri label:last-child {
    grid-column: span 2;
  }
  .irraggiungibili {
    display: flex;
    gap: 0.5rem;
    align-items: flex-start;
    padding: 0.5rem 0.75rem;
    background: #fdf3e1;
    color: #5c3a00;
    font-size: 0.82rem;
    line-height: 1.35;
    border-bottom: 1px solid var(--ri-bordo);
  }
  .irraggiungibili :global(.icon) {
    fill: var(--ri-avviso);
    flex: none;
  }
  .irraggiungibili strong {
    word-break: break-all;
  }
  .voci {
    list-style: none;
    margin: 0;
    padding: 0;
    flex: 1;
    min-height: 0;
  }
  .voce {
    display: flex;
    gap: 0.65rem;
    width: 100%;
    border: 0;
    background: none;
    color: inherit;
    text-align: left;
    padding: 0.6rem 0.75rem 0.6rem 0.9rem;
    border: 0 !important;
  }
  .voci > li.riga-elenco {
    border-bottom: 1px solid var(--ri-bordo);
    /* Altezza fissa, uguale ad ALTEZZA_RIGA: serve all'elenco virtuale. */
    height: 74px;
    overflow: hidden;
  }
  .voci > li.riga-elenco > .voce {
    height: 73px;
  }
  .voce:focus:not(:focus-visible) {
    box-shadow: none !important;
    outline: none !important;
  }
  .voce.selezionata:focus:not(:focus-visible) {
    box-shadow: inset 3px 0 0 var(--ri-primario) !important;
  }
  /* Fuoco da tastiera ben visibile: la riga ha overflow nascosto, quindi
     l'indicatore va disegnato all'interno. */
  /* Bootstrap Italia disegna il fuoco all'esterno (qui nascosto dalla riga):
     per il fuoco da tastiera si usa un bordo interno. */
  .voci > li.riga-elenco > .voce:focus:not([data-focus-mouse="true"]) {
    box-shadow: inset 0 0 0 3px var(--ri-primario) !important;
  }
  .voce:hover {
    background: var(--ri-sfondo);
  }
  .voce.selezionata {
    background: var(--ri-selezione);
    box-shadow: inset 3px 0 0 var(--ri-primario);
  }
  .voce.non-letta .soggetto {
    font-weight: 700;
  }
  .voce.non-letta .avatar-ri::after {
    content: "";
    position: absolute;
    top: -1px;
    right: -1px;
    width: 10px;
    height: 10px;
    border-radius: 50%;
    background: var(--ri-primario);
    border: 2px solid var(--ri-pannello);
  }
  .avatar-ri {
    position: relative;
    flex: none;
    width: 36px;
    height: 36px;
    border-radius: 50%;
    background: #d9f2e6;
    color: #006644;
    display: flex;
    align-items: center;
    justify-content: center;
    font-weight: 700;
    font-size: 0.85rem;
  }
  .avatar-ri.emessa {
    background: #d6e8fa;
    color: var(--ri-primario-scuro);
  }
  .dati {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
    gap: 0.05rem;
    line-height: 1.3;
  }
  .riga {
    display: flex;
    justify-content: space-between;
    gap: 0.5rem;
    font-size: 0.93rem;
  }
  .soggetto {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    font-weight: 600;
  }
  .totale {
    font-weight: 600;
    font-variant-numeric: tabular-nums;
    white-space: nowrap;
  }
  .totale.negativo {
    color: var(--ri-errore);
  }
  .riga.secondaria {
    font-size: 0.82rem;
    color: var(--ri-testo-2);
  }
  .riga.secondaria > span:first-child {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .dir {
    font-style: italic;
  }
  .segni {
    display: flex;
    align-items: center;
    gap: 0.35rem;
    min-height: 1rem;
    line-height: 1;
  }
  .segni :global(.icon) {
    fill: var(--ri-testo-2);
  }
  .segni :global(.icon.stella) {
    fill: #a37400;
  }
  .segni :global(.icon.errore-controllo) {
    fill: var(--ri-errore);
  }
  .segni :global(.icon.anomalia) {
    fill: var(--ri-avviso);
  }
  .scadenza {
    color: var(--ri-testo-2);
    font-size: 0.75rem;
    margin-left: auto;
  }
  .nessuno {
    padding: 1.5rem 1rem;
    text-align: center;
  }
  .piede {
    display: flex;
    justify-content: space-between;
    padding: 0.5rem 0.75rem;
    border-top: 1px solid var(--ri-bordo);
    font-size: 0.85rem;
    color: var(--ri-testo-2);
  }
  .piede strong {
    color: var(--ri-testo);
    font-variant-numeric: tabular-nums;
  }
  .dettaglio {
    min-width: 0;
    min-height: 0;
  }
  .nessuna-selezione {
    height: 100%;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    text-align: center;
  }
  .nessuna-selezione :global(.icon) {
    fill: var(--ri-bordo);
    width: 64px;
    height: 64px;
  }

  /* Primo avvio. */
  .benvenuto {
    height: 100%;
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 2rem;
  }
  .scheda-benvenuto {
    max-width: 640px;
    padding: 2.5rem;
    border-radius: 12px;
    border: 1px solid var(--ri-bordo);
    box-shadow: 0 6px 24px rgba(0, 40, 80, 0.08);
  }
  .scheda-benvenuto ol {
    padding-left: 1.2rem;
  }
  .scheda-benvenuto li {
    margin-bottom: 0.5rem;
  }
  .scheda-benvenuto :global(.icon) {
    fill: currentColor;
  }
  .scheda-benvenuto .btn-outline-primary :global(.icon) {
    fill: var(--ri-primario);
  }

  /* Messaggi SdI. */
  .messaggi {
    height: 100%;
    padding: 2rem 2.5rem;
  }
  .messaggi .table {
    font-size: 0.9rem;
    color: var(--ri-testo);
  }
  .percorso {
    word-break: break-all;
  }
  .sigla {
    display: inline-block;
    min-width: 2.2rem;
    text-align: center;
    font-weight: 700;
    font-size: 0.8rem;
    border-radius: 4px;
    padding: 0.05rem 0.3rem;
    background: var(--ri-selezione);
    color: var(--ri-primario-scuro);
  }
  .sigla-errore {
    background: #f8d7dd;
    color: #8c1c30;
  }
  .sigla-ok {
    background: #d9f2e6;
    color: #006644;
  }
  .sigla-avviso {
    background: #fdecc8;
    color: #6b4000;
  }
  .legenda {
    display: grid;
    grid-template-columns: max-content 1fr;
    gap: 0.4rem 1rem;
    font-size: 0.88rem;
    background: var(--ri-pannello);
    border: 1px solid var(--ri-bordo);
    border-radius: 6px;
    padding: 0.75rem 1rem;
  }
  .legenda dt {
    font-weight: 600;
  }
  .legenda dd {
    margin: 0;
    color: var(--ri-testo-2);
  }
  .piccolo {
    font-size: 0.8rem;
  }

  /* Trascina e rilascia. */
  .rilascio {
    position: fixed;
    inset: 0;
    background: rgba(0, 102, 204, 0.88);
    color: #fff;
    display: flex;
    align-items: center;
    justify-content: center;
    text-align: center;
    z-index: 50;
    pointer-events: none;
    border: 6px dashed rgba(255, 255, 255, 0.7);
  }
  .rilascio :global(.icon) {
    fill: #fff;
    width: 72px;
    height: 72px;
  }
  .velo {
    position: fixed;
    inset: 0;
    background: rgba(0, 20, 40, 0.45);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 60;
  }
  .finestra {
    max-width: 560px;
    padding: 1.75rem;
    border-radius: 10px;
    box-shadow: 0 12px 40px rgba(0, 0, 0, 0.25);
  }
  .form-check {
    display: flex;
    align-items: center;
    gap: 0.5rem;
  }
  .form-check label {
    position: static;
    transform: none;
    font-size: 0.9rem;
    padding-left: 0;
  }
  .form-check input {
    position: static;
    opacity: 1;
    width: auto;
    height: auto;
  }

  .notifica {
    position: fixed;
    bottom: 1.25rem;
    left: 50%;
    transform: translateX(-50%);
    background: #17324d;
    color: #fff;
    padding: 0.6rem 0.9rem 0.6rem 1.1rem;
    border-radius: 8px;
    box-shadow: 0 6px 20px rgba(0, 0, 0, 0.25);
    z-index: 70;
    display: flex;
    gap: 0.75rem;
    align-items: center;
    max-width: 70vw;
    font-size: 0.92rem;
  }
  .notifica.errore {
    background: var(--ri-errore);
  }
  .notifica button {
    border: 0;
    background: none;
  }
  .notifica :global(.icon) {
    fill: #fff;
  }
  .avvio {
    height: 100vh;
    display: flex;
    align-items: center;
    justify-content: center;
  }

  /* Collegamento per saltare al contenuto: visibile solo con il fuoco. */
  .salta:focus {
    position: fixed;
    top: 0.5rem;
    left: 0.5rem;
    z-index: 3000;
    background: #fff;
    color: var(--ri-primario);
    padding: 0.5rem 1rem;
    border-radius: 4px;
    font-weight: 600;
  }
  .principale:focus {
    outline: none;
  }

  /* Finestra stretta o testo ingrandito (zoom al 200% e oltre): il menu va
     sopra il contenuto e l'elenco sopra l'anteprima, senza scorrimento
     orizzontale della pagina. */
  @media (max-width: 900px) {
    .testata {
      flex-wrap: wrap;
      height: auto;
      padding: 0.5rem 1rem;
      gap: 0.5rem 1rem;
    }
    .marchio {
      width: auto;
    }
    .cerca {
      order: 3;
      flex-basis: 100%;
      max-width: none;
    }
    .corpo {
      flex-direction: column;
    }
    .btn-testata.menu-stretto {
      display: inline-flex;
    }
    .laterale {
      display: none;
    }
    .laterale.aperto {
      display: block;
      width: auto;
      max-height: 60vh;
      border-right: 0;
      border-bottom: 1px solid var(--ri-bordo);
      padding: 0.25rem 0;
    }
    .laterale ul {
      display: flex;
      flex-wrap: wrap;
    }
    .laterale ul button {
      width: auto;
      padding: 0.35rem 0.75rem;
    }
    .titoletto {
      display: none;
    }
    /* Elenco e anteprima uno sotto l'altro, con scorrimento verticale. */
    .principale {
      overflow-y: auto;
    }
    .documenti {
      grid-template-columns: 1fr;
      grid-template-rows: 75vh 90vh;
      height: auto;
    }
    .elenco {
      border-right: 0;
      border-bottom: 1px solid var(--ri-bordo);
    }
  }
</style>
