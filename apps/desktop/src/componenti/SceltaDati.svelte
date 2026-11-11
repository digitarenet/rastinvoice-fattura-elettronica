<!--
  Copyright (c) 2026 Andrea Re - info@digitare.net
  Autore: Andrea Re @ 2026 - info@digitare.net

  Scelta della cartella in cui salvare i dati dell'applicazione: profilo
  utente oppure accanto al programma (modalità portable, per chiavette USB).
-->
<script lang="ts">
  import { untrack } from "svelte";
  import { api, type Avvio } from "../lib/api";

  let {
    info,
    etichettaConferma = "Conferma",
    onFatto,
    onErrore,
  }: {
    info: Avvio;
    etichettaConferma?: string;
    onFatto: () => void;
    onErrore?: (m: string) => void;
  } = $props();
  let errore = $state("");

  // Scelta iniziale: la posizione attuale.
  let portable = $state(untrack(() => info.portable));
  let attesa = $state(false);
  const cambia = $derived(portable !== info.portable);

  async function conferma() {
    attesa = true;
    errore = "";
    try {
      // Se la cartella cambia, i dati vengono copiati e l'app si riavvia.
      await api.cambiaCartellaDati(portable);
      onFatto();
    } catch (e) {
      errore = `Impossibile usare questa cartella: ${e}`;
      onErrore?.(errore);
    } finally {
      attesa = false;
    }
  }
</script>

<p class="spiegazione">
  Qui vengono salvati impostazioni, elenco delle cartelle, indice per avvii rapidi, foglio di stile e copie dei file
  trascinati nell'app. Le fatture originali restano dove sono.
</p>
<div class="opzioni" role="radiogroup" aria-label="Cartella dei dati">
  <label class="opzione" class:scelta={!portable}>
    <input type="radio" name="dove-dati" checked={!portable} onchange={() => (portable = false)} />
    <span>
      <strong>Nel profilo utente</strong> <span class="consigliato">consigliato</span><br />
      <small>Ogni utente del computer ha i propri dati.</small><br />
      <code class="selezionabile">{info.cartella_standard}</code>
    </span>
  </label>
  <label class="opzione" class:scelta={portable} class:disabilitata={!info.portable_possibile}>
    <input type="radio" name="dove-dati" checked={portable} disabled={!info.portable_possibile} onchange={() => (portable = true)} />
    <span>
      <strong>Accanto al programma (portable)</strong><br />
      <small>
        {#if info.portable_possibile}
          Per usare l'app da una chiavetta USB o da una cartella condivisa: la cartella <code>dati</code> viene creata
          automaticamente.
        {:else}
          Non disponibile: nella cartella del programma non si può scrivere.
        {/if}
      </small><br />
      <code class="selezionabile">{info.cartella_portable}</code>
    </span>
  </label>
</div>
{#if cambia && !info.primo_avvio}
  <p class="nota">I dati attuali vengono copiati nella nuova cartella e l'applicazione si riavvia.</p>
{/if}
<button class="btn btn-primary btn-sm mt-2" disabled={attesa || (!cambia && !info.primo_avvio)} onclick={conferma}>
  {attesa ? "Attendere..." : cambia && !info.primo_avvio ? "Sposta i dati e riavvia" : etichettaConferma}
</button>

{#if errore}<p class="errore" role="alert">{errore}</p>{/if}

<style>
  .errore {
    color: var(--ri-errore);
    font-size: 0.88rem;
    margin: 0.5rem 0 0;
  }
  .spiegazione {
    font-size: 0.92rem;
  }
  .opzioni {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
  }
  .opzione {
    display: flex;
    gap: 0.75rem;
    align-items: flex-start;
    border: 1px solid var(--ri-bordo);
    border-radius: 6px;
    padding: 0.75rem 1rem;
    cursor: pointer;
    font-size: 0.92rem;
    background: var(--ri-pannello);
  }
  .opzione.scelta {
    border-color: var(--ri-primario);
    box-shadow: inset 3px 0 0 var(--ri-primario);
  }
  .opzione.disabilitata {
    opacity: 0.6;
    cursor: not-allowed;
  }
  .opzione input {
    margin-top: 0.3rem;
  }
  .opzione code {
    font-size: 0.8rem;
    word-break: break-all;
  }
  .consigliato {
    font-size: 0.75rem;
    font-weight: 600;
    color: var(--ri-successo);
  }
  .nota {
    font-size: 0.85rem;
    color: var(--ri-avviso);
    margin: 0.5rem 0 0;
  }
</style>
