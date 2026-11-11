<!--
  Copyright (c) 2026 Andrea Re - info@digitare.net
  Autore: Andrea Re @ 2026 - info@digitare.net

  Anteprima del documento selezionato: vista Moderna, foglio di stile
  standard, XML originale e allegati.
-->
<script lang="ts">
  import { untrack } from "svelte";
  import { api, type Dettaglio, type Voce } from "../lib/api";
  import * as xslt from "../lib/xslt";
  import { formatta, type Riga } from "../lib/xmlformato";
  import { infoSigla, statoAttuale } from "../lib/sdi";
  import { dataOra as dataOraIso } from "../lib/formato";
  import type { VoceMessaggio } from "../lib/api";
  import Icona from "./Icona.svelte";
  import Visualizzatore from "./Visualizzatore.svelte";

  type Scheda = "moderna" | "standard" | "xml";

  let {
    voce,
    contrassegnata,
    letta = true,
    onNonLetta = () => {},
    schedaIniziale = "moderna",
    foglioPresente,
    onContrassegna,
    onVistaCambiata,
    onApriImpostazioni,
    onErrore,
    onAvviso = () => {},
    notifiche = [],
  }: {
    notifiche?: VoceMessaggio[];
    voce: Voce;
    contrassegnata: boolean;
    letta?: boolean;
    onNonLetta?: () => void;
    schedaIniziale?: "moderna" | "standard";
    foglioPresente: boolean;
    onContrassegna: () => void;
    onVistaCambiata: (v: "moderna" | "standard") => void;
    onApriImpostazioni: () => void;
    onErrore: (m: string) => void;
    onAvviso?: (m: string) => void;
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
        let html: string;
        try {
          // Motore nativo, indipendente dalla webview.
          html = await api.trasformaStandard(d.xml);
        } catch (errNativo) {
          // Riserva: motore XSLT della webview, finché è disponibile.
          if (!xslt.disponibile()) throw errNativo;
          foglioXsl ??= await api.foglioLeggi();
          if (!foglioXsl) throw errNativo;
          html = xslt.trasforma(d.xml, foglioXsl);
        }
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

  const SCHEDE: [Scheda, string][] = [
    ["moderna", "Moderna"],
    ["standard", "AssoSoftware"],
    ["xml", "XML"],
  ];

  /** Frecce, Home e Fine spostano la scheda attiva, come nello schema ARIA. */
  function tastiSchede(e: KeyboardEvent) {
    const i = SCHEDE.findIndex(([s]) => s === scheda);
    let j = i;
    if (e.key === "ArrowRight") j = (i + 1) % SCHEDE.length;
    else if (e.key === "ArrowLeft") j = (i + SCHEDE.length - 1) % SCHEDE.length;
    else if (e.key === "Home") j = 0;
    else if (e.key === "End") j = SCHEDE.length - 1;
    else return;
    e.preventDefault();
    e.stopPropagation();
    cambiaScheda(SCHEDE[j][0]);
    document.getElementById(`scheda-${SCHEDE[j][0]}`)?.focus();
  }

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

  // Controlli formali e di calcolo.
  let controlliAperti = $state(false);
  const erroriControlli = $derived((dettaglio?.controlli ?? []).filter((c) => c.gravita === "errore").length);

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

  // Stato presso il Sistema di Interscambio, dalle notifiche trovate.
  const ultimaNotifica = $derived(statoAttuale(notifiche));
  let notificheAperte = $state(false);

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

  // Nome del file senza estensioni, usato come nome proposto nei salvataggi.
  function nomeBase() {
    return voce.nome_file.replace(/\.(xml|p7m)$/gi, "").replace(/\.xml$/i, "");
  }

  // Salvataggio in PDF tramite la finestra di stampa del sistema, che offre
  // sempre la destinazione "Salva come PDF". Il titolo del documento diventa
  // il nome proposto per il file.
  function salvaPdf() {
    const documento = cornice?.contentDocument;
    if (!documento) return stampa();
    const titolo = documento.title;
    documento.title = nomeBase();
    onAvviso("Nella finestra di stampa scegli \"Salva come PDF\" o \"Microsoft Print to PDF\".");
    stampa();
    documento.title = titolo;
  }

  async function salvaHtml() {
    const html = scheda === "standard" ? htmlStandard : dettaglio?.html;
    if (!html) return;
    await api.salvaTesto(`${nomeBase()}.html`, "html", html).catch((e) => onErrore(String(e)));
  }

  async function salvaXml() {
    if (!dettaglio) return;
    const base = voce.nome_file.replace(/\.p7m$/i, "");
    await api.salvaTesto(base, "xml", dettaglio.xml).catch((e) => onErrore(String(e)));
  }

  async function apriAllegato(i: number) {
    try {
      await api.apriAllegato(voce.file, voce.corpo, i);
    } catch (e) {
      onErrore(String(e));
    }
  }

  // Allegato aperto nel visualizzatore integrato.
  let allegatoAperto = $state<number | null>(null);
  // Cambiando documento il visualizzatore si chiude.
  $effect(() => {
    void voce.id;
    allegatoAperto = null;
  });

  async function salvaAllegato(i: number) {
    try {
      await api.salvaAllegato(voce.file, voce.corpo, i);
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
    <div class="schede" role="tablist" aria-label="Viste del documento" tabindex="-1" onkeydown={tastiSchede}>
      {#each SCHEDE as [s, etichetta] (s)}
        <button role="tab" id="scheda-{s}" aria-controls="pannello-documento" class:attiva={scheda === s} aria-selected={scheda === s}
          tabindex={scheda === s ? 0 : -1} onclick={() => cambiaScheda(s)}>{etichetta}</button>
      {/each}
    </div>
    <div class="azioni">
      <div class="zoom" role="group" aria-label="Zoom">
        <button class="btn btn-xs btn-link" title="Riduci (Ctrl -)" aria-label="Riduci" onclick={() => cambiaZoom(-1)} disabled={zoom <= LIVELLI_ZOOM[0]}>−</button>
        <button class="btn btn-xs btn-link percentuale" title="Dimensione reale (Ctrl 0)" aria-label="{Math.round(zoom * 100)}%, dimensione reale" onclick={() => cambiaZoom(0)}>{Math.round(zoom * 100)}%</button>
        <button class="btn btn-xs btn-link" title="Ingrandisci (Ctrl +)" aria-label="Ingrandisci" onclick={() => cambiaZoom(1)} disabled={zoom >= LIVELLI_ZOOM[LIVELLI_ZOOM.length - 1]}>+</button>
      </div>
      {#if scheda === "xml"}
        <button class="btn btn-xs btn-link interruttore" aria-pressed={xmlFormattato} title="Alterna tra XML formattato e testo originale del file"
          onclick={() => (xmlFormattato = !xmlFormattato)}>{xmlFormattato ? "Mostra originale" : "Formatta"}</button>
      {/if}
      <button class="btn btn-xs btn-link" title={contrassegnata ? "Togli contrassegno" : "Contrassegna"} aria-label="Contrassegna" aria-pressed={contrassegnata} onclick={onContrassegna}>
        <Icona nome={contrassegnata ? "it-star-full" : "it-star-outline"} classe={contrassegnata ? "stella" : ""} />
      </button>
      <button class="btn btn-xs btn-link" title="Segna come da leggere" aria-label="Segna come da leggere" disabled={!letta} onclick={onNonLetta}>
        <Icona nome="it-mail" />
      </button>
      {#if scheda !== "xml"}
        <button class="btn btn-xs btn-link" title="Stampa (Ctrl+P)" aria-label="Stampa" onclick={stampa}>
          <Icona nome="it-print" />
        </button>
        <button class="btn btn-xs btn-link" title="Salva in PDF" aria-label="Salva in PDF" onclick={salvaPdf}>
          <Icona nome="it-file-pdf" />
        </button>
        <button class="btn btn-xs btn-link" title="Salva come pagina HTML" aria-label="Salva come pagina HTML" onclick={salvaHtml}>
          <Icona nome="it-download" />
        </button>
      {/if}
      <button class="btn btn-xs btn-link" title="Salva l'XML estratto" aria-label="Salva l'XML estratto" onclick={salvaXml}>
        <Icona nome="it-file-xml" />
      </button>
      <button class="btn btn-xs btn-link" title="Mostra il file nella cartella" aria-label="Mostra il file nella cartella" onclick={() => api.mostraNellaCartella(voce.file).catch((e) => onErrore(String(e)))}>
        <Icona nome="it-folder" />
      </button>
    </div>
  </div>

  {#if dettaglio && (dettaglio.firmato || allegati.length > 0 || dettaglio.controlli.length > 0 || ultimaNotifica)}
    <div class="info">
      {#if ultimaNotifica}
        {@const i = infoSigla(ultimaNotifica)}
        <button class="chip-ri sdi-{i.gravita}" onclick={() => (notificheAperte = !notificheAperte)} aria-expanded={notificheAperte}
          title="Mostra le notifiche del Sistema di Interscambio">
          <Icona nome={i.gravita === "errore" ? "it-error" : i.gravita === "avviso" ? "it-warning-circle" : "it-mail"} dimensione="xs" />
          SdI: {i.stato}{#if ultimaNotifica.data}&nbsp;· {dataOraIso(ultimaNotifica.data).slice(0, 10)}{/if}
        </button>
      {/if}
      {#if dettaglio.firmato}
        <button class="chip-ri firma-{esitoFirme}" onclick={() => (firmeAperte = !firmeAperte)} aria-expanded={firmeAperte}
          title="Mostra i dettagli della firma">
          <Icona nome={esitoFirme === "non_valida" ? "it-warning-circle" : "it-file-signed"} dimensione="xs" />
          {etichettaFirme}
        </button>
      {/if}
      {#if dettaglio.controlli.length > 0}
        <button class="chip-ri controlli-{erroriControlli > 0 ? 'errore' : 'avviso'}" onclick={() => (controlliAperti = !controlliAperti)}
          aria-expanded={controlliAperti} title="Mostra i controlli">
          <Icona nome={erroriControlli > 0 ? "it-error" : "it-warning-circle"} dimensione="xs" />
          {erroriControlli > 0
            ? `${erroriControlli} ${erroriControlli === 1 ? "controllo non superato" : "controlli non superati"}`
            : `${dettaglio.controlli.length} ${dettaglio.controlli.length === 1 ? "avviso" : "avvisi"}`}
        </button>
      {/if}
      {#each allegati as a, i (i)}
        <span class="chip-ri allegato">
          <Icona nome="it-clip" dimensione="xs" />
          <button class="btn btn-link p-0 nome" title={a.descrizione ? `${a.descrizione} · Visualizza` : "Visualizza"}
            onclick={() => (allegatoAperto = i)}>{a.nome ?? `allegato ${i + 1}`}</button>
          <span class="text-secondary-ri">{dimensione(a.dimensione)}</span>
          <button class="btn btn-link p-0" title="Apri con il programma predefinito" aria-label="Apri {a.nome ?? `allegato ${i + 1}`} con il programma predefinito" onclick={() => apriAllegato(i)}><Icona nome="it-external-link" dimensione="xs" /></button>
          <button class="btn btn-link p-0" title="Salva" aria-label="Salva {a.nome ?? `allegato ${i + 1}`}" onclick={() => salvaAllegato(i)}><Icona nome="it-download" dimensione="xs" /></button>
        </span>
      {/each}
    </div>
  {/if}

  {#if dettaglio && notificheAperte && notifiche.length > 0}
    <div class="pannello-firme">
      <ol class="cronologia">
        {#each notifiche as m (m.file)}
          {@const i = infoSigla(m)}
          <li class="sdi-{i.gravita}">
            <div><strong>{i.stato}</strong> <span class="text-secondary-ri">· {i.nome}{#if m.data} · {dataOraIso(m.data)}{/if}</span></div>
            {#if m.note}<div>{m.note}</div>{/if}
            {#each m.errori as e, j (j)}
              <div>Errore {e.codice}: {e.descrizione}{#if e.suggerimento} <span class="text-secondary-ri">({e.suggerimento})</span>{/if}</div>
            {/each}
          </li>
        {/each}
      </ol>
    </div>
  {/if}

  {#if dettaglio && controlliAperti && dettaglio.controlli.length > 0}
    <div class="pannello-firme">
      <ul class="elenco-controlli">
        {#each dettaglio.controlli as c, i (i)}
          <li class="controllo-{c.gravita}">
            <Icona nome={c.gravita === "errore" ? "it-error" : "it-warning-circle"} dimensione="xs" />
            <span>{c.messaggio}{#if c.codice}<small class="codice"> · codice SdI {c.codice}</small>{/if}</span>
          </li>
        {/each}
      </ul>
      <p class="nota">
        Controlli formali e di calcolo basati sulle specifiche pubbliche del Sistema di Interscambio: aiutano a individuare
        le anomalie più comuni, ma non sostituiscono i controlli dello SdI.
      </p>
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

  <div class="contenuto" id="pannello-documento" role="tabpanel" aria-labelledby="scheda-{scheda}" aria-busy={caricamento}>
    {#if caricamento && !dettaglio}
      <div class="stato" role="status"><div class="progress-spinner progress-spinner-active"></div><span class="visually-hidden">Apertura del documento</span></div>
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
          <div class="stato" role="status"><div class="progress-spinner progress-spinner-active"></div><span class="visually-hidden">Apertura del documento</span></div>
        {/if}
      {:else}
        <pre class="xml selezionabile" class:a-capo={!righeXml} style="font-size: {0.8 * zoom}rem">{#if righeXml}{#each righeXml as r, i (i)}<span class="riga-xml" style="padding-left: {r.rientro * 1.5}em">{#each r.parti as p, j (j)}<span class="x-{p.tipo}">{p.testo}</span>{/each}</span>
{/each}{:else}{dettaglio.xml}{/if}</pre>
      {/if}
    {/if}
  </div>
  {#if allegatoAperto !== null}
    <Visualizzatore
      file={voce.file}
      corpo={voce.corpo}
      indice={allegatoAperto}
      nome={allegati[allegatoAperto]?.nome ?? `allegato ${allegatoAperto + 1}`}
      onChiudi={() => (allegatoAperto = null)}
      {onErrore}
    />
  {/if}
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
    fill: #a37400;
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
  .chip-ri.sdi-ok {
    color: var(--ri-successo);
  }
  .chip-ri.sdi-ok :global(.icon) {
    fill: var(--ri-successo);
  }
  .chip-ri.sdi-errore {
    color: var(--ri-errore);
    border-color: var(--ri-errore);
  }
  .chip-ri.sdi-errore :global(.icon) {
    fill: var(--ri-errore);
  }
  .chip-ri.sdi-avviso {
    color: var(--ri-avviso);
  }
  .chip-ri.sdi-avviso :global(.icon) {
    fill: var(--ri-avviso);
  }
  .chip-ri.sdi-info :global(.icon) {
    fill: var(--ri-primario);
  }
  .cronologia {
    list-style: none;
    margin: 0;
    padding: 0 0 0 0.25rem;
    font-size: 0.88rem;
  }
  .cronologia li {
    border-left: 3px solid var(--ri-bordo);
    padding: 0.2rem 0 0.5rem 0.75rem;
  }
  .cronologia li.sdi-ok {
    border-left-color: var(--ri-successo);
  }
  .cronologia li.sdi-errore {
    border-left-color: var(--ri-errore);
  }
  .cronologia li.sdi-avviso {
    border-left-color: var(--ri-avviso);
  }
  .chip-ri.controlli-errore {
    color: var(--ri-errore);
    border-color: var(--ri-errore);
  }
  .chip-ri.controlli-errore :global(.icon) {
    fill: var(--ri-errore);
  }
  .chip-ri.controlli-avviso {
    color: var(--ri-avviso);
  }
  .chip-ri.controlli-avviso :global(.icon) {
    fill: var(--ri-avviso);
  }
  .elenco-controlli {
    list-style: none;
    margin: 0 0 0.5rem;
    padding: 0;
  }
  .elenco-controlli li {
    display: flex;
    gap: 0.5rem;
    align-items: flex-start;
    background: var(--ri-pannello);
    border: 1px solid var(--ri-bordo);
    border-radius: 6px;
    padding: 0.4rem 0.6rem;
    margin-bottom: 0.35rem;
  }
  .elenco-controlli :global(.icon) {
    flex: none;
    margin-top: 0.2rem;
  }
  .controllo-errore :global(.icon) {
    fill: var(--ri-errore);
  }
  .controllo-avviso :global(.icon) {
    fill: var(--ri-avviso);
  }
  .codice {
    color: var(--ri-testo-2);
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
    color: var(--ri-testo-2);
    font-style: italic;
  }
  .x-pi {
    color: var(--ri-testo-2);
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
  /* Pulsanti piccoli degli allegati: area attiva di almeno 24 px (WCAG 2.5.8). */
  .allegato :global(.btn) {
    min-width: 24px;
    min-height: 24px;
  }
</style>
