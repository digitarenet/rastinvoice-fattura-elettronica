<!--
  Copyright (c) 2026 Andrea Re - info@digitare.net
  Autore: Andrea Re @ 2026 - info@digitare.net

  Anteprima del documento selezionato: vista Moderna, foglio di stile
  standard, XML originale e allegati.
-->
<script lang="ts">
  import { save } from "@tauri-apps/plugin-dialog";
  import { openPath, revealItemInDir } from "@tauri-apps/plugin-opener";
  import { api, type Dettaglio, type Voce } from "../lib/api";
  import * as xslt from "../lib/xslt";
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
    htmlStandard = "";
    erroreStandard = "";
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

  // Applica il foglio di stile standard solo quando serve.
  $effect(() => {
    if (scheda !== "standard" || !dettaglio || htmlStandard || erroreStandard) return;
    const xml = dettaglio.xml;
    (async () => {
      try {
        if (!xslt.disponibile()) throw new Error("il motore XSLT non è disponibile in questo sistema");
        foglioXsl ??= await api.foglioLeggi();
        if (!foglioXsl) throw new Error("foglio di stile non presente");
        htmlStandard = xslt.trasforma(xml, foglioXsl);
      } catch (e) {
        erroreStandard = String(e instanceof Error ? e.message : e);
      }
    })();
  });

  // Se il foglio viene scaricato dopo, al prossimo tentativo va riletto.
  $effect(() => {
    if (foglioPresente) {
      foglioXsl = null;
      erroreStandard = "";
    }
  });

  function cambiaScheda(s: Scheda) {
    scheda = s;
    if (s !== "xml") onVistaCambiata(s);
  }

  const allegati = $derived(dettaglio?.fattura.corpi[dettaglio.fattura.corpi.length > 1 ? voce.corpo : 0]?.allegati ?? []);

  function stampa() {
    cornice?.contentWindow?.focus();
    cornice?.contentWindow?.print();
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

<section class="anteprima ri-pannello">
  <div class="barra">
    <div class="schede" role="tablist">
      <button role="tab" class:attiva={scheda === "moderna"} aria-selected={scheda === "moderna"} onclick={() => cambiaScheda("moderna")}>Moderna</button>
      <button role="tab" class:attiva={scheda === "standard"} aria-selected={scheda === "standard"} onclick={() => cambiaScheda("standard")}>Foglio di stile</button>
      <button role="tab" class:attiva={scheda === "xml"} aria-selected={scheda === "xml"} onclick={() => cambiaScheda("xml")}>XML</button>
    </div>
    <div class="azioni">
      <button class="btn btn-xs btn-link" title={contrassegnata ? "Togli contrassegno" : "Contrassegna"} onclick={onContrassegna}>
        <Icona nome={contrassegnata ? "it-star-full" : "it-star-outline"} classe={contrassegnata ? "stella" : ""} />
      </button>
      <button class="btn btn-xs btn-link" title="Stampa (Ctrl+P)" onclick={stampa} disabled={scheda === "xml"}>
        <Icona nome="it-print" />
      </button>
      <button class="btn btn-xs btn-link" title="Salva come pagina HTML" onclick={salvaHtml} disabled={scheda === "xml"}>
        <Icona nome="it-download" />
      </button>
      <button class="btn btn-xs btn-link" title="Salva l'XML estratto" onclick={salvaXml}>
        <Icona nome="it-file-xml" />
      </button>
      <button class="btn btn-xs btn-link" title="Mostra il file nella cartella" onclick={() => revealItemInDir(voce.file)}>
        <Icona nome="it-folder" />
      </button>
    </div>
  </div>

  {#if dettaglio && (dettaglio.firmato || allegati.length > 0)}
    <div class="info">
      {#if dettaglio.firmato}
        <span class="chip-ri firmato" title="Busta CAdES (.p7m): contenuto estratto. La verifica della firma non è ancora disponibile.">
          <Icona nome="it-file-signed" dimensione="xs" />
          Firmato digitalmente{dettaglio.livelli_firma > 1 ? ` (${dettaglio.livelli_firma} buste)` : ""} · {dettaglio.certificati}
          {dettaglio.certificati === 1 ? "certificato" : "certificati"}
        </span>
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

  <div class="contenuto">
    {#if caricamento && !dettaglio}
      <div class="stato"><div class="progress-spinner progress-spinner-active"></div></div>
    {:else if errore}
      <div class="stato">
        <div class="alert alert-danger" role="alert">Impossibile aprire il documento: {errore}</div>
      </div>
    {:else if dettaglio}
      {#if scheda === "moderna"}
        <iframe bind:this={cornice} title="Fattura" srcdoc={dettaglio.html} sandbox="allow-same-origin allow-modals"></iframe>
      {:else if scheda === "standard"}
        {#if htmlStandard}
          <iframe bind:this={cornice} title="Fattura (foglio di stile)" srcdoc={htmlStandard} sandbox="allow-same-origin allow-modals"></iframe>
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
        <pre class="xml selezionabile">{dettaglio.xml}</pre>
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
  .chip-ri.firmato {
    color: var(--ri-successo);
    border-color: color-mix(in srgb, var(--ri-successo) 40%, transparent);
  }
  .chip-ri.firmato :global(.icon) {
    fill: var(--ri-successo);
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
    font-size: 0.8rem;
    white-space: pre-wrap;
    word-break: break-all;
    color: var(--ri-testo);
  }
</style>
