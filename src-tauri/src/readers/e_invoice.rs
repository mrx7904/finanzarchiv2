//! The e-invoice reader (FA-17, #12): the values of a ZUGFeRD, Factur-X or XRechnung invoice from
//! the Cross Industry Invoice XML its PDF carries. They are what the sender's system wrote, so they
//! count as exact (knowledge/decision-reading.md).

use std::path::Path;

use jiff::civil::Date;
use quick_xml::NsReader;
use quick_xml::events::Event;
use quick_xml::name::ResolveResult;
use rusqlite::{Connection, OptionalExtension, params};

use super::{Input, Normalized, Reader, Value};
use crate::money::Cents;

/// The namespace of the root `CrossIndustryInvoice` of UN/CEFACT, used by ZUGFeRD 2, Factur-X and the
/// CII syntax of XRechnung (checked on 2026-11-25 in the reference messages of the KoSIT test suite
/// of XRechnung 3.0, `itplr-kosit/xrechnung-testsuite`). ZUGFeRD 1 used another one and is not read.
pub const CII: &str = "urn:un:unece:uncefact:data:standard:CrossIndustryInvoice:100";
/// An attachment larger than this is no invoice of a person and is not read.
pub const MAX_BYTES: usize = 5 * 1024 * 1024;
/// The names of the XML in the PDF, the more specific first. Any other `.xml` attachment comes after.
const KNOWN_NAMES: &[&str] = &[
    "factur-x.xml",
    "zugferd-invoice.xml",
    "xrechnung.xml",
    "zugferd_invoice.xml",
];

/// Why the XML of an e-invoice gave no values. German, for the owner.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum Unreadable {
    #[error("Die E-Rechnung ist größer als 5 MB und wird nicht gelesen.")]
    TooLarge,
    #[error("Die E-Rechnung ist nicht in UTF-8 geschrieben.")]
    NotUtf8,
    #[error("Die XML der E-Rechnung ist beschädigt ({0}).")]
    Broken(String),
    #[error(
        "Die angehängte XML ist keine Cross Industry Invoice (ZUGFeRD 2, Factur-X, XRechnung)."
    )]
    NotCii,
    #[error("Die E-Rechnung ist in {0}, Beträge werden nur in Euro gelesen.")]
    Currency(String),
}

/// What the XML gave: the values and, when something was left out or nothing could be read, why.
#[derive(Debug, Clone, PartialEq)]
pub struct Read {
    pub values: Vec<Value>,
    pub problem: Option<Unreadable>,
}

#[derive(Clone, Copy)]
enum Form {
    Text,
    Date,
    Amount,
    Percent,
    Iban,
}

const HEAD: &[&str] = &["SupplyChainTradeTransaction"];
const SETTLEMENT: &[&str] = &[
    "SupplyChainTradeTransaction",
    "ApplicableHeaderTradeSettlement",
];
const SUMMATION: &[&str] = &[
    "SupplyChainTradeTransaction",
    "ApplicableHeaderTradeSettlement",
    "SpecifiedTradeSettlementHeaderMonetarySummation",
];

/// Each field, the path of its element below the root and its form.
const FIELDS: &[(&str, &[&str], &[&str], Form)] = &[
    (
        "invoice_number",
        &[],
        &["ExchangedDocument", "ID"],
        Form::Text,
    ),
    (
        "invoice_date",
        &[],
        &["ExchangedDocument", "IssueDateTime", "DateTimeString"],
        Form::Date,
    ),
    (
        "sender",
        HEAD,
        &["ApplicableHeaderTradeAgreement", "SellerTradeParty", "Name"],
        Form::Text,
    ),
    (
        "due_date",
        SETTLEMENT,
        &[
            "SpecifiedTradePaymentTerms",
            "DueDateDateTime",
            "DateTimeString",
        ],
        Form::Date,
    ),
    ("net", SUMMATION, &["TaxBasisTotalAmount"], Form::Amount),
    ("vat", SUMMATION, &["TaxTotalAmount"], Form::Amount),
    ("gross", SUMMATION, &["GrandTotalAmount"], Form::Amount),
    (
        "vat_rate",
        SETTLEMENT,
        &["ApplicableTradeTax", "RateApplicablePercent"],
        Form::Percent,
    ),
    (
        "iban",
        SETTLEMENT,
        &[
            "SpecifiedTradeSettlementPaymentMeans",
            "PayeePartyCreditorFinancialAccount",
            "IBANID",
        ],
        Form::Iban,
    ),
];

