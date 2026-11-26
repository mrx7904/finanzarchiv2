//! Parties and their categories (FA-30, #25): who money goes to or comes from is one party however
//! it is spelled, and the party carries its category (knowledge/decision-parties.md).

use jiff::Timestamp;
use rusqlite::{Connection, OptionalExtension, params};

/// The categories a new archive starts with, chosen for everyday money (the owner asked for 20 to
/// 30 that can be changed and extended in the app, 2026-11-26). Money in first.
pub const DEFAULT_CATEGORIES: &[(&str, &str)] = &[
    ("Gehalt und Lohn", "in"),
    ("Rente und Sozialleistungen", "in"),
    ("Erstattungen", "in"),
    ("Zinsen und Erträge", "in"),
    ("Sonstige Einnahmen", "in"),
    ("Miete und Wohnen", "out"),
    ("Strom und Heizung", "out"),
    ("Internet und Telefon", "out"),
    ("Versicherungen", "out"),
    ("Lebensmittel", "out"),
    ("Drogerie und Haushalt", "out"),
    ("Restaurant und Café", "out"),
    ("Auto und Tanken", "out"),
    ("Bus und Bahn", "out"),
    ("Gesundheit und Apotheke", "out"),
    ("Kleidung", "out"),
    ("Freizeit und Hobby", "out"),
    ("Reisen und Urlaub", "out"),
    ("Abos und Medien", "out"),
    ("Bildung", "out"),
    ("Kinder", "out"),
    ("Geschenke und Spenden", "out"),
    ("Steuern und Abgaben", "out"),
    ("Bankgebühren", "out"),
    ("Kredit und Tilgung", "out"),
    ("Bargeld", "out"),
    ("Sparen und Anlegen", "out"),
    ("Sonstige Ausgaben", "out"),
];

/// Fills the categories once, when the archive has none.
pub fn seed_categories(conn: &Connection) -> rusqlite::Result<usize> {
    // An archive whose migrations stopped before #25 has no table to fill.
    let table: i64 = conn.query_row(
        "SELECT count(*) FROM sqlite_master WHERE type = 'table' AND name = 'categories'",
        [],
        |row| row.get(0),
    )?;
    if table == 0 {
        return Ok(0);
    }
    let have: i64 = conn.query_row("SELECT count(*) FROM categories", [], |row| row.get(0))?;
    if have > 0 {
        return Ok(0);
    }
    for (position, (name, direction)) in DEFAULT_CATEGORIES.iter().enumerate() {
        conn.execute(
            "INSERT INTO categories (name, direction, position) VALUES (?1, ?2, ?3)",
            params![name, direction, i64::try_from(position).unwrap_or(i64::MAX)],
        )?;
    }
    Ok(DEFAULT_CATEGORIES.len())
}

/// Why a change of the categories was not taken. German, for the owner.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum CategoryError {
    #[error("Eine Kategorie braucht einen Namen.")]
    Empty,
    #[error("Die Kategorie „{0}“ gibt es schon.")]
    Taken(String),
    #[error(transparent)]
    Database(#[from] DatabaseError),
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("Datenbank")]
pub struct DatabaseError;

impl From<rusqlite::Error> for CategoryError {
    fn from(_: rusqlite::Error) -> Self {
        Self::Database(DatabaseError)
    }
}

fn clean_name(name: &str) -> Result<String, CategoryError> {
    let name = name.split_whitespace().collect::<Vec<_>>().join(" ");
    if name.is_empty() {
        return Err(CategoryError::Empty);
    }
    Ok(name)
}

fn taken(conn: &Connection, name: &str, other_than: Option<i64>) -> rusqlite::Result<bool> {
    Ok(conn
        .query_row(
            "SELECT id FROM categories WHERE lower(name) = lower(?1)",
            [name],
            |row| row.get::<_, i64>(0),
        )
        .optional()?
        .is_some_and(|id| Some(id) != other_than))
}

