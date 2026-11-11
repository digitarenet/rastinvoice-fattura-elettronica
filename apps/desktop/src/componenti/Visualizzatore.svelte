<!--
  Copyright (c) 2026 Andrea Re - info@digitare.net
  Autore: Andrea Re @ 2026 - info@digitare.net

  Visualizzatore degli allegati: PDF, immagini, Word, fogli di calcolo, CSV,
  testo, XML, pagine HTML e archivi zip. Nessun visualizzatore esegue script
  contenuti nel file.
-->
<script lang="ts">
  import { modale } from "../lib/modale";
  import { onDestroy, onMount, tick, untrack } from "svelte";
  import { api, type AnteprimaAllegato } from "../lib/api";
  import { formatta } from "../lib/xmlformato";
  import Icona from "./Icona.svelte";

  let {
    file,
    corpo,
    indice,
    nome,
    onChiudi,
    onErrore,
  }: {
    file: string;
    corpo: number;
    indice: number;
    nome: string;
    onChiudi: () => void;
    onErrore: (m: string) => void;
  } = $props();

  // Percorso dentro gli archivi zip (vuoto = l'allegato stesso).
  let interno = $state<string[]>([]);
  let anteprima = $state<AnteprimaAllegato | null>(null);
  let errore = $state("");
  let caricamento = $state(true);
  let urlImmagine = $state("");
  let htmlWord = $state("");
  let foglioAttivo = $state(0);
  let zoom = $state(1);
  let pagine = $state(0);
  let contenitorePdf = $state<HTMLElement | null>(null);
  let docPdf: import("pdfjs-dist").PDFDocumentProxy | null = null;
  let generazione = 0;

  const titolo = $derived(interno.length ? interno[interno.length - 1].split("/").pop() : nome);
  const righeXml = $derived(anteprima?.tipo === "testo" && anteprima.formato === "xml" ? formatta(anteprima.testo) : null);
  const zoomabile = $derived(anteprima?.tipo === "pdf" || anteprima?.tipo === "immagine");

  // Si ricarica solo quando cambia il percorso, non per lo stato interno.
  $effect(() => {
    const percorso = [...interno];
    untrack(() => void carica(percorso));
  });

  // Accessibilità: il fuoco va nella finestra all'apertura e torna dove
  // era alla chiusura (schema dei modali di Bootstrap Italia).
  let finestraEl = $state<HTMLElement | null>(null);
  const precedente = document.activeElement as HTMLElement | null;
  onMount(() => finestraEl?.focus());
  onDestroy(() => {
    pulisci();
    precedente?.focus?.();
  });

  function pulisci() {
    if (urlImmagine) URL.revokeObjectURL(urlImmagine);
    urlImmagine = "";
    htmlWord = "";
    void docPdf?.loadingTask.destroy();
    docPdf = null;
    pagine = 0;
  }

  async function carica(percorso: string[]) {
    const mio = ++generazione;
    pulisci();
    caricamento = true;
    errore = "";
    anteprima = null;
    foglioAttivo = 0;
    zoom = 1;
    try {
      const a = await api.anteprimaAllegato(file, corpo, indice, percorso);
      if (mio !== generazione) return;
      if (a.tipo === "immagine" || a.tipo === "pdf" || a.tipo === "word") {
        const dati = await api.byteAllegato(file, corpo, indice, percorso);
        if (mio !== generazione) return;
        if (a.tipo === "immagine") {
          urlImmagine = URL.createObjectURL(new Blob([dati], { type: a.mime }));
        } else if (a.tipo === "word") {
          const { wordInHtml } = await import("../lib/word");
          htmlWord = await wordInHtml(dati);
        } else {
          const { apriPdf } = await import("../lib/pdf");
          docPdf = await apriPdf(dati);
          if (mio !== generazione) return;
          pagine = docPdf.numPages;
        }
      }
      anteprima = a;
      caricamento = false;
      if (a.tipo === "pdf") {
        await tick();
        await disegnaPdf(mio);
      }
    } catch (e) {
      if (mio !== generazione) return;
      errore = String(e);
      caricamento = false;
    }
  }

  async function disegnaPdf(mio = generazione) {
    if (!docPdf || !contenitorePdf) return;
    const { disegnaPagina } = await import("../lib/pdf");
    const larghezza = Math.min(contenitorePdf.clientWidth - 48, 900) * zoom;
    const tele = contenitorePdf.querySelectorAll("canvas");
    for (let i = 0; i < tele.length; i++) {
      if (mio !== generazione || !docPdf) return;
      await disegnaPagina(docPdf, i + 1, tele[i], larghezza);
    }
  }

  function cambiaZoom(fattore: number) {
    zoom = fattore === 0 ? 1 : Math.min(4, Math.max(0.25, zoom * fattore));
    if (anteprima?.tipo === "pdf") void disegnaPdf();
  }

  function tasti(e: KeyboardEvent) {
    if (e.key === "Escape") {
      e.preventDefault();
      if (interno.length) interno = interno.slice(0, -1);
      else onChiudi();
    }
  }

  function dimensione(n: number): string {
    if (n < 1024) return `${n} B`;
    if (n < 1024 * 1024) return `${(n / 1024).toFixed(0)} kB`;
    return `${(n / 1024 / 1024).toFixed(1)} MB`;
  }

  async function apriEsterno() {
    try {
      await api.apriAllegato(file, corpo, indice);
    } catch (e) {
      onErrore(String(e));
    }
  }

  async function salva() {
    try {
      await api.salvaAllegato(file, corpo, indice);
    } catch (e) {
      onErrore(String(e));
    }
  }