/// One element with text, as found: its path below the root, its text and the attributes a field needs.
struct Found {
    path: Vec<String>,
    text: String,
    currency: Option<String>,
    format: Option<String>,
}

fn matches(path: &[String], base: &[&str], rest: &[&str]) -> bool {
    path.len() == base.len() + rest.len()
        && path
            .iter()
            .zip(base.iter().chain(rest))
            .all(|(have, want)| have == want)
}

fn attribute(start: &quick_xml::events::BytesStart, name: &str) -> Option<String> {
    start
        .attributes()
        .flatten()
        .find(|attribute| attribute.key.local_name().as_ref() == name)
        .and_then(|attribute| {
            attribute
                .normalized_value(quick_xml::XmlVersion::Implicit1_0)
                .ok()
                .map(|value| value.into_owned())
        })
}

/// Every element with text below a `CrossIndustryInvoice` root.
fn elements(xml: &str) -> Result<Vec<Found>, Unreadable> {
    let mut reader = NsReader::from_str(xml);
    let mut stack: Vec<String> = Vec::new();
    let mut open: Vec<(Option<String>, Option<String>)> = Vec::new();
    let mut text = String::new();
    let mut found = Vec::new();
    let mut rooted = false;
    loop {
        let (namespace, event) = reader
            .read_resolved_event()
            .map_err(|error| Unreadable::Broken(error.to_string()))?;
        match event {
            Event::Start(start) => {
                let local = start.local_name().as_ref().to_owned();
                if !rooted {
                    let is_cii =
                        matches!(&namespace, ResolveResult::Bound(ns) if ns.as_ref() == CII);
                    if local != "CrossIndustryInvoice" || !is_cii {
                        return Err(Unreadable::NotCii);
                    }
                    rooted = true;
                    open.push((None, None));
                    continue;
                }
                stack.push(local);
                open.push((attribute(&start, "currencyID"), attribute(&start, "format")));
                text.clear();
            }
            Event::Text(chunk) => text.push_str(&chunk.xml10_content()),
            Event::CData(chunk) => text.push_str(&chunk.xml10_content()),
            Event::GeneralRef(reference) => {
                let resolved = match reference.resolve_char_ref() {
                    Ok(Some(c)) => Some(c),
                    _ => match reference.as_ref() {
                        "amp" => Some('&'),
                        "lt" => Some('<'),
                        "gt" => Some('>'),
                        "quot" => Some('"'),
                        "apos" => Some('\''),
                        _ => None,
                    },
                };
                text.extend(resolved);
            }
            Event::End(_) => {
                let (currency, format) = open.pop().unwrap_or((None, None));
                if !text.trim().is_empty() {
                    found.push(Found {
                        path: stack.clone(),
                        text: text.trim().to_owned(),
                        currency,
                        format,
                    });
                }
                text.clear();
                stack.pop();
            }
            Event::Empty(_) if !rooted => return Err(Unreadable::NotCii),
            Event::Eof => break,
            _ => {}
        }
    }
    if rooted {
        Ok(found)
    } else {
        Err(Unreadable::NotCii)
    }
}

