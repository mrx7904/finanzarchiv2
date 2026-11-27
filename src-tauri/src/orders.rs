//! Amazon's orders (#32, FA-36): a mail of Amazon's own domain tells an order by its number, its
//! total and its day, and is kept as an order, not as a document. The owner downloads the invoice
//! (the app never signs in at Amazon, decided 2026-11-26); an invoice that names the order number
//! is matched to it, and to its booking by the number in the purpose (`links`).

use std::sync::LazyLock;

use jiff::Timestamp;
use regex::Regex;
use rusqlite::{Connection, params};

use crate::money::Cents;
use crate::readers::parse::{first_date, last_amount};

static DOMAIN: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)^([a-z0-9-]+\.)*amazon\.(de|com|at|co\.uk|fr|it|es|nl|pl|se|com\.be)$")
        .unwrap_or_else(|_| unreachable!("a fixed pattern"))
});

static NUMBER: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"\b\d{3}-\d{7}-\d{7}\b").unwrap_or_else(|_| unreachable!("a fixed pattern"))
});

pub const SHOP: &str = "Amazon";

/// Whether a sender, `Amazon.de <bestellbestaetigung@amazon.de>`, is one of Amazon's own domains.
pub fn from_amazon(from: &str) -> bool {
    let address = match (from.rfind('<'), from.rfind('>')) {
        (Some(open), Some(close)) if open < close => &from[open + 1..close],
        _ => from.trim(),
    };
    address
        .rsplit_once('@')
        .is_some_and(|(_, domain)| DOMAIN.is_match(domain.trim()))
}

/// An order a mail tells.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Order {
    pub number: String,
    /// ISO day.
    pub day: Option<String>,
    pub total: Option<Cents>,
}

/// The orders of the text of a mail of Amazon: each order number once, the total of the mail
/// where it names a single order, the day of the mail.
pub fn read(lines: &[String]) -> Vec<Order> {
    let day = lines
        .iter()
        .take(6)
        .find(|line| line.starts_with("Datum:"))
        .and_then(|line| first_date(line))
        .map(|(_, date)| date.to_string());
    let mut numbers: Vec<String> = Vec::new();
    for line in lines {
        for found in NUMBER.find_iter(line) {
            if !numbers.iter().any(|number| number == found.as_str()) {
                numbers.push(found.as_str().to_owned());
            }
        }
    }
    let total = (numbers.len() == 1)
        .then(|| {
            lines.iter().find_map(|line| {
                let lower = line.to_lowercase();
                ["gesamtbetrag", "bestellsumme", "gesamtsumme", "summe"]
                    .iter()
                    .any(|label| lower.starts_with(label))
                    .then(|| last_amount(line).map(|(_, cents)| cents))
                    .flatten()
            })
        })
        .flatten();
    numbers
        .into_iter()
        .map(|number| Order {
            number,
            day: day.clone(),
            total,
        })
        .collect()
}

/// Keeps the orders of a mail. An order known by its number gets what it lacked.
pub fn note(
    conn: &Connection,
    hash: &str,
    orders: &[Order],
    now: Timestamp,
) -> rusqlite::Result<()> {
    for order in orders {
        conn.execute(
            "INSERT INTO orders (shop, order_number, ordered_at, total_cents, mail_hash, found_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6)
             ON CONFLICT (order_number) DO UPDATE SET
                ordered_at = coalesce(orders.ordered_at, excluded.ordered_at),
                total_cents = coalesce(orders.total_cents, excluded.total_cents)",
            params![SHOP, order.number, order.day, order.total.map(|cents| cents.value()), hash, now.to_string()],
        )?;
    }
    conn.execute(
        "UPDATE mail_messages SET orders_read = 1 WHERE message_hash = ?1",
        [hash],
    )?;
    Ok(())
}

/// Matches the orders without an invoice to an invoice that names their number. Returns how many.
pub fn match_invoices(conn: &Connection) -> rusqlite::Result<usize> {
    conn.execute(
        "UPDATE orders SET document_id = (
            SELECT v.document_id FROM field_values v JOIN documents d ON d.id = v.document_id
            WHERE v.field = 'order_number' AND v.value = orders.order_number AND d.kind = 'invoice'
            ORDER BY v.document_id LIMIT 1)
         WHERE document_id IS NULL AND EXISTS (
            SELECT 1 FROM field_values v JOIN documents d ON d.id = v.document_id
            WHERE v.field = 'order_number' AND v.value = orders.order_number AND d.kind = 'invoice')",
        [],
    )
}

/// An order as "Bestellungen" shows it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Listed {
    pub id: i64,
    pub shop: String,
    pub number: String,
    pub day: Option<String>,
    pub total: Option<Cents>,
    /// The invoice and its name, once there is one.
    pub invoice: Option<(i64, String)>,
    /// The owner said it needs no invoice (#46).
    pub no_invoice: bool,
}

pub fn list(conn: &Connection) -> rusqlite::Result<Vec<Listed>> {
    conn.prepare(
        "SELECT o.id, o.shop, o.order_number, o.ordered_at, o.total_cents, o.document_id, coalesce(d.name, d.original_name), o.no_invoice_at IS NOT NULL
         FROM orders o LEFT JOIN documents d ON d.id = o.document_id
         ORDER BY coalesce(o.ordered_at, o.found_at) DESC, o.id DESC",
    )?
    .query_map([], |row| {
        let document: Option<i64> = row.get(5)?;
        let name: Option<String> = row.get(6)?;
        Ok(Listed {
            id: row.get(0)?,
            shop: row.get(1)?,
            number: row.get(2)?,
            day: row.get(3)?,
            total: row.get::<_, Option<i64>>(4)?.map(Cents::new),
            invoice: document.zip(name),
            no_invoice: row.get(7)?,
        })
    })?
    .collect()
}

