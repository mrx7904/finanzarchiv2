//! The code reader of invoices (FA-16): values from their labelled lines, as printed.

use super::parse::{after_label, first_date, has_label, iban, last_amount, percent};
use super::{Input, Line, Normalized, Reader, Value};

/// Labels of each field, the stronger first. Lower case, compared with the lowered line.
const NUMBER: &[&str] = &[
    "rechnungsnummer",
    "rechnungsnr",
    "rechnung nr",
    "rechnungs-nr",
    "invoice number",
    "invoice no",
];
/// "Datum" comes last and only at the start of a word, so the "Geburtsdatum" of a dental invoice
/// is never read as its date (seen in the owner's invoices on 2026-11-25).
const DATE: &[&str] = &[
    "rechnungsdatum",
    "rechnung vom",
    "datum der rechnung",
    "belegdatum",
    "invoice date",
    "date of issue",
    "datum",
];
const DUE: &[&str] = &[
    "zahlbar bis",
    "fällig am",
    "fällig bis",
    "zahlungsziel",
    "due date",
    "zahlbar am",
];
const NET: &[&str] = &["nettobetrag", "summe netto", "gesamt netto", "netto"];
const VAT: &[&str] = &["umsatzsteuer", "mehrwertsteuer", "mwst", "ust"];
const GROSS: &[&str] = &[
    "rechnungsbetrag",
    "zahlbetrag",
    "gesamtbetrag",
    "gesamtsumme",
    "gesamtpreis",
    "bruttobetrag",
    "endbetrag",
    "zu zahlender betrag",
    "offener betrag",
    "zu zahlen",
    "amount due",
    "brutto",
    "gesamt",
    "summe",
    "total",
];
/// Weak labels of the gross that also stand on the line of the net or the tax.
const WEAK_GROSS: &[&str] = &["gesamt", "summe", "total"];

pub struct InvoiceCode;

fn amount_on(lines: &[Line], labels: &[&str], field: &'static str) -> Option<Value> {
    labels.iter().find_map(|label| {
        lines
            .iter()
            .filter(|line| {
                let lower = line.text.to_lowercase();
                has_label(&lower, label)
                    && !(WEAK_GROSS.contains(label)
                        && ["netto", "mwst", "ust", "steuer"]
                            .iter()
                            .any(|other| has_label(&lower, other)))
            })
            .find_map(|line| {
                let (printed, cents) = last_amount(&line.text)?;
                Some(Value::on(field, printed, Normalized::Cents(cents), line))
            })
    })
}

fn date_on(lines: &[Line], labels: &[&str], field: &'static str) -> Option<Value> {
    labels.iter().find_map(|label| {
        lines
            .iter()
            .filter(|line| has_label(&line.text.to_lowercase(), label))
            .find_map(|line| {
                let (printed, date) = first_date(&line.text)?;
                Some(Value::on(field, printed, Normalized::Date(date), line))
            })
    })
}

fn invoice_number(lines: &[Line]) -> Option<Value> {
    NUMBER.iter().find_map(|label| {
        lines.iter().find_map(|line| {
            let rest = after_label(&line.text, label)?;
            let token = rest.split_whitespace().next()?.trim_end_matches([',', ';']);
            token.chars().any(|c| c.is_ascii_digit()).then(|| {
                Value::on(
                    "invoice_number",
                    token,
                    Normalized::Text(token.to_owned()),
                    line,
                )
            })
        })
    })
}

fn vat(lines: &[Line]) -> Vec<Value> {
    let Some(line) = VAT.iter().find_map(|label| {
        lines.iter().find(|line| {
            let lower = line.text.to_lowercase();
            has_label(&lower, label) && last_amount(&line.text).is_some()
        })
    }) else {
        return Vec::new();
    };
    let mut values = Vec::new();
    if let Some((printed, rate)) = percent(&line.text) {
        values.push(Value::on(
            "vat_rate",
            printed,
            Normalized::Percent(rate),
            line,
        ));
    }
    if let Some((printed, cents)) = last_amount(&line.text) {
        values.push(Value::on("vat", printed, Normalized::Cents(cents), line));
    }
    values
}

/// The sender is the first line of the first page that is no label, no date and no amount: on a
/// letter the name of the sender stands at the top. The model reads it where this guess is wrong.
fn sender(lines: &[Line]) -> Option<Value> {
    lines
        .iter()
        .filter(|line| line.page == 1)
        .find(|line| {
            let text = line.text.trim();
            text.chars().filter(|c| c.is_alphabetic()).count() >= 3
                && first_date(text).is_none()
                && last_amount(text).is_none()
                && !text.to_lowercase().starts_with("rechnung")
        })
        .map(|line| {
            Value::on(
                "sender",
                line.text.trim(),
                Normalized::Text(line.text.trim().to_owned()),
                line,
            )
        })
}

fn iban_of(lines: &[Line]) -> Option<Value> {
    lines.iter().find_map(|line| {
        let found = iban(&line.text)?;
        let compact: String = found.chars().filter(|c| !c.is_whitespace()).collect();
        Some(Value::on("iban", found, Normalized::Text(compact), line))
    })
}

