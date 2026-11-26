//! Readers turn the text of a document into values with their source (FA-15, #11). Code, the
//! e-invoice XML, the local model and the page image are readers side by side, measured alike
//! (knowledge/decision-reading.md).

pub mod e_invoice;
pub mod image;
pub mod invoice;
pub mod model;
pub mod parse;
pub mod payslip;

use jiff::Timestamp;
use jiff::civil::Date;
use rusqlite::{Connection, params};

use crate::money::Cents;

/// Raised when a reader reads differently, so older documents are read again at the next start.
pub const READERS_VERSION: i64 = 4;

/// A line of a page with its place, as the stage `text` stored it.
#[derive(Debug, Clone, PartialEq)]
pub struct Line {
    pub page: u32,
    pub text: String,
    pub left: f64,
    pub bottom: f64,
    pub right: f64,
    pub top: f64,
}

/// What a reader gets: the kind, the lines of every page, top to bottom, and the XML of an
/// e-invoice when the PDF carries one.
#[derive(Debug, Clone, PartialEq)]
pub struct Input {
    pub kind: crate::kinds::Kind,
    pub lines: Vec<Line>,
    pub e_invoice: Option<Vec<u8>>,
}

/// A value in its normalized form. Amounts are cents, dates are dates, the rest is text.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Normalized {
    Cents(Cents),
    Date(Date),
    Percent(u32),
    Text(String),
}

impl Normalized {
    /// The form stored in `field_values.value`.
    pub fn stored(&self) -> String {
        match self {
            Self::Cents(cents) => cents.value().to_string(),
            Self::Date(date) => date.to_string(),
            Self::Percent(percent) => percent.to_string(),
            Self::Text(text) => text.clone(),
        }
    }
}

/// One value a reader found, as printed and normalized, with the line it stands on or the XML
/// element it comes from.
#[derive(Debug, Clone, PartialEq)]
pub struct Value {
    pub field: &'static str,
    pub printed: String,
    pub value: Normalized,
    pub line: Option<Line>,
    pub element: Option<String>,
    /// The page a value was read from as an image, when it has no line (#15).
    pub page: Option<u32>,
}

impl Value {
    /// A value from an element of the XML of an e-invoice, named by its path below the root.
    pub fn in_element(
        field: &'static str,
        printed: impl Into<String>,
        value: Normalized,
        element: impl Into<String>,
    ) -> Self {
        Self {
            field,
            printed: printed.into(),
            value,
            line: None,
            element: Some(element.into()),
            page: None,
        }
    }

    pub fn on(
        field: &'static str,
        printed: impl Into<String>,
        value: Normalized,
        line: &Line,
    ) -> Self {
        Self {
            field,
            printed: printed.into(),
            value,
            line: Some(line.clone()),
            element: None,
            page: None,
        }
    }
}

pub trait Reader {
    /// The name stored with each value, for example `code:1`.
    fn name(&self) -> String;
    fn read(&self, input: &Input) -> Vec<Value>;
}

/// The lines of a document with their boxes, page after page.
pub fn lines_of(conn: &Connection, document: i64) -> rusqlite::Result<Vec<Line>> {
    let mut statement = conn.prepare(
        "SELECT page, text, left, bottom, right, top FROM page_lines WHERE document_id = ?1 ORDER BY page, line",
    )?;
    let lines = statement.query_map([document], |row| {
        Ok(Line {
            page: row.get(0)?,
            text: row.get(1)?,
            left: row.get(2)?,
            bottom: row.get(3)?,
            right: row.get(4)?,
            top: row.get(5)?,
        })
    })?;
    lines.collect()
}

/// Replaces what the given readers read of a document, in any of their versions (`code:1` goes when
/// `code:2` reads). Values of other readers, the owner's above all, stay.
pub fn save(
    conn: &Connection,
    document: i64,
    readers: &[String],
    values: &[(String, Value)],
    now: Timestamp,
) -> rusqlite::Result<()> {
    for reader in readers {
        conn.execute(
            "DELETE FROM field_values WHERE document_id = ?1 AND (reader = ?2 OR reader LIKE ?3)",
            params![document, reader, format!("{}:%", family(reader))],
        )?;
    }
    let mut insert = conn.prepare(
        "INSERT INTO field_values (document_id, field, printed, value, reader, page, left, bottom, right, top, element, read_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
    )?;
    for (reader, value) in values {
        let line = value.line.as_ref();
        insert.execute(params![
            document,
            value.field,
            value.printed,
            value.value.stored(),
            reader,
            line.map(|l| l.page).or(value.page),
            line.map(|l| l.left),
            line.map(|l| l.bottom),
            line.map(|l| l.right),
            line.map(|l| l.top),
            value.element,
            now.to_string()
        ])?;
    }
    Ok(())
}

/// The first line of page 1 that is no date, no amount and does not start with one of the given
/// words: on a letter the name of the sender stands at the top.
pub(crate) fn head_line<'a>(lines: &'a [Line], refused_starts: &[&str]) -> Option<&'a Line> {
    lines.iter().filter(|line| line.page == 1).find(|line| {
        let text = line.text.trim();
        let lower = text.to_lowercase();
        text.chars().filter(|c| c.is_alphabetic()).count() >= 3
            && parse::first_date(text).is_none()
            && parse::last_amount(text).is_none()
            && !refused_starts.iter().any(|start| lower.starts_with(start))
    })
}

