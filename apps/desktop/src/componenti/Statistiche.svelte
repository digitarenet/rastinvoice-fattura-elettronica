<!--
  Copyright (c) 2026 Andrea Re - info@digitare.net
  Autore: Andrea Re @ 2026 - info@digitare.net

  Statistiche globali: fatturato, acquisti, IVA a debito e a credito,
  andamento per mese o per anno e classifica di clienti e fornitori.
-->
<script lang="ts">
  import { untrack } from "svelte";
  import type { Voce } from "../lib/api";
  import { breve, calcola, euro, type PerControparte } from "../lib/statistiche";
  import Icona from "./Icona.svelte";

  let {
    voci,
    propri,
    onApriImpostazioni,
    onMostraContatto,
  }: {
    voci: Voce[];
    propri: Set<string>;
    onApriImpostazioni: () => void;
    onMostraContatto: (chiave: string) => void;
  } = $props();

  // Anno mostrato: inizialmente il più recente.
  let anno = $state<number | null>(
    untrack(() => {
      const a = voci.map((v) => v.anno).filter((x) => x > 0);
      return a.length ? Math.max(...a) : null;
    }),
  );
  const s = $derived(calcola(voci, propri, anno));
  const saldoIva = $derived(s.emesse.imposta - s.emesse.scissione - s.ricevute.imposta);

  let tuttiClienti = $state(false);
  let tuttiFornitori = $state(false);
  let tabellaGrafico = $state(false);

  // Grafico a barre affiancate (imponibile emesse/ricevute per periodo).
  const ALTEZZA = 220;
  const MARGINE_SX = 52;
  const MARGINE_BASSO = 24;
  let larghezza = $state(700);
  let sopra = $state<number | null>(null);
  // Fondo scala arrotondato a un valore "tondo" (1, 2, 2,5 o 5 per una potenza di 10).
  function tondo(n: number): number {
    if (n <= 0) return 0;
    const p = 10 ** Math.floor(Math.log10(n));
    for (const f of [1, 2, 2.5, 5, 10]) if (n <= f * p) return f * p;
    return 10 * p;
  }
  const massimo = $derived(tondo(Math.max(1, ...s.periodi.flatMap((p) => [p.emesse, p.ricevute]))));
  const minimo = $derived(-tondo(-Math.min(0, ...s.periodi.flatMap((p) => [p.emesse, p.ricevute]))));
  const scala = $derived((n: number) => ((massimo - n) / (massimo - minimo)) * (ALTEZZA - MARGINE_BASSO - 8) + 8);
  const tacche = $derived([0, 0.2, 0.4, 0.6, 0.8, 1].map((f) => minimo + (massimo - minimo) * f));
  const gruppo = $derived((larghezza - MARGINE_SX) / Math.max(1, s.periodi.length));
  const barra = $derived(Math.max(4, Math.min(28, gruppo * 0.32)));

  function percentuale(c: PerControparte, elenco: PerControparte[]): number {
    const max = Math.max(...elenco.map((x) => Math.abs(x.imponibile)), 1);
    return (Math.abs(c.imponibile) / max) * 100;
  }
</script>