/// A date of the form `102` (`YYYYMMDD`), the form EN 16931 asks for.
fn date_102(text: &str, format: Option<&str>) -> Option<Date> {
    if format.is_some_and(|format| format != "102") || text.len() != 8 || !text.is_ascii() {
        return None;
    }
    let year: i16 = text[..4].parse().ok()?;
    let month: i8 = text[4..6].parse().ok()?;
    let day: i8 = text[6..].parse().ok()?;
    Date::new(year, month, day).ok()
}

/// A whole percent: "19", "19.00". A rate such as 5.5 % is left to the other readers.
fn whole_percent(text: &str) -> Option<u32> {
    let (whole, fraction) = text.split_once('.').unwrap_or((text, ""));
    (!whole.is_empty() && fraction.bytes().all(|b| b == b'0'))
        .then(|| whole.parse().ok())
        .flatten()
}

/// Reads the values of an e-invoice from its XML.
pub fn read_xml(bytes: &[u8]) -> Read {
    let failed = |problem| Read {
        values: Vec::new(),
        problem: Some(problem),
    };
    if bytes.len() > MAX_BYTES {
        return failed(Unreadable::TooLarge);
    }
    let bytes = bytes.strip_prefix(b"\xEF\xBB\xBF").unwrap_or(bytes);
    let Ok(xml) = std::str::from_utf8(bytes) else {
        return failed(Unreadable::NotUtf8);
    };
    let found = match elements(xml) {
        Ok(found) => found,
        Err(problem) => return failed(problem),
    };
    let currency = found
        .iter()
        .find(|element| matches(&element.path, SETTLEMENT, &["InvoiceCurrencyCode"]))
        .map(|element| element.text.clone());
    let in_euro = currency.as_deref().is_none_or(|code| code == "EUR");
    let mut values = Vec::new();
    for (field, base, rest, form) in FIELDS {
        let mut candidates = found
            .iter()
            .filter(|element| matches(&element.path, base, rest));
        let element = match form {
            // The total tax may stand twice, once in the currency of the tax. The invoice's counts.
            Form::Amount => candidates.find(|element| {
                element
                    .currency
                    .as_deref()
                    .is_none_or(|code| currency.as_deref().is_none_or(|own| own == code))
            }),
            // With several rates the document has no one rate.
            Form::Percent => {
                let all: Vec<&Found> = candidates.collect();
                if all.len() == 1 {
                    all.first().copied()
                } else {
                    None
                }
            }
            _ => candidates.next(),
        };
        let Some(element) = element else { continue };
        let normalized = match form {
            Form::Text => Some(Normalized::Text(element.text.clone())),
            Form::Date => date_102(&element.text, element.format.as_deref()).map(Normalized::Date),
            Form::Amount if in_euro => Cents::parse_decimal(&element.text).map(Normalized::Cents),
            Form::Amount => None,
            Form::Percent => whole_percent(&element.text).map(Normalized::Percent),
            Form::Iban => super::parse::iban_is_valid(&element.text).then(|| {
                Normalized::Text(
                    element
                        .text
                        .chars()
                        .filter(|c| !c.is_whitespace())
                        .collect(),
                )
            }),
        };
        if let Some(normalized) = normalized {
            values.push(Value::in_element(
                field,
                element.text.clone(),
                normalized,
                element.path.join("/"),
            ));
        }
    }
    Read {
        values,
        problem: (!in_euro).then(|| Unreadable::Currency(currency.unwrap_or_default())),
    }
}

pub struct EInvoice;

impl Reader for EInvoice {
    fn name(&self) -> String {
        format!("e-invoice:{}", super::READERS_VERSION)
    }

    fn read(&self, input: &Input) -> Vec<Value> {
        input
            .e_invoice
            .as_deref()
            .map(|xml| read_xml(xml).values)
            .unwrap_or_default()
    }
}

