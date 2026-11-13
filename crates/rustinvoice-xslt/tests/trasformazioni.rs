// Copyright (c) 2026 Andrea Re - info@digitare.net
// Autore: Andrea Re @ 2026 - info@digitare.net
//
// Prove del motore XSLT su fogli di stile scritti per l'occasione. I
// risultati attesi sono stati confrontati con un motore XSLT di riferimento.

use rustinvoice_xslt::{trasforma, Errore};

const FOGLIO: &str = r##"<xsl:stylesheet version="1.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform">
<xsl:output method="html"/>
<xsl:variable name="rtf"><b>grassetto</b> e testo</xsl:variable>
<xsl:template match="/"><html><body>
 <xsl:for-each select="//r"><xsl:sort select="@n" data-type="number" order="descending"/><p class="r{position()}" id="{@n}"><xsl:value-of select="concat(name(), ':', ., ':', last())"/></p></xsl:for-each>
 <xsl:for-each select="//r"><xsl:sort select="."/><i><xsl:value-of select="."/></i></xsl:for-each>
 <xsl:apply-templates select="radice/*" mode="m"/>
 <xsl:copy-of select="$rtf"/><xsl:copy-of select="//r[2]"/>
 <div><xsl:attribute name="data-x"><xsl:value-of select="count(//r) * 2 div 3"/></xsl:attribute>
 <xsl:value-of select="substring('12345', 1.5, 2.6)"/>|<xsl:value-of select="substring-after('a=b','=')"/>|<xsl:value-of select="translate('abc','ab','A')"/>|<xsl:value-of select="round(-2.5)"/>|<xsl:value-of select="floor(2.7) + ceiling(2.1)"/>|<xsl:value-of select="10 mod 3"/>|<xsl:value-of select="1 div 0"/>|<xsl:value-of select="0.1 + 0.2"/>|<xsl:value-of select="sum(//r/@n)"/>|<xsl:value-of select="//r[@n &gt; 1][last()]/@n"/>|<xsl:value-of select="count(//r[1]/following-sibling::*)"/>|<xsl:value-of select="//r[3]/preceding-sibling::r[1]"/>|<xsl:value-of select="count(//r/ancestor::*)"/>|<xsl:value-of select="boolean(//x) = false()"/>|<xsl:value-of select="//r = 'beta'"/>|<xsl:value-of select="format-number(1234.5, '#,##0.00')"/>|<xsl:value-of select="string-length(normalize-space('  a   b '))"/>
 </div>
 <xsl:element name="h{1+1}">titolo &amp; &lt;x&gt;</xsl:element><br/><script>if (a &lt; b) x();</script>
 <xsl:comment>nota</xsl:comment><xsl:text disable-output-escaping="yes">&lt;hr&gt;</xsl:text>
</body></html></xsl:template>
<xsl:template match="r" mode="m"><em><xsl:copy><xsl:value-of select="@n"/></xsl:copy></em></xsl:template>
<xsl:template match="r[@n='2']" mode="m"><strong>due</strong></xsl:template>
<xsl:template match="text()" mode="m"/>
</xsl:stylesheet>
"##;

const DOCUMENTO: &str = r##"<radice><r n="2">beta</r><r n="10">alfa</r><r n="1">gamma &amp; delta</r><!-- c --></radice>
"##;

const ATTESO: &str = r##"<!DOCTYPE html>
<html><body><p class="r1" id="10">r:alfa:3</p><p class="r2" id="2">r:beta:3</p><p class="r3" id="1">r:gamma &amp; delta:3</p><i>alfa</i><i>beta</i><i>gamma &amp; delta</i><strong>due</strong><em><r>10</r></em><em><r>1</r></em><b>grassetto</b> e testo<r n="10">alfa</r><div data-x="2">234|b|Ac|-2|5|1|Infinity|0.3|13|10|2|alfa|1|true|true|1,234.50|3</div><h2>titolo &amp; &lt;x&gt;</h2><br><script>if (a < b) x();</script><!--nota--><hr></body></html>"##;

#[test]
fn istruzioni_e_funzioni_principali() {
    assert_eq!(trasforma(FOGLIO, DOCUMENTO).unwrap(), ATTESO);
}

#[test]
fn spazi_dei_nomi_e_parametri() {
    let foglio = r#"<xsl:stylesheet version="1.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform" xmlns:p="urn:prova">
      <xsl:output method="html"/>
      <xsl:template match="/"><xsl:call-template name="saluto"><xsl:with-param name="chi" select="p:Radice/Nome"/></xsl:call-template></xsl:template>
      <xsl:template name="saluto"><xsl:param name="chi"/><xsl:param name="come" select="'Ciao'"/><p><xsl:value-of select="concat($come, ' ', $chi)"/></p></xsl:template>
    </xsl:stylesheet>"#;
    let documento = r#"<q:Radice xmlns:q="urn:prova"><Nome>Andrea</Nome></q:Radice>"#;
    assert_eq!(
        trasforma(foglio, documento).unwrap(),
        "<!DOCTYPE html>
<p>Ciao Andrea</p>"
    );
}

#[test]
fn ricorsione_infinita_interrotta() {
    let foglio = r#"<xsl:stylesheet version="1.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform">
      <xsl:template match="/"><xsl:call-template name="giro"/></xsl:template>
      <xsl:template name="giro"><xsl:call-template name="giro"/></xsl:template>
    </xsl:stylesheet>"#;
    assert!(matches!(trasforma(foglio, "<a/>"), Err(Errore::Limite(_))));
}

#[test]
fn funzionalita_non_supportate_segnalate() {
    let foglio = r#"<xsl:stylesheet version="1.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform">
      <xsl:key name="k" match="a" use="@id"/>
    </xsl:stylesheet>"#;
    assert!(matches!(
        trasforma(foglio, "<a/>"),
        Err(Errore::NonSupportato(_))
    ));
}

#[test]
fn spazio_dei_nomi_predefinito_annullato() {
    // Fatture con la radice nello spazio dei nomi predefinito e i figli con `xmlns=""`
    // (per esempio quelle prodotte dal sistema BRAV): i figli non hanno spazio dei nomi.
    let foglio = r#"<xsl:stylesheet version="1.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform" xmlns:a="urn:f">
      <xsl:template match="/"><xsl:value-of select="a:F/H/N"/>|<xsl:value-of select="count(a:F/*[namespace-uri()=''])"/>|<xsl:apply-templates select="a:F/H"/></xsl:template>
      <xsl:template match="H">h</xsl:template>
    </xsl:stylesheet>"#;
    let xml = r#"<F xmlns="urn:f"><H xmlns=""><N>42</N></H></F>"#;
    assert_eq!(trasforma(foglio, xml).unwrap(), "42|1|h");
}
