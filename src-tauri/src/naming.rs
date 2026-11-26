//! Names by template (FA-23, #18): a filed document is named by the template of its kind, filled by
//! code from its values. A name the owner typed is kept. The file on disk keeps its hash name.

use std::collections::BTreeMap;

use rusqlite::{Connection, OptionalExtension, params};

use crate::checks::{self, Seen};
use crate::kinds::Kind;
use crate::money::Cents;

/// The placeholders a template may use, with what they mean, for the owner.
pub const PLACEHOLDERS: &[(&str, &str)] = &[
    ("datum", "Datum des Dokuments, Jahr zuerst"),
    ("monat", "Monat einer Abrechnung, Jahr zuerst"),
    ("absender", "Absender oder Arbeitgeber ohne Rechtsform"),
    ("thema", "Art des Dokuments mit seiner Nummer"),
    ("nummer", "Rechnungsnummer"),
    ("betrag", "Bruttobetrag oder Netto"),
];

/// The default template of a kind, date first as the owner chose in 1.0. `None`: the kind keeps the
/// name of its file.
pub fn default_template(kind: Kind) -> Option<&'static str> {
    match kind {
        Kind::Invoice | Kind::Receipt => Some("{datum}_{absender}_{thema}"),
        Kind::Payslip => Some("{monat}_{absender}_Gehaltsabrechnung"),
        Kind::Statement => Some("{datum}_{absender}_Kontoauszug"),
        Kind::Other | Kind::Unknown => None,
    }
}

/// The kinds that have a template, in the order the owner sees them.
pub const NAMED_KINDS: [Kind; 4] = [Kind::Invoice, Kind::Receipt, Kind::Payslip, Kind::Statement];

fn setting(kind: Kind) -> String {
    format!("name.template.{}", kind.code())
}

/// The template of a kind: the owner's, else the default.
pub fn template(conn: &Connection, kind: Kind) -> rusqlite::Result<Option<String>> {
    let own: Option<String> = conn
        .query_row(
            "SELECT value FROM settings WHERE key = ?1",
            [setting(kind)],
            |row| row.get(0),
        )
        .optional()?;
    Ok(own
        .filter(|own| !own.trim().is_empty())
        .or_else(|| default_template(kind).map(str::to_owned)))
}

/// Why a template was not taken. German, for the owner.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum TemplateError {
    #[error("„{{{0}}}“ ist kein Platzhalter. Möglich sind {1}.")]
    Unknown(String, String),
    #[error("Die Vorlage braucht mindestens einen Platzhalter.")]
    Empty,
}

/// Stores the owner's template of a kind, an empty one restores the default.
pub fn set_template(conn: &Connection, kind: Kind, template: &str) -> Result<(), TemplateError> {
    let template = template.trim();
    if template.is_empty() {
        let _ = conn.execute("DELETE FROM settings WHERE key = ?1", [setting(kind)]);
        return Ok(());
    }
    let used = placeholders(template);
    if used.is_empty() {
        return Err(TemplateError::Empty);
    }
    if let Some(unknown) = used
        .iter()
        .find(|name| !PLACEHOLDERS.iter().any(|(known, _)| known == name))
    {
        let known: Vec<String> = PLACEHOLDERS
            .iter()
            .map(|(name, _)| format!("{{{name}}}"))
            .collect();
        return Err(TemplateError::Unknown(unknown.clone(), known.join(", ")));
    }
    let _ = conn.execute(
        "INSERT INTO settings (key, value) VALUES (?1, ?2) ON CONFLICT (key) DO UPDATE SET value = excluded.value",
        params![setting(kind), template],
    );
    Ok(())
}

fn placeholders(template: &str) -> Vec<String> {
    let mut found = Vec::new();
    let mut rest = template;
    while let Some(start) = rest.find('{') {
        let Some(end) = rest[start..].find('}') else {
            break;
        };
        found.push(rest[start + 1..start + end].to_owned());
        rest = &rest[start + end + 1..];
    }
    found
}

/// Legal forms left out of a sender, the longer first so "GmbH & Co. KG" goes as one.
const LEGAL_FORMS: &[&str] = &[
    "gmbh & co. kgaa",
    "gmbh & co. kg",
    "ug (haftungsbeschränkt)",
    "gmbh",
    "mbh",
    "kgaa",
    "ag",
    "kg",
    "ohg",
    "gbr",
    "e.v.",
    "e. v.",
    "e.k.",
    "e. k.",
    "se",
    "ug",
    "ltd.",
    "ltd",
    "inc.",
    "inc",
    "llc",
];