/// The XML attachment of a PDF that may be its e-invoice: a known name first, then any `.xml`.
pub fn attachment(pdfium: &crate::pdf::Pdfium, bytes: &[u8]) -> Option<(String, Vec<u8>)> {
    let document = pdfium.load_pdf_from_byte_slice(bytes, None).ok()?;
    let attachments = document.attachments();
    let named: Vec<_> = attachments
        .iter()
        .map(|attachment| (attachment.name().to_lowercase(), attachment))
        .filter(|(name, _)| name.ends_with(".xml"))
        .collect();
    let (name, chosen) = named
        .iter()
        .find(|(name, _)| KNOWN_NAMES.contains(&name.as_str()))
        .or_else(|| named.first())?;
    if chosen.len() > MAX_BYTES {
        return None;
    }
    Some((name.clone(), chosen.save_to_bytes().ok()?))
}

/// Stores the attachment the stage `text` found, or that there is none, so it is looked for once.
pub fn save_attachment(
    conn: &Connection,
    document: i64,
    found: Option<&(String, Vec<u8>)>,
) -> rusqlite::Result<()> {
    conn.execute("DELETE FROM e_invoices WHERE document_id = ?1", [document])?;
    if let Some((name, xml)) = found {
        conn.execute(
            "INSERT INTO e_invoices (document_id, name, xml) VALUES (?1, ?2, ?3)",
            params![document, name, xml],
        )?;
    }
    conn.execute(
        "UPDATE documents SET attachments_read = 1 WHERE id = ?1",
        [document],
    )?;
    Ok(())
}

pub fn attachment_of(conn: &Connection, document: i64) -> rusqlite::Result<Option<Vec<u8>>> {
    conn.query_row(
        "SELECT xml FROM e_invoices WHERE document_id = ?1",
        [document],
        |row| row.get(0),
    )
    .optional()
}

/// Looks for the attachment of every PDF stored before the stage `text` did (an archive of #06 to
/// #11). Returns how many PDFs were looked at.
pub fn extract_missing(
    conn: &Connection,
    root: &Path,
    pdfium: &crate::pdf::Pdfium,
) -> rusqlite::Result<usize> {
    let documents: Vec<i64> = conn
        .prepare("SELECT id FROM documents WHERE attachments_read = 0 AND file_type = 'pdf' AND text_state IS NOT NULL")?
        .query_map([], |row| row.get(0))?
        .collect::<rusqlite::Result<_>>()?;
    let tx = conn.unchecked_transaction()?;
    for id in &documents {
        let found = crate::documents::stored_of(&tx, root, *id)?
            .and_then(|(path, _)| std::fs::read(path).ok())
            .and_then(|bytes| attachment(pdfium, &bytes));
        save_attachment(&tx, *id, found.as_ref())?;
    }
    tx.commit()?;
    Ok(documents.len())
}

#[cfg(test)]
mod tests {
    use super::*;
    use jiff::civil::date;

    const XML: &str = crate::testing::E_INVOICE_XML;

