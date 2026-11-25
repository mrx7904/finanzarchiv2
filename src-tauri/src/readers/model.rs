//! The model reader (FA-19, #14): the local model reads the fields code left empty. It copies each
//! value as printed and names its line, code checks that the value stands there and parses it. The
//! model is never asked how sure it is (lesson of 1.0, knowledge/decision-reading.md).

use serde_json::{Value as Json, json};

use super::parse::{first_date, iban_is_valid, last_amount, month_of};
use super::{Line, Normalized, Value};
use crate::kinds::Kind;
use crate::ollama::{LocalModel, ModelError, Question};

/// The form of a field, which decides how code parses what the model copied.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Form {
    Text,
    Date,
    Amount,
    Percent,
    Iban,
    Month,
}

/// A field the model may read: its name, what it means in German for the model, and its form.
pub type Field = (&'static str, &'static str, Form);

const INVOICE: &[Field] = &[
    (
        "sender",
        "Name des Absenders, der die Rechnung stellt",
        Form::Text,
    ),
    ("invoice_number", "Rechnungsnummer", Form::Text),
    ("invoice_date", "Rechnungsdatum", Form::Date),
    ("due_date", "Fälligkeitsdatum, zahlbar bis", Form::Date),
    ("net", "Nettobetrag, Summe ohne Umsatzsteuer", Form::Amount),
    ("vat_rate", "Umsatzsteuersatz in Prozent", Form::Percent),
    ("vat", "Betrag der Umsatzsteuer", Form::Amount),
    (
        "gross",
        "Bruttobetrag, der zu zahlende Gesamtbetrag",
        Form::Amount,
    ),
    ("iban", "IBAN des Empfängers der Zahlung", Form::Iban),
];

const PAYSLIP: &[Field] = &[
    ("employer", "Name des Arbeitgebers", Form::Text),
    ("period", "Abrechnungsmonat", Form::Month),
    ("brutto", "Gesamtbrutto", Form::Amount),
    ("lohnsteuer", "abgezogene Lohnsteuer", Form::Amount),
    (
        "solidaritaetszuschlag",
        "abgezogener Solidaritätszuschlag",
        Form::Amount,
    ),
    ("kirchensteuer", "abgezogene Kirchensteuer", Form::Amount),
    (
        "krankenversicherung",
        "Arbeitnehmeranteil Krankenversicherung",
        Form::Amount,
    ),
    (
        "rentenversicherung",
        "Arbeitnehmeranteil Rentenversicherung",
        Form::Amount,
    ),
    (
        "arbeitslosenversicherung",
        "Arbeitnehmeranteil Arbeitslosenversicherung",
        Form::Amount,
    ),
    (
        "pflegeversicherung",
        "Arbeitnehmeranteil Pflegeversicherung",
        Form::Amount,
    ),
    ("netto", "Auszahlungsbetrag, Netto", Form::Amount),
];

/// The fields of a kind. A kind without fields is not given to the model.
pub fn fields_of(kind: Kind) -> &'static [Field] {
    match kind {
        Kind::Invoice | Kind::Receipt => INVOICE,
        Kind::Payslip => PAYSLIP,
        _ => &[],
    }
}

pub const MODEL_SETTING: &str = "model.name";
/// The model that reads when the owner chose none, picked by measurement (knowledge/decision-model.md).
pub const DEFAULT_MODEL: &str = "gemma4:12b-it-qat";

/// The model the owner set, else the default.
pub fn configured(conn: &rusqlite::Connection) -> String {
    conn.query_row(
        "SELECT value FROM settings WHERE key = ?1",
        [MODEL_SETTING],
        |row| row.get::<_, String>(0),
    )
    .ok()
    .filter(|name| !name.trim().is_empty())
    .unwrap_or_else(|| DEFAULT_MODEL.to_owned())
}

/// At most this many lines go to the model, the head of a long document.
pub const MAX_LINES: usize = 400;

