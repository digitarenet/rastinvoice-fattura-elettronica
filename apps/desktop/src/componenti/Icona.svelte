<!--
  Copyright (c) 2026 Andrea Re - info@digitare.net
  Autore: Andrea Re @ 2026 - info@digitare.net

  Icona dallo sprite SVG di Bootstrap Italia.
-->
<script lang="ts">
  import sprite from "bootstrap-italia/dist/svg/sprites.svg?url";

  let {
    nome,
    classe = "",
    dimensione = "sm",
    titolo = "",
  }: { nome: string; classe?: string; dimensione?: "xs" | "sm" | "" | "lg"; titolo?: string } = $props();
</script>

<svg
  class="icon {dimensione ? `icon-${dimensione}` : ''} {classe}"
  aria-hidden={titolo ? undefined : "true"}
  role={titolo ? "img" : undefined}
>
  {#if titolo}<title>{titolo}</title>{/if}
  <use href="{sprite}#{nome}"></use>
</svg>