/// Adds a category at the end of its direction.
pub fn add_category(conn: &Connection, name: &str, direction: &str) -> Result<i64, CategoryError> {
    let name = clean_name(name)?;
    if taken(conn, &name, None)? {
        return Err(CategoryError::Taken(name));
    }
    let direction = if direction == "in" { "in" } else { "out" };
    conn.execute(
        "INSERT INTO categories (name, direction, position) VALUES (?1, ?2, (SELECT coalesce(max(position), 0) + 1 FROM categories))",
        params![name, direction],
    )?;
    Ok(conn.last_insert_rowid())
}

pub fn rename_category(conn: &Connection, id: i64, name: &str) -> Result<(), CategoryError> {
    let name = clean_name(name)?;
    if taken(conn, &name, Some(id))? {
        return Err(CategoryError::Taken(name));
    }
    conn.execute(
        "UPDATE categories SET name = ?2 WHERE id = ?1",
        params![id, name],
    )?;
    Ok(())
}

/// Removes a category. Parties and bookings that carried it carry none, the owner's choices for it
/// stay in the corrections.
pub fn remove_category(conn: &Connection, id: i64) -> Result<(), CategoryError> {
    let tx = conn.unchecked_transaction()?;
    tx.execute(
        "UPDATE parties SET category_id = NULL WHERE category_id = ?1",
        [id],
    )?;
    tx.execute(
        "UPDATE bookings SET category_id = NULL WHERE category_id = ?1",
        [id],
    )?;
    tx.execute(
        "UPDATE corrections SET category_id = NULL WHERE category_id = ?1",
        [id],
    )?;
    tx.execute("DELETE FROM categories WHERE id = ?1", [id])?;
    tx.commit()?;
    Ok(())
}

/// The spelling of a party folded for matching: lower case, the legal form, spaces and signs left out.
pub fn fold(spelling: &str) -> String {
    crate::naming::without_legal_form(spelling)
        .to_lowercase()
        .chars()
        .filter(|c| c.is_alphanumeric())
        .collect()
}

/// The party of a spelling: by its folded spelling, else by the IBAN beside it, else a new one.
pub fn party_of(
    conn: &Connection,
    spelling: &str,
    iban: Option<&str>,
    now: Timestamp,
) -> rusqlite::Result<Option<i64>> {
    let folded = fold(spelling);
    if folded.is_empty() {
        return Ok(None);
    }
    if let Some(id) = conn
        .query_row(
            "SELECT party_id FROM party_aliases WHERE folded = ?1",
            [&folded],
            |row| row.get(0),
        )
        .optional()?
    {
        return Ok(Some(id));
    }
    let by_iban: Option<i64> = match iban {
        Some(iban) => conn
            .query_row(
                "SELECT party_id FROM party_aliases WHERE iban = ?1 LIMIT 1",
                [iban],
                |row| row.get(0),
            )
            .optional()?,
        None => None,
    };
    let party = match by_iban {
        Some(party) => party,
        None => {
            let name = crate::naming::without_legal_form(spelling);
            conn.execute(
                "INSERT INTO parties (name, created_at) VALUES (?1, ?2)",
                params![name.trim(), now.to_string()],
            )?;
            conn.last_insert_rowid()
        }
    };
    conn.execute(
        "INSERT INTO party_aliases (party_id, folded, spelling, iban) VALUES (?1, ?2, ?3, ?4)",
        params![party, folded, spelling.trim(), iban],
    )?;
    Ok(Some(party))
}

/// The IBAN of the other side a booking names in its purpose, when its check sum holds.
fn iban_in(purpose: &str) -> Option<String> {
    crate::readers::parse::iban(purpose)
        .map(|found| found.chars().filter(|c| !c.is_whitespace()).collect())
}