pub(crate) const SYSTEM_TEXT: &str = "Du liest Werte aus dem Text eines Dokuments. Kopiere jeden Wert genau so, \
wie er im Text steht, Zeichen für Zeichen, Beträge mit Punkt und Komma wie gedruckt, Daten wie gedruckt. \
Gib zu jedem Wert die Nummer der Zeile an, in der er steht. Steht ein Wert nicht im Text, gib einen leeren \
Text und die Zeile 0 an. Rechne nichts aus, ergänze nichts und erfinde nichts.";

/// A value the model gave that code did not take, with the German reason.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Finding {
    pub field: &'static str,
    pub detail: String,
}

/// What the model read: the values code took and the findings of those it dropped.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Read {
    pub values: Vec<Value>,
    pub findings: Vec<Finding>,
}

fn kind_label(kind: Kind) -> &'static str {
    match kind {
        Kind::Invoice => "Rechnung",
        Kind::Receipt => "Zahlungsbeleg",
        Kind::Payslip => "Gehaltsabrechnung",
        Kind::Statement => "Kontoauszug",
        Kind::Other | Kind::Unknown => "Dokument",
    }
}

/// The JSON schema of the answer: per field the value as printed and its line.
pub fn schema(fields: &[Field]) -> Json {
    let properties: serde_json::Map<String, Json> = fields
        .iter()
        .map(|(name, _, _)| {
            (
                (*name).to_owned(),
                json!({
                    "type": "object",
                    "properties": { "printed": { "type": "string" }, "line": { "type": "integer" } },
                    "required": ["printed", "line"],
                }),
            )
        })
        .collect();
    let required: Vec<&str> = fields.iter().map(|(name, _, _)| *name).collect();
    json!({ "type": "object", "properties": properties, "required": required })
}

/// The question: the kind, the fields with their meaning and the numbered lines.
pub fn prompt(kind: Kind, fields: &[Field], lines: &[Line]) -> String {
    let mut text = format!("Art des Dokuments: {}\n\nFelder:\n", kind_label(kind));
    for (name, meaning, _) in fields {
        text.push_str(&format!("- {name}: {meaning}\n"));
    }
    text.push_str("\nText, eine Zeile je Nummer:\n");
    for (index, line) in lines.iter().take(MAX_LINES).enumerate() {
        text.push_str(&format!("{}: {}\n", index + 1, line.text));
    }
    text
}

fn fold(text: &str) -> String {
    text.split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .to_lowercase()
}

/// The value code reads from what the model copied, in the form of the field.
pub(crate) fn normalize(form: Form, printed: &str) -> Option<(String, Normalized)> {
    match form {
        Form::Text => Some((printed.to_owned(), Normalized::Text(printed.to_owned()))),
        Form::Date => first_date(printed).map(|(printed, date)| (printed, Normalized::Date(date))),
        Form::Amount => {
            last_amount(printed).map(|(printed, cents)| (printed, Normalized::Cents(cents)))
        }
        Form::Percent => {
            let number = printed.trim().trim_end_matches('%').trim();
            let (whole, fraction) = number.split_once(',').unwrap_or((number, ""));
            let rate: u32 = whole.parse().ok()?;
            fraction
                .bytes()
                .all(|b| b == b'0')
                .then(|| (printed.to_owned(), Normalized::Percent(rate)))
        }
        Form::Iban => iban_is_valid(printed).then(|| {
            (
                printed.to_owned(),
                Normalized::Text(printed.chars().filter(|c| !c.is_whitespace()).collect()),
            )
        }),
        Form::Month => month_of(printed).map(|(printed, year, month)| {
            (printed, Normalized::Text(format!("{year:04}-{month:02}")))
        }),
    }
}

