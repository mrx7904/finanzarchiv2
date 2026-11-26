//! Payments over PayPal (#32, FA-36). A PayPal mail stands for the payment until the seller's
//! invoice comes: same amount, within 10 days, the seller's name or the order number. Then it is
//! that invoice's receipt (`receipts_of`), and the booking it was linked to belongs to the invoice
//! as well. Two mails of one payment, the same transaction code, are one.

use std::path::Path;

use jiff::Timestamp;
use jiff::civil::Date;
use rusqlite::{Connection, OptionalExtension, params};
use serde_json::json;

use crate::readers::paypal::VIA;

pub const RULE_PAIR: &str = "receipt.seller-invoice:1";
pub const RULE_LINK: &str = "link.receipt:1";
/// The days between the payment and the seller's invoice, either way.
pub const DAYS: i64 = 10;

fn value(conn: &Connection, document: i64, field: &str) -> rusqlite::Result<Option<String>> {
    conn.query_row(
        "SELECT value FROM field_values WHERE document_id = ?1 AND field = ?2 ORDER BY id LIMIT 1",
        params![document, field],
        |row| row.get(0),
    )
    .optional()
}

fn day(text: Option<String>) -> Option<Date> {
    text?.parse().ok()
}

/// The documents of PayPal payments.
fn paypal_documents(conn: &Connection) -> rusqlite::Result<Vec<i64>> {
    conn.prepare(
        "SELECT DISTINCT document_id FROM field_values WHERE field = ?1 AND value = 'PayPal' ORDER BY document_id",
    )?
    .query_map([VIA], |row| row.get(0))?
    .collect()
}

fn names_match(payee: &str, sender: &str) -> bool {
    let (payee, sender) = (crate::parties::fold(payee), crate::parties::fold(sender));
    !payee.is_empty() && !sender.is_empty() && (payee.contains(&sender) || sender.contains(&payee))
}

