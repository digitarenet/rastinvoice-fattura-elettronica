<!--
  Copyright (c) 2026 Andrea Re - info@digitare.net
  Autore: Andrea Re @ 2026 - info@digitare.net

  Pagina Credits: informazioni sull'applicazione, autore, librerie
  utilizzate e registro delle modifiche.
-->
<script lang="ts">
  import { api } from "../lib/api";
  import { blocchiMarkdown } from "../lib/markdown";
  import changelog from "../../../../CHANGELOG.md?raw";

  let { versione, onErrore }: { versione: string; onErrore: (m: string) => void } = $props();

  const librerie = [
    { nome: 'roxmltree', autori: 'Yevhenii Reizner', licenza: 'MIT / Apache-2.0' },
    { nome: 'encoding_rs', autori: 'Henri Sivonen', licenza: '(MIT / Apache-2.0) e BSD-3-Clause' },
    { nome: 'rust_decimal', autori: 'Paul Mason e contributori', licenza: 'MIT' },
    { nome: 'chrono', autori: 'Kang Seonghoon, Brandon W Maister e contributori', licenza: 'MIT / Apache-2.0' },
    { nome: 'serde, serde_json', autori: 'Erick Tryzelaar, David Tolnay', licenza: 'MIT / Apache-2.0' },
    { nome: 'thiserror', autori: 'David Tolnay', licenza: 'MIT / Apache-2.0' },
    { nome: 'base64', autori: 'Marshall Pierce, Alice Maz', licenza: 'MIT / Apache-2.0' },
    { nome: 'clap', autori: 'Kevin K., Ed Page e contributori', licenza: 'MIT / Apache-2.0' },
    { nome: 'walkdir', autori: 'Andrew Gallant', licenza: 'MIT / Unlicense' },
    { nome: 'Tauri e plugin dialog, opener', autori: 'Tauri Programme within The Commons Conservancy', licenza: 'MIT / Apache-2.0' },
    { nome: 'rayon', autori: 'Niko Matsakis, Josh Stone', licenza: 'MIT / Apache-2.0' },
    { nome: 'ureq', autori: 'Martin Algesten', licenza: 'MIT / Apache-2.0' },
    { nome: 'zip', autori: 'Mathijs van de Nes, Marli Frost, Ryan Levick, Chris Hennick', licenza: 'MIT' },
    { nome: 'rsa, sha1, sha2, p256, p384', autori: 'RustCrypto Developers', licenza: 'MIT / Apache-2.0' },
    { nome: 'calamine', autori: 'Johann Tuffe e contributori', licenza: 'MIT' },
    { nome: 'PDF.js', autori: 'Mozilla e contributori', licenza: 'Apache-2.0' },
    { nome: 'mammoth', autori: 'Michael Williamson', licenza: 'BSD-2-Clause' },
    { nome: 'Svelte', autori: 'Rich Harris e contributori', licenza: 'MIT' },
    { nome: 'Vite', autori: 'Evan You e contributori', licenza: 'MIT' },
    { nome: 'TypeScript', autori: 'Microsoft', licenza: 'Apache-2.0' },
    { nome: 'Bootstrap Italia', autori: 'Designers Italia / Dipartimento per la trasformazione digitale', licenza: 'BSD-3-Clause' },
    { nome: 'Titillium Web', autori: 'Accademia di Belle Arti di Urbino', licenza: 'SIL Open Font License 1.1' },
    { nome: 'Roboto Mono', autori: 'Christian Robertson', licenza: 'Apache-2.0' },
  ];

  // Il titolo e l'introduzione del registro sono già nella pagina.
  const registro = blocchiMarkdown(changelog.slice(Math.max(0, changelog.indexOf("\n## "))));
</script>