/// Takes what the model answered: a value only when it stands in the line the model named and
/// code can parse it. Everything else becomes a finding.
pub fn take(answer: &str, fields: &[Field], lines: &[Line]) -> Result<Read, ModelError> {
    let answer: Json =
        serde_json::from_str(answer).map_err(|error| ModelError::BadAnswer(error.to_string()))?;
    let mut read = Read::default();
    let shown = lines.len().min(MAX_LINES);
    for &(field, _, form) in fields {
        let entry = &answer[field];
        let printed = entry["printed"].as_str().unwrap_or("").trim();
        let number = entry["line"].as_u64().unwrap_or(0);
        if printed.is_empty() || number == 0 {
            continue;
        }
        let finding = |detail: String| Finding { field, detail };
        let Some(line) = usize::try_from(number)
            .ok()
            .filter(|number| *number <= shown)
            .and_then(|number| lines.get(number - 1))
        else {
            read.findings.push(finding(format!(
                "Das Modell nannte Zeile {number}, die es nicht gibt."
            )));
            continue;
        };
        if !fold(&line.text).contains(&fold(printed)) {
            read.findings.push(finding(format!(
                "„{printed}“ steht nicht in Zeile {number}, das Modell hat es nicht abgeschrieben."
            )));
            continue;
        }
        match normalize(form, printed) {
            Some((printed, value)) => read.values.push(Value::on(field, printed, value, line)),
            None => read.findings.push(finding(format!(
                "„{printed}“ lässt sich nicht als Wert dieses Feldes lesen."
            ))),
        }
    }
    Ok(read)
}

/// The check under which a value the model gave and code dropped is kept, with its reason.
pub const MODEL_CHECK: &str = "model:source";

/// Stores what the model read as `model:<name>`, in place of what any model read before, and its
/// findings as failed checks.
pub fn save(
    conn: &rusqlite::Connection,
    document: i64,
    model: &str,
    read: &Read,
    now: jiff::Timestamp,
) -> rusqlite::Result<()> {
    let reader = format!("model:{model}");
    let values: Vec<(String, Value)> = read
        .values
        .iter()
        .map(|value| (reader.clone(), value.clone()))
        .collect();
    super::save(conn, document, std::slice::from_ref(&reader), &values, now)?;
    conn.execute(
        "DELETE FROM field_checks WHERE document_id = ?1 AND check_id = ?2",
        rusqlite::params![document, MODEL_CHECK],
    )?;
    for finding in &read.findings {
        conn.execute(
            "INSERT INTO field_checks (document_id, field, check_id, passed, detail, checked_at) VALUES (?1, ?2, ?3, 0, ?4, ?5)",
            rusqlite::params![
                document,
                (!finding.field.is_empty()).then_some(finding.field),
                MODEL_CHECK,
                finding.detail,
                now.to_string()
            ],
        )?;
    }
    Ok(())
}