/// The name of a reader without its version: `code` of `code:2`.
fn family(reader: &str) -> &str {
    reader.split_once(':').map_or(reader, |(family, _)| family)
}

/// The check that says whether the XML of an e-invoice could be read (`field_checks`).
pub const E_INVOICE_CHECK: &str = "e-invoice:readable";

/// The readers that fit a kind of document. The e-invoice reader gives nothing without an XML.
pub fn for_kind(kind: crate::kinds::Kind) -> Vec<Box<dyn Reader>> {
    match kind {
        crate::kinds::Kind::Invoice | crate::kinds::Kind::Receipt => {
            vec![
                Box::new(e_invoice::EInvoice),
                Box::new(invoice::InvoiceCode),
            ]
        }
        crate::kinds::Kind::Payslip => vec![Box::new(payslip::PayslipCode)],
        crate::kinds::Kind::Statement => vec![Box::new(crate::statements::StatementCode)],
        _ => Vec::new(),
    }
}

/// Runs the readers of the kind and stores their values with the version of the readers. An
/// attached e-invoice is read whatever the kind, and whether its XML could be read is a check.
pub fn read_document(
    conn: &Connection,
    document: i64,
    kind: crate::kinds::Kind,
    now: Timestamp,
) -> rusqlite::Result<usize> {
    let input = Input {
        kind,
        lines: lines_of(conn, document)?,
        e_invoice: e_invoice::attachment_of(conn, document)?,
    };
    let mut readers = for_kind(kind);
    let e_invoice_name = e_invoice::EInvoice.name();
    if input.e_invoice.is_some() && !readers.iter().any(|reader| reader.name() == e_invoice_name) {
        readers.insert(0, Box::new(e_invoice::EInvoice));
    }
    conn.execute(
        "DELETE FROM field_checks WHERE document_id = ?1 AND check_id = ?2",
        params![document, E_INVOICE_CHECK],
    )?;
    if let Some(xml) = &input.e_invoice {
        let problem = e_invoice::read_xml(xml).problem;
        conn.execute(
            "INSERT INTO field_checks (document_id, field, check_id, passed, detail, checked_at) VALUES (?1, NULL, ?2, ?3, ?4, ?5)",
            params![
                document,
                E_INVOICE_CHECK,
                problem.is_none(),
                problem.map(|problem| problem.to_string()),
                now.to_string()
            ],
        )?;
    }
    // A statement's bookings go to their own tables (#20), a document of another kind keeps none.
    if kind == crate::kinds::Kind::Statement {
        crate::statements::store(conn, document, &input.lines, now)?;
    } else {
        crate::statements::forget(conn, document)?;
    }
    let names: Vec<String> = readers.iter().map(|reader| reader.name()).collect();
    let values: Vec<(String, Value)> = readers
        .iter()
        .flat_map(|reader| {
            reader
                .read(&input)
                .into_iter()
                .map(move |value| (reader.name(), value))
        })
        .collect();
    save(conn, document, &names, &values, now)?;
    conn.execute(
        "UPDATE documents SET read_version = ?2 WHERE id = ?1",
        params![document, READERS_VERSION],
    )?;
    Ok(values.len())
}

