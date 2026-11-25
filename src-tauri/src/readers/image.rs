//! Reading the page as an image (FA-20, #15): a scan, or a decisive value no reader found in the
//! text, is read by the local vision model from the picture of the page. The model copies each
//! value as printed and names its page, code parses it (knowledge/decision-reading.md).

use base64::Engine;
use serde_json::{Value as Json, json};

use super::Value;
use super::model::{Field, fields_of, normalize};
use crate::kinds::Kind;
use crate::ollama::{LocalModel, ModelError, Question};

/// The width of the page image the model gets, chosen by measurement (knowledge/decision-model.md).
pub const WIDTH: u32 = 1280;
/// At most this many pages go to the model, the head of a long document.
pub const MAX_PAGES: u32 = 3;

const SYSTEM: &str = "Du liest Werte aus dem Bild von Seiten eines Dokuments. Kopiere jeden Wert genau so, \
wie er gedruckt ist, Beträge mit Punkt und Komma wie gedruckt, Daten wie gedruckt. Gib zu jedem Wert die \
Nummer der Seite an, auf der er steht, die erste Seite ist 1. Steht ein Wert nicht auf den Seiten, gib einen \
leeren Text und die Seite 0 an. Rechne nichts aus, ergänze nichts und erfinde nichts.";

/// The kinds the model may name for a scan, with their German words.
const KINDS: &[(Kind, &str)] = &[
    (Kind::Invoice, "Rechnung"),
    (Kind::Receipt, "Zahlungsbeleg oder Quittung"),
    (Kind::Payslip, "Gehaltsabrechnung"),
    (Kind::Statement, "Kontoauszug"),
    (Kind::Other, "etwas anderes"),
];

/// The values a decision rests on: a document with text is read as an image only for these.
pub fn decisive(kind: Kind) -> &'static [&'static str] {
    match kind {
        Kind::Invoice | Kind::Receipt => &["gross"],
        Kind::Payslip => &["brutto", "netto"],
        _ => &[],
    }
}

/// What the image gave: the kind when it was asked for, the values and the findings.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Read {
    pub kind: Option<Kind>,
    pub values: Vec<Value>,
    pub findings: Vec<super::model::Finding>,
}

/// Every field a scan may carry, each once: those of an invoice and those of a payslip.
fn all_fields() -> Vec<Field> {
    let mut fields: Vec<Field> = Vec::new();
    for kind in [Kind::Invoice, Kind::Payslip] {
        for field in fields_of(kind) {
            if !fields.iter().any(|(name, _, _)| name == &field.0) {
                fields.push(*field);
            }
        }
    }
    fields
}

pub fn schema(fields: &[Field], with_kind: bool) -> Json {
    let mut properties: serde_json::Map<String, Json> = fields
        .iter()
        .map(|(name, _, _)| {
            (
                (*name).to_owned(),
                json!({
                    "type": "object",
                    "properties": { "printed": { "type": "string" }, "page": { "type": "integer" } },
                    "required": ["printed", "page"],
                }),
            )
        })
        .collect();
    let mut required: Vec<&str> = fields.iter().map(|(name, _, _)| *name).collect();
    if with_kind {
        let codes: Vec<&str> = KINDS.iter().map(|(kind, _)| kind.code()).collect();
        properties.insert("kind".into(), json!({ "type": "string", "enum": codes }));
        required.insert(0, "kind");
    }
    json!({ "type": "object", "properties": properties, "required": required })
}

fn prompt(kind: Option<Kind>, fields: &[Field], pages: usize) -> String {
    let mut text = match kind {
        Some(kind) => format!("Art des Dokuments: {}\n", kind.code()),
        None => {
            let mut text = String::from("Bestimme zuerst die Art des Dokuments (kind):\n");
            for (kind, label) in KINDS {
                text.push_str(&format!("- {}: {label}\n", kind.code()));
            }
            text
        }
    };
    text.push_str(&format!("\n{pages} Seite(n) als Bild. Felder:\n"));
    for (name, meaning, _) in fields {
        text.push_str(&format!("- {name}: {meaning}\n"));
    }
    text
}

