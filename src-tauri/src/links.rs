//! Documents linked to the bookings that paid them (FA-29, #24). Rules with stable ids, the surest
//! first, each link a decision with the values it saw, which "Rückgängig" takes back. Where more
//! than one booking fits, the document waits as a question in "Prüfen".

use std::collections::BTreeMap;

use jiff::Timestamp;
use jiff::civil::Date;
use rusqlite::{Connection, OptionalExtension, params};
use serde_json::json;

use crate::kinds::Kind;

/// The invoice number stands in the purpose and the amount fits. One booking may pay several
/// invoices whose numbers it names, when their sum is its amount to the cent.
pub const RULE_NUMBER: &str = "link.number:1";
/// The amount with the party, within the days around the invoice.
pub const RULE_PARTY_AMOUNT: &str = "link.party-amount:1";
/// The amount within the days, and a word of the sender's name in the booking's other side or
/// purpose: the bank spells the party otherwise than the document.
pub const RULE_AMOUNT_NAME: &str = "link.amount-name:1";
/// A payslip's Netto from its employer, in its month or the first days after it.
pub const RULE_PAYSLIP: &str = "link.payslip:1";
/// The owner chose the booking in "Prüfen".
pub const RULE_OWNER: &str = "link.owner:1";

/// The parameters of the rules, in one place.
pub mod parameters {
    /// Days before the invoice date a payment may lie (paid in advance, a direct debit).
    pub const DAYS_BEFORE: i64 = 5;
    /// Days after the due date, or after the invoice date without one, a payment may lie.
    pub const DAYS_AFTER: i64 = 45;
    /// Days into the next month a salary may be paid.
    pub const SALARY_DAYS_AFTER_MONTH: i64 = 10;
    /// An invoice number shorter than this is too common to find in a purpose.
    pub const MIN_NUMBER: usize = 4;
    /// A word of a name shorter than this is too common to tell a party.
    pub const MIN_WORD: usize = 4;
    /// At most this many bookings are offered in a question.
    pub const MAX_CANDIDATES: usize = 5;
}

/// What a document brings to the rules.
struct Document {
    id: i64,
    kind: Kind,
    party: Option<i64>,
    cents: i64,
    number: Option<String>,
    /// The words of the sender's or employer's name that may tell its bookings.
    words: Vec<String>,
    from: Date,
    until: Date,
}

fn alnum(text: &str) -> String {
    text.chars()
        .filter(|c| c.is_alphanumeric())
        .collect::<String>()
        .to_lowercase()
}

fn day(text: &str) -> Option<Date> {
    text.parse().ok()
}

fn plus(day: Date, days: i64) -> Date {
    day.checked_add(jiff::Span::new().days(days)).unwrap_or(day)
}

/// The filed invoices, receipts and payslips without a link and never asked about (a question the
/// owner answered, or a link undone, leaves a document to the owner), with what the rules need.
fn open_documents(conn: &Connection) -> rusqlite::Result<Vec<Document>> {
    let rows: Vec<(i64, String, Option<i64>)> = conn
        .prepare(
            "SELECT id, kind, party_id FROM documents d WHERE state = 'filed' AND kind IN ('invoice', 'receipt', 'payslip')
             AND NOT EXISTS (SELECT 1 FROM links l WHERE l.document_id = d.id AND l.undone_at IS NULL)
             AND NOT EXISTS (SELECT 1 FROM link_questions q WHERE q.document_id = d.id)",
        )?
        .query_map([], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)))?
        .collect::<rusqlite::Result<_>>()?;
    let mut documents = Vec::new();
    for (id, kind, party) in rows {
        let kind = Kind::from_code(&kind);
        let chosen = crate::checks::chosen(&crate::checks::values(conn, id)?);
        let get = |field: &str| chosen.get(field).map(|seen| seen.value.clone());
        let words: Vec<String> = get("sender")
            .or_else(|| get("employer"))
            .map(|name| {
                crate::naming::without_legal_form(&name)
                    .to_lowercase()
                    .split(|c: char| !c.is_alphanumeric())
                    .filter(|word| word.chars().count() >= parameters::MIN_WORD)
                    .map(str::to_owned)
                    .collect()
            })
            .unwrap_or_default();
        let document = if kind == Kind::Payslip {
            let (Some(cents), Some(month)) =
                (get("netto").and_then(|v| v.parse().ok()), get("period"))
            else {
                continue;
            };
            let Some(first) = day(&format!("{month}-01")) else {
                continue;
            };
            let last = first.last_of_month();
            Document {
                id,
                kind,
                party,
                cents,
                number: None,
                words,
                from: first,
                until: plus(last, parameters::SALARY_DAYS_AFTER_MONTH),
            }
        } else {
            let Some(cents) = get("gross").and_then(|v| v.parse::<i64>().ok()) else {
                continue;
            };
            let Some(issued) = get("invoice_date").and_then(|v| day(&v)) else {
                continue;
            };
            let due = get("due_date").and_then(|v| day(&v)).unwrap_or(issued);
            Document {
                id,
                kind,
                party,
                // An invoice is paid by money out.
                cents: -cents,
                number: get("invoice_number")
                    .filter(|number| alnum(number).len() >= parameters::MIN_NUMBER),
                words,
                from: plus(issued, -parameters::DAYS_BEFORE),
                until: plus(due.max(issued), parameters::DAYS_AFTER),
            }
        };
        documents.push(document);
    }
    Ok(documents)
}