<section class="credits">
  <div class="intestazione">
    <h1 class="h3 mb-1">RastInvoice</h1>
    <p class="mb-1">Visualizzatore di fatture elettroniche FatturaPA, versione <strong>{versione}</strong>.</p>
    <p class="mb-3">Ideato e sviluppato da <strong>Andrea Re</strong>, che progetta software su misura per aziende,
      centri di ricerca e startup.</p>
    <button class="btn btn-primary btn-sm" onclick={() => api.apriSito().catch((e) => onErrore(String(e)))}>
      Visita digitare.net
    </button>
    <p class="piccolo mt-3 mb-0">Copyright © 2026 Andrea Re · info@digitare.net</p>
    <p class="piccolo mt-1 mb-0">Software libero con licenza pubblica dell'Unione europea EUPL-1.2.</p>
  </div>

  <h2 class="h5 mt-4">Librerie utilizzate</h2>
  <p>Si ringraziano gli autori delle librerie open source su cui si basa il programma.</p>
  <table class="table table-sm">
    <thead><tr><th>Libreria</th><th>Autori</th><th>Licenza</th></tr></thead>
    <tbody>
      {#each librerie as l (l.nome)}
        <tr><td>{l.nome}</td><td>{l.autori}</td><td>{l.licenza}</td></tr>
      {/each}
    </tbody>
  </table>
  <p class="piccolo">La vista AssoSoftware usa il foglio di stile pubblicato da AssoSoftware, scaricato dal sito
    dell'associazione. L'interfaccia è realizzata con Bootstrap Italia, il kit ufficiale del design system di
    Designers Italia, e ne segue le linee guida.</p>

  <h2 class="h5 mt-4">Accessibilità</h2>
  <p>RastInvoice è progettato per i requisiti WCAG 2.1 livello AA, richiamati dalla norma EN 301 549 e dalle Linee
    guida AgID. Si usa interamente da tastiera, con il fuoco sempre visibile. I contrasti sono verificati, i messaggi
    di stato vengono annunciati e l'interfaccia si ingrandisce fino al 200% dalle Impostazioni. Il controllo
    automatico con axe-core non rileva violazioni; le prove con i lettori di schermo sono in corso. Per segnalare un
    problema: info@digitare.net.</p>

  <h2 class="h5 mt-4">Registro delle modifiche</h2>
  <div class="registro">
    {#each registro as b, i (i)}
      {#if b.tipo === "titolo"}
        <svelte:element this={b.livello === 2 ? "h3" : "h4"} class={b.livello === 2 ? "h6 mt-3" : "sottotitolo"}>{@html b.html}</svelte:element>
      {:else if b.tipo === "elenco"}
        <ul>{#each b.voci as v, j (j)}<li>{@html v}</li>{/each}</ul>
      {:else}
        <p>{@html b.html}</p>
      {/if}
    {/each}
  </div>
</section>

<style>
  .credits {
    width: 100%;
    padding: 1.5rem 2rem 3rem;
    overflow: auto;
    height: 100%;
  }
  .intestazione {
    border-bottom: 1px solid var(--ri-bordo);
    padding-bottom: 1.25rem;
  }
  .piccolo {
    font-size: 0.85rem;
    color: var(--ri-testo-2);
  }
  table {
    font-size: 0.9rem;
  }
  /* Registro più compatto del resto della pagina (scala tipografica
     di Bootstrap Italia: testo piccolo 0.875rem, titoli ridotti). */
  .registro {
    font-size: 0.875rem;
    line-height: 1.5;
  }
  .registro :global(h3) {
    font-size: 1rem;
  }
  .registro ul {
    padding-left: 1.25rem;
    margin-bottom: 0.5rem;
  }
  /* Bootstrap Italia assegna una dimensione propria a paragrafi ed elenchi. */
  .registro li,
  .registro p {
    font-size: inherit;
    line-height: inherit;
    margin-bottom: 0.15rem;
  }
  .registro :global(.sottotitolo) {
    font-size: 0.8rem;
    text-transform: uppercase;
    letter-spacing: 0.04em;
    font-weight: 700;
    margin: 0.75rem 0 0.25rem;
    color: var(--ri-testo-2);
  }
  .registro :global(code) {
    font-family: "Roboto Mono", monospace;
    font-size: 0.85em;
  }
</style>
