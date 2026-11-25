//! The kind of a document from the signals printed in its text (FA-14, #10). No model: the
//! kind decides which readers run, so it must be the same every time and say why.

/// The kinds of documents. Stored as their code.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Kind {
    Invoice,
    Payslip,
    Receipt,
    Statement,
    Other,
    /// No text to read (a scan or an image), or the signals of two kinds are equally strong.
    Unknown,
}

impl Kind {
    pub fn code(self) -> &'static str {
        match self {
            Self::Invoice => "invoice",
            Self::Payslip => "payslip",
            Self::Receipt => "receipt",
            Self::Statement => "statement",
            Self::Other => "other",
            Self::Unknown => "unknown",
        }
    }
}

/// What decided the kind: the kind and the signals found, as written in the signal lists.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Classified {
    pub kind: Kind,
    pub signals: Vec<&'static str>,
}

/// A payment that was made. One of these makes a receipt, before any count, because a receipt
/// carries the signals of its invoice too (lesson of 1.0, the Stripe receipts beside invoices).
const RECEIPT: &[&str] = &[
    "zahlungsbestätigung",
    "zahlungsbeleg",
    "quittung",
    "kassenbon",
    "kassenbeleg",
    "date paid",
    "amount paid",
    "payment receipt",
    "auftragsbestätigung überweisung",
    "ausführungsbestätigung",
];

const INVOICE: &[&str] = &[
    "rechnung nr",
    "rechnungs-nr",
    "rechnungsnummer",
    "rechnungsdatum",
    "rechnungsbetrag",
    "umsatzsteuer",
    "mehrwertsteuer",
    "mwst",
    "ust-idnr",
    "nettobetrag",
    "zahlbar bis",
    "zahlungsziel",
    "invoice",
];

const PAYSLIP: &[&str] = &[
    "verdienstabrechnung",
    "entgeltabrechnung",
    "gehaltsabrechnung",
    "lohnabrechnung",
    "gesamtbrutto",
    "auszahlungsbetrag",
    "lohnsteuer",
    "steuerklasse",
    "rentenversicherung",
    "arbeitslosenversicherung",
    "personalnummer",
];

const STATEMENT: &[&str] = &[
    "kontoauszug",
    "umsatzübersicht",
    "umsatzanzeige",
    "alter kontostand",
    "neuer kontostand",
    "saldo alt",
    "saldo neu",
    "buchungstag",
    "valuta",
];

/// Two signals of one kind are needed, one alone is too easy to meet by chance.
const NEEDED: usize = 2;

/// The signals that stand at the start of a word, so "abrechnung" does not count as "rechnung".
fn found(text: &str, signals: &[&'static str]) -> Vec<&'static str> {
    signals
        .iter()
        .copied()
        .filter(|signal| {
            text.match_indices(signal).any(|(at, _)| {
                text[..at]
                    .chars()
                    .next_back()
                    .is_none_or(|before| !before.is_alphanumeric())
            })
        })
        .collect()
}

/// The kind of a document from the lines of its pages.
pub fn classify<'a>(lines: impl IntoIterator<Item = &'a str>) -> Classified {
    let text = lines
        .into_iter()
        .map(str::to_lowercase)
        .collect::<Vec<_>>()
        .join("\n");
    if text.trim().is_empty() {
        return Classified {
            kind: Kind::Unknown,
            signals: Vec::new(),
        };
    }
    let receipt = found(&text, RECEIPT);
    if !receipt.is_empty() {
        return Classified {
            kind: Kind::Receipt,
            signals: receipt,
        };
    }
    let mut scored = [
        (Kind::Invoice, found(&text, INVOICE)),
        (Kind::Payslip, found(&text, PAYSLIP)),
        (Kind::Statement, found(&text, STATEMENT)),
    ];
    scored.sort_by_key(|(_, signals)| std::cmp::Reverse(signals.len()));
    let [(first, first_signals), (_, second_signals), _] = scored;
    if first_signals.len() < NEEDED {
        let mut all = first_signals;
        all.extend(second_signals);
        return Classified {
            kind: Kind::Other,
            signals: all,
        };
    }
    if second_signals.len() == first_signals.len() {
        let mut all = first_signals;
        all.extend(second_signals);
        return Classified {
            kind: Kind::Unknown,
            signals: all,
        };
    }
    Classified {
        kind: first,
        signals: first_signals,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn kind(lines: &[&str]) -> Kind {
        classify(lines.iter().copied()).kind
    }

    #[test]
    fn names_the_kinds_by_their_signals() {
        assert_eq!(
            kind(&["Rechnung", "Rechnungsnummer R-1", "Umsatzsteuer 19 % 1,90"]),
            Kind::Invoice
        );
        assert_eq!(
            kind(&[
                "Verdienstabrechnung März 2026",
                "Gesamtbrutto 3.800,00",
                "Lohnsteuer 512,33"
            ]),
            Kind::Payslip
        );
        assert_eq!(
            kind(&[
                "Kontoauszug Nr. 3",
                "Alter Kontostand 1,00",
                "Neuer Kontostand 2,00"
            ]),
            Kind::Statement
        );
        assert_eq!(
            kind(&["Liebe Kundin, lieber Kunde", "wir ändern unsere AGB"]),
            Kind::Other
        );
    }

    #[test]
    fn a_payment_confirmation_is_a_receipt_even_with_the_signals_of_its_invoice() {
        let found = classify([
            "Receipt from Beispiel GmbH",
            "Invoice number R-7",
            "Amount paid €21,42",
            "Date paid 28.05.2026",
        ]);
        assert_eq!(found.kind, Kind::Receipt);
        assert_eq!(found.signals, ["date paid", "amount paid"]);
    }

    #[test]
    fn one_signal_is_not_enough_and_a_tie_stays_open() {
        assert_eq!(
            kind(&["Ihre Rechnung finden Sie im Kundenportal"]),
            Kind::Other
        );
        assert_eq!(
            kind(&[
                "Rechnungsnummer 1",
                "Umsatzsteuer",
                "Lohnsteuer",
                "Steuerklasse 1"
            ]),
            Kind::Unknown
        );
    }

    #[test]
    fn a_signal_counts_only_at_the_start_of_a_word() {
        assert_eq!(
            found(
                "verdienstabrechnung",
                &["verdienstabrechnung", "abrechnung"]
            ),
            ["verdienstabrechnung"]
        );
        assert!(found("die verdienstabrechnung", &["rechnungsdatum"]).is_empty());
        assert_eq!(
            found("x rechnungsdatum", &["rechnungsdatum"]),
            ["rechnungsdatum"]
        );
    }

    #[test]
    fn no_text_is_unknown() {
        assert_eq!(classify([]).kind, Kind::Unknown);
        assert_eq!(kind(&["   "]), Kind::Unknown);
    }

    #[test]
    fn the_signals_are_lower_case_so_the_text_can_be_lowered_once() {
        for list in [RECEIPT, INVOICE, PAYSLIP, STATEMENT] {
            assert!(
                list.iter().all(|signal| signal.to_lowercase() == *signal),
                "{list:?}"
            );
        }
    }
}
