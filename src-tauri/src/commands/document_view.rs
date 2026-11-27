//! The commands of the view "Dokumente" as designed (#46): the read values of a document with where
//! each stands and the bookings that paid it, deleting a document, showing its file, and the page
//! "Bestellungen" with "Rechnung zuordnen …" and "Braucht keine Rechnung".

use std::sync::{Arc, Mutex};

use serde::Serialize;
use specta::Type;
use tauri::State;

use super::CommandError;
use crate::archive::Opened;
use crate::checks;
use crate::money::Cents;

/// The fields the window shows, in this order, with their German names. Amounts are cents.
const FIELDS: [(&str, &str, bool); 13] = [
    ("sender", "Absender", false),
    ("employer", "Arbeitgeber", false),
    ("invoice_number", "Nummer", false),
    ("order_number", "Bestellung", false),
    ("invoice_date", "Datum", false),
    ("due_date", "Fällig", false),
    ("period", "Monat", false),
    ("net", "Netto", true),
    ("vat", "Umsatzsteuer", true),
    ("gross", "Brutto", true),
    ("brutto", "Brutto", true),
    ("netto", "Netto", true),
    ("iban", "IBAN", false),
];

/// A read value: its name, the value as the window shows it, and where it stands.
#[derive(Debug, Clone, Serialize, Type, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct DocumentValue {
    pub field: String,
    pub name: String,
    pub value: String,
    pub amount: bool,
    /// "read", "checked", "sender" or "owner".
    pub source: String,
}

/// A booking that paid a document.
#[derive(Debug, Clone, Serialize, Type, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct PaidBy {
    pub booking: u32,
    pub account: u32,
    pub account_name: String,
    /// ISO day.
    pub day: String,
    pub amount: String,
    pub incoming: bool,
}

#[derive(Debug, Clone, Serialize, Type, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct DocumentDetail {
    pub values: Vec<DocumentValue>,
    pub paid_by: Vec<PaidBy>,
}

fn with_ready<T>(
    archive: &Mutex<Opened>,
    run: impl FnOnce(&crate::archive::Archive) -> Result<T, CommandError>,
) -> Result<T, CommandError> {
    let opened = archive.lock().map_err(|_| CommandError::internal())?;
    let Opened::Ready(ready) = &*opened else {
        return Err(CommandError::archive_not_ready());
    };
    run(ready)
}

fn internal<E>(_: E) -> CommandError {
    CommandError::internal()
}

fn id32(id: i64) -> Result<u32, CommandError> {
    u32::try_from(id).map_err(internal)
}

/// Where a value stands: the owner's, the sender's own data, checked by the archive, or read.
pub fn source_of(family: &str, checked: bool) -> &'static str {
    match family {
        "owner" => "owner",
        "e-invoice" => "sender",
        _ if checked => "checked",
        _ => "read",
    }
}

pub fn detail(conn: &rusqlite::Connection, document: i64) -> Result<DocumentDetail, CommandError> {
    let chosen = checks::chosen(&checks::values(conn, document).map_err(internal)?);
    let passed: Vec<String> = conn
        .prepare("SELECT field FROM field_checks WHERE document_id = ?1 AND passed = 1 AND field IS NOT NULL AND check_id IN (?2, ?3)")
        .map_err(internal)?
        .query_map(rusqlite::params![document, checks::AGREE, checks::SUM], |row| row.get(0))
        .map_err(internal)?
        .collect::<rusqlite::Result<_>>()
        .map_err(internal)?;
    let values = FIELDS
        .iter()
        .filter_map(|(field, name, amount)| {
            let seen = chosen.get(*field)?;
            let value = if *amount {
                seen.value.parse::<i64>().map_or_else(
                    |_| seen.printed.clone(),
                    |cents| Cents::new(cents).to_string(),
                )
            } else if seen.value.len() == 10 && seen.value.as_bytes().get(4) == Some(&b'-') {
                seen.value.split('-').rev().collect::<Vec<_>>().join(".")
            } else {
                seen.value.clone()
            };
            Some(DocumentValue {
                field: (*field).to_owned(),
                name: (*name).to_owned(),
                value,
                amount: *amount,
                source: source_of(seen.family, passed.iter().any(|done| done == field)).to_owned(),
            })
        })
        .collect();
    let paid_by = conn
        .prepare(
            "SELECT b.id, b.account_id, a.name, b.day, b.amount_cents FROM links l
             JOIN bookings b ON b.id = l.booking_id JOIN accounts a ON a.id = b.account_id
             WHERE l.document_id = ?1 AND l.undone_at IS NULL ORDER BY b.day",
        )
        .map_err(internal)?
        .query_map([document], |row| {
            Ok((
                row.get::<_, i64>(0)?,
                row.get::<_, i64>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
                row.get::<_, i64>(4)?,
            ))
        })
        .map_err(internal)?
        .collect::<rusqlite::Result<Vec<_>>>()
        .map_err(internal)?
        .into_iter()
        .map(|(booking, account, account_name, day, cents)| {
            Ok(PaidBy {
                booking: id32(booking)?,
                account: id32(account)?,
                account_name,
                day,
                amount: Cents::new(cents.abs()).to_string(),
                incoming: cents > 0,
            })
        })
        .collect::<Result<_, CommandError>>()?;
    Ok(DocumentDetail { values, paid_by })
}