/// The sender without its legal form at the end: "Beispiel Stadtwerke GmbH" gives "Beispiel Stadtwerke".
pub fn without_legal_form(sender: &str) -> String {
    let mut name = sender.trim().trim_end_matches(',').trim().to_owned();
    loop {
        let lower = name.to_lowercase();
        let Some(form) = LEGAL_FORMS.iter().find(|form| {
            lower.ends_with(*form)
                && lower[..lower.len() - form.len()]
                    .chars()
                    .next_back()
                    .is_some_and(|before| before == ' ' || before == ',')
        }) else {
            break;
        };
        let cut = name.len() - form.len();
        name = name[..cut].trim().trim_end_matches(',').trim().to_owned();
    }
    if name.is_empty() {
        sender.trim().to_owned()
    } else {
        name
    }
}

/// Removes what a file system of Windows, macOS or Linux refuses in a name, and folds spaces.
pub fn safe(text: &str) -> String {
    let cleaned: String = text
        .chars()
        .map(|c| match c {
            '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|' => ' ',
            c if c.is_control() => ' ',
            c => c,
        })
        .collect();
    let folded = cleaned.split_whitespace().collect::<Vec<_>>().join(" ");
    let trimmed = folded.trim_matches([' ', '.']);
    trimmed
        .chars()
        .take(120)
        .collect::<String>()
        .trim_end_matches([' ', '.'])
        .to_owned()
}

fn kind_word(kind: Kind) -> &'static str {
    match kind {
        Kind::Invoice => "Rechnung",
        Kind::Receipt => "Zahlungsbeleg",
        Kind::Payslip => "Gehaltsabrechnung",
        Kind::Statement => "Kontoauszug",
        Kind::Other | Kind::Unknown => "Dokument",
    }
}

/// The values of the placeholders, from the values that count for each field.
fn filling(kind: Kind, chosen: &BTreeMap<String, Seen>) -> BTreeMap<&'static str, String> {
    let get = |field: &str| chosen.get(field).map(|seen| seen.value.clone());
    let mut filled = BTreeMap::new();
    if let Some(date) = get("invoice_date")
        .or_else(|| get("period_to"))
        .or_else(|| get("period").map(|month| format!("{month}-01")))
    {
        filled.insert("datum", date);
    }
    if let Some(month) = get("period") {
        filled.insert("monat", month);
    }
    if let Some(sender) = get("sender").or_else(|| get("employer")) {
        filled.insert("absender", without_legal_form(&sender));
    }
    if let Some(number) = get("invoice_number") {
        filled.insert("nummer", number.clone());
        filled.insert("thema", format!("{} {number}", kind_word(kind)));
    } else {
        filled.insert("thema", kind_word(kind).to_owned());
    }
    if let Some(cents) = get("gross")
        .or_else(|| get("netto"))
        .and_then(|value| value.parse::<i64>().ok())
    {
        filled.insert("betrag", Cents::new(cents).to_string());
    }
    filled
}

/// The name a template gives. A placeholder without a value is left out with its separator. A name
/// with no value of the document in it is no name: `None`, the file keeps its name.
pub fn render(template: &str, filled: &BTreeMap<&str, String>) -> Option<String> {
    let mut parts: Vec<String> = Vec::new();
    let mut any_value = false;
    for part in template.split('_') {
        let mut text = part.to_owned();
        for name in placeholders(part) {
            let value = filled
                .get(name.as_str())
                .map(|value| safe(value))
                .unwrap_or_default();
            // "thema" alone is the kind's word, no value of the document.
            any_value |= !value.is_empty() && (name != "thema" || value.contains(' '));
            text = text.replace(&format!("{{{name}}}"), &value);
        }
        let text = safe(&text);
        if !text.is_empty() {
            parts.push(text);
        }
    }
    (any_value && !parts.is_empty()).then(|| parts.join("_"))
}

/// Names a document by the template of its kind, unless the owner named it. Returns the name.
pub fn name_document(conn: &Connection, document: i64) -> rusqlite::Result<Option<String>> {
    let (kind, by_owner): (String, bool) = conn.query_row(
        "SELECT kind, name_by_owner FROM documents WHERE id = ?1",
        [document],
        |row| Ok((row.get(0)?, row.get(1)?)),
    )?;
    if by_owner {
        return Ok(None);
    }
    let kind = Kind::from_code(&kind);
    let Some(template) = template(conn, kind)? else {
        return Ok(None);
    };
    let name = render(
        &template,
        &filling(kind, &checks::chosen(&checks::values(conn, document)?)),
    );
    conn.execute(
        "UPDATE documents SET name = ?2 WHERE id = ?1",
        params![document, name],
    )?;
    Ok(name)
}

