// Copyright (c) 2026 Andrea Re - info@digitare.net
// Autore: Andrea Re @ 2026 - info@digitare.net
//
// Tabelle dei codici previsti dalle specifiche tecniche FatturaPA
// (versione 1.9.1, in vigore dal 15 maggio 2026), con la descrizione da
// mostrare all'utente. Un codice assente dalle tabelle restituisce `None`:
// l'interfaccia lo mostrerà come "codice non previsto", senza bloccare la
// visualizzazione.
//
// Le tabelle sono mantenute a mano: a ogni nuova versione delle specifiche
// vanno confrontate con l'allegato ufficiale.

/// Cerca un codice in una tabella `(codice, descrizione)`.
fn cerca(tabella: &[(&'static str, &'static str)], codice: &str) -> Option<&'static str> {
    let codice = codice.trim();
    tabella
        .iter()
        .find(|(c, _)| c.eq_ignore_ascii_case(codice))
        .map(|(_, d)| *d)
}

/// Tipi documento (`TipoDocumento`).
pub const TIPO_DOCUMENTO: &[(&str, &str)] = &[
    ("TD01", "Fattura"),
    ("TD02", "Acconto/anticipo su fattura"),
    ("TD03", "Acconto/anticipo su parcella"),
    ("TD04", "Nota di credito"),
    ("TD05", "Nota di debito"),
    ("TD06", "Parcella"),
    ("TD07", "Fattura semplificata"),
    ("TD08", "Nota di credito semplificata"),
    ("TD09", "Nota di debito semplificata"),
    ("TD16", "Integrazione fattura reverse charge interno"),
    (
        "TD17",
        "Integrazione/autofattura per acquisto servizi dall'estero",
    ),
    ("TD18", "Integrazione per acquisto di beni intracomunitari"),
    (
        "TD19",
        "Integrazione/autofattura per acquisto di beni ex art. 17 c. 2 DPR 633/72",
    ),
    (
        "TD20",
        "Autofattura per regolarizzazione e integrazione delle fatture",
    ),
    ("TD21", "Autofattura per splafonamento"),
    ("TD22", "Estrazione beni da Deposito IVA"),
    (
        "TD23",
        "Estrazione beni da Deposito IVA con versamento dell'IVA",
    ),
    ("TD24", "Fattura differita (art. 21 c. 4 lett. a)"),
    (
        "TD25",
        "Fattura differita (art. 21 c. 4 terzo periodo lett. b)",
    ),
    (
        "TD26",
        "Cessione di beni ammortizzabili e per passaggi interni",
    ),
    (
        "TD27",
        "Fattura per autoconsumo o per cessioni gratuite senza rivalsa",
    ),
    ("TD28", "Acquisti da San Marino con IVA (fattura cartacea)"),
    ("TD29", "Comunicazione per omessa o irregolare fatturazione"),
];

/// Natura dell'operazione (`Natura`), inclusi i codici generici storici
/// N2, N3, N6 non più ammessi dal 2021 ma presenti negli archivi.
pub const NATURA: &[(&str, &str)] = &[
    ("N1", "Escluse ex art. 15 DPR 633/72"),
    ("N2", "Non soggette (codice non più valido dal 2021)"),
    (
        "N2.1",
        "Non soggette ad IVA ai sensi degli artt. da 7 a 7-septies DPR 633/72",
    ),
    ("N2.2", "Non soggette - altri casi"),
    ("N3", "Non imponibili (codice non più valido dal 2021)"),
    ("N3.1", "Non imponibili - esportazioni"),
    ("N3.2", "Non imponibili - cessioni intracomunitarie"),
    ("N3.3", "Non imponibili - cessioni verso San Marino"),
    (
        "N3.4",
        "Non imponibili - operazioni assimilate alle cessioni all'esportazione",
    ),
    (
        "N3.5",
        "Non imponibili - a seguito di dichiarazioni d'intento",
    ),
    (
        "N3.6",
        "Non imponibili - altre operazioni che non concorrono alla formazione del plafond",
    ),
    ("N4", "Esenti"),
    ("N5", "Regime del margine / IVA non esposta in fattura"),
    (
        "N6",
        "Inversione contabile (codice non più valido dal 2021)",
    ),
    (
        "N6.1",
        "Inversione contabile - cessione di rottami e altri materiali di recupero",
    ),
    ("N6.2", "Inversione contabile - cessione di oro e argento"),
    (
        "N6.3",
        "Inversione contabile - subappalto nel settore edile",
    ),
    ("N6.4", "Inversione contabile - cessione di fabbricati"),
    (
        "N6.5",
        "Inversione contabile - cessione di telefoni cellulari",
    ),
    (
        "N6.6",
        "Inversione contabile - cessione di prodotti elettronici",
    ),
    (
        "N6.7",
        "Inversione contabile - prestazioni comparto edile e settori connessi",
    ),
    (
        "N6.8",
        "Inversione contabile - operazioni settore energetico",
    ),
    ("N6.9", "Inversione contabile - altri casi"),
    ("N7", "IVA assolta in altro stato UE"),
];