impl Reader for InvoiceCode {
    fn name(&self) -> String {
        format!("code:{}", super::READERS_VERSION)
    }

    fn read(&self, input: &Input) -> Vec<Value> {
        let lines = &input.lines;
        let mut values: Vec<Value> = [
            invoice_number(lines),
            date_on(lines, DATE, "invoice_date"),
            date_on(lines, DUE, "due_date"),
            amount_on(lines, NET, "net"),
            amount_on(lines, GROSS, "gross"),
            iban_of(lines),
            sender(lines),
        ]
        .into_iter()
        .flatten()
        .collect();
        values.extend(vat(lines));
        values
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::kinds::Kind;
    use crate::money::Cents;
    use jiff::civil::date;

    fn input(texts: &[&str]) -> Input {
        let lines = texts
            .iter()
            .enumerate()
            .map(|(index, text)| Line {
                page: 1,
                text: (*text).into(),
                left: 60.0,
                bottom: 700.0 - 20.0 * index as f64,
                right: 400.0,
                top: 711.0 - 20.0 * index as f64,
            })
            .collect();
        Input {
            kind: Kind::Invoice,
            lines,
            e_invoice: None,
        }
    }

    fn value<'a>(values: &'a [Value], field: &str) -> Option<&'a Normalized> {
        values
            .iter()
            .find(|value| value.field == field)
            .map(|value| &value.value)
    }

    #[test]
    fn reads_an_invoice_from_its_labelled_lines() {
        let values = InvoiceCode.read(&input(&[
            "Beispiel Stadtwerke GmbH",
            "Beispielweg 1, 12345 Musterstadt",
            "Rechnung",
            "Rechnungsnummer R-2026-0042",
            "Rechnungsdatum 20.11.2026",
            "Nettobetrag 1.234,56",
            "Umsatzsteuer 19 % 234,57",
            "Rechnungsbetrag 1.469,13",
            "Zahlbar bis 04.12.2026",
            "Musterbank, IBAN DE89 3704 0044 0532 0130 00",
        ]));
        assert_eq!(
            value(&values, "sender"),
            Some(&Normalized::Text("Beispiel Stadtwerke GmbH".into()))
        );
        assert_eq!(
            value(&values, "invoice_number"),
            Some(&Normalized::Text("R-2026-0042".into()))
        );
        assert_eq!(
            value(&values, "invoice_date"),
            Some(&Normalized::Date(date(2026, 11, 20)))
        );
        assert_eq!(
            value(&values, "due_date"),
            Some(&Normalized::Date(date(2026, 12, 4)))
        );
        assert_eq!(
            value(&values, "net"),
            Some(&Normalized::Cents(Cents::new(123_456)))
        );
        assert_eq!(value(&values, "vat_rate"), Some(&Normalized::Percent(19)));
        assert_eq!(
            value(&values, "vat"),
            Some(&Normalized::Cents(Cents::new(23_457)))
        );
        assert_eq!(
            value(&values, "gross"),
            Some(&Normalized::Cents(Cents::new(146_913)))
        );
        assert_eq!(
            value(&values, "iban"),
            Some(&Normalized::Text("DE89370400440532013000".into()))
        );
        let gross = values.iter().find(|value| value.field == "gross").unwrap();
        assert_eq!(gross.printed, "1.469,13");
        assert_eq!(
            gross.line.as_ref().map(|line| line.text.as_str()),
            Some("Rechnungsbetrag 1.469,13")
        );
    }

    #[test]
    fn the_labels_seen_on_real_invoices_are_read() {
        let values = InvoiceCode.read(&input(&[
            "Zahnarztpraxis Beispiel",
            "Geburtsdatum 01.01.1990",
            "Rechnung vom 03.03.2025",
            "Gesamt netto 84,03",
            "Zahlbetrag 100,00",
        ]));
        assert_eq!(
            value(&values, "invoice_date"),
            Some(&Normalized::Date(date(2025, 3, 3)))
        );
        assert_eq!(
            value(&values, "gross"),
            Some(&Normalized::Cents(Cents::new(10_000)))
        );
        let weak = InvoiceCode.read(&input(&["Shop", "Gesamt netto 84,03", "Gesamt 100,00"]));
        assert_eq!(
            value(&weak, "gross"),
            Some(&Normalized::Cents(Cents::new(10_000))),
            "a weak label skips the line of the net"
        );
    }

    #[test]
    fn the_stronger_label_wins_and_missing_fields_stay_empty() {
        let values = InvoiceCode.read(&input(&[
            "Muster Shop",
            "Gesamt 10,00",
            "Rechnungsbetrag 11,90",
        ]));
        assert_eq!(
            value(&values, "gross"),
            Some(&Normalized::Cents(Cents::new(1_190)))
        );
        assert_eq!(value(&values, "invoice_number"), None);
        assert_eq!(value(&values, "vat"), None);
    }
}