/// The owner's name of a document, kept from now on. An empty name gives the document back to its
/// template.
pub fn rename(conn: &Connection, document: i64, name: &str) -> rusqlite::Result<Option<String>> {
    let name = safe(name);
    if name.is_empty() {
        conn.execute(
            "UPDATE documents SET name_by_owner = 0 WHERE id = ?1",
            [document],
        )?;
        let state: String = conn.query_row(
            "SELECT state FROM documents WHERE id = ?1",
            [document],
            |row| row.get(0),
        )?;
        if state == "filed" {
            return name_document(conn, document);
        }
        conn.execute("UPDATE documents SET name = NULL WHERE id = ?1", [document])?;
        return Ok(None);
    }
    conn.execute(
        "UPDATE documents SET name = ?2, name_by_owner = 1 WHERE id = ?1",
        params![document, name],
    )?;
    Ok(Some(name))
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    fn filled(pairs: &[(&'static str, &str)]) -> BTreeMap<&'static str, String> {
        pairs
            .iter()
            .map(|(key, value)| (*key, (*value).to_owned()))
            .collect()
    }

    #[test]
    fn an_invoice_is_named_date_first_without_the_legal_form() {
        let name = render(
            "{datum}_{absender}_{thema}",
            &filled(&[
                ("datum", "2026-11-20"),
                ("absender", &without_legal_form("Beispiel Stadtwerke GmbH")),
                ("thema", "Rechnung R-2026/0042"),
            ]),
        );
        assert_eq!(
            name.as_deref(),
            Some("2026-11-20_Beispiel Stadtwerke_Rechnung R-2026 0042")
        );
    }

    #[test]
    fn legal_forms_go_and_the_name_stays() {
        assert_eq!(without_legal_form("Muster Bau GmbH & Co. KG"), "Muster Bau");
        assert_eq!(without_legal_form("Beispielverein e.V."), "Beispielverein");
        assert_eq!(without_legal_form("Kirchengemeinde"), "Kirchengemeinde");
        assert_eq!(without_legal_form("Beispielverein e. V."), "Beispielverein");
        assert_eq!(without_legal_form("Muster AG,"), "Muster");
        assert_eq!(without_legal_form("Agentur Muster"), "Agentur Muster");
        assert_eq!(
            without_legal_form("GmbH"),
            "GmbH",
            "nothing left keeps the name"
        );
    }

    #[test]
    fn a_missing_value_leaves_its_part_out_and_no_value_gives_no_name() {
        let name = render(
            "{datum}_{absender}_{thema}",
            &filled(&[("absender", "Muster Shop"), ("thema", "Rechnung")]),
        );
        assert_eq!(name.as_deref(), Some("Muster Shop_Rechnung"));
        assert_eq!(render("{datum}_{absender}_Kontoauszug", &filled(&[])), None);
        assert_eq!(
            render(
                "{datum}_{absender}_{thema}",
                &filled(&[("thema", "Rechnung")])
            ),
            None
        );
    }

    #[test]
    fn a_template_with_an_unknown_placeholder_is_refused() {
        let dir = tempfile::tempdir().unwrap();
        let conn = Connection::open(dir.path().join("t.db")).unwrap();
        conn.execute_batch(include_str!("../migrations/0001_settings.sql"))
            .unwrap();
        let error = set_template(&conn, Kind::Invoice, "{datum}_{firma}").unwrap_err();
        assert!(
            error.to_string().contains("„{firma}“ ist kein Platzhalter"),
            "{error}"
        );
        assert_eq!(
            set_template(&conn, Kind::Invoice, "Rechnung"),
            Err(TemplateError::Empty)
        );
        set_template(&conn, Kind::Invoice, "{absender}_{datum}").unwrap();
        assert_eq!(
            template(&conn, Kind::Invoice).unwrap().as_deref(),
            Some("{absender}_{datum}")
        );
        set_template(&conn, Kind::Invoice, "").unwrap();
        assert_eq!(
            template(&conn, Kind::Invoice).unwrap().as_deref(),
            default_template(Kind::Invoice)
        );
    }

    proptest! {
        #[test]
        fn a_name_never_holds_a_separator_of_a_path(sender in "\\PC{0,40}", number in "\\PC{0,20}") {
            let name = render("{datum}_{absender}_{thema}", &filled(&[("datum", "2026-01-02"), ("absender", &sender), ("thema", &format!("Rechnung {number}"))]));
            if let Some(name) = name {
                prop_assert!(!name.contains('/') && !name.contains('\\'), "{}", name);
                prop_assert!(!name.chars().any(|c| c.is_control() || ":*?\"<>|".contains(c)), "{}", name);
                prop_assert!(!name.ends_with('.') && !name.ends_with(' '));
            }
        }
    }
}
