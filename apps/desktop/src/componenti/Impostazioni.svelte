<!--
  Copyright (c) 2026 Andrea Re - info@digitare.net
  Autore: Andrea Re @ 2026 - info@digitare.net

  Pannello delle impostazioni.
-->
<script lang="ts">
  import { open } from "@tauri-apps/plugin-dialog";
  import SceltaDati from "./SceltaDati.svelte";
  import { api, type Avvio, type Impostazioni, type StatoFoglio } from "../lib/api";
  import { normalizzaIdentificativo } from "../lib/archivio";
  import Icona from "./Icona.svelte";

  let {
    imp,
    info,
    foglio,
    suggerito,
    onSalva,
    onFoglio,
    onAggiungiCartella,
  }: {
    imp: Impostazioni;
    info: Avvio;
    foglio: StatoFoglio | null;
    suggerito: { chiave: string; nome: string } | null;
    onSalva: (i: Impostazioni, riscansiona?: boolean) => void;
    onFoglio: (s: StatoFoglio) => void;
    onAggiungiCartella: () => void;
  } = $props();

  let nuovoId = $state("");
  let scaricamento = $state(false);
  let esitoFoglio = $state<{ ok: boolean; testo: string } | null>(null);

  function aggiungiId(valore: string) {
    const id = normalizzaIdentificativo(valore);
    if (!id || imp.identificativi_propri.includes(id)) return;
    onSalva({ ...imp, identificativi_propri: [...imp.identificativi_propri, id] });
    nuovoId = "";
  }

  function togliId(id: string) {
    onSalva({ ...imp, identificativi_propri: imp.identificativi_propri.filter((x) => x !== id) });
  }

  function togliCartella(c: string) {
    onSalva({ ...imp, cartelle: imp.cartelle.filter((x) => x !== c) }, true);
  }

  function togliFile(f: string) {
    onSalva({ ...imp, file_singoli: imp.file_singoli.filter((x) => x !== f) }, true);
  }

  async function scaricaFoglio() {
    scaricamento = true;
    esitoFoglio = null;
    try {
      const s = await api.foglioAggiorna();
      onFoglio(s);
      esitoFoglio = { ok: true, testo: "Foglio di stile scaricato correttamente." };
    } catch (e) {
      esitoFoglio = { ok: false, testo: `Download non riuscito: ${e}` };
    } finally {
      scaricamento = false;
    }
  }

  async function importaFoglio() {
    const scelto = await open({
      multiple: false,
      filters: [{ name: "Foglio di stile", extensions: ["xsl", "xslt", "zip"] }],
    });
    if (!scelto) return;
    try {
      const s = await api.foglioImporta(scelto as string);
      onFoglio(s);
      esitoFoglio = { ok: true, testo: "Foglio di stile importato correttamente." };
    } catch (e) {
      esitoFoglio = { ok: false, testo: `Importazione non riuscita: ${e}` };
    }
  }
</script>

