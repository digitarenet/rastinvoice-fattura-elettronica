<!--
  Copyright (c) 2026 Andrea Re - info@digitare.net
  Autore: Andrea Re @ 2026 - info@digitare.net

  Anteprima del documento selezionato: vista Moderna, foglio di stile
  standard, XML originale e allegati.
-->
<script lang="ts">
  import { untrack } from "svelte";
  import { save } from "@tauri-apps/plugin-dialog";
  import { openPath, revealItemInDir } from "@tauri-apps/plugin-opener";
  import { api, type Dettaglio, type Voce } from "../lib/api";
  import * as xslt from "../lib/xslt";
  import { formatta, type Riga } from "../lib/xmlformato";
  import Icona from "./Icona.svelte";

  type Scheda = "moderna" | "standard" | "xml";

  let {
    voce,
    contrassegnata,
    schedaIniziale = "moderna",
    foglioPresente,
    onContrassegna,
    onVistaCambiata,
    onApriImpostazioni,
    onErrore,
  }: {
    voce: Voce;
    contrassegnata: boolean;
    schedaIniziale?: "moderna" | "standard";
    foglioPresente: boolean;
    onContrassegna: () => void;
    onVistaCambiata: (v: "moderna" | "standard") => void;
    onApriImpostazioni: () => void;
    onErrore: (m: string) => void;
  } = $props();

  let scheda = $state<Scheda>("moderna");
  let dettaglio = $state<Dettaglio | null>(null);
  let caricamento = $state(false);
  let errore = $state("");
  let htmlStandard = $state("");
  let erroreStandard = $state("");
  let cornice = $state<HTMLIFrameElement | null>(null);
  let foglioXsl: string | null = null;

  $effect(() => {
    scheda = schedaIniziale;
  });

  // Ricarica il dettaglio quando cambia il documento selezionato.
  $effect(() => {
    const { file, corpo } = voce;
    let annullato = false;
    caricamento = true;
    errore = "";
    api
      .dettaglio(file, corpo)
      .then((d) => {
        if (annullato) return;
        dettaglio = d;
      })
      .catch((e) => {
        if (!annullato) errore = String(e);
      })
      .finally(() => {
        if (!annullato) caricamento = false;
      });
    return () => {
      annullato = true;
    };
  });

  // Applica il foglio di stile standard solo quando serve. La trasformazione
  // è legata al dettaglio da cui nasce: cambiando documento si rifà, senza
  // mai mostrare il risultato di un documento precedente.
  let standardPer: Dettaglio | null = null;
  let tentativoFoglio = $state(0);
  $effect(() => {
    void tentativoFoglio;
    const d = dettaglio;
    if (scheda !== "standard" || !d || standardPer === d) return;
    standardPer = d;
    erroreStandard = "";
    (async () => {
      try {
        if (!xslt.disponibile()) throw new Error("il motore XSLT non è disponibile in questo sistema");
        foglioXsl ??= await api.foglioLeggi();
        if (!foglioXsl) throw new Error("foglio di stile non presente");
        const html = xslt.trasforma(d.xml, foglioXsl);
        if (dettaglio === d) htmlStandard = html;
      } catch (e) {
        if (dettaglio === d) {
          htmlStandard = "";
          erroreStandard = String(e instanceof Error ? e.message : e);
        }
      }
    })();
  });

  // Se il foglio viene scaricato dopo, al prossimo tentativo va riletto.
  $effect(() => {
    if (foglioPresente) {
      foglioXsl = null;
      standardPer = null;
      untrack(() => tentativoFoglio++);
    }
  });

  function cambiaScheda(s: Scheda) {
    scheda = s;
    if (s !== "xml") onVistaCambiata(s);
  }

  const allegati = $derived(dettaglio?.fattura.corpi[dettaglio.fattura.corpi.length > 1 ? voce.corpo : 0]?.allegati ?? []);

  // Zoom delle viste HTML, ricordato tra una sessione e l'altra.
  const LIVELLI_ZOOM = [0.5, 0.67, 0.75, 0.8, 0.9, 1, 1.1, 1.25, 1.5, 1.75, 2];
  let zoom = $state(leggiZoom());

  function leggiZoom(): number {
    try {
      const z = Number(localStorage.getItem("rustinvoice.zoom"));
      return LIVELLI_ZOOM.includes(z) ? z : 1;
    } catch {
      return 1;
    }
  }

  function cambiaZoom(verso: -1 | 0 | 1) {
    if (verso === 0) zoom = 1;
    else {
      const i = LIVELLI_ZOOM.indexOf(zoom);
      zoom = LIVELLI_ZOOM[Math.max(0, Math.min(LIVELLI_ZOOM.length - 1, i + verso))];
    }
    try {
      localStorage.setItem("rustinvoice.zoom", String(zoom));
    } catch {
      // Memorizzazione non disponibile: lo zoom vale solo per questa sessione.
    }
  }

  function applicaZoom(valore = zoom) {
    const radice = cornice?.contentDocument?.documentElement;
    if (radice) radice.style.zoom = String(valore);
  }

  $effect(() => {
    applicaZoom(zoom);
  });

  function tastiZoom(e: KeyboardEvent) {
    if (!(e.ctrlKey || e.metaKey)) return;
    if (e.key === "+" || e.key === "=") cambiaZoom(1);
    else if (e.key === "-") cambiaZoom(-1);
    else if (e.key === "0") cambiaZoom(0);
    else return;
    e.preventDefault();
  }

  // Riepilogo della verifica delle firme.
  let firmeAperte = $state(false);
  const esitoFirme = $derived.by(() => {
    const f = dettaglio?.firme ?? [];
    if (f.length === 0) return "non_verificabile";
    if (f.some((x) => x.esito === "non_valida")) return "non_valida";
    if (f.every((x) => x.esito === "valida")) return "valida";
    return "non_verificabile";
  });
  const etichettaFirme = $derived.by(() => {
    const f = dettaglio?.firme ?? [];
    const nomi = [...new Set(f.map((x) => x.nome).filter(Boolean))].join(", ");
    const base =
      esitoFirme === "valida" ? "Firma valida" : esitoFirme === "non_valida" ? "Firma NON valida" : "Firma non verificabile";
    return nomi ? `${base} · ${nomi}` : base;
  });

  function dataOra(s: string): string {
    // "2026-11-03 10:15:00" → "03/11/2026 10:15"
    return s.length >= 16 ? `${s.slice(8, 10)}/${s.slice(5, 7)}/${s.slice(0, 4)} ${s.slice(11, 16)}` : s;
  }

  // XML formattato per la lettura (il file resta com'è).
  let xmlFormattato = $state(true);
  const righeXml = $derived<Riga[] | null>(dettaglio && scheda === "xml" && xmlFormattato ? formatta(dettaglio.xml) : null);

  function stampa() {
    // Si stampa sempre a grandezza naturale.
    applicaZoom(1);
    cornice?.contentWindow?.focus();
    cornice?.contentWindow?.print();
    applicaZoom();
  }

  async function salvaHtml() {
    const html = scheda === "standard" ? htmlStandard : dettaglio?.html;
    if (!html) return;
    const base = voce.nome_file.replace(/\.(xml|p7m)$/gi, "").replace(/\.xml$/i, "");
    const percorso = await save({
      defaultPath: `${base}.html`,
      filters: [{ name: "Pagina HTML", extensions: ["html"] }],
    });
    if (percorso) await api.scriviTesto(percorso, html).catch((e) => onErrore(String(e)));
  }

  async function salvaXml() {
    if (!dettaglio) return;
    const base = voce.nome_file.replace(/\.p7m$/i, "");
    const percorso = await save({
      defaultPath: base.toLowerCase().endsWith(".xml") ? base : `${base}.xml`,
      filters: [{ name: "File XML", extensions: ["xml"] }],
    });
    if (percorso) await api.scriviTesto(percorso, dettaglio.xml).catch((e) => onErrore(String(e)));
  }

  async function apriAllegato(i: number) {
    try {
      const p = await api.apriAllegato(voce.file, voce.corpo, i);
      await openPath(p);
    } catch (e) {
      onErrore(String(e));
    }
  }

  async function salvaAllegato(i: number) {
    try {
      const nome = await api.nomeAllegato(voce.file, voce.corpo, i);
      const percorso = await save({ defaultPath: nome });
      if (percorso) await api.salvaAllegato(voce.file, voce.corpo, i, percorso);
    } catch (e) {
      onErrore(String(e));
    }
  }

  function dimensione(n: number): string {
    if (n < 1024) return `${n} B`;
    if (n < 1024 * 1024) return `${(n / 1024).toFixed(0)} kB`;
    return `${(n / 1024 / 1024).toFixed(1)} MB`;
  }
