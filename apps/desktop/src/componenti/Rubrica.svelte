<!--
  Copyright (c) 2026 Andrea Re - info@digitare.net
  Autore: Andrea Re @ 2026 - info@digitare.net

  Rubrica di clienti e fornitori ricavata dai documenti in archivio.
-->
<script lang="ts">
  import type { Voce } from "../lib/api";
  import { rubrica, importoConSegno, type Contatto } from "../lib/archivio";
  import { data, importo, iniziali } from "../lib/formato";
  import Icona from "./Icona.svelte";

  let {
    voci,
    propri,
    onApriDocumenti,
    onApriDocumento,
  }: {
    voci: Voce[];
    propri: Set<string>;
    onApriDocumenti: (chiave: string) => void;
    onApriDocumento: (v: Voce) => void;
  } = $props();

  let ruolo = $state<"tutti" | "cliente" | "fornitore">("tutti");
  let ricerca = $state("");
  let dal = $state("");
  let al = $state("");
  let ordine = $state<"nome" | "totale" | "recenti">("nome");
  let selezionato = $state<string | null>(null);

  const contatti = $derived.by(() => {
    const t = ricerca.toLowerCase().trim();
    const elenco = rubrica(voci, propri, dal, al).filter(
      (c) =>
        (ruolo === "tutti" || c.ruoli.has(ruolo)) &&
        (!t || c.nome.toLowerCase().includes(t) || c.chiave.toLowerCase().includes(t) || c.comune.toLowerCase().includes(t)),
    );
    const cmp: Record<typeof ordine, (a: Contatto, b: Contatto) => number> = {
      nome: (a, b) => a.nome.localeCompare(b.nome, "it", { sensitivity: "base" }),
      totale: (a, b) => b.totale - a.totale,
      recenti: (a, b) => b.ultimo.localeCompare(a.ultimo),
    };
    return elenco.sort(cmp[ordine]);
  });

  const corrente = $derived(contatti.find((c) => c.chiave === selezionato) ?? null);

  const documenti = $derived(
    corrente
      ? voci
          .filter(
            (v) =>
              (v.cedente.chiave === corrente.chiave || v.cessionario.chiave === corrente.chiave) &&
              (!dal || v.data >= dal) &&
              (!al || v.data <= al),
          )
          .sort((a, b) => b.data.localeCompare(a.data))
      : [],
  );

  // Totali per anno del contatto selezionato.
  const perAnno = $derived.by(() => {
    const m = new Map<number, number>();
    for (const v of documenti) m.set(v.anno, (m.get(v.anno) ?? 0) + importoConSegno(v));
    return [...m.entries()].sort((a, b) => b[0] - a[0]);
  });

  function etichettaChiave(c: string): string {
    return c.startsWith("CF:") ? `C.F. ${c.slice(3)}` : `P.IVA ${c}`;
  }
</script>