/// Pairs every PayPal mail without its seller's invoice to the one invoice that fits. Returns how
/// many pairs it made.
pub fn pair(conn: &Connection, now: Timestamp) -> rusqlite::Result<usize> {
    let mut made = 0;
    for receipt in paypal_documents(conn)? {
        let paired: bool = conn
            .query_row(
                "SELECT 1 FROM receipts_of WHERE receipt_id = ?1",
                [receipt],
                |_| Ok(()),
            )
            .optional()?
            .is_some();
        if paired {
            continue;
        }
        let (Some(gross), Some(paid)) = (
            value(conn, receipt, "gross")?,
            day(value(conn, receipt, "invoice_date")?),
        ) else {
            continue;
        };
        let payee = value(conn, receipt, "sender")?.unwrap_or_default();
        let order = value(conn, receipt, "order_number")?;
        let invoices: Vec<i64> = conn
            .prepare(
                "SELECT DISTINCT v.document_id FROM field_values v JOIN documents d ON d.id = v.document_id
                 WHERE v.field = 'gross' AND v.value = ?1 AND d.kind = 'invoice' AND d.id <> ?2
                 AND NOT EXISTS (SELECT 1 FROM field_values p WHERE p.document_id = d.id AND p.field = ?3)",
            )?
            .query_map(params![gross, receipt, VIA], |row| row.get(0))?
            .collect::<rusqlite::Result<_>>()?;
        let mut fitting = Vec::new();
        for invoice in invoices {
            let Some(issued) = day(value(conn, invoice, "invoice_date")?) else {
                continue;
            };
            if (issued - paid).get_days().abs() > DAYS as i32 {
                continue;
            }
            let sender = value(conn, invoice, "sender")?.unwrap_or_default();
            let numbered = match &order {
                Some(order) => {
                    let numbers = [
                        value(conn, invoice, "order_number")?,
                        value(conn, invoice, "invoice_number")?,
                    ];
                    numbers.iter().flatten().any(|number| number == order)
                }
                None => false,
            };
            if numbered || names_match(&payee, &sender) {
                fitting.push(invoice);
            }
        }
        let [invoice] = fitting[..] else {
            continue;
        };
        conn.execute(
            "INSERT INTO receipts_of (receipt_id, invoice_id, rule, decided_at) VALUES (?1, ?2, ?3, ?4)",
            params![receipt, invoice, RULE_PAIR, now.to_string()],
        )?;
        // The booking the PayPal mail was linked to paid the invoice as well.
        let booking: Option<(i64, i64)> = conn
            .query_row(
                "SELECT booking_id, cents FROM links WHERE document_id = ?1 AND undone_at IS NULL LIMIT 1",
                [receipt],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .optional()?;
        let invoice_linked: bool = conn
            .query_row(
                "SELECT 1 FROM links WHERE document_id = ?1 AND undone_at IS NULL",
                [invoice],
                |_| Ok(()),
            )
            .optional()?
            .is_some();
        if let Some((booking, cents)) = booking
            && !invoice_linked
        {
            conn.execute(
                "INSERT INTO links (document_id, booking_id, cents, rule, inputs, decided_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                params![invoice, booking, cents, RULE_LINK, json!({ "receipt": receipt }).to_string(), now.to_string()],
            )?;
        }
        made += 1;
    }
    Ok(made)
}

/// The receipt and the invoice a document is part of, for the window: the other document and
/// whether this one is the receipt.
pub fn pair_of(conn: &Connection, document: i64) -> rusqlite::Result<Option<(i64, bool)>> {
    conn.query_row(
        "SELECT CASE WHEN receipt_id = ?1 THEN invoice_id ELSE receipt_id END, receipt_id = ?1
         FROM receipts_of WHERE (receipt_id = ?1 OR invoice_id = ?1) AND undone_at IS NULL LIMIT 1",
        [document],
        |row| Ok((row.get(0)?, row.get(1)?)),
    )
    .optional()
}

/// Two mails of one PayPal payment are one: of the documents with the same transaction code, the
/// first stays and the others go to the Papierkorb, unless the owner did something with them.
/// The one that stays is decided again. Returns how many went.
pub fn merge_twins(conn: &Connection, root: &Path, now: Timestamp) -> rusqlite::Result<usize> {
    let groups: Vec<(String, String)> = conn
        .prepare(
            "SELECT n.value, group_concat(n.document_id) FROM field_values n
             WHERE n.field = 'invoice_number' AND n.document_id IN (SELECT document_id FROM field_values WHERE field = ?1 AND value = 'PayPal')
             GROUP BY n.value HAVING count(DISTINCT n.document_id) > 1",
        )?
        .query_map([VIA], |row| Ok((row.get(0)?, row.get(1)?)))?
        .collect::<rusqlite::Result<_>>()?;
    let mut gone = 0;
    for (_, ids) in groups {
        let mut ids: Vec<i64> = ids.split(',').filter_map(|id| id.parse().ok()).collect();
        ids.sort_unstable();
        ids.dedup();
        let Some((&keep, twins)) = ids.split_first() else {
            continue;
        };
        for twin in twins {
            let touched: bool = conn
                .query_row(
                    "SELECT 1 FROM field_values WHERE document_id = ?1 AND reader = 'owner' LIMIT 1",
                    [twin],
                    |_| Ok(()),
                )
                .optional()?
                .is_some();
            if !touched && crate::documents::delete(conn, root, *twin).is_ok() {
                gone += 1;
            }
        }
        let waiting: bool = conn
            .query_row(
                "SELECT state = 'waiting' FROM documents WHERE id = ?1",
                [keep],
                |row| row.get(0),
            )
            .optional()?
            .unwrap_or(false);
        if waiting {
            crate::checks::decide(conn, keep, now)?;
        }
    }
    Ok(gone)
}

/// The waiting PayPal mails are decided again once their reader read them (#32), a held one is
/// left. Returns how many were filed.
pub fn recheck(conn: &Connection, now: Timestamp) -> rusqlite::Result<usize> {
    let waiting: Vec<i64> = conn
        .prepare(
            "SELECT d.id FROM documents d WHERE d.state = 'waiting'
             AND EXISTS (SELECT 1 FROM field_values v WHERE v.document_id = d.id AND v.field = ?1 AND v.value = 'PayPal')
             AND NOT EXISTS (SELECT 1 FROM field_values v WHERE v.document_id = d.id AND v.reader = 'owner' AND v.field = 'hold' AND v.value = '1')
             AND NOT EXISTS (SELECT 1 FROM intake_jobs j WHERE j.document_id = d.id AND j.stage NOT IN ('filed', 'waiting', 'failed'))",
        )?
        .query_map([VIA], |row| row.get(0))?
        .collect::<rusqlite::Result<_>>()?;
    let mut filed = 0;
    for document in waiting {
        if crate::checks::decide(conn, document, now)?.filed() {
            filed += 1;
        }
    }
    Ok(filed)
}

#[cfg(test)]
mod tests;
