// Copyright (c) 2026 Andrea Re - info@digitare.net
// Autore: Andrea Re @ 2026 - info@digitare.net
//
// Avvio dell'interfaccia.

import "bootstrap-italia/dist/css/bootstrap-italia.min.css";
import "./app.css";
import { mount } from "svelte";
import App from "./App.svelte";

// Come lo script di Bootstrap Italia: l'evidenziazione del focus resta solo
// per la navigazione da tastiera e non compare dopo un clic del mouse.
let ultimoDaMouse = false;
document.addEventListener("mousedown", () => (ultimoDaMouse = true), true);
document.addEventListener("keydown", (e) => {
  if (e.key === "Tab") {
    ultimoDaMouse = false;
    document.querySelectorAll("[data-focus-mouse]").forEach((el) => el.removeAttribute("data-focus-mouse"));
  }
}, true);
document.addEventListener("focusin", (e) => {
  const el = e.target as Element | null;
  if (!el?.setAttribute) return;
  if (ultimoDaMouse) el.setAttribute("data-focus-mouse", "true");
  else el.removeAttribute("data-focus-mouse");
});

const app = mount(App, { target: document.getElementById("app")! });

export default app;
