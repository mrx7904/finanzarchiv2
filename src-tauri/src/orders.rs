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
}

pub fn list(conn: &Connection) -> rusqlite::Result<Vec<Listed>> {
    conn.prepare(
        "SELECT o.id, o.shop, o.order_number, o.ordered_at, o.total_cents, o.document_id, coalesce(d.name, d.original_name)
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
        })
    })?
    .collect()
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