/// Asks the model for the given fields of a document and takes its answer.
pub fn ask(
    model: &dyn LocalModel,
    kind: Kind,
    fields: &[Field],
    lines: &[Line],
) -> Result<Read, ModelError> {
    if fields.is_empty() || lines.is_empty() {
        return Ok(Read::default());
    }
    let schema = schema(fields);
    let prompt = prompt(kind, fields, lines);
    let answer = model.ask(&Question {
        system: SYSTEM_TEXT,
        prompt: &prompt,
        schema: &schema,
        images: &[],
    })?;
    take(&answer, fields, lines)
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use crate::money::Cents;
    use std::sync::Mutex;

    /// A model that gives fixed answers in turn and keeps the questions.
    pub struct Scripted {
        pub answers: Mutex<Vec<Result<String, ModelError>>>,
        pub prompts: Mutex<Vec<String>>,
    }

    impl Scripted {
        pub fn new(answers: Vec<Result<String, ModelError>>) -> Self {
            Self {
                answers: Mutex::new(answers),
                prompts: Mutex::new(Vec::new()),
            }
        }
    }

    impl LocalModel for Scripted {
        fn name(&self) -> String {
            "beispiel:1b".into()
        }

        fn ask(&self, question: &Question) -> Result<String, ModelError> {
            self.prompts
                .lock()
                .unwrap()
                .push(question.prompt.to_owned());
            let mut answers = self.answers.lock().unwrap();
            if answers.is_empty() {
                Ok("{}".into())
            } else {
                answers.remove(0)
            }
        }
    }

    fn lines(texts: &[&str]) -> Vec<Line> {
        texts
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
            .collect()
    }

    #[test]
    fn a_value_is_taken_only_from_the_line_the_model_named() {
        let lines = lines(&[
            "Beispiel Stadtwerke GmbH",
            "Belegnr. 2026/77",
            "Datum: 3. März 2026",
            "Zu zahlen: 1.469,13 EUR",
        ]);
        let answer = json!({
            "sender": { "printed": "Beispiel Stadtwerke GmbH", "line": 1 },
            "invoice_number": { "printed": "2026/77", "line": 2 },
            "invoice_date": { "printed": "3. März 2026", "line": 3 },
            "gross": { "printed": "1.469,13", "line": 4 },
            "net": { "printed": "1.234,56", "line": 4 },
            "vat": { "printed": "234,57", "line": 9 },
            "iban": { "printed": "", "line": 0 },
            "due_date": { "printed": "zwei Wochen", "line": 3 },
        })
        .to_string();
        let read = take(&answer, INVOICE, &lines).unwrap();
        let value = |field: &str| {
            read.values
                .iter()
                .find(|value| value.field == field)
                .map(|value| value.value.clone())
        };
        assert_eq!(value("gross"), Some(Normalized::Cents(Cents::new(146_913))));
        assert_eq!(
            value("invoice_number"),
            Some(Normalized::Text("2026/77".into()))
        );
        assert_eq!(
            value("invoice_date"),
            Some(Normalized::Date(jiff::civil::date(2026, 3, 3)))
        );
        assert_eq!(value("net"), None, "not in the line it named, an invention");
        assert_eq!(value("vat"), None);
        assert_eq!(value("iban"), None, "not found is no finding");
        let fields: Vec<&str> = read.findings.iter().map(|finding| finding.field).collect();
        assert_eq!(fields, ["due_date", "net", "vat"]);
        assert!(
            read.findings
                .iter()
                .all(|finding| !finding.detail.is_empty())
        );
    }

    #[test]
    fn the_question_names_the_fields_and_numbers_the_lines() {
        let model = Scripted::new(vec![Ok(
            json!({ "gross": { "printed": "11,90", "line": 2 } }).to_string(),
        )]);
        let lines = lines(&["Muster Shop", "Summe 11,90"]);
        let fields: Vec<Field> = INVOICE
            .iter()
            .copied()
            .filter(|(name, _, _)| *name == "gross")
            .collect();
        let read = ask(&model, Kind::Invoice, &fields, &lines).unwrap();
        assert_eq!(read.values.len(), 1);
        let prompts = model.prompts.lock().unwrap();
        assert!(prompts[0].contains("- gross: Bruttobetrag"));
        assert!(prompts[0].contains("2: Summe 11,90"));
        assert!(
            !prompts[0].contains("invoice_number"),
            "only the fields asked for"
        );
        assert_eq!(schema(&fields)["required"], json!(["gross"]));
    }

    #[test]
    fn a_broken_answer_and_odd_forms_are_findings_not_values() {
        assert!(matches!(
            take("kein json", INVOICE, &[]),
            Err(ModelError::BadAnswer(_))
        ));
        assert_eq!(
            normalize(Form::Percent, "19 %").map(|(_, v)| v),
            Some(Normalized::Percent(19))
        );
        assert_eq!(normalize(Form::Percent, "7,5 %"), None);
        assert_eq!(
            normalize(Form::Month, "Mai 2026").map(|(_, v)| v),
            Some(Normalized::Text("2026-05".into()))
        );
        assert_eq!(normalize(Form::Iban, "DE00 1234"), None);
        assert_eq!(normalize(Form::Amount, "ca. zwölf Euro"), None);
    }
}