/// A row of a booking as read: id, day, amount, party, purpose, other side.
type BookingRow = (i64, String, i64, Option<i64>, String, Option<String>);

/// A booking asked about: id, day, amount in cents, other side.
pub type Candidate = (i64, String, i64, Option<String>);

/// A booking the rules may link: not replaced, no move, not linked yet.
struct Booking {
    id: i64,
    day: Date,
    cents: i64,
    party: Option<i64>,
    purpose: String,
}

fn free_bookings(conn: &Connection) -> rusqlite::Result<Vec<Booking>> {
    let rows: Vec<BookingRow> = conn
        .prepare(
            "SELECT id, day, amount_cents, party_id, purpose, other_side FROM bookings b WHERE replaced_by IS NULL AND made_by IS NULL
             AND NOT EXISTS (SELECT 1 FROM links l WHERE l.booking_id = b.id AND l.undone_at IS NULL)
             AND NOT EXISTS (SELECT 1 FROM booking_pairs p WHERE (p.out_booking = b.id OR p.in_booking = b.id) AND p.undone_at IS NULL)",
        )?
        .query_map([], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?, row.get(4)?, row.get(5)?)))?
        .collect::<rusqlite::Result<_>>()?;
    Ok(rows
        .into_iter()
        .filter_map(|(id, on, cents, party, purpose, other)| {
            Some(Booking {
                id,
                day: day(&on)?,
                cents,
                party,
                purpose: format!("{} {purpose}", other.unwrap_or_default()),
            })
        })
        .collect())
}

fn link(
    conn: &Connection,
    document: i64,
    booking: i64,
    cents: i64,
    rule: &str,
    inputs: serde_json::Value,
    now: Timestamp,
) -> rusqlite::Result<()> {
    conn.execute(
        "INSERT INTO links (document_id, booking_id, cents, rule, inputs, decided_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        params![document, booking, cents, rule, inputs.to_string(), now.to_string()],
    )?;
    Ok(())
}

/// Runs the rules over every filed document without a link. Returns the links made and the
/// questions asked.
pub fn find(conn: &Connection, now: Timestamp) -> rusqlite::Result<(usize, usize)> {
    let documents = open_documents(conn)?;
    let mut bookings = free_bookings(conn)?;
    let mut linked: Vec<i64> = Vec::new();
    let mut used: Vec<i64> = Vec::new();
    let mut made = 0;
    // 1. Numbers in a purpose: the documents a booking names, when their amounts make its own.
    let mut by_booking: BTreeMap<i64, Vec<&Document>> = BTreeMap::new();
    for document in documents
        .iter()
        .filter(|document| document.number.is_some())
    {
        let number = alnum(document.number.as_deref().unwrap_or_default());
        for booking in &bookings {
            if alnum(&booking.purpose).contains(&number)
                && booking.day >= document.from
                && booking.day <= document.until
            {
                by_booking.entry(booking.id).or_default().push(document);
            }
        }
    }
    for (booking, named) in &by_booking {
        let Some(found) = bookings.iter().find(|candidate| candidate.id == *booking) else {
            continue;
        };
        let sum: i64 = named.iter().map(|document| document.cents).sum();
        if sum != found.cents || named.iter().any(|document| linked.contains(&document.id)) {
            continue;
        }
        for document in named {
            link(
                conn,
                document.id,
                *booking,
                document.cents,
                RULE_NUMBER,
                json!({ "number": document.number, "cents": document.cents, "day": found.day.to_string() }),
                now,
            )?;
            linked.push(document.id);
            made += 1;
        }
        used.push(*booking);
    }
    bookings.retain(|booking| !used.contains(&booking.id));
    // 2. The amount with the party in the days, or a payslip's Netto from its employer.
    let mut questions = 0;
    for document in documents
        .iter()
        .filter(|document| !linked.contains(&document.id))
    {
        let in_days: Vec<&Booking> = bookings
            .iter()
            .filter(|booking| {
                booking.cents == document.cents
                    && booking.day >= document.from
                    && booking.day <= document.until
            })
            .collect();
        let by_party: Vec<&Booking> = in_days
            .iter()
            .copied()
            .filter(|booking| document.party.is_some() && booking.party == document.party)
            .collect();
        let by_name: Vec<&Booking> = in_days
            .iter()
            .copied()
            .filter(|booking| {
                let text = alnum(&booking.purpose);
                document
                    .words
                    .iter()
                    .any(|word| text.contains(&alnum(word)))
            })
            .collect();
        let sure = match (by_party.as_slice(), by_name.as_slice()) {
            ([one], _) => Some((
                *one,
                if document.kind == Kind::Payslip {
                    RULE_PAYSLIP
                } else {
                    RULE_PARTY_AMOUNT
                },
            )),
            ([], [one]) => Some((*one, RULE_AMOUNT_NAME)),
            _ => None,
        };
        if let Some((one, rule)) = sure {
            let booking = one.id;
            link(
                conn,
                document.id,
                booking,
                document.cents,
                rule,
                json!({ "party": document.party, "cents": document.cents, "day": one.day.to_string() }),
                now,
            )?;
            bookings.retain(|candidate| candidate.id != booking);
            made += 1;
        } else if !in_days.is_empty() {
            // The amount fits in the days, but no rule is sure which booking it is: the owner says.
            let candidates: Vec<i64> = in_days
                .iter()
                .take(parameters::MAX_CANDIDATES)
                .map(|booking| booking.id)
                .collect();
            conn.execute(
                "INSERT INTO link_questions (document_id, candidates, asked_at) VALUES (?1, ?2, ?3)
                 ON CONFLICT (document_id) DO UPDATE SET candidates = excluded.candidates, asked_at = excluded.asked_at, answered_at = NULL",
                params![document.id, json!(candidates).to_string(), now.to_string()],
            )?;
            questions += 1;
        }
    }
    Ok((made, questions))
}