/// Modalità di pagamento (`ModalitaPagamento`).
pub const MODALITA_PAGAMENTO: &[(&str, &str)] = &[
    ("MP01", "Contanti"),
    ("MP02", "Assegno"),
    ("MP03", "Assegno circolare"),
    ("MP04", "Contanti presso Tesoreria"),
    ("MP05", "Bonifico"),
    ("MP06", "Vaglia cambiario"),
    ("MP07", "Bollettino bancario"),
    ("MP08", "Carta di pagamento"),
    ("MP09", "RID"),
    ("MP10", "RID utenze"),
    ("MP11", "RID veloce"),
    ("MP12", "RIBA"),
    ("MP13", "MAV"),
    ("MP14", "Quietanza erario"),
    ("MP15", "Giroconto su conti di contabilità speciale"),
    ("MP16", "Domiciliazione bancaria"),
    ("MP17", "Domiciliazione postale"),
    ("MP18", "Bollettino di c/c postale"),
    ("MP19", "SEPA Direct Debit"),
    ("MP20", "SEPA Direct Debit CORE"),
    ("MP21", "SEPA Direct Debit B2B"),
    ("MP22", "Trattenuta su somme già riscosse"),
    ("MP23", "PagoPA"),
];

/// Regime fiscale del cedente/prestatore (`RegimeFiscale`).
pub const REGIME_FISCALE: &[(&str, &str)] = &[
    ("RF01", "Ordinario"),
    ("RF02", "Contribuenti minimi"),
    ("RF04", "Agricoltura e attività connesse e pesca"),
    ("RF05", "Vendita sali e tabacchi"),
    ("RF06", "Commercio fiammiferi"),
    ("RF07", "Editoria"),
    ("RF08", "Gestione servizi telefonia pubblica"),
    (
        "RF09",
        "Rivendita documenti di trasporto pubblico e di sosta",
    ),
    ("RF10", "Intrattenimenti, giochi e altre attività"),
    ("RF11", "Agenzie viaggi e turismo"),
    ("RF12", "Agriturismo"),
    ("RF13", "Vendite a domicilio"),
    (
        "RF14",
        "Rivendita beni usati, oggetti d'arte, d'antiquariato o da collezione",
    ),
    (
        "RF15",
        "Agenzie di vendite all'asta di oggetti d'arte, antiquariato o da collezione",
    ),
    ("RF16", "IVA per cassa P.A."),
    ("RF17", "IVA per cassa"),
    ("RF18", "Altro"),
    ("RF19", "Regime forfettario"),
    ("RF20", "Regime transfrontaliero di franchigia IVA"),
];

/// Tipo di cassa previdenziale (`TipoCassa`).
pub const TIPO_CASSA: &[(&str, &str)] = &[
    (
        "TC01",
        "Cassa nazionale previdenza e assistenza avvocati e procuratori legali",
    ),
    ("TC02", "Cassa previdenza dottori commercialisti"),
    ("TC03", "Cassa previdenza e assistenza geometri"),
    (
        "TC04",
        "Cassa nazionale previdenza e assistenza ingegneri e architetti liberi professionisti",
    ),
    ("TC05", "Cassa nazionale del notariato"),
    (
        "TC06",
        "Cassa nazionale previdenza e assistenza ragionieri e periti commerciali",
    ),
    ("TC07", "ENASARCO"),
    ("TC08", "ENPACL"),
    ("TC09", "ENPAM"),
    ("TC10", "ENPAF"),
    ("TC11", "ENPAV"),
    ("TC12", "ENPAIA"),
    (
        "TC13",
        "Fondo previdenza impiegati imprese di spedizione e agenzie marittime",
    ),
    ("TC14", "INPGI"),
    ("TC15", "ONAOSI"),
    ("TC16", "CASAGIT"),
    ("TC17", "EPPI"),
    ("TC18", "EPAP"),
    ("TC19", "ENPAB"),
    ("TC20", "ENPAPI"),
    ("TC21", "ENPAP"),
    ("TC22", "INPS"),
];

