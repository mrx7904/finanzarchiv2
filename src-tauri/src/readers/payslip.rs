//! The code reader of payslips (FA-18, #13): employer, month, Brutto, each deduction and Netto from
//! their labelled lines. A deduction is the amount deducted, never its rate (lesson of 1.0).

use super::parse::{has_label, last_amount, month_of};
use super::{Input, Line, Normalized, Reader, Value};

/// Labels of each amount, the stronger first. Lower case, compared at the start of a word.
const AMOUNTS: &[(&str, &[&str])] = &[
    (
        "brutto",
        &[
            "gesamtbrutto",
            "gesamt-brutto",
            "gesamt brutto",
            "bruttoverdienst",
            "bruttobezüge",
            "bruttolohn",
            "bruttogehalt",
            "summe brutto",
            "brutto",
        ],
    ),
    ("lohnsteuer", &["lohnsteuer"]),
    (
        "solidaritaetszuschlag",
        &["solidaritätszuschlag", "solidaritaetszuschlag", "solz"],
    ),
    ("kirchensteuer", &["kirchensteuer"]),
    (
        "krankenversicherung",
        &["krankenversicherung", "kv-beitrag", "kv beitrag"],
    ),
    (
        "rentenversicherung",
        &["rentenversicherung", "rv-beitrag", "rv beitrag"],
    ),
    (
        "arbeitslosenversicherung",
        &["arbeitslosenversicherung", "av-beitrag", "av beitrag"],
    ),
    (
        "pflegeversicherung",
        &["pflegeversicherung", "pv-beitrag", "pv beitrag"],
    ),
    (
        "netto",
        &[
            "auszahlungsbetrag",
            "überweisungsbetrag",
            "auszahlung",
            "nettoverdienst",
            "nettobezüge",
            "nettolohn",
            "nettogehalt",
            "netto",
        ],
    ),
];

/// Lines of the employer's share, the base of a contribution or a total of the year: their amount is
/// no deduction of this month.
const NOT_THIS_DEDUCTION: &[&str] = &[
    "arbeitgeber",
    "ag-anteil",
    "ag anteil",
    "brutto",
    "jahres",
    "kumuliert",
    "steuerklasse",
];

/// Lines that name the month of the payslip, the stronger first.
const PERIOD: &[&str] = &[
    "verdienstabrechnung",
    "entgeltabrechnung",
    "gehaltsabrechnung",
    "lohnabrechnung",
    "bezügemitteilung",
    "abrechnungsmonat",
    "abrechnungszeitraum",
    "abrechnung",
    "zeitraum",
    "monat",
];

/// The words of the title, never the employer.
const TITLE: &[&str] = &[
    "verdienstabrechnung",
    "entgeltabrechnung",
    "gehaltsabrechnung",
    "lohnabrechnung",
    "bezügemitteilung",
    "abrechnung",
];

pub struct PayslipCode;

fn amount(lines: &[Line], field: &'static str, labels: &[&str]) -> Option<Value> {
    let is_deduction = !matches!(field, "brutto" | "netto");
    labels.iter().find_map(|label| {
        lines
            .iter()
            .filter(|line| {
                let lower = line.text.to_lowercase();
                has_label(&lower, label)
                    && !(is_deduction
                        && NOT_THIS_DEDUCTION
                            .iter()
                            .any(|other| has_label(&lower, other)))
            })
            .find_map(|line| {
                let (printed, cents) = last_amount(&line.text)?;
                Some(Value::on(field, printed, Normalized::Cents(cents), line))
            })
    })
}

/// The month, stored as `YYYY-MM`, from the first line that names the period and a month.
fn period(lines: &[Line]) -> Option<Value> {
    PERIOD.iter().find_map(|label| {
        lines
            .iter()
            .filter(|line| has_label(&line.text.to_lowercase(), label))
            .find_map(|line| {
                let (printed, year, month) = month_of(&line.text)?;
                Some(Value::on(
                    "period",
                    printed,
                    Normalized::Text(format!("{year:04}-{month:02}")),
                    line,
                ))
            })
    })
}