/// Gives every booking without a party, and every filed document without one, its party. Returns
/// how many got one.
pub fn assign(conn: &Connection, now: Timestamp) -> rusqlite::Result<usize> {
    // Runs in the transaction of its caller: the stage that read or filed the document.
    let tx = conn;
    let bookings: Vec<(i64, String, String)> = tx
        .prepare("SELECT id, other_side, purpose FROM bookings WHERE party_id IS NULL AND other_side IS NOT NULL")?
        .query_map([], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)))?
        .collect::<rusqlite::Result<_>>()?;
    let mut assigned = 0;
    for (id, other_side, purpose) in bookings {
        if let Some(party) = party_of(tx, &other_side, iban_in(&purpose).as_deref(), now)? {
            tx.execute(
                "UPDATE bookings SET party_id = ?2 WHERE id = ?1",
                params![id, party],
            )?;
            assigned += 1;
        }
    }
    let documents: Vec<i64> = tx
        .prepare("SELECT id FROM documents WHERE party_id IS NULL AND kind IN ('invoice', 'receipt', 'payslip')")?
        .query_map([], |row| row.get(0))?
        .collect::<rusqlite::Result<_>>()?;
    for document in documents {
        let chosen = crate::checks::chosen(&crate::checks::values(tx, document)?);
        let Some(sender) = chosen.get("sender").or_else(|| chosen.get("employer")) else {
            continue;
        };
        let iban = chosen.get("iban").map(|seen| seen.value.clone());
        if let Some(party) = party_of(tx, &sender.value, iban.as_deref(), now)? {
            tx.execute(
                "UPDATE documents SET party_id = ?2 WHERE id = ?1",
                params![document, party],
            )?;
            assigned += 1;
        }
    }
    Ok(assigned)
}

/// The category of a booking: its own, else its party's.
pub fn category_of(conn: &Connection, booking: i64) -> rusqlite::Result<Option<i64>> {
    conn.query_row(
        "SELECT coalesce(b.category_id, p.category_id) FROM bookings b LEFT JOIN parties p ON p.id = b.party_id WHERE b.id = ?1",
        [booking],
        |row| row.get(0),
    )
}

/// The owner's category for one booking, or with `for_party` for all of its party ("Für alle von …").
pub fn choose_category(
    conn: &Connection,
    booking: i64,
    category: Option<i64>,
    for_party: bool,
    now: Timestamp,
) -> rusqlite::Result<()> {
    let tx = conn.unchecked_transaction()?;
    let party: Option<i64> = tx.query_row(
        "SELECT party_id FROM bookings WHERE id = ?1",
        [booking],
        |row| row.get(0),
    )?;
    match (for_party, party) {
        (true, Some(party)) => {
            tx.execute(
                "UPDATE parties SET category_id = ?2 WHERE id = ?1",
                params![party, category],
            )?;
            // The booking follows its party from now on.
            tx.execute(
                "UPDATE bookings SET category_id = NULL WHERE id = ?1",
                [booking],
            )?;
            tx.execute(
                "INSERT INTO corrections (kind, party_id, booking_id, category_id, made_at) VALUES ('party-category', ?1, ?2, ?3, ?4)",
                params![party, booking, category, now.to_string()],
            )?;
        }
        _ => {
            tx.execute(
                "UPDATE bookings SET category_id = ?2 WHERE id = ?1",
                params![booking, category],
            )?;
            tx.execute(
                "INSERT INTO corrections (kind, party_id, booking_id, category_id, made_at) VALUES ('booking-category', ?1, ?2, ?3, ?4)",
                params![party, booking, category, now.to_string()],
            )?;
        }
    }
    tx.commit()?;
    Ok(())
}

/// Merges `from` into `into`: its spellings, bookings and documents go over, and the change is a
/// correction that `undo` takes back. Returns the id of the correction.
pub fn merge(conn: &Connection, from: i64, into: i64, now: Timestamp) -> rusqlite::Result<i64> {
    let tx = conn.unchecked_transaction()?;
    let ids = |sql: &str| -> rusqlite::Result<Vec<i64>> {
        tx.prepare(sql)?
            .query_map([from], |row| row.get(0))?
            .collect()
    };
    let aliases = ids("SELECT id FROM party_aliases WHERE party_id = ?1")?;
    let bookings = ids("SELECT id FROM bookings WHERE party_id = ?1")?;
    let documents = ids("SELECT id FROM documents WHERE party_id = ?1")?;
    let category: Option<i64> = tx.query_row(
        "SELECT category_id FROM parties WHERE id = ?1",
        [from],
        |row| row.get(0),
    )?;
    tx.execute(
        "UPDATE party_aliases SET party_id = ?2 WHERE party_id = ?1",
        params![from, into],
    )?;
    tx.execute(
        "UPDATE bookings SET party_id = ?2 WHERE party_id = ?1",
        params![from, into],
    )?;
    tx.execute(
        "UPDATE documents SET party_id = ?2 WHERE party_id = ?1",
        params![from, into],
    )?;
    let detail = serde_json::json!({ "from": from, "into": into, "aliases": aliases, "bookings": bookings, "documents": documents, "category": category });
    tx.execute(
        "INSERT INTO corrections (kind, party_id, detail, made_at) VALUES ('merge', ?1, ?2, ?3)",
        params![into, detail.to_string(), now.to_string()],
    )?;
    let id = tx.last_insert_rowid();
    tx.commit()?;
    Ok(id)
}