<div class="rubrica">
  <aside class="elenco ri-pannello">
    <div class="filtri">
      <div class="gruppo" role="group" aria-label="Ruolo">
        <button class:attivo={ruolo === "tutti"} onclick={() => (ruolo = "tutti")}>Tutti</button>
        <button class:attivo={ruolo === "cliente"} onclick={() => (ruolo = "cliente")}>Clienti</button>
        <button class:attivo={ruolo === "fornitore"} onclick={() => (ruolo = "fornitore")}>Fornitori</button>
      </div>
      <input class="form-control form-control-sm" type="search" placeholder="Cerca per nome, P.IVA, comune" bind:value={ricerca} />
      <div class="date">
        <label>Dal <input class="form-control form-control-sm" type="date" bind:value={dal} /></label>
        <label>Al <input class="form-control form-control-sm" type="date" bind:value={al} /></label>
      </div>
      <div class="d-flex justify-content-between align-items-center">
        <small class="text-secondary-ri">{contatti.length} {contatti.length === 1 ? "contatto" : "contatti"}</small>
        <select class="form-select form-select-sm w-auto" bind:value={ordine} aria-label="Ordina">
          <option value="nome">Nome A-Z</option>
          <option value="totale">Totale</option>
          <option value="recenti">Più recenti</option>
        </select>
      </div>
    </div>
    <ul class="ri-scroll">
      {#each contatti as c (c.chiave)}
        <li>
          <button class:selezionato={c.chiave === selezionato} onclick={() => (selezionato = c.chiave)}>
            <span class="avatar-ri" class:fornitore={!c.ruoli.has("cliente")}>{iniziali(c.nome)}</span>
            <span class="testo">
              <strong>{c.nome || c.chiave}</strong>
              <small class="text-secondary-ri">
                {[...c.ruoli].map((r) => (r === "cliente" ? "Cliente" : "Fornitore")).join(" e ")} · {c.documenti} doc.
              </small>
            </span>
            <span class="importo">{importo(c.totale.toFixed(2))}</span>
          </button>
        </li>
      {:else}
        <li class="nessuno text-secondary-ri">Nessun contatto con questi filtri.</li>
      {/each}
    </ul>
  </aside>

  <section class="scheda ri-scroll">
    {#if corrente}
      <header>
        <span class="avatar-ri grande" class:fornitore={!corrente.ruoli.has("cliente")}>{iniziali(corrente.nome)}</span>
        <div>
          <h2 class="h4 mb-1 selezionabile">{corrente.nome}</h2>
          <div class="text-secondary-ri selezionabile">
            {etichettaChiave(corrente.chiave)}{corrente.comune ? ` · ${corrente.comune}` : ""}
          </div>
        </div>
        <button class="btn btn-outline-primary btn-sm ms-auto text-nowrap" onclick={() => onApriDocumenti(corrente.chiave)}>
          <Icona nome="it-list" /> Vedi in elenco
        </button>
      </header>

      <div class="riquadri">
        <div class="riquadro"><span>Documenti</span><strong>{documenti.length}</strong></div>
        <div class="riquadro"><span>Totale</span><strong>{importo(corrente.totale.toFixed(2))}</strong></div>
        <div class="riquadro"><span>Primo</span><strong>{data(corrente.primo)}</strong></div>
        <div class="riquadro"><span>Ultimo</span><strong>{data(corrente.ultimo)}</strong></div>
      </div>

      {#if perAnno.length > 1}
        <h3 class="h6 mt-4">Totale per anno</h3>
        <div class="anni">
          {#each perAnno as [anno, tot] (anno)}
            <span class="badge-anno"><strong>{anno}</strong> {importo(tot.toFixed(2))}</span>
          {/each}
        </div>
      {/if}

      <h3 class="h6 mt-4">Documenti</h3>
      <table class="table table-sm table-hover">
        <thead>
          <tr><th>Data</th><th>Numero</th><th>Tipo</th><th>Ruolo</th><th class="text-end">Totale</th></tr>
        </thead>
        <tbody>
          {#each documenti as v (v.id)}
            <tr onclick={() => onApriDocumento(v)} class="cliccabile">
              <td>{data(v.data)}</td>
              <td>{v.numero}</td>
              <td title={v.tipo_documento}>{v.descrizione_tipo || v.tipo_documento}</td>
              <td>{v.cedente.chiave === corrente.chiave ? "Fornitore" : "Cliente"}</td>
              <td class="text-end">{importo(v.totale, v.divisa)}</td>
            </tr>
          {/each}
        </tbody>
      </table>
    {:else}
      <div class="vuoto text-secondary-ri">
        <Icona nome="it-team-digitale" dimensione="lg" />
        <p class="mt-3">
          Seleziona un contatto per vedere i documenti.<br />
          {#if propri.size === 0}
            Imposta la tua partita IVA nelle impostazioni per separare clienti e fornitori da te stesso.
          {/if}
        </p>
      </div>
    {/if}
  </section>
</div>

<style>
  .rubrica {
    display: grid;
    grid-template-columns: 380px 1fr;
    height: 100%;
    min-height: 0;
  }
  .elenco {
    display: flex;
    flex-direction: column;
    border-right: 1px solid var(--ri-bordo);
    min-height: 0;
  }
  .filtri {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
    padding: 0.75rem;
    border-bottom: 1px solid var(--ri-bordo);
  }
  .gruppo {
    display: flex;
    border: 1px solid var(--ri-primario);
    border-radius: 6px;
    overflow: hidden;
  }
  .gruppo button {
    flex: 1;
    border: 0;
    background: none;
    color: var(--ri-primario);
    font-weight: 600;
    padding: 0.3rem;
    font-size: 0.9rem;
  }
  .gruppo button.attivo {
    background: var(--ri-primario);
    color: #fff;
  }
  .date {
    display: flex;
    gap: 0.5rem;
  }
  .date label {
    flex: 1;
    font-size: 0.8rem;
    color: var(--ri-testo-2);
    display: flex;
    align-items: center;
    gap: 0.3rem;
    position: static;
    transform: none;
    width: auto;
  }
  ul {
    list-style: none;
    margin: 0;
    padding: 0;
    flex: 1;
  }
  li button {
    display: flex;
    align-items: center;
    gap: 0.6rem;
    width: 100%;
    border: 0;
    background: none;
    color: inherit;
    text-align: left;
    padding: 0.55rem 0.75rem 0.55rem 0.9rem;
    border: 0 !important;
  }
  li {
    border-bottom: 1px solid var(--ri-bordo);
  }
  li button:focus:not(:focus-visible) {
    box-shadow: none !important;
    outline: none !important;
  }
  li button.selezionato:focus:not(:focus-visible) {
    box-shadow: inset 3px 0 0 var(--ri-primario) !important;
  }
  li button:hover {
    background: var(--ri-sfondo);
  }
  li button.selezionato {
    background: var(--ri-selezione);
    box-shadow: inset 3px 0 0 var(--ri-primario);
  }
  .testo {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
    line-height: 1.25;
  }
  .testo strong {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    font-size: 0.92rem;
  }
  .importo {
    font-size: 0.85rem;
    font-variant-numeric: tabular-nums;
    white-space: nowrap;
  }
  .nessuno {
    padding: 1rem;
  }
  .avatar-ri {
    flex: none;
    width: 36px;
    height: 36px;
    border-radius: 50%;
    background: #d6e8fa;
    color: var(--ri-primario-scuro);
    display: flex;
    align-items: center;
    justify-content: center;
    font-weight: 700;
    font-size: 0.85rem;
  }
  .avatar-ri.fornitore {
    background: #d9f2e6;
    color: #006644;
  }
  .avatar-ri.grande {
    width: 56px;
    height: 56px;
    font-size: 1.2rem;
  }
  .scheda {
    padding: 1.5rem 2rem;
    min-height: 0;
  }
  header {
    display: flex;
    align-items: center;
    gap: 1rem;
  }
  .riquadri {
    display: grid;
    grid-template-columns: repeat(4, 1fr);
    gap: 0.75rem;
    margin-top: 1.25rem;
  }
  .riquadro {
    background: var(--ri-pannello);
    border: 1px solid var(--ri-bordo);
    border-radius: 8px;
    padding: 0.75rem 1rem;
    display: flex;
    flex-direction: column;
  }
  .riquadro span {
    font-size: 0.8rem;
    color: var(--ri-testo-2);
  }
  .riquadro strong {
    font-size: 1.15rem;
  }
  .anni {
    display: flex;
    flex-wrap: wrap;
    gap: 0.5rem;
  }
  .badge-anno {
    border: 1px solid var(--ri-bordo);
    border-radius: 6px;
    padding: 0.2rem 0.6rem;
    font-size: 0.85rem;
    background: var(--ri-pannello);
  }
  .table {
    font-size: 0.9rem;
    color: var(--ri-testo);
  }
  .cliccabile {
    cursor: pointer;
  }
  .vuoto {
    height: 100%;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    text-align: center;
  }
  .vuoto :global(.icon) {
    fill: var(--ri-testo-2);
  }
</style>