/// Reads again every document an older version of the readers read. Returns how many.
pub fn read_outdated(conn: &Connection, now: Timestamp) -> rusqlite::Result<usize> {
    let documents: Vec<(i64, String)> = conn
        .prepare("SELECT id, kind FROM documents WHERE text_state IS NOT NULL AND kind_signals IS NOT NULL AND read_version < ?1")?
        .query_map([READERS_VERSION], |row| Ok((row.get(0)?, row.get(1)?)))?
        .collect::<rusqlite::Result<_>>()?;
    let tx = conn.unchecked_transaction()?;
    for (id, kind) in &documents {
        read_document(&tx, *id, crate::kinds::Kind::from_code(kind), now)?;
    }
    tx.commit()?;
    Ok(documents.len())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::archive::{self, Opened};
    use crate::documents::{self, Outcome, Source};
    use crate::testing;

    fn ready(dir: &std::path::Path) -> archive::Archive {
        match archive::open(&dir.join(archive::FILE_NAME), testing::t0()).unwrap() {
            Opened::Ready(ready) => ready,
            other => panic!("{other:?}"),
        }
    }

    fn invoice(ready: &archive::Archive, root: &std::path::Path) -> i64 {
        let bytes = testing::pdf(&[
            "Beispiel Stadtwerke GmbH",
            "Rechnungsnummer R-7",
            "Umsatzsteuer 19 % 1,90",
            "Rechnungsbetrag 11,90",
        ]);
        let Outcome::Added { id } = documents::import_bytes(
            ready.connection(),
            root,
            Some(testing::pdfium()),
            &bytes,
            "r.pdf",
            Source::Dialog,
            testing::t0(),
        ) else {
            panic!("not added")
        };
        let read = crate::pages::read(testing::pdfium(), &bytes).unwrap();
        documents::save_pages(ready.connection(), id, &read, "text").unwrap();
        id
    }

    #[test]
    fn a_value_without_a_reader_is_refused() {
        let dir = tempfile::tempdir().unwrap();
        let ready = ready(dir.path());
        let id = invoice(&ready, &dir.path().join("Archiv"));
        let refused = ready.connection().execute(
            "INSERT INTO field_values (document_id, field, printed, value, reader, read_at) VALUES (?1, 'gross', '1,00', '100', '', 'x')",
            [id],
        );
        assert!(refused.is_err(), "an empty reader must be refused");
    }

    #[test]
    fn reading_stores_each_value_with_its_line_and_keeps_the_owners() {
        let dir = tempfile::tempdir().unwrap();
        let ready = ready(dir.path());
        let id = invoice(&ready, &dir.path().join("Archiv"));
        let conn = ready.connection();
        conn.execute(
            "INSERT INTO field_values (document_id, field, printed, value, reader, read_at) VALUES (?1, 'gross', '11,90', '1190', 'owner', 'x')",
            [id],
        )
        .unwrap();
        assert!(read_document(conn, id, crate::kinds::Kind::Invoice, testing::t0()).unwrap() >= 4);
        assert!(
            read_document(conn, id, crate::kinds::Kind::Invoice, testing::t0()).unwrap() >= 4,
            "reading again replaces, never doubles"
        );
        let (printed, page, top): (String, i64, f64) = conn
            .query_row("SELECT printed, page, top FROM field_values WHERE document_id = ?1 AND field = 'gross' AND reader LIKE 'code:%'", [id], |row| {
                Ok((row.get(0)?, row.get(1)?, row.get(2)?))
            })
            .unwrap();
        assert_eq!((printed.as_str(), page), ("11,90", 1));
        assert!(top > 0.0);
        let owners: i64 = conn
            .query_row(
                "SELECT count(*) FROM field_values WHERE reader = 'owner'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(owners, 1, "the owner's value stays");
        let doubled: i64 = conn
            .query_row("SELECT count(*) FROM field_values WHERE document_id = ?1 AND field = 'gross' AND reader LIKE 'code:%'", [id], |row| row.get(0))
            .unwrap();
        assert_eq!(doubled, 1);
    }

    #[test]
    fn a_newer_version_replaces_the_values_of_the_older_one() {
        let dir = tempfile::tempdir().unwrap();
        let ready = ready(dir.path());
        let id = invoice(&ready, &dir.path().join("Archiv"));
        let conn = ready.connection();
        conn.execute(            "INSERT INTO field_values (document_id, field, printed, value, reader, read_at) VALUES (?1, 'gross', '9,99', '999', 'code:1', 'x')",            [id],        )        .unwrap();
        read_document(conn, id, crate::kinds::Kind::Invoice, testing::t0()).unwrap();
        let old: i64 = conn
            .query_row(
                "SELECT count(*) FROM field_values WHERE reader = 'code:1'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(old, 0);
    }
    #[test]
    fn the_attachments_of_older_pdfs_are_looked_for_once() {
        let dir = tempfile::tempdir().unwrap();
        let ready = ready(dir.path());
        let root = dir.path().join("Archiv");
        let bytes = testing::pdf_with_xml(
            &["Rechnungsnummer R-1"],
            "factur-x.xml",
            testing::E_INVOICE_XML,
        );
        let Outcome::Added { id } = documents::import_bytes(
            ready.connection(),
            &root,
            Some(testing::pdfium()),
            &bytes,
            "e.pdf",
            Source::Dialog,
            testing::t0(),
        ) else {
            panic!("not added")
        };
        let read = crate::pages::read(testing::pdfium(), &bytes).unwrap();
        documents::save_pages(ready.connection(), id, &read, "text").unwrap();
        let conn = ready.connection();
        assert_eq!(
            e_invoice::extract_missing(conn, &root, testing::pdfium()).unwrap(),
            1
        );
        assert_eq!(
            e_invoice::extract_missing(conn, &root, testing::pdfium()).unwrap(),
            0
        );
        let xml = e_invoice::attachment_of(conn, id).unwrap().unwrap();
        assert_eq!(xml, testing::E_INVOICE_XML.as_bytes());
    }
    #[test]
    fn documents_of_an_older_reader_are_read_again_once() {
        let dir = tempfile::tempdir().unwrap();
        let ready = ready(dir.path());
        let id = invoice(&ready, &dir.path().join("Archiv"));
        let conn = ready.connection();
        documents::set_kind(conn, id, crate::kinds::Kind::Invoice, &["rechnungsnummer"]).unwrap();
        assert_eq!(read_outdated(conn, testing::t0()).unwrap(), 1);
        assert_eq!(read_outdated(conn, testing::t0()).unwrap(), 0);
        let values: i64 = conn
            .query_row(
                "SELECT count(*) FROM field_values WHERE document_id = ?1",
                [id],
                |row| row.get(0),
            )
            .unwrap();
        assert!(values >= 4);
    }
}