/// The read values of a document and the bookings that paid it.
#[tauri::command]
#[specta::specta]
pub fn document_detail(
    archive: State<'_, Arc<Mutex<Opened>>>,
    document: u32,
) -> Result<DocumentDetail, CommandError> {
    with_ready(&archive, |ready| {
        detail(ready.connection(), i64::from(document))
    })
}

/// "Löschen …": the document and every row about it go, a statement with its bookings. The file
/// goes to the Papierkorb of the archive.
#[tauri::command]
#[specta::specta]
pub fn delete_document(
    archive: State<'_, Arc<Mutex<Opened>>>,
    document: u32,
) -> Result<(), CommandError> {
    with_ready(&archive, |ready| {
        let conn = ready.connection();
        let id = i64::from(document);
        crate::statements::forget(conn, id).map_err(internal)?;
        let root = crate::documents::archive_root(ready.path());
        crate::documents::delete(conn, &root, id).map_err(internal)?;
        // A document that waited as the twin of this one is judged again.
        let twins: Vec<i64> = conn
            .prepare(
                "SELECT DISTINCT document_id FROM field_checks WHERE check_id = ?1 AND passed = 0",
            )
            .map_err(internal)?
            .query_map([checks::DUPLICATE], |row| row.get(0))
            .map_err(internal)?
            .collect::<rusqlite::Result<_>>()
            .map_err(internal)?;
        for twin in twins {
            checks::decide(conn, twin, jiff::Timestamp::now()).map_err(internal)?;
        }
        Ok(())
    })
}

/// "Datei im Ordner zeigen": the file manager of the system opens with the file chosen.
#[tauri::command]
#[specta::specta]
pub fn reveal_document(
    archive: State<'_, Arc<Mutex<Opened>>>,
    document: u32,
) -> Result<(), CommandError> {
    let path = with_ready(&archive, |ready| {
        let root = crate::documents::archive_root(ready.path());
        crate::documents::stored_of(ready.connection(), &root, i64::from(document))
            .map_err(internal)?
            .map(|(path, _)| path)
            .ok_or_else(CommandError::not_found)
    })?;
    let mut command = match std::env::consts::OS {
        "windows" => {
            let mut command = std::process::Command::new("explorer");
            command.arg(format!("/select,{}", path.display()));
            command
        }
        "macos" => {
            let mut command = std::process::Command::new("open");
            command.arg("-R").arg(&path);
            command
        }
        _ => {
            let mut command = std::process::Command::new("xdg-open");
            command.arg(path.parent().unwrap_or(&path));
            command
        }
    };
    command.spawn().map(|_| ()).map_err(internal)
}

/// "Braucht keine Rechnung" (`needs` false) and its undoing (`needs` true).
#[tauri::command]
#[specta::specta]
pub fn set_order_needs_invoice(
    archive: State<'_, Arc<Mutex<Opened>>>,
    order: u32,
    needs: bool,
) -> Result<(), CommandError> {
    with_ready(&archive, |ready| {
        crate::orders::set_needs_invoice(
            ready.connection(),
            i64::from(order),
            needs,
            jiff::Timestamp::now(),
        )
        .map_err(internal)?
        .then_some(())
        .ok_or_else(CommandError::not_found)
    })
}

#[derive(Debug, Clone, Serialize, Type, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct InvoiceCandidate {
    pub document: u32,
    pub name: String,
    pub gross: Option<String>,
    /// ISO day.
    pub day: Option<String>,
}

/// The invoices no order holds, the likeliest for this order first.
#[tauri::command]
#[specta::specta]
pub fn order_invoice_candidates(
    archive: State<'_, Arc<Mutex<Opened>>>,
    order: u32,
) -> Result<Vec<InvoiceCandidate>, CommandError> {
    with_ready(&archive, |ready| {
        crate::orders::candidates(ready.connection(), i64::from(order))
            .map_err(internal)?
            .into_iter()
            .map(|candidate| {
                Ok(InvoiceCandidate {
                    document: id32(candidate.document)?,
                    name: candidate.name,
                    gross: candidate.gross.map(|cents| cents.to_string()),
                    day: candidate.day,
                })
            })
            .collect()
    })
}

/// "Rechnung zuordnen …".
#[tauri::command]
#[specta::specta]
pub fn assign_order_invoice(
    archive: State<'_, Arc<Mutex<Opened>>>,
    order: u32,
    document: u32,
) -> Result<(), CommandError> {
    with_ready(&archive, |ready| {
        crate::orders::assign(ready.connection(), i64::from(order), i64::from(document))
            .map_err(internal)?
            .then_some(())
            .ok_or_else(CommandError::not_found)
    })
}

#[cfg(test)]
mod tests;