/// Tipo di ritenuta (`TipoRitenuta`).
pub const TIPO_RITENUTA: &[(&str, &str)] = &[
    ("RT01", "Ritenuta persone fisiche"),
    ("RT02", "Ritenuta persone giuridiche"),
    ("RT03", "Contributo INPS"),
    ("RT04", "Contributo ENASARCO"),
    ("RT05", "Contributo ENPAM"),
    ("RT06", "Altro contributo previdenziale"),
];

/// Esigibilità IVA (`EsigibilitaIVA`).
pub const ESIGIBILITA_IVA: &[(&str, &str)] = &[
    ("I", "Immediata"),
    ("D", "Differita"),
    ("S", "Scissione dei pagamenti"),
];

/// Condizioni di pagamento (`CondizioniPagamento`).
pub const CONDIZIONI_PAGAMENTO: &[(&str, &str)] = &[
    ("TP01", "Pagamento a rate"),
    ("TP02", "Pagamento completo"),
    ("TP03", "Anticipo"),
];

pub fn tipo_documento(codice: &str) -> Option<&'static str> {
    cerca(TIPO_DOCUMENTO, codice)
}
pub fn natura(codice: &str) -> Option<&'static str> {
    cerca(NATURA, codice)
}
pub fn modalita_pagamento(codice: &str) -> Option<&'static str> {
    cerca(MODALITA_PAGAMENTO, codice)
}
pub fn regime_fiscale(codice: &str) -> Option<&'static str> {
    cerca(REGIME_FISCALE, codice)
}
pub fn tipo_cassa(codice: &str) -> Option<&'static str> {
    cerca(TIPO_CASSA, codice)
}
pub fn tipo_ritenuta(codice: &str) -> Option<&'static str> {
    cerca(TIPO_RITENUTA, codice)
}
pub fn esigibilita_iva(codice: &str) -> Option<&'static str> {
    cerca(ESIGIBILITA_IVA, codice)
}
pub fn condizioni_pagamento(codice: &str) -> Option<&'static str> {
    cerca(CONDIZIONI_PAGAMENTO, codice)
}

/// Valori di `AltriDatiGestionali/TipoDato` con un significato normativo.
pub const TIPO_DATO_NORMATIVO: &[(&str, &str)] = &[(
    "ESENZSPORT",
    "Compenso esente per lavoro sportivo dilettantistico",
)];

pub fn tipo_dato_normativo(codice: &str) -> Option<&'static str> {
    cerca(TIPO_DATO_NORMATIVO, codice)
}

/// Indica se il tipo documento rappresenta una nota di credito, cioè un
/// documento il cui importo va letto con segno negativo nei totali.
pub fn e_nota_di_credito(codice: &str) -> bool {
    matches!(codice.trim().to_uppercase().as_str(), "TD04" | "TD08")
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn codici_noti_e_ignoti() {
        assert_eq!(tipo_documento("td01"), Some("Fattura"));
        assert_eq!(natura("N2.2"), Some("Non soggette - altri casi"));
        assert!(tipo_documento("TD99").is_none());
        assert!(e_nota_di_credito("TD04"));
    }

    #[test]
    fn tabelle_senza_duplicati() {
        for tabella in [
            TIPO_DOCUMENTO,
            NATURA,
            MODALITA_PAGAMENTO,
            REGIME_FISCALE,
            TIPO_CASSA,
            TIPO_RITENUTA,
        ] {
            let mut codici: Vec<_> = tabella.iter().map(|(c, _)| *c).collect();
            let prima = codici.len();
            codici.sort();
            codici.dedup();
            assert_eq!(prima, codici.len());
        }
    }
}
