// Copyright (c) 2026 Andrea Re - info@digitare.net
// Autore: Andrea Re @ 2026 - info@digitare.net
//
// Comportamento accessibile delle finestre modali: il fuoco entra nella
// finestra all'apertura, resta al suo interno con Tab e Maiusc+Tab e torna
// all'elemento di partenza alla chiusura. Esc chiama `chiudi`, se indicato.

const FOCALIZZABILI =
  'a[href], button:not([disabled]), input:not([disabled]), select:not([disabled]), textarea:not([disabled]), iframe, [tabindex]:not([tabindex="-1"])';

export function modale(nodo: HTMLElement, chiudi?: () => void) {
  const precedente = document.activeElement as HTMLElement | null;
  let chiusura = chiudi;

  const elementi = () =>
    Array.from(nodo.querySelectorAll<HTMLElement>(FOCALIZZABILI)).filter((e) => e.offsetParent !== null);

  // Primo campo selezionato, altrimenti il primo elemento attivabile.
  queueMicrotask(() => {
    const scelto = nodo.querySelector<HTMLElement>("input:checked, [autofocus]") ?? elementi()[0] ?? nodo;
    if (!nodo.contains(document.activeElement)) scelto.focus();
  });

  function tasti(e: KeyboardEvent) {
    if (e.key === "Escape" && chiusura) {
      e.preventDefault();
      e.stopPropagation();
      chiusura();
      return;
    }
    if (e.key !== "Tab") return;
    const lista = elementi();
    if (lista.length === 0) {
      e.preventDefault();
      return;
    }
    const primo = lista[0];
    const ultimo = lista[lista.length - 1];
    if (e.shiftKey && document.activeElement === primo) {
      e.preventDefault();
      ultimo.focus();
    } else if (!e.shiftKey && document.activeElement === ultimo) {
      e.preventDefault();
      primo.focus();
    }
  }

  nodo.addEventListener("keydown", tasti);
  return {
    update(nuova?: () => void) {
      chiusura = nuova;
    },
    destroy() {
      nodo.removeEventListener("keydown", tasti);
      if (precedente && document.contains(precedente)) precedente.focus();
    },
  };
}
