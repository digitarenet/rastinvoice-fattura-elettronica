# Copyright (c) 2026 Andrea Re - info@digitare.net
# Autore: Andrea Re @ 2026 - info@digitare.net
#
# Genera i file SVG del logo di RastInvoice (marchio «Foglio» e logotipo).
# La R e' un foglio di fattura con l'angolo piegato; la gamba verde e' il documento verificato.
# Il logotipo e' in Titillium Web convertito in tracciati, cosi' i file non dipendono dal font.
# Uso: pip install fonttools && python3 store/immagini/logo/genera.py
from pathlib import Path
from fontTools.ttLib import TTFont
from fontTools.pens.svgPathPen import SVGPathPen
from fontTools.pens.transformPen import TransformPen

QUI = Path(__file__).resolve().parent
RADICE = QUI.parents[2]
FONT = RADICE / "crates/rustinvoice-render/font/TitilliumWeb-{}.ttf"
font = {p: TTFont(str(FONT).format(p)) for p in ("Bold", "Regular")}

# Colori di Bootstrap Italia
BLU, NOTTE, VERDE, CHIARO, MENTA = "#0066CC", "#17324D", "#008055", "#A6C8F0", "#7DE3B8"
INTESTAZIONE = "<!-- Copyright (c) 2026 Andrea Re - info@digitare.net - Autore: Andrea Re @ 2026 - info@digitare.net -->"


def testo(s, peso, corpo, x, y, spaziatura=0.0):
    """Converte il testo in un tracciato SVG; restituisce (d, larghezza)."""
    f = font[peso]
    glifi, mappa, metriche = f.getGlyphSet(), f.getBestCmap(), f["hmtx"]
    scala, cx, parti = corpo / 1000, x, []
    for c in s:
        nome = mappa[ord(c)]
        penna = SVGPathPen(glifi)
        glifi[nome].draw(TransformPen(penna, (scala, 0, 0, -scala, cx, y)))
        parti.append(penna.getCommands())
        cx += metriche[nome][0] * scala + spaziatura
    return " ".join(parti), cx - x - spaziatura


def marchio(r=BLU, gamba=VERDE, piega=CHIARO):
    """Marchio su griglia 100x100: R-foglio, angolo piegato e gamba staccata."""
    return (f'<path fill="{r}" fill-rule="evenodd" d="M14 8H64L88 32V36A24 24 0 0 1 64 60H40V92H14Z '
            f'M40 30H60A8 8 0 0 1 60 46H40Z"/>\n'
            f'  <path fill="{piega}" d="M64 8V26A6 6 0 0 0 70 32H88Z"/>\n'
            f'  <path fill="{gamba}" d="M48 64H73L92 92H66Z"/>')


def marchio_negativo():
    return marchio("#FFFFFF", MENTA, CHIARO)


def svg(larghezza, altezza, corpo):
    return (f'<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {larghezza:.0f} {altezza:.0f}" '
            f'role="img" aria-labelledby="t">\n  {INTESTAZIONE}\n  <title id="t">RastInvoice</title>\n  {corpo}\n</svg>\n')


def logo(negativo=False, slogan=True):
    """Marchio + «RastInvoice» (Rast in grassetto, Invoice in tondo), con o senza slogan."""
    corpo, larghezza = logo_corpo(negativo, slogan)
    return svg(larghezza, 100, corpo)


def logo_su_fondo():
    """Logo su un riquadro bianco arrotondato: leggibile su fondo chiaro e scuro (README di GitHub)."""
    corpo, larghezza = logo_corpo()
    m = 24
    return svg(larghezza + 2 * m, 100 + 2 * m,
               f'<rect width="{larghezza + 2 * m:.0f}" height="{100 + 2 * m}" rx="18" fill="#FFFFFF"/>\n'
               f'  <g transform="translate({m} {m})">{corpo}</g>')


def logo_corpo(negativo=False, slogan=True):
    c1, c2 = ("#FFFFFF", "#FFFFFF") if negativo else (NOTTE, BLU)
    base, x0 = (72 if slogan else 84), 122
    d1, l1 = testo("Rast", "Bold", 92, x0, base)
    d2, l2 = testo("Invoice", "Regular", 92, x0 + l1 + 1, base)
    corpo = (f'<g>{marchio_negativo() if negativo else marchio()}</g>\n'
             f'  <path fill="{c1}" d="{d1}"/>\n  <path fill="{c2}" d="{d2}"/>')
    larghezza = x0 + l1 + l2 + 6
    if slogan:
        d3, l3 = testo("VISUALIZZATORE DI FATTURE ELETTRONICHE", "Regular", 17.5, x0 + 3, 96, 2.2)
        corpo += f'\n  <path fill="{c1}" opacity=".8" d="{d3}"/>'
        larghezza = max(larghezza, x0 + l3 + 6)
    return corpo, larghezza


def icona(piccola=False):
    """Icona del programma: foglio bianco arrotondato con il marchio (piu' grande per 16-32 px)."""
    t, s = ("6 6", ".88") if piccola else ("12 12", ".76")
    bordo = "" if piccola else (f'\n  <rect x="1" y="1" width="98" height="98" rx="21" fill="none" '
                                f'stroke="{BLU}" stroke-opacity=".18" stroke-width="2"/>')
    return svg(100, 100, f'<rect width="100" height="100" rx="22" fill="#FFFFFF"/>{bordo}\n'
                         f'  <g transform="translate({t}) scale({s})">{marchio()}</g>')


FILE = {
    "marchio.svg": svg(100, 100, marchio()),
    "marchio-negativo.svg": svg(100, 100, marchio_negativo()),
    "logo.svg": logo(),
    "logo-negativo.svg": logo(negativo=True),
    "logo-su-fondo.svg": logo_su_fondo(),
    "logo-breve.svg": logo(slogan=False),
    "logo-breve-negativo.svg": logo(negativo=True, slogan=False),
    "icona.svg": icona(),
    "icona-piccola.svg": icona(piccola=True),
}

if __name__ == "__main__":
    for nome, contenuto in FILE.items():
        (QUI / nome).write_text(contenuto, encoding="utf-8")
        print(nome)