    fn value<'a>(read: &'a Read, field: &str) -> Option<&'a Value> {
        read.values.iter().find(|value| value.field == field)
    }

    #[test]
    fn every_field_comes_from_its_element() {
        let read = read_xml(XML.as_bytes());
        assert_eq!(read.problem, None);
        let normalized = |field| value(&read, field).map(|value| value.value.clone());
        assert_eq!(
            normalized("invoice_number"),
            Some(Normalized::Text("R-2026-0042".into()))
        );
        assert_eq!(
            normalized("invoice_date"),
            Some(Normalized::Date(date(2026, 11, 20)))
        );
        assert_eq!(
            normalized("due_date"),
            Some(Normalized::Date(date(2026, 12, 4)))
        );
        assert_eq!(
            normalized("sender"),
            Some(Normalized::Text("Beispiel Stadtwerke & Co. GmbH".into())),
            "the seller, not the buyer or a product"
        );
        assert_eq!(
            normalized("net"),
            Some(Normalized::Cents(Cents::new(123_456)))
        );
        assert_eq!(
            normalized("vat"),
            Some(Normalized::Cents(Cents::new(23_457))),
            "the tax in the currency of the invoice"
        );
        assert_eq!(
            normalized("gross"),
            Some(Normalized::Cents(Cents::new(146_913)))
        );
        assert_eq!(normalized("vat_rate"), Some(Normalized::Percent(19)));
        assert_eq!(
            normalized("iban"),
            Some(Normalized::Text("DE89370400440532013000".into()))
        );
        let gross = value(&read, "gross").unwrap();
        assert_eq!(gross.printed, "1469.13");
        assert_eq!(gross.line, None);
        assert_eq!(
            gross.element.as_deref(),
            Some(
                "SupplyChainTradeTransaction/ApplicableHeaderTradeSettlement/SpecifiedTradeSettlementHeaderMonetarySummation/GrandTotalAmount"
            )
        );
    }

    #[test]
    fn a_broken_or_foreign_xml_gives_no_values_and_says_why() {
        let cut = &XML[..XML.len() / 2];
        let broken = read_xml(cut.as_bytes());
        assert!(broken.values.is_empty());
        assert!(
            matches!(
                broken.problem,
                Some(Unreadable::Broken(_)) | Some(Unreadable::NotCii)
            ),
            "{:?}",
            broken.problem
        );
        let mismatched = read_xml(
            XML.replace("</ram:GrandTotalAmount>", "</ram:Gross>")
                .as_bytes(),
        );
        assert!(
            matches!(mismatched.problem, Some(Unreadable::Broken(_))),
            "{:?}",
            mismatched.problem
        );
        assert!(mismatched.values.is_empty());
        let ubl = read_xml(br#"<Invoice xmlns="urn:oasis:names:specification:ubl:schema:xsd:Invoice-2"><ID>1</ID></Invoice>"#);
        assert_eq!(ubl.problem, Some(Unreadable::NotCii));
        let ferd1 = read_xml(br#"<rsm:CrossIndustryInvoice xmlns:rsm="urn:ferd:CrossIndustryDocument:invoice:1p0"/>"#);
        assert_eq!(
            ferd1.problem,
            Some(Unreadable::NotCii),
            "ZUGFeRD 1 is not read"
        );
        assert_eq!(
            read_xml(&[0xff, 0xfe, 0x00]).problem,
            Some(Unreadable::NotUtf8)
        );
        assert_eq!(
            read_xml(&vec![b' '; MAX_BYTES + 1]).problem,
            Some(Unreadable::TooLarge)
        );
    }

    #[test]
    fn a_foreign_currency_keeps_the_amounts_out() {
        let read = read_xml(
            XML.replace(
                "<ram:InvoiceCurrencyCode>EUR</ram:InvoiceCurrencyCode>",
                "<ram:InvoiceCurrencyCode>CHF</ram:InvoiceCurrencyCode>",
            )
            .as_bytes(),
        );
        assert_eq!(read.problem, Some(Unreadable::Currency("CHF".into())));
        assert!(value(&read, "gross").is_none());
        assert!(value(&read, "invoice_number").is_some());
    }

    #[test]
    fn several_rates_give_no_rate_and_odd_dates_none() {
        let two = XML.replace(
            "<ram:SpecifiedTradePaymentTerms>",
            "<ram:ApplicableTradeTax><ram:RateApplicablePercent>7</ram:RateApplicablePercent></ram:ApplicableTradeTax><ram:SpecifiedTradePaymentTerms>",
        );
        assert!(value(&read_xml(two.as_bytes()), "vat_rate").is_none());
        assert_eq!(date_102("20261131", Some("102")), None);
        assert_eq!(date_102("202611", Some("610")), None);
        assert_eq!(date_102("20260229", None), None);
        assert_eq!(whole_percent("5.5"), None);
        assert_eq!(whole_percent("7"), Some(7));
    }
}