/// "Braucht keine Rechnung" and its undoing (#46). An invoice linked later counts over it.
pub fn set_needs_invoice(
    conn: &Connection,
    order: i64,
    needs: bool,
    now: jiff::Timestamp,
) -> rusqlite::Result<bool> {
    let changed = if needs {
        conn.execute(
            "UPDATE orders SET no_invoice_at = NULL WHERE id = ?1",
            [order],
        )?
    } else {
        conn.execute(
            "UPDATE orders SET no_invoice_at = coalesce(no_invoice_at, ?2) WHERE id = ?1",
            rusqlite::params![order, now.to_string()],
        )?
    };
    Ok(changed > 0)
}

/// "Rechnung zuordnen …": the owner links an invoice to an order. It ends "Braucht keine Rechnung".
pub fn assign(conn: &Connection, order: i64, document: i64) -> rusqlite::Result<bool> {
    let changed = conn.execute(
        "UPDATE orders SET document_id = ?2, no_invoice_at = NULL WHERE id = ?1
         AND EXISTS (SELECT 1 FROM documents WHERE id = ?2 AND kind = 'invoice')",
        rusqlite::params![order, document],
    )?;
    Ok(changed > 0)
}

/// An invoice that may belong to an order: its name, gross and day.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Candidate {
    pub document: i64,
    pub name: String,
    pub gross: Option<Cents>,
    pub day: Option<String>,
}

/// The invoices no order holds, the likeliest for this order first: the same total, then the
/// nearest day, then the newest.
pub fn candidates(conn: &Connection, order: i64) -> rusqlite::Result<Vec<Candidate>> {
    let (total, ordered): (Option<i64>, Option<String>) = conn.query_row(
        "SELECT total_cents, ordered_at FROM orders WHERE id = ?1",
        [order],
        |row| Ok((row.get(0)?, row.get(1)?)),
    )?;
    let ids: Vec<(i64, String)> = conn
        .prepare(
            "SELECT d.id, coalesce(d.name, d.original_name) FROM documents d WHERE d.kind = 'invoice'
             AND NOT EXISTS (SELECT 1 FROM orders o WHERE o.document_id = d.id) ORDER BY d.id DESC",
        )?
        .query_map([], |row| Ok((row.get(0)?, row.get(1)?)))?
        .collect::<rusqlite::Result<_>>()?;
    let ordered = ordered.and_then(|day| {
        day.get(..10)
            .and_then(|day| day.parse::<jiff::civil::Date>().ok())
    });
    let mut found = Vec::new();
    for (document, name) in ids {
        let chosen = crate::checks::chosen(&crate::checks::values(conn, document)?);
        let gross = chosen
            .get("gross")
            .and_then(|seen| seen.value.parse::<i64>().ok())
            .map(Cents::new);
        let day = chosen.get("invoice_date").map(|seen| seen.value.clone());
        found.push(Candidate {
            document,
            name,
            gross,
            day,
        });
    }
    let distance = |candidate: &Candidate| -> i32 {
        let day = candidate
            .day
            .as_deref()
            .and_then(|day| day.parse::<jiff::civil::Date>().ok());
        match (ordered, day) {
            (Some(ordered), Some(day)) => (day - ordered).get_days().abs(),
            _ => i32::MAX,
        }
    };
    found.sort_by_key(|candidate| {
        (
            total.is_none() || candidate.gross.map(Cents::value) != total,
            distance(candidate),
        )
    });
    Ok(found)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn lines(texts: &[&str]) -> Vec<String> {
        texts.iter().map(|text| (*text).to_owned()).collect()
    }

    #[test]
    fn only_amazon_own_domains_tell_orders() {
        for own in [
            "Amazon.de <bestellbestaetigung@amazon.de>",
            "auto-confirm@amazon.com",
            "x@marketplace.amazon.de",
        ] {
            assert!(from_amazon(own), "{own}");
        }
        assert!(
            !from_amazon("Amazon.de <amazon@example.com>"),
            "a name is no domain"
        );
        for other in ["amazon.de.example.com", "notamazon.de", "amazon-versand.de"] {
            assert!(!DOMAIN.is_match(other), "{other}");
        }
    }

    #[test]
    fn a_mail_tells_its_orders_with_the_total_of_a_single_one() {
        let one = read(&lines(&[
            "Von: Amazon.de <bestellbestaetigung@amazon.de>",
            "Datum: 24.11.2026 09:30",
            "Betreff: Bestellt: „Beispiel Buch“",
            "",
            "Bestellnr. 302-1234567-7654321",
            "Gesamtbetrag 23,80 €",
        ]));
        assert_eq!(
            one,
            [Order {
                number: "302-1234567-7654321".into(),
                day: Some("2026-11-24".into()),
                total: Some(Cents::new(2380))
            }]
        );
        let two = read(&lines(&[
            "Datum: 24.11.2026 09:30",
            "Bestellnr. 302-1234567-7654321",
            "Bestellnr. 028-7654321-1234567",
            "Gesamtbetrag 40,00 €",
            "Bestellnr. 302-1234567-7654321",
        ]));
        assert_eq!(two.len(), 2, "each number once");
        assert!(
            two.iter().all(|order| order.total.is_none()),
            "a total of two orders is no order's"
        );
    }
}