<section class="statistiche ri-scroll">
  <header class="intestazione">
    <div>
      <h1 class="h4 mb-1">Statistiche</h1>
      <p class="text-secondary-ri mb-0">
        Valori indicativi calcolati dalle fatture lette: non sostituiscono i registri IVA né la liquidazione periodica.
      </p>
    </div>
    <label class="scelta-anno">
      <span>Periodo</span>
      <select class="form-select form-select-sm" bind:value={anno}>
        <option value={null}>Tutti gli anni</option>
        {#each [...s.anni].reverse() as a (a)}<option value={a}>{a}</option>{/each}
      </select>
    </label>
  </header>

  {#if propri.size === 0}
    <div class="callout-ri">
      <Icona nome="it-info-circle" />
      <div>
        <strong>Indica la tua partita IVA</strong> per distinguere fatture emesse (fatturato, IVA a debito) e ricevute
        (acquisti, IVA a credito).
        <button class="btn btn-link btn-xs p-0" onclick={onApriImpostazioni}>Vai alle impostazioni</button>
      </div>
    </div>
  {:else}
    <div class="riquadri">
      <div class="riquadro">
        <span class="etichetta"><span class="punto emesse"></span>Fatturato (imponibile)</span>
        <strong class="valore">€ {euro(s.emesse.imponibile)}</strong>
        <span class="dettaglio">{s.emesse.documenti} documenti emessi · totale € {euro(s.emesse.totale)}</span>
      </div>
      <div class="riquadro">
        <span class="etichetta">IVA a debito</span>
        <strong class="valore">€ {euro(s.emesse.imposta)}</strong>
        <span class="dettaglio">
          {#if s.emesse.scissione}di cui € {euro(s.emesse.scissione)} in scissione dei pagamenti{:else}sulle fatture emesse{/if}
        </span>
      </div>
      <div class="riquadro">
        <span class="etichetta"><span class="punto ricevute"></span>Acquisti (imponibile)</span>
        <strong class="valore">€ {euro(s.ricevute.imponibile)}</strong>
        <span class="dettaglio">{s.ricevute.documenti} documenti ricevuti · totale € {euro(s.ricevute.totale)}</span>
      </div>
      <div class="riquadro">
        <span class="etichetta">IVA a credito</span>
        <strong class="valore">€ {euro(s.ricevute.imposta)}</strong>
        <span class="dettaglio">sulle fatture ricevute</span>
      </div>
      <div class="riquadro">
        <span class="etichetta">Saldo IVA indicativo</span>
        <strong class="valore">€ {euro(saldoIva)}</strong>
        <span class="dettaglio">{saldoIva >= 0 ? "a debito" : "a credito"} (debito − scissione − credito)</span>
      </div>
    </div>

    <div class="pannello">
      <div class="titolo-pannello">
        <h2 class="h6 mb-0">Imponibile {anno === null ? "per anno" : `per mese nel ${anno}`}</h2>
        <div class="legenda" aria-hidden={tabellaGrafico}>
          <span><span class="punto emesse"></span>Emesse</span>
          <span><span class="punto ricevute"></span>Ricevute</span>
          <button class="btn btn-link btn-xs p-0" aria-pressed={tabellaGrafico} onclick={() => (tabellaGrafico = !tabellaGrafico)}>
            {tabellaGrafico ? "Mostra grafico" : "Mostra tabella"}
          </button>
        </div>
      </div>
      {#if tabellaGrafico}
        <table class="table table-sm tabella-dati">
          <caption class="visually-hidden">Imponibile delle fatture emesse e ricevute per periodo</caption>
          <thead><tr><th>Periodo</th><th class="num">Emesse</th><th class="num">Ricevute</th></tr></thead>
          <tbody>
            {#each s.periodi as p (p.etichetta)}
              <tr><td>{p.etichetta}</td><td class="num">€ {euro(p.emesse)}</td><td class="num">€ {euro(p.ricevute)}</td></tr>
            {/each}
          </tbody>
        </table>
      {:else}
        <div class="grafico" bind:clientWidth={larghezza}>
          <svg width={larghezza} height={ALTEZZA} role="img" aria-label="Grafico a barre dell'imponibile di emesse e ricevute per periodo: i valori sono nella tabella, pulsante Mostra tabella">
            {#each tacche as t (t)}
              <line class="griglia" x1={MARGINE_SX} x2={larghezza} y1={scala(t)} y2={scala(t)} />
              <text class="asse" x={MARGINE_SX - 8} y={scala(t) + 4} text-anchor="end">{breve(t)}</text>
            {/each}
            <line class="base" x1={MARGINE_SX} x2={larghezza} y1={scala(0)} y2={scala(0)} />
            {#each s.periodi as p, i (p.etichetta)}
              {@const x = MARGINE_SX + i * gruppo + gruppo / 2}
              <g
                role="presentation"
                onmouseenter={() => (sopra = i)}
                onmouseleave={() => (sopra = null)}
              >
                <rect class="bersaglio" x={x - gruppo / 2} y="0" width={gruppo} height={ALTEZZA - MARGINE_BASSO} />
                <rect class="barra emesse" x={x - barra - 1} width={barra} y={Math.min(scala(p.emesse), scala(0))}
                  height={Math.max(p.emesse === 0 ? 0 : 1, Math.abs(scala(0) - scala(p.emesse)))} rx="3" />
                <rect class="barra ricevute" x={x + 1} width={barra} y={Math.min(scala(p.ricevute), scala(0))}
                  height={Math.max(p.ricevute === 0 ? 0 : 1, Math.abs(scala(0) - scala(p.ricevute)))} rx="3" />
                <text class="asse" x={x} y={ALTEZZA - 6} text-anchor="middle">{p.etichetta}</text>
              </g>
            {/each}
          </svg>
          {#if sopra !== null && s.periodi[sopra]}
            {@const p = s.periodi[sopra]}
            <div class="suggerimento" style="left: {Math.min(larghezza - 190, MARGINE_SX + sopra * gruppo + gruppo / 2 + 12)}px">
              <strong>{p.etichetta}</strong>
              <div><span class="punto emesse"></span>Emesse € {euro(p.emesse)}</div>
              <div><span class="punto ricevute"></span>Ricevute € {euro(p.ricevute)}</div>
            </div>
          {/if}
        </div>
      {/if}
    </div>

    <div class="colonne">
      {#each [{ titolo: "Clienti", elenco: s.clienti, tutti: tuttiClienti, classe: "emesse", iva: "IVA a debito" }, { titolo: "Fornitori", elenco: s.fornitori, tutti: tuttiFornitori, classe: "ricevute", iva: "IVA a credito" }] as blocco (blocco.titolo)}
        <div class="pannello">
          <div class="titolo-pannello">
            <h2 class="h6 mb-0">{blocco.titolo} <span class="text-secondary-ri">· {blocco.elenco.length}</span></h2>
          </div>
          {#if blocco.elenco.length === 0}
            <p class="text-secondary-ri mb-0">Nessun documento nel periodo.</p>
          {:else}
            <table class="table table-sm classifica">
              <thead>
                <tr><th>Nome</th><th class="num">Doc.</th><th class="num">Imponibile</th><th class="num">{blocco.iva}</th><th class="num">Totale</th></tr>
              </thead>
              <tbody>
                {#each blocco.tutti ? blocco.elenco : blocco.elenco.slice(0, 10) as c (c.chiave + c.nome)}
                  <tr>
                    <td>
                      <button class="btn btn-link btn-xs p-0 text-start" title="Mostra i documenti" onclick={() => onMostraContatto(c.chiave)}>{c.nome}</button>
                      <div class="quota" aria-hidden="true"><span class="{blocco.classe}" style="width: {percentuale(c, blocco.elenco)}%"></span></div>
                    </td>
                    <td class="num">{c.documenti}</td>
                    <td class="num">{euro(c.imponibile)}</td>
                    <td class="num">{euro(c.imposta)}</td>
                    <td class="num">{euro(c.totale)}</td>
                  </tr>
                {/each}
              </tbody>
            </table>
            {#if blocco.elenco.length > 10}
              <button class="btn btn-link btn-xs p-0" onclick={() => (blocco.titolo === "Clienti" ? (tuttiClienti = !tuttiClienti) : (tuttiFornitori = !tuttiFornitori))}>
                {blocco.tutti ? "Mostra i primi 10" : `Mostra tutti (${blocco.elenco.length})`}
              </button>
            {/if}
          {/if}
        </div>
      {/each}
    </div>
  {/if}
</section>

<style>
  .statistiche {
    height: 100%;
    overflow: auto;
    padding: 1.5rem 2rem 3rem;
    --serie-emesse: #0066cc;
    --serie-ricevute: #c26a00;
  }
  .intestazione {
    display: flex;
    justify-content: space-between;
    align-items: flex-end;
    gap: 1rem;
    margin-bottom: 1.25rem;
    flex-wrap: wrap;
  }
  .intestazione p {
    font-size: 0.88rem;
  }
  .scelta-anno {
    display: flex;
    flex-direction: column;
    font-size: 0.8rem;
    font-weight: 600;
    color: var(--ri-testo-2);
    min-width: 11rem;
  }
  .callout-ri {
    display: flex;
    gap: 0.75rem;
    align-items: flex-start;
    background: var(--ri-pannello);
    border: 1px solid var(--ri-bordo);
    border-left: 4px solid var(--ri-primario);
    padding: 1rem;
    border-radius: 4px;
  }
  .callout-ri :global(.icon) {
    fill: var(--ri-primario);
    flex: none;
  }
  .riquadri {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(190px, 1fr));
    gap: 0.75rem;
    margin-bottom: 1rem;
  }
  .riquadro {
    display: flex;
    flex-direction: column;
    gap: 0.15rem;
    background: var(--ri-pannello);
    border: 1px solid var(--ri-bordo);
    border-radius: 6px;
    padding: 0.85rem 1rem;
  }
  .etichetta {
    font-size: 0.78rem;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.03em;
    color: var(--ri-testo-2);
  }
  .valore {
    font-size: 1.35rem;
    font-variant-numeric: tabular-nums;
    color: var(--ri-testo);
  }
  .dettaglio {
    font-size: 0.8rem;
    color: var(--ri-testo-2);
  }
  .pannello {
    background: var(--ri-pannello);
    border: 1px solid var(--ri-bordo);
    border-radius: 6px;
    padding: 1rem 1.25rem;
    margin-bottom: 1rem;
    min-width: 0;
  }
  .titolo-pannello {
    display: flex;
    justify-content: space-between;
    align-items: center;
    gap: 1rem;
    margin-bottom: 0.75rem;
    flex-wrap: wrap;
  }
  .legenda {
    display: flex;
    gap: 1rem;
    align-items: center;
    font-size: 0.82rem;
    color: var(--ri-testo-2);
  }
  .punto {
    display: inline-block;
    width: 10px;
    height: 10px;
    border-radius: 2px;
    margin-right: 0.35rem;
    vertical-align: baseline;
  }
  .punto.emesse,
  .quota .emesse {
    background: var(--serie-emesse);
  }
  .punto.ricevute,
  .quota .ricevute {
    background: var(--serie-ricevute);
  }
  .grafico {
    position: relative;
    width: 100%;
  }
  svg {
    display: block;
  }
  .griglia {
    stroke: var(--ri-bordo);
    stroke-width: 1;
  }
  .base {
    stroke: var(--ri-testo-2);
    stroke-width: 1;
  }
  .asse {
    font-size: 11px;
    fill: var(--ri-testo-2);
  }
  .bersaglio {
    fill: transparent;
  }
  g:hover .bersaglio {
    fill: color-mix(in srgb, var(--ri-primario) 6%, transparent);
  }
  .barra.emesse {
    fill: var(--serie-emesse);
  }
  .barra.ricevute {
    fill: var(--serie-ricevute);
  }
  .suggerimento {
    position: absolute;
    top: 8px;
    width: 180px;
    background: var(--ri-pannello);
    border: 1px solid var(--ri-bordo);
    box-shadow: 0 4px 14px rgb(0 0 0 / 0.12);
    border-radius: 4px;
    padding: 0.5rem 0.65rem;
    font-size: 0.8rem;
    pointer-events: none;
    font-variant-numeric: tabular-nums;
  }
  .colonne {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(420px, 1fr));
    gap: 1rem;
  }
  .classifica,
  .tabella-dati {
    font-size: 0.85rem;
    color: var(--ri-testo);
    margin-bottom: 0.5rem;
  }
  .num {
    text-align: right;
    font-variant-numeric: tabular-nums;
    white-space: nowrap;
  }
  .quota {
    height: 4px;
    background: var(--ri-sfondo);
    border-radius: 2px;
    margin-top: 0.2rem;
    max-width: 220px;
  }
  .quota span {
    display: block;
    height: 100%;
    border-radius: 2px;
  }
</style>