<div class="impostazioni ri-scroll">
  <h1 class="h3 mb-4">Impostazioni</h1>

  <section>
    <h2 class="h5">La tua azienda</h2>
    <p class="text-secondary-ri">
      Inserisci la tua partita IVA (o il codice fiscale) per distinguere le fatture ricevute da quelle emesse e per
      escluderti dalla rubrica. Puoi indicarne più di una.
    </p>
    {#if suggerito && !imp.identificativi_propri.includes(suggerito.chiave)}
      <div class="alert alert-info d-flex align-items-center gap-3 py-2" role="alert">
        <span class="flex-grow-1">Sembra che i documenti siano di <strong>{suggerito.nome}</strong> ({suggerito.chiave}).</span>
        <button class="btn btn-primary btn-xs" onclick={() => aggiungiId(suggerito.chiave)}>Usa questo</button>
      </div>
    {/if}
    <div class="elenco-chip">
      {#each imp.identificativi_propri as id (id)}
        <span class="chip-imp">{id}<button aria-label="Rimuovi {id}" onclick={() => togliId(id)}><Icona nome="it-close" dimensione="xs" /></button></span>
      {/each}
    </div>
    <form
      class="d-flex gap-2 mt-2"
      onsubmit={(e) => {
        e.preventDefault();
        aggiungiId(nuovoId);
      }}
    >
      <input class="form-control form-control-sm w-auto" placeholder="es. 01234567890" aria-label="Partita IVA o codice fiscale da aggiungere" bind:value={nuovoId} />
      <button class="btn btn-outline-primary btn-xs" type="submit">Aggiungi</button>
    </form>
  </section>

  <section>
    <h2 class="h5">Cartelle e file</h2>
    <p class="text-secondary-ri">Le cartelle vengono lette con tutte le sottocartelle. I file originali non vengono mai modificati.</p>
    <ul class="percorsi">
      {#each imp.cartelle as c (c)}
        <li><Icona nome="it-folder" /> <span class="selezionabile">{c}</span>
          <button class="btn btn-link btn-xs" aria-label="Apri la cartella {c}" onclick={() => api.apriCartella(c)}>Apri</button>
          <button class="btn btn-link btn-xs text-danger" aria-label="Rimuovi la cartella {c}" onclick={() => togliCartella(c)}>Rimuovi</button></li>
      {/each}
      {#each imp.file_singoli as f (f)}
        <li><Icona nome="it-file" /> <span class="selezionabile">{f}</span>
          <button class="btn btn-link btn-xs text-danger" aria-label="Rimuovi il file {f}" onclick={() => togliFile(f)}>Rimuovi</button></li>
      {/each}
      {#if imp.cartelle.length === 0 && imp.file_singoli.length === 0}
        <li class="text-secondary-ri">Nessuna cartella.</li>
      {/if}
    </ul>
    <button class="btn btn-outline-primary btn-xs" onclick={onAggiungiCartella}><Icona nome="it-plus" /> Aggiungi cartella</button>

    <div class="mt-3">
      <label class="etichetta" for="trascinati">Quando trascini dei file nella finestra</label>
      <select id="trascinati" class="form-select form-select-sm w-auto" value={imp.file_trascinati}
        onchange={(e) => onSalva({ ...imp, file_trascinati: (e.currentTarget as HTMLSelectElement).value as Impostazioni["file_trascinati"] })}>
        <option value="chiedi">Chiedi ogni volta</option>
        <option value="copia">Copiali nell'archivio dell'app</option>
        <option value="lascia">Lasciali dove sono</option>
      </select>
    </div>
  </section>

  <section>
    <h2 class="h5">Foglio di stile ufficiale</h2>
    <p class="text-secondary-ri">
      La vista "AssoSoftware" usa il foglio di stile pubblicato da AssoSoftware. Non è incluso nell'applicazione:
      puoi scaricarlo dal sito ufficiale oppure importarlo da un file .xsl o .zip.
    </p>
    <p>
      {#if foglio?.presente}
        <Icona nome="it-check-circle" classe="ok" /> Presente, aggiornato il {foglio.aggiornato_il}
      {:else}
        <Icona nome="it-warning-circle" classe="ko" /> Non presente
      {/if}
    </p>
    <div class="d-flex gap-2">
      <button class="btn btn-primary btn-xs" onclick={scaricaFoglio} disabled={scaricamento}>
        <Icona nome="it-download" classe="bianca" /> {scaricamento ? "Download in corso..." : foglio?.presente ? "Aggiorna dal sito" : "Scarica dal sito"}
      </button>
      <button class="btn btn-outline-primary btn-xs" onclick={importaFoglio}><Icona nome="it-upload" /> Importa da file</button>
    </div>
    {#if esitoFoglio}
      <div class="alert {esitoFoglio.ok ? 'alert-success' : 'alert-danger'} mt-3 py-2" role="status">{esitoFoglio.testo}</div>
    {/if}
  </section>

  <section>
    <h2 class="h5">Leggibilità</h2>
    <label class="etichetta" for="zoom">Dimensione dell'interfaccia</label>
    <select id="zoom" class="form-select form-select-sm w-auto" value={String(imp.zoom)}
      onchange={(e) => onSalva({ ...imp, zoom: Number((e.currentTarget as HTMLSelectElement).value) })}>
      {#each [1, 1.25, 1.5, 1.75, 2] as z (z)}
        <option value={String(z)}>{Math.round(z * 100)}%</option>
      {/each}
    </select>
    <p class="text-secondary-ri mt-2 mb-0"><small>Ingrandisce testi, pulsanti e menu; con valori alti la disposizione si adatta alla finestra.</small></p>
  </section>

  <section>
    <h2 class="h5">Dati dell'applicazione</h2>
    <SceltaDati {info} onFatto={() => {}} />
    <p class="selezionabile mt-3 mb-0"><small>In uso: <code>{info.cartella_dati}</code></small>
      <button class="btn btn-link btn-xs" aria-label="Apri la cartella dei dati" onclick={() => api.apriCartella(info.cartella_dati)}>Apri</button></p>
  </section>

</div>

<style>
  .impostazioni {
    height: 100%;
    padding: 2rem 2.5rem;
    width: 100%;
  }
  section {
    margin-bottom: 2rem;
    padding-bottom: 1.5rem;
    border-bottom: 1px solid var(--ri-bordo);
  }
  .elenco-chip {
    display: flex;
    flex-wrap: wrap;
    gap: 0.4rem;
  }
  .chip-imp {
    display: inline-flex;
    align-items: center;
    gap: 0.3rem;
    background: var(--ri-selezione);
    border-radius: 999px;
    padding: 0.15rem 0.3rem 0.15rem 0.75rem;
    font-family: "Roboto Mono", monospace;
    font-size: 0.85rem;
  }
  .chip-imp button {
    border: 0;
    background: none;
    padding: 0 0.2rem;
  }
  .percorsi {
    list-style: none;
    padding: 0;
  }
  .percorsi li {
    display: flex;
    align-items: center;
    gap: 0.4rem;
    padding: 0.25rem 0;
    font-size: 0.92rem;
  }
  .percorsi :global(.icon) {
    fill: var(--ri-primario);
  }
  .etichetta {
    display: block;
    font-weight: 600;
    font-size: 0.9rem;
    margin-bottom: 0.3rem;
  }
  :global(.icon.ok) {
    fill: var(--ri-successo);
  }
  :global(.icon.ko) {
    fill: var(--ri-avviso);
  }
  :global(.icon.bianca) {
    fill: #fff;
  }
  .btn :global(.icon) {
    fill: currentColor;
  }
  .btn-outline-primary :global(.icon) {
    fill: var(--ri-primario);
  }
</style>