</script>

<svelte:window onkeydown={tasti} />

<div class="sfondo" role="presentation" onclick={(e) => e.target === e.currentTarget && onChiudi()}>
  <div class="finestra ri-pannello" role="dialog" aria-modal="true" aria-label="Allegato {titolo}" tabindex="-1" bind:this={finestraEl} use:modale>
    <header class="barra">
      <div class="titolo">
        <Icona nome="it-clip" dimensione="sm" />
        {#if interno.length}
          <button class="btn btn-link btn-xs p-0" onclick={() => (interno = [])}>{nome}</button>
          {#each interno as parte, i (i)}
            <span class="separatore">›</span>
            {#if i < interno.length - 1}
              <button class="btn btn-link btn-xs p-0" onclick={() => (interno = interno.slice(0, i + 1))}>{parte.split("/").pop()}</button>
            {:else}
              <strong>{parte.split("/").pop()}</strong>
            {/if}
          {/each}
        {:else}
          <strong>{nome}</strong>
        {/if}
        {#if anteprima?.tipo === "pdf" && pagine}<span class="text-secondary-ri">· {pagine} {pagine === 1 ? "pagina" : "pagine"}</span>{/if}
      </div>
      <div class="azioni">
        {#if zoomabile}
          <div class="zoom" role="group" aria-label="Zoom">
            <button class="btn btn-xs btn-link" title="Riduci" aria-label="Riduci" onclick={() => cambiaZoom(1 / 1.25)}>−</button>
            <button class="btn btn-xs btn-link percentuale" title="Adatta" aria-label="{Math.round(zoom * 100)}%, adatta alla finestra" onclick={() => cambiaZoom(0)}>{Math.round(zoom * 100)}%</button>
            <button class="btn btn-xs btn-link" title="Ingrandisci" aria-label="Ingrandisci" onclick={() => cambiaZoom(1.25)}>+</button>
          </div>
        {/if}
        <button class="btn btn-xs btn-link" title="Apri con il programma predefinito" aria-label="Apri con il programma predefinito" onclick={apriEsterno}>
          <Icona nome="it-external-link" />
        </button>
        <button class="btn btn-xs btn-link" title="Salva l'allegato" aria-label="Salva l'allegato" onclick={salva}>
          <Icona nome="it-download" />
        </button>
        <button class="btn btn-xs btn-link" title="Chiudi (Esc)" aria-label="Chiudi" onclick={onChiudi}>
          <Icona nome="it-close-big" />
        </button>
      </div>
    </header>

    <div class="contenuto">
      {#if caricamento}
        <div class="centro text-secondary-ri">Apertura dell'allegato...</div>
      {:else if errore}
        <div class="centro"><p class="text-danger">Impossibile mostrare l'allegato: {errore}</p></div>
      {:else if anteprima?.tipo === "immagine"}
        <div class="immagine ri-scroll">
          <img src={urlImmagine} alt={titolo} style={zoom === 1 ? "" : `max-width: none; width: ${zoom * 100}%`} />
        </div>
      {:else if anteprima?.tipo === "pdf"}
        <div class="pdf ri-scroll" bind:this={contenitorePdf}>
          {#each Array(pagine) as _, i (i)}
            <canvas aria-label="Pagina {i + 1}"></canvas>
          {/each}
        </div>
      {:else if anteprima?.tipo === "word"}
        <iframe title={titolo} sandbox="" srcdoc={htmlWord}></iframe>
      {:else if anteprima?.tipo === "html"}
        <iframe title={titolo} sandbox="" srcdoc={anteprima.html}></iframe>
      {:else if anteprima?.tipo === "testo"}
        {#if anteprima.troncato}<p class="avviso">Testo molto lungo: è mostrata solo la prima parte.</p>{/if}
        <pre class="testo ri-scroll selezionabile">{#if righeXml}{#each righeXml as r, i (i)}<span class="riga-xml" style="padding-left: {r.rientro * 1.5}em">{#each r.parti as p, j (j)}<span class="x-{p.tipo}">{p.testo}</span>{/each}</span>
{/each}{:else}{anteprima.testo}{/if}</pre>
      {:else if anteprima?.tipo === "tabella"}
        {@const foglio = anteprima.fogli[foglioAttivo]}
        {#if anteprima.fogli.length > 1}
          <div class="fogli" role="tablist">
            {#each anteprima.fogli as f, i (i)}
              <button role="tab" class:attiva={i === foglioAttivo} aria-selected={i === foglioAttivo} onclick={() => (foglioAttivo = i)}>{f.nome}</button>
            {/each}
          </div>
        {/if}
        {#if foglio.troncato}<p class="avviso">Foglio molto grande: sono mostrate solo le prime righe.</p>{/if}
        <div class="tabella ri-scroll selezionabile">
          <table>
            <tbody>
              {#each foglio.righe as r, i (i)}
                <tr><th class="numero">{i + 1}</th>{#each r as c, j (j)}<td>{c}</td>{/each}</tr>
              {/each}
            </tbody>
          </table>
        </div>
      {:else if anteprima?.tipo === "archivio"}
        <ul class="archivio ri-scroll">
          {#each anteprima.voci as v (v.nome)}
            <li>
              <button onclick={() => (interno = [...interno, v.nome])}>
                <Icona nome="it-file" dimensione="sm" />
                <span class="nome">{v.nome}</span>
                <span class="text-secondary-ri">{dimensione(v.dimensione)}</span>
              </button>
            </li>
          {:else}
            <li class="text-secondary-ri p-3">Archivio vuoto.</li>
          {/each}
        </ul>
      {:else if anteprima?.tipo === "non_disponibile"}
        <div class="centro">
          <div class="vuoto">
            <Icona nome="it-file" dimensione="lg" />
            <p class="mt-3">{anteprima.motivo}</p>
            {#if !interno.length}
              <button class="btn btn-primary btn-sm me-2" onclick={apriEsterno}>Apri con il programma predefinito</button>
            {/if}
            <button class="btn btn-outline-primary btn-sm" onclick={salva}>Salva</button>
          </div>
        </div>
      {/if}
    </div>
  </div>
</div>

<style>
  .sfondo {
    position: fixed;
    inset: 0;
    background: rgb(0 0 0 / 0.45);
    z-index: 50;
    display: flex;
    padding: 1.5rem;
  }
  .finestra {
    outline: none;
    flex: 1;
    display: flex;
    flex-direction: column;
    min-width: 0;
    min-height: 0;
    overflow: hidden;
    box-shadow: 0 10px 40px rgb(0 0 0 / 0.3);
  }
  .barra {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 1rem;
    padding: 0.5rem 0.75rem 0.5rem 1rem;
    border-bottom: 1px solid var(--ri-bordo);
  }
  .titolo {
    display: flex;
    align-items: center;
    gap: 0.4rem;
    min-width: 0;
    overflow: hidden;
    white-space: nowrap;
  }
  .titolo :global(.icon) {
    fill: var(--ri-testo-2);
    flex: none;
  }
  .separatore {
    color: var(--ri-testo-2);
  }
  .azioni {
    display: flex;
    align-items: center;
    flex: none;
  }
  .azioni .btn {
    text-decoration: none;
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
    min-width: 3.2rem;
  }
  .contenuto {
    flex: 1;
    min-height: 0;
    display: flex;
    flex-direction: column;
    background: var(--ri-sfondo);
  }
  .centro {
    flex: 1;
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 2rem;
    text-align: center;
  }
  .vuoto {
    max-width: 460px;
  }
  .vuoto :global(.icon) {
    fill: var(--ri-testo-2);
  }
  .immagine {
    flex: 1;
    overflow: auto;
    display: flex;
    align-items: flex-start;
    justify-content: center;
    padding: 1.5rem;
  }
  .immagine img {
    max-width: 100%;
    background: #fff;
    box-shadow: 0 1px 6px rgb(0 0 0 / 0.15);
  }
  .pdf {
    flex: 1;
    overflow: auto;
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 1rem;
    padding: 1.5rem;
  }
  .pdf canvas {
    background: #fff;
    box-shadow: 0 1px 6px rgb(0 0 0 / 0.2);
    flex: none;
  }
  iframe {
    flex: 1;
    border: 0;
    background: #fff;
  }
  .testo {
    flex: 1;
    margin: 0;
    overflow: auto;
    padding: 1rem 1.25rem;
    font-family: "Roboto Mono", monospace;
    font-size: 0.82rem;
    line-height: 1.5;
    white-space: pre-wrap;
    word-break: break-word;
    color: var(--ri-testo);
    background: var(--ri-pannello);
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
    font-weight: 600;
  }
  .x-commento,
  .x-pi {
    color: var(--ri-testo-2);
  }
  .avviso {
    margin: 0;
    padding: 0.4rem 1rem;
    font-size: 0.85rem;
    background: #fff4e0;
    color: #6b4800;
  }
  .fogli {
    display: flex;
    gap: 0.25rem;
    padding: 0.4rem 0.75rem 0;
    border-bottom: 1px solid var(--ri-bordo);
    overflow-x: auto;
  }
  .fogli button {
    border: 0;
    background: none;
    padding: 0.35rem 0.8rem;
    font-weight: 600;
    color: var(--ri-testo-2);
    border-bottom: 3px solid transparent;
    white-space: nowrap;
  }
  .fogli button.attiva {
    color: var(--ri-primario);
    border-bottom-color: var(--ri-primario);
  }
  .tabella {
    flex: 1;
    overflow: auto;
    background: var(--ri-pannello);
  }
  table {
    border-collapse: collapse;
    font-size: 0.85rem;
  }
  td,
  th {
    border: 1px solid var(--ri-bordo);
    padding: 0.2rem 0.5rem;
    white-space: nowrap;
    max-width: 32rem;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  th.numero {
    position: sticky;
    left: 0;
    background: var(--ri-sfondo);
    color: var(--ri-testo-2);
    font-weight: 600;
    text-align: right;
  }
  .archivio {
    list-style: none;
    margin: 0;
    padding: 0;
    overflow: auto;
    flex: 1;
    background: var(--ri-pannello);
  }
  .archivio button {
    display: flex;
    align-items: center;
    gap: 0.6rem;
    width: 100%;
    border: 0;
    border-bottom: 1px solid var(--ri-bordo);
    background: none;
    padding: 0.6rem 1rem;
    text-align: left;
    color: inherit;
  }
  .archivio button:hover {
    background: var(--ri-sfondo);
  }
  .archivio .nome {
    flex: 1;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .archivio :global(.icon) {
    fill: var(--ri-primario);
  }
</style>
