/*
 * Copyright (c) 2026 Andrea Re - info@digitare.net
 * Autore: Andrea Re @ 2026 - info@digitare.net
 *
 * Comportamenti del sito: menu su schermi piccoli, galleria a schede, campagna nei link degli store,
 * invio del modulo contatti.
 * Il sito resta utilizzabile anche senza JavaScript.
 */
(function () {
  "use strict";

  // Menu su schermi piccoli
  var pulsante = document.querySelector(".apri-menu");
  var menu = document.getElementById("menu");
  if (pulsante && menu) {
    pulsante.addEventListener("click", function () {
      var aperto = menu.classList.toggle("aperto");
      pulsante.setAttribute("aria-expanded", aperto ? "true" : "false");
    });
    menu.addEventListener("click", function (e) {
      if (e.target.closest("a")) {
        menu.classList.remove("aperto");
        pulsante.setAttribute("aria-expanded", "false");
      }
    });
    document.addEventListener("keydown", function (e) {
      if (e.key === "Escape" && menu.classList.contains("aperto")) {
        menu.classList.remove("aperto");
        pulsante.setAttribute("aria-expanded", "false");
        pulsante.focus();
      }
    });
  }

  // Campagna nei link degli store: se la pagina è aperta con ?cid=<campagna> (per esempio da un annuncio
  // o da una guida), il codice passa ai pulsanti del Microsoft Store, così Partner Center conta le
  // installazioni per campagna. Sono ammessi solo lettere, cifre, trattino e trattino basso.
  var campagna = (window.location.search.match(/[?&]cid=([A-Za-z0-9_-]{1,64})(?:&|$)/) || [])[1];
  if (campagna) {
    Array.prototype.forEach.call(document.querySelectorAll('a[data-store="microsoft"]'), function (a) {
      var url = new URL(a.href);
      url.searchParams.set("cid", campagna);
      a.href = url.toString();
    });
  }

  // Galleria a schede (modello ARIA "tabs", con frecce, Home e Fine)
  var schede = Array.prototype.slice.call(document.querySelectorAll('.galleria-schede [role="tab"]'));
  var pannello = document.getElementById("vista-galleria");
  var immagine = document.getElementById("img-galleria");
  var didascalia = document.getElementById("testo-galleria");
  function seleziona(scheda, sposta) {
    schede.forEach(function (s) {
      var attiva = s === scheda;
      s.setAttribute("aria-selected", attiva ? "true" : "false");
      s.tabIndex = attiva ? 0 : -1;
    });
    var nome = scheda.getAttribute("data-img");
    var testo = scheda.getAttribute("data-testo");
    immagine.src = "img/" + nome + ".webp";
    immagine.srcset = "img/" + nome + "-400.webp 400w, img/" + nome + "-800.webp 800w, img/" + nome + ".webp 1600w";
    immagine.alt = testo;
    didascalia.textContent = testo;
    pannello.setAttribute("aria-labelledby", scheda.id);
    if (sposta) scheda.focus();
  }
  schede.forEach(function (s, i) {
    s.addEventListener("click", function () { seleziona(s, false); });
    s.addEventListener("keydown", function (e) {
      var j = null;
      if (e.key === "ArrowRight") j = (i + 1) % schede.length;
      else if (e.key === "ArrowLeft") j = (i - 1 + schede.length) % schede.length;
      else if (e.key === "Home") j = 0;
      else if (e.key === "End") j = schede.length - 1;
      if (j !== null) { e.preventDefault(); seleziona(schede[j], true); }
    });
  });

  // Modulo contatti
  var modulo = document.getElementById("modulo-contatti");
  if (!modulo) return;
  var esito = document.getElementById("esito");
  var gettone = document.getElementById("gettone-modulo");

  // Gettone firmato dal server: certifica l'ora di apertura del modulo (protezione dagli invii automatici)
  function nuovoGettone() {
    if (!gettone) return;
    fetch(modulo.action + "?gettone=1", { headers: { "Accept": "application/json" }, cache: "no-store" })
      .then(function (r) { return r.json(); })
      .then(function (dati) { if (dati && dati.gettone) gettone.value = dati.gettone; })
      .catch(function () { /* senza gettone l'invio segue il percorso senza JavaScript */ });
  }
  nuovoGettone();

  function avviso(tipo, testo) {
    esito.innerHTML = "";
    var div = document.createElement("div");
    div.className = "alert alert-" + tipo;
    div.textContent = testo;
    esito.appendChild(div);
  }

  function controlla() {
    var errori = [];
    ["nome", "email", "messaggio"].forEach(function (id) {
      var campo = document.getElementById(id);
      var valido = campo.value.trim() !== "" && campo.checkValidity();
      campo.setAttribute("aria-invalid", valido ? "false" : "true");
      if (!valido) errori.push(campo);
    });
    var privacy = document.getElementById("privacy");
    if (!privacy.checked) errori.push(privacy);
    return errori;
  }

  // Ritorno dal modulo inviato senza JavaScript, con un errore
  if (/[?&]errore=1/.test(window.location.search)) {
    avviso("danger", "Non è stato possibile inviare il messaggio. Controlla i campi e riprova, oppure scrivi a rastinvoice@digitare.net.");
  }

  modulo.addEventListener("submit", function (e) {
    e.preventDefault();
    var errori = controlla();
    if (errori.length) {
      avviso("danger", "Controlla i campi: nome, email valida, messaggio e consenso all'informativa sono obbligatori.");
      errori[0].focus();
      return;
    }
    var invio = modulo.querySelector('button[type="submit"]');
    invio.disabled = true;
    avviso("info", "Invio in corso…");
    fetch(modulo.action, {
      method: "POST",
      body: new FormData(modulo),
      headers: { "Accept": "application/json" }
    })
      .then(function (r) { return r.json().catch(function () { return { ok: false }; }); })
      .then(function (dati) {
        if (dati.ok) {
          modulo.reset();
          nuovoGettone();
          avviso("success", "Grazie, il messaggio è stato inviato. Ti risponderemo il prima possibile.");
        } else {
          avviso("danger", dati.errore || "Non è stato possibile inviare il messaggio. Riprova più tardi o scrivi a rastinvoice@digitare.net.");
        }
      })
      .catch(function () {
        avviso("danger", "Non è stato possibile inviare il messaggio. Riprova più tardi o scrivi a rastinvoice@digitare.net.");
      })
      .then(function () { invio.disabled = false; });
  });
})();