</script>

<svelte:window onkeydown={tastiZoom} />

<section class="anteprima ri-pannello">
  <div class="barra">
    <div class="schede" role="tablist">
      <button role="tab" class:attiva={scheda === "moderna"} aria-selected={scheda === "moderna"} onclick={() => cambiaScheda("moderna")}>Moderna</button>
      <button role="tab" class:attiva={scheda === "standard"} aria-selected={scheda === "standard"} onclick={() => cambiaScheda("standard")}>AssoSoftware</button>
      <button role="tab" class:attiva={scheda === "xml"} aria-selected={scheda === "xml"} onclick={() => cambiaScheda("xml")}>XML</button>
    </div>
    <div class="azioni">
      <div class="zoom" role="group" aria-label="Zoom">
        <button class="btn btn-xs btn-link" title="Riduci (Ctrl -)" onclick={() => cambiaZoom(-1)} disabled={zoom <= LIVELLI_ZOOM[0]}>−</button>
        <button class="btn btn-xs btn-link percentuale" title="Dimensione reale (Ctrl 0)" onclick={() => cambiaZoom(0)}>{Math.round(zoom * 100)}%</button>
        <button class="btn btn-xs btn-link" title="Ingrandisci (Ctrl +)" onclick={() => cambiaZoom(1)} disabled={zoom >= LIVELLI_ZOOM[LIVELLI_ZOOM.length - 1]}>+</button>
      </div>
      {#if scheda === "xml"}
        <button class="btn btn-xs btn-link interruttore" aria-pressed={xmlFormattato} title="Alterna tra XML formattato e testo originale del file"
          onclick={() => (xmlFormattato = !xmlFormattato)}>{xmlFormattato ? "Mostra originale" : "Formatta"}</button>
      {/if}
      <button class="btn btn-xs btn-link" title={contrassegnata ? "Togli contrassegno" : "Contrassegna"} onclick={onContrassegna}>
        <Icona nome={contrassegnata ? "it-star-full" : "it-star-outline"} classe={contrassegnata ? "stella" : ""} />
      </button>
      {#if scheda !== "xml"}
        <button class="btn btn-xs btn-link" title="Stampa (Ctrl+P)" onclick={stampa}>
          <Icona nome="it-print" />
        </button>
        <button class="btn btn-xs btn-link" title="Salva come pagina HTML" onclick={salvaHtml}>
          <Icona nome="it-download" />
        </button>
      {/if}
      <button class="btn btn-xs btn-link" title="Salva l'XML estratto" onclick={salvaXml}>
        <Icona nome="it-file-xml" />
      </button>
      <button class="btn btn-xs btn-link" title="Mostra il file nella cartella" onclick={() => revealItemInDir(voce.file.split("!/")[0])}>
        <Icona nome="it-folder" />
      </button>
    </div>
  </div>

  {#if dettaglio && (dettaglio.firmato || allegati.length > 0)}
    <div class="info">
      {#if dettaglio.firmato}
        <button class="chip-ri firma-{esitoFirme}" onclick={() => (firmeAperte = !firmeAperte)} aria-expanded={firmeAperte}
          title="Mostra i dettagli della firma">
          <Icona nome={esitoFirme === "non_valida" ? "it-warning-circle" : "it-file-signed"} dimensione="xs" />
          {etichettaFirme}
        </button>
      {/if}
      {#each allegati as a, i (i)}
        <span class="chip-ri allegato">
          <Icona nome="it-clip" dimensione="xs" />
          <span class="nome" title={a.descrizione ?? ""}>{a.nome ?? `allegato ${i + 1}`}</span>
          <span class="text-secondary-ri">{dimensione(a.dimensione)}</span>
          <button class="btn btn-link p-0" title="Apri" onclick={() => apriAllegato(i)}><Icona nome="it-external-link" dimensione="xs" /></button>
          <button class="btn btn-link p-0" title="Salva" onclick={() => salvaAllegato(i)}><Icona nome="it-download" dimensione="xs" /></button>
        </span>
      {/each}
    </div>
  {/if}

  {#if dettaglio && firmeAperte && dettaglio.firme.length > 0}
    <div class="pannello-firme">
      {#each dettaglio.firme as f, i (i)}
        <div class="firma-voce firma-{f.esito}">
          <div class="d-flex justify-content-between gap-3">
            <strong>{f.nome || "Firmatario sconosciuto"}</strong>
            <span class="esito">{f.esito === "valida" ? "Firma valida" : f.esito === "non_valida" ? "Firma NON valida" : "Non verificabile"}</span>
          </div>
          <div>{f.dettaglio}</div>
          <dl>
            {#if f.codice_fiscale}<dt>Codice fiscale</dt><dd class="selezionabile">{f.codice_fiscale}</dd>{/if}
            {#if f.organizzazione}<dt>Organizzazione</dt><dd>{f.organizzazione}</dd>{/if}
            {#if f.data_firma}<dt>Firmato il</dt><dd>{dataOra(f.data_firma)} (UTC)</dd>{/if}
            {#if f.emittente}<dt>Certificatore</dt><dd>{f.emittente}</dd>{/if}
            {#if f.valido_dal}<dt>Certificato valido</dt><dd>dal {dataOra(f.valido_dal)} al {dataOra(f.valido_al)}
              {#if !f.certificato_in_corso_di_validita}<span class="text-danger"> · non valido alla data della firma</span>{/if}</dd>{/if}
            {#if f.algoritmo}<dt>Algoritmo</dt><dd>{f.algoritmo}</dd>{/if}
            {#if dettaglio.livelli_firma > 1}<dt>Busta</dt><dd>{f.livello} di {dettaglio.livelli_firma}</dd>{/if}
          </dl>
        </div>
      {/each}
      <p class="nota">
        La verifica controlla l'integrità del documento e la corrispondenza con il certificato del firmatario. Non
        controlla ancora l'accreditamento del certificatore né la revoca del certificato.
      </p>
    </div>
  {/if}

  <div class="contenuto">
    {#if caricamento && !dettaglio}
      <div class="stato"><div class="progress-spinner progress-spinner-active"></div></div>
    {:else if errore}
      <div class="stato">
        <div class="alert alert-danger" role="alert">Impossibile aprire il documento: {errore}</div>
      </div>
    {:else if dettaglio}
      {#if scheda === "moderna"}
        <iframe bind:this={cornice} title="Fattura" srcdoc={dettaglio.html} sandbox="allow-same-origin allow-modals" onload={() => applicaZoom()}></iframe>
      {:else if scheda === "standard"}
        {#if htmlStandard}
          <iframe bind:this={cornice} title="Fattura (foglio di stile)" srcdoc={htmlStandard} sandbox="allow-same-origin allow-modals" onload={() => applicaZoom()}></iframe>
        {:else if erroreStandard}
          <div class="stato">
            <div class="vuoto">
              <Icona nome="it-file" dimensione="lg" />
              <h3 class="h5 mt-3">Foglio di stile non disponibile</h3>
              <p class="text-secondary-ri">
                {#if !foglioPresente}
                  Il foglio di stile ufficiale AssoSoftware si scarica dal sito dell'associazione, oppure si importa da un
                  file .xsl o .zip.
                {:else}
                  {erroreStandard}
                {/if}
              </p>
              <button class="btn btn-primary btn-sm" onclick={onApriImpostazioni}>Vai alle impostazioni</button>
            </div>
          </div>
        {:else}
          <div class="stato"><div class="progress-spinner progress-spinner-active"></div></div>
        {/if}
      {:else}
        <pre class="xml selezionabile" class:a-capo={!righeXml} style="font-size: {0.8 * zoom}rem">{#if righeXml}{#each righeXml as r, i (i)}<span class="riga-xml" style="padding-left: {r.rientro * 1.5}em">{#each r.parti as p, j (j)}<span class="x-{p.tipo}">{p.testo}</span>{/each}</span>
{/each}{:else}{dettaglio.xml}{/if}</pre>
      {/if}
    {/if}
  </div>
</section>

<style>
  .anteprima {
    display: flex;
    flex-direction: column;
    height: 100%;
    min-width: 0;
  }
  .barra {
    display: flex;
    align-items: center;
    justify-content: space-between;
    border-bottom: 1px solid var(--ri-bordo);
    padding: 0 0.75rem;
    min-height: 48px;
  }
  .schede {
    display: flex;
    gap: 0.25rem;
    align-self: stretch;
  }
  .schede button {
    border: 0;
    background: none;
    color: var(--ri-testo-2);
    font-weight: 600;
    padding: 0 0.85rem;
    border-bottom: 3px solid transparent;
    font-size: 0.95rem;
  }
  .schede button.attiva {
    color: var(--ri-primario);
    border-bottom-color: var(--ri-primario);
  }
  .azioni {
    display: flex;
    gap: 0.1rem;
  }
  .azioni :global(.icon) {
    fill: var(--ri-primario);
  }
  .azioni :global(.stella) {
    fill: #e8a400;
  }
  .info {
    display: flex;
    flex-wrap: wrap;
    gap: 0.5rem;
    padding: 0.5rem 0.75rem;
    border-bottom: 1px solid var(--ri-bordo);
    background: var(--ri-sfondo);
  }
  .chip-ri {
    display: inline-flex;
    align-items: center;
    gap: 0.35rem;
    font-size: 0.82rem;
    border: 1px solid var(--ri-bordo);
    border-radius: 999px;
    padding: 0.15rem 0.6rem;
    background: var(--ri-pannello);
  }
  button.chip-ri {
    cursor: pointer;
    font-weight: 600;
  }
  .chip-ri.firma-valida {
    color: var(--ri-successo);
    border-color: color-mix(in srgb, var(--ri-successo) 40%, transparent);
  }
  .chip-ri.firma-valida :global(.icon) {
    fill: var(--ri-successo);
  }
  .chip-ri.firma-non_valida {
    color: var(--ri-errore);
    border-color: var(--ri-errore);
    background: color-mix(in srgb, var(--ri-errore) 8%, var(--ri-pannello));
  }
  .chip-ri.firma-non_valida :global(.icon) {
    fill: var(--ri-errore);
  }
  .chip-ri.firma-non_verificabile {
    color: var(--ri-avviso);
  }
  .chip-ri.firma-non_verificabile :global(.icon) {
    fill: var(--ri-avviso);
  }
  .pannello-firme {
    padding: 0.75rem;
    border-bottom: 1px solid var(--ri-bordo);
    background: var(--ri-sfondo);
    font-size: 0.88rem;
    max-height: 40%;
    overflow: auto;
  }
  .firma-voce {
    background: var(--ri-pannello);
    border: 1px solid var(--ri-bordo);
    border-left: 4px solid var(--ri-avviso);
    border-radius: 6px;
    padding: 0.6rem 0.8rem;
    margin-bottom: 0.5rem;
  }
  .firma-voce.firma-valida {
    border-left-color: var(--ri-successo);
  }
  .firma-voce.firma-non_valida {
    border-left-color: var(--ri-errore);
  }
  .firma-valida .esito {
    color: var(--ri-successo);
    font-weight: 700;
  }
  .firma-non_valida .esito {
    color: var(--ri-errore);
    font-weight: 700;
  }
  .firma-non_verificabile .esito {
    color: var(--ri-avviso);
    font-weight: 700;
  }
  .firma-voce dl {
    font-size: 0.85rem;
    display: grid;
    grid-template-columns: max-content 1fr;
    gap: 0.1rem 0.75rem;
    margin: 0.4rem 0 0;
  }
  .firma-voce dt {
    font-weight: 400;
    color: var(--ri-testo-2);
  }
  .firma-voce dd {
    margin: 0;
  }
  .nota {
    margin: 0;
    font-size: 0.8rem;
    color: var(--ri-testo-2);
  }
  .chip-ri .nome {
    max-width: 260px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .chip-ri :global(.icon) {
    fill: var(--ri-primario);
  }
  .contenuto {
    flex: 1;
    min-height: 0;
    position: relative;
  }
  iframe {
    border: 0;
    width: 100%;
    height: 100%;
    background: #fff;
  }
  .stato {
    display: flex;
    align-items: center;
    justify-content: center;
    height: 100%;
    padding: 2rem;
  }
  .vuoto {
    text-align: center;
    max-width: 420px;
  }
  .vuoto :global(.icon) {
    fill: var(--ri-testo-2);
  }
  .xml {
    margin: 0;
    height: 100%;
    overflow: auto;
    padding: 1rem;
    font-family: "Roboto Mono", monospace;
    white-space: pre;
    color: var(--ri-testo);
    line-height: 1.5;
  }
  .xml.a-capo {
    white-space: pre-wrap;
    word-break: break-all;
  }
  .riga-xml {
    display: inline-block;
  }
  .x-tag {
    color: #0b5394;
  }
  .x-attr {
    color: #8a4b08;
  }
  .x-valore {
    color: #1d7a3e;
  }
  .x-testo {
    color: var(--ri-testo);
    font-weight: 600;
  }
  .x-commento {
    color: #7a8794;
    font-style: italic;
  }
  .x-pi {
    color: #7a8794;
  }
  :global(:root[data-tema="scuro"]) .x-tag {
    color: #79b8ff;
  }
  :global(:root[data-tema="scuro"]) .x-attr {
    color: #f0b46b;
  }
  :global(:root[data-tema="scuro"]) .x-valore {
    color: #85e89d;
  }
  .zoom {
    display: flex;
    align-items: center;
    margin-right: 0.5rem;
    border-right: 1px solid var(--ri-bordo);
    padding-right: 0.5rem;
  }
  .zoom .btn {
    font-weight: 700;
    font-size: 1.05rem;
    min-width: 2rem;
    padding: 0.1rem 0.3rem;
  }
  .zoom .percentuale {
    font-size: 0.8rem;
    font-weight: 600;
    min-width: 3.2rem;
  }
  .interruttore {
    font-size: 0.85rem;
    font-weight: 600;
    margin-right: 0.5rem;
    white-space: nowrap;
  }
  .azioni .btn {
    text-decoration: none;
  }
</style>