/// Takes the answer: the kind, and each value code can parse from what the model copied.
pub fn take(
    answer: &str,
    fields: &[Field],
    pages: u32,
    with_kind: bool,
) -> Result<Read, ModelError> {
    let answer: Json =
        serde_json::from_str(answer).map_err(|error| ModelError::BadAnswer(error.to_string()))?;
    let kind = with_kind.then(|| {
        let code = answer["kind"].as_str().unwrap_or("unknown");
        KINDS
            .iter()
            .find(|(kind, _)| kind.code() == code)
            .map_or(Kind::Unknown, |(kind, _)| *kind)
    });
    let wanted: Vec<&str> = match kind {
        Some(kind) => fields_of(kind).iter().map(|(name, _, _)| *name).collect(),
        None => fields.iter().map(|(name, _, _)| *name).collect(),
    };
    let mut read = Read {
        kind,
        ..Read::default()
    };
    for &(field, _, form) in fields.iter().filter(|(name, _, _)| wanted.contains(name)) {
        let entry = &answer[field];
        let printed = entry["printed"].as_str().unwrap_or("").trim();
        let page = entry["page"].as_u64().unwrap_or(0);
        if printed.is_empty() || page == 0 {
            continue;
        }
        let finding = |detail: String| super::model::Finding { field, detail };
        let Some(page) = u32::try_from(page).ok().filter(|page| *page <= pages) else {
            read.findings.push(finding(format!(
                "Das Modell nannte Seite {page}, die es nicht gibt."
            )));
            continue;
        };
        match normalize(form, printed) {
            Some((printed, value)) => read.values.push(Value {
                field,
                printed,
                value,
                line: None,
                element: None,
                page: Some(page),
            }),
            None => read.findings.push(finding(format!(
                "„{printed}“ lässt sich nicht als Wert dieses Feldes lesen."
            ))),
        }
    }
    Ok(read)
}

/// Asks the vision model about the page images (PNG). With `kind` `None` it names the kind first and
/// reads every field of that kind, with a kind it reads the given fields.
pub fn ask(
    model: &dyn LocalModel,
    kind: Option<Kind>,
    fields: &[Field],
    pngs: &[Vec<u8>],
) -> Result<Read, ModelError> {
    let fields: Vec<Field> = match kind {
        Some(_) => fields.to_vec(),
        None => all_fields(),
    };
    if pngs.is_empty() || fields.is_empty() {
        return Ok(Read::default());
    }
    let images: Vec<String> = pngs
        .iter()
        .map(|png| base64::engine::general_purpose::STANDARD.encode(png))
        .collect();
    let schema = schema(&fields, kind.is_none());
    let prompt = prompt(kind, &fields, pngs.len());
    let answer = model.ask(&Question {
        system: SYSTEM,
        prompt: &prompt,
        schema: &schema,
        images: &images,
    })?;
    take(
        &answer,
        &fields,
        u32::try_from(pngs.len()).unwrap_or(u32::MAX),
        kind.is_none(),
    )
}

/// The page images of a stored file for the model: up to `MAX_PAGES` pages of a PDF, or the image.
pub fn pages_of(
    pdfium: &crate::pdf::Pdfium,
    bytes: &[u8],
    file_type: crate::documents::FileType,
    width: u32,
) -> Vec<Vec<u8>> {
    match file_type {
        crate::documents::FileType::Pdf => (1..=MAX_PAGES)
            .map_while(|number| crate::pages::render_pdf_page(pdfium, bytes, number, width).ok())
            .collect(),
        _ => crate::pages::render_image(bytes, width)
            .ok()
            .into_iter()
            .collect(),
    }
}

/// Whether the values of a kind still lack a decisive one.
pub fn lacks_decisive(kind: Kind, found: &[String]) -> Vec<Field> {
    fields_of(kind)
        .iter()
        .copied()
        .filter(|(name, _, _)| decisive(kind).contains(name) && !found.iter().any(|f| f == name))
        .collect()
}