/// The owner's answer to a link question: the booking that paid it, or none.
pub fn answer(
    conn: &Connection,
    document: i64,
    booking: Option<i64>,
    now: Timestamp,
) -> rusqlite::Result<()> {
    let tx = conn.unchecked_transaction()?;
    if let Some(booking) = booking {
        let cents: i64 = tx.query_row(
            "SELECT amount_cents FROM bookings WHERE id = ?1",
            [booking],
            |row| row.get(0),
        )?;
        link(
            &tx,
            document,
            booking,
            cents,
            RULE_OWNER,
            json!({ "chosen": booking }),
            now,
        )?;
    }
    tx.execute(
        "UPDATE link_questions SET answered_at = ?2 WHERE document_id = ?1",
        params![document, now.to_string()],
    )?;
    tx.commit()?;
    Ok(())
}

/// The bookings asked about for a document, with their day, amount and other side.
pub fn question(conn: &Connection, document: i64) -> rusqlite::Result<Option<Vec<Candidate>>> {
    let candidates: Option<String> = conn
        .query_row(
            "SELECT candidates FROM link_questions WHERE document_id = ?1 AND answered_at IS NULL",
            [document],
            |row| row.get(0),
        )
        .optional()?;
    let Some(candidates) = candidates else {
        return Ok(None);
    };
    let ids: Vec<i64> = serde_json::from_str(&candidates).unwrap_or_default();
    let mut found = Vec::new();
    for id in ids {
        if let Some(row) = conn
            .query_row(
                "SELECT id, day, amount_cents, other_side FROM bookings WHERE id = ?1",
                [id],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
            )
            .optional()?
        {
            found.push(row);
        }
    }
    Ok(Some(found))
}

/// The documents with an open link question.
pub fn asked(conn: &Connection) -> rusqlite::Result<Vec<i64>> {
    conn.prepare("SELECT document_id FROM link_questions WHERE answered_at IS NULL ORDER BY asked_at, document_id")?
        .query_map([], |row| row.get(0))?
        .collect()
}

/// "Rückgängig": the link is no more, and the rules do not make it again.
pub fn undo(conn: &Connection, link: i64, now: Timestamp) -> rusqlite::Result<bool> {
    let changed = conn.execute(
        "UPDATE links SET undone_at = ?2 WHERE id = ?1 AND undone_at IS NULL",
        params![link, now.to_string()],
    )?;
    if changed > 0 {
        // The document is answered as unpaid, so the rules leave it alone.
        let document: i64 = conn.query_row(
            "SELECT document_id FROM links WHERE id = ?1",
            [link],
            |row| row.get(0),
        )?;
        conn.execute(
            "INSERT INTO link_questions (document_id, candidates, asked_at, answered_at) VALUES (?1, '[]', ?2, ?2)
             ON CONFLICT (document_id) DO UPDATE SET answered_at = excluded.answered_at",
            params![document, now.to_string()],
        )?;
    }
    Ok(changed > 0)
}

#[cfg(test)]
mod tests;
