// Copyright (c) 2026 Andrea Re - info@digitare.net
// Autore: Andrea Re @ 2026 - info@digitare.net
//
// Formattazione leggibile dell'XML, solo per la visualizzazione: il file
// originale non viene mai toccato.

/** Una riga formattata, divisa in parti per la colorazione. */
export interface Parte {
  tipo: "tag" | "attr" | "valore" | "testo" | "commento" | "pi";
  testo: string;
}
export type Riga = { rientro: number; parti: Parte[] };

const MAX_TESTO_IN_LINEA = 120;

/**
 * Restituisce l'XML indentato riga per riga, oppure `null` se non è ben
 * formato (in tal caso si mostra il testo originale).
 */
export function formatta(xml: string): Riga[] | null {
  const doc = new DOMParser().parseFromString(xml, "application/xml");
  if (doc.getElementsByTagName("parsererror").length > 0) return null;
  const righe: Riga[] = [];

  // Dichiarazione XML iniziale, che il DOM non conserva.
  const dich = xml.match(/^﻿?\s*(<\?xml[^?]*\?>)/);
  if (dich) righe.push({ rientro: 0, parti: [{ tipo: "pi", testo: dich[1] }] });

  const apertura = (el: Element, chiudi: "" | "/"): Parte[] => {
    const p: Parte[] = [{ tipo: "tag", testo: `<${el.nodeName}` }];
    for (const a of Array.from(el.attributes)) {
      p.push({ tipo: "attr", testo: ` ${a.name}=` });
      p.push({ tipo: "valore", testo: `"${escAttr(a.value)}"` });
    }
    p.push({ tipo: "tag", testo: `${chiudi}>` });
    return p;
  };

  const visita = (nodo: Node, rientro: number) => {
    switch (nodo.nodeType) {
      case Node.ELEMENT_NODE: {
        const el = nodo as Element;
        const figli = Array.from(el.childNodes).filter(
          (n) => !(n.nodeType === Node.TEXT_NODE && !n.nodeValue?.trim()),
        );
        if (figli.length === 0) {
          righe.push({ rientro, parti: apertura(el, "/") });
        } else if (
          figli.length === 1 &&
          figli[0].nodeType === Node.TEXT_NODE &&
          (figli[0].nodeValue ?? "").length <= MAX_TESTO_IN_LINEA &&
          !(figli[0].nodeValue ?? "").includes("\n")
        ) {
          // Elemento semplice su una sola riga: <Numero>123</Numero>
          righe.push({
            rientro,
            parti: [
              ...apertura(el, ""),
              { tipo: "testo", testo: escTesto(figli[0].nodeValue ?? "") },
              { tipo: "tag", testo: `</${el.nodeName}>` },
            ],
          });
        } else {
          righe.push({ rientro, parti: apertura(el, "") });
          for (const f of figli) visita(f, rientro + 1);
          righe.push({ rientro, parti: [{ tipo: "tag", testo: `</${el.nodeName}>` }] });
        }
        break;
      }
      case Node.TEXT_NODE:
      case Node.CDATA_SECTION_NODE: {
        const t = nodo.nodeValue ?? "";
        if (t.trim()) {
          const testo = nodo.nodeType === Node.CDATA_SECTION_NODE ? `<![CDATA[${t}]]>` : escTesto(t.trim());
          // Testi lunghi (es. allegati in base64) vengono accorciati.
          righe.push({ rientro, parti: [{ tipo: "testo", testo: accorcia(testo) }] });
        }
        break;
      }
      case Node.COMMENT_NODE:
        righe.push({ rientro, parti: [{ tipo: "commento", testo: `<!--${nodo.nodeValue}-->` }] });
        break;
      case Node.PROCESSING_INSTRUCTION_NODE: {
        const pi = nodo as ProcessingInstruction;
        righe.push({ rientro, parti: [{ tipo: "pi", testo: `<?${pi.target} ${pi.data}?>` }] });
        break;
      }
    }
  };

  for (const n of Array.from(doc.childNodes)) visita(n, 0);
  return righe;
}

/** I contenuti molto lunghi (allegati in base64) restano leggibili. */
function accorcia(s: string): string {
  if (s.length <= 2000) return s;
  return `${s.slice(0, 160)} … [${(s.length / 1024).toFixed(0)} kB omessi] … ${s.slice(-40)}`;
}

function escTesto(s: string): string {
  return s.replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;");
}

function escAttr(s: string): string {
  return escTesto(s).replace(/"/g, "&quot;");
}