/// Splits one spelling off its party into a party of its own, with the bookings of that spelling.
pub fn split(conn: &Connection, alias: i64, now: Timestamp) -> rusqlite::Result<i64> {
    let tx = conn.unchecked_transaction()?;
    let (from, spelling, folded): (i64, String, String) = tx.query_row(
        "SELECT party_id, spelling, folded FROM party_aliases WHERE id = ?1",
        [alias],
        |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
    )?;
    tx.execute(
        "INSERT INTO parties (name, created_at) VALUES (?1, ?2)",
        params![
            crate::naming::without_legal_form(&spelling),
            now.to_string()
        ],
    )?;
    let into = tx.last_insert_rowid();
    tx.execute(
        "UPDATE party_aliases SET party_id = ?2 WHERE id = ?1",
        params![alias, into],
    )?;
    let bookings: Vec<i64> = tx
        .prepare("SELECT id, other_side FROM bookings WHERE party_id = ?1")?
        .query_map([from], |row| {
            Ok((row.get::<_, i64>(0)?, row.get::<_, Option<String>>(1)?))
        })?
        .filter_map(|row| match row {
            Ok((id, Some(other))) if fold(&other) == folded => Some(Ok(id)),
            Ok(_) => None,
            Err(error) => Some(Err(error)),
        })
        .collect::<rusqlite::Result<_>>()?;
    for booking in &bookings {
        tx.execute(
            "UPDATE bookings SET party_id = ?2 WHERE id = ?1",
            params![booking, into],
        )?;
    }
    let detail = serde_json::json!({ "from": from, "into": into, "aliases": [alias], "bookings": bookings, "documents": [] });
    tx.execute(
        "INSERT INTO corrections (kind, party_id, detail, made_at) VALUES ('split', ?1, ?2, ?3)",
        params![into, detail.to_string(), now.to_string()],
    )?;
    let id = tx.last_insert_rowid();
    tx.commit()?;
    Ok(id)
}

/// Takes a merge or split back: what moved goes back where it came from.
pub fn undo(conn: &Connection, correction: i64, now: Timestamp) -> rusqlite::Result<bool> {
    let tx = conn.unchecked_transaction()?;
    let row: Option<(String, String)> = tx
        .query_row(
            "SELECT kind, detail FROM corrections WHERE id = ?1 AND undone_at IS NULL AND kind IN ('merge', 'split')",
            [correction],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .optional()?;
    let Some((_, detail)) = row else {
        return Ok(false);
    };
    let detail: serde_json::Value = serde_json::from_str(&detail).unwrap_or_default();
    let from = detail["from"].as_i64().unwrap_or_default();
    let list = |key: &str| -> Vec<i64> {
        detail[key]
            .as_array()
            .map(|items| items.iter().filter_map(serde_json::Value::as_i64).collect())
            .unwrap_or_default()
    };
    for (table, key) in [
        ("party_aliases", "aliases"),
        ("bookings", "bookings"),
        ("documents", "documents"),
    ] {
        for id in list(key) {
            tx.execute(
                &format!("UPDATE {table} SET party_id = ?2 WHERE id = ?1"),
                params![id, from],
            )?;
        }
    }
    tx.execute(
        "UPDATE corrections SET undone_at = ?2 WHERE id = ?1",
        params![correction, now.to_string()],
    )?;
    tx.commit()?;
    Ok(true)
}

#[cfg(test)]
mod tests;
