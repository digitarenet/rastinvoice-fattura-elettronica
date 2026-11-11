// Copyright (c) 2026 Andrea Re - info@digitare.net
// Autore: Andrea Re @ 2026 - info@digitare.net
//
// Tipi per i moduli importati da percorsi senza dichiarazioni proprie.

declare module "mammoth/mammoth.browser.js" {
  const mammoth: typeof import("mammoth");
  export default mammoth;
}