/// The check under which a value the vision model gave and code dropped is kept, with its reason.
pub const IMAGE_CHECK: &str = "image:source";

/// Stores what the vision model read as `image:<name>` with the page, in place of what any model
/// read from the image before, and its findings as failed checks.
pub fn save(
    conn: &rusqlite::Connection,
    document: i64,
    model: &str,
    read: &Read,
    now: jiff::Timestamp,
) -> rusqlite::Result<()> {
    let reader = format!("image:{model}");
    let values: Vec<(String, Value)> = read
        .values
        .iter()
        .map(|value| (reader.clone(), value.clone()))
        .collect();
    super::save(conn, document, std::slice::from_ref(&reader), &values, now)?;
    conn.execute(
        "DELETE FROM field_checks WHERE document_id = ?1 AND check_id = ?2",
        rusqlite::params![document, IMAGE_CHECK],
    )?;
    for finding in &read.findings {
        conn.execute(
            "INSERT INTO field_checks (document_id, field, check_id, passed, detail, checked_at) VALUES (?1, ?2, ?3, 0, ?4, ?5)",
            rusqlite::params![
                document,
                (!finding.field.is_empty()).then_some(finding.field),
                IMAGE_CHECK,
                finding.detail,
                now.to_string()
            ],
        )?;
    }
    Ok(())
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::money::Cents;
    use crate::readers::Normalized;
    use crate::readers::model::tests::Scripted;

    #[test]
    fn a_scan_gets_its_kind_and_the_values_of_that_kind() {
        let model = Scripted::new(vec![Ok(json!({
            "kind": "invoice",
            "gross": { "printed": "1.469,13", "page": 1 },
            "invoice_number": { "printed": "R-2026-0042", "page": 1 },
            "brutto": { "printed": "4.000,00", "page": 1 },
            "vat": { "printed": "234,57", "page": 4 },
            "net": { "printed": "viel", "page": 1 },
        })
        .to_string())]);
        let read = ask(&model, None, &[], &[vec![0x89, b'P']]).unwrap();
        assert_eq!(read.kind, Some(Kind::Invoice));
        let gross = read
            .values
            .iter()
            .find(|value| value.field == "gross")
            .unwrap();
        assert_eq!(gross.value, Normalized::Cents(Cents::new(146_913)));
        assert_eq!((gross.page, gross.line.as_ref()), (Some(1), None));
        assert!(
            read.values.iter().all(|value| value.field != "brutto"),
            "a payslip field is no value of an invoice"
        );
        let fields: Vec<&str> = read.findings.iter().map(|finding| finding.field).collect();
        assert_eq!(fields, ["net", "vat"]);
        let prompt = model.prompts.lock().unwrap().remove(0);
        assert!(prompt.contains("Bestimme zuerst die Art"));
    }

    #[test]
    fn a_text_document_is_asked_only_for_its_missing_decisive_values() {
        assert_eq!(
            lacks_decisive(Kind::Payslip, &["brutto".into()])
                .iter()
                .map(|(name, _, _)| *name)
                .collect::<Vec<_>>(),
            ["netto"]
        );
        assert!(lacks_decisive(Kind::Invoice, &["gross".into()]).is_empty());
        assert!(lacks_decisive(Kind::Statement, &[]).is_empty());
        let model = Scripted::new(vec![Ok(
            json!({ "netto": { "printed": "2.594,40", "page": 1 } }).to_string(),
        )]);
        let fields = lacks_decisive(Kind::Payslip, &["brutto".into()]);
        let read = ask(&model, Some(Kind::Payslip), &fields, &[vec![1]]).unwrap();
        assert_eq!(read.kind, None);
        assert_eq!(read.values.len(), 1);
        assert_eq!(
            schema(&fields, false)["required"],
            json!(["netto"]),
            "no kind asked of a document whose kind is known"
        );
    }
}