fn employer(lines: &[Line]) -> Option<Value> {
    super::head_line(lines, TITLE).map(|line| {
        Value::on(
            "employer",
            line.text.trim(),
            Normalized::Text(line.text.trim().to_owned()),
            line,
        )
    })
}

impl Reader for PayslipCode {
    fn name(&self) -> String {
        format!("code:{}", super::READERS_VERSION)
    }

    fn read(&self, input: &Input) -> Vec<Value> {
        let lines = &input.lines;
        let mut values: Vec<Value> = [employer(lines), period(lines)]
            .into_iter()
            .flatten()
            .collect();
        values.extend(
            AMOUNTS
                .iter()
                .filter_map(|(field, labels)| amount(lines, field, labels)),
        );
        values
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::kinds::Kind;

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
            kind: Kind::Payslip,
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

    fn cents(values: &[Value], field: &str) -> Option<i64> {
        match value(values, field) {
            Some(Normalized::Cents(cents)) => Some(cents.value()),
            _ => None,
        }
    }

    #[test]
    fn reads_a_payslip_from_its_labelled_lines() {
        let values = PayslipCode.read(&input(&[
            "Beispiel Maschinenbau GmbH",
            "Verdienstabrechnung März 2026",
            "Personalnummer 00042, Steuerklasse 1",
            "Gesamtbrutto 4.000,00",
            "Lohnsteuer 520,00",
            "Solidaritätszuschlag 0,00",
            "Kirchensteuer 41,60",
            "Krankenversicherung 7,30 % 348,00",
            "Rentenversicherung 372,00 9,30 %",
            "Arbeitslosenversicherung 52,00",
            "Pflegeversicherung 72,00",
            "Auszahlungsbetrag 2.594,40",
        ]));
        assert_eq!(
            value(&values, "employer"),
            Some(&Normalized::Text("Beispiel Maschinenbau GmbH".into()))
        );
        assert_eq!(
            value(&values, "period"),
            Some(&Normalized::Text("2026-03".into()))
        );
        let found: Vec<(&str, i64)> = [
            "brutto",
            "lohnsteuer",
            "solidaritaetszuschlag",
            "kirchensteuer",
            "krankenversicherung",
            "rentenversicherung",
            "arbeitslosenversicherung",
            "pflegeversicherung",
            "netto",
        ]
        .into_iter()
        .filter_map(|field| cents(&values, field).map(|cents| (field, cents)))
        .collect();
        assert_eq!(
            found,
            [
                ("brutto", 400_000),
                ("lohnsteuer", 52_000),
                ("solidaritaetszuschlag", 0),
                ("kirchensteuer", 4_160),
                ("krankenversicherung", 34_800),
                ("rentenversicherung", 37_200),
                ("arbeitslosenversicherung", 5_200),
                ("pflegeversicherung", 7_200),
                ("netto", 259_440),
            ]
        );
        let period = values.iter().find(|value| value.field == "period").unwrap();
        assert_eq!(period.printed, "März 2026");
    }

    #[test]
    fn the_employers_share_and_the_base_are_no_deduction() {
        let values = PayslipCode.read(&input(&[
            "Muster AG",
            "Entgeltabrechnung 11/2026",
            "KV-Brutto 4.000,00",
            "Krankenversicherung Arbeitgeberanteil 292,00",
            "Krankenversicherung 348,00",
            "Lohnsteuer Jahressumme 6.240,00",
            "Lohnsteuer 520,00",
            "Netto 2.600,00",
        ]));
        assert_eq!(cents(&values, "krankenversicherung"), Some(34_800));
        assert_eq!(cents(&values, "lohnsteuer"), Some(52_000));
        assert_eq!(cents(&values, "netto"), Some(260_000));
        assert_eq!(
            value(&values, "period"),
            Some(&Normalized::Text("2026-11".into()))
        );
        assert_eq!(cents(&values, "kirchensteuer"), None, "not on the slip");
    }

    #[test]
    fn the_title_is_never_the_employer() {
        let values = PayslipCode.read(&input(&[
            "Verdienstabrechnung Mai 2026",
            "Beispiel Pflegedienst e.V.",
        ]));
        assert_eq!(
            value(&values, "employer"),
            Some(&Normalized::Text("Beispiel Pflegedienst e.V.".into()))
        );
    }
}
