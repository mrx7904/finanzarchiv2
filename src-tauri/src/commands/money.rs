//! The commands of "Konten" and "Umsätze" (#26): the accounts with their balance against the bank,
//! the bookings with their party, category, documents and moves, and the categories.

use std::sync::{Arc, Mutex};

use rusqlite::params;
use serde::{Deserialize, Serialize};
use specta::Type;
use tauri::State;

use super::CommandError;
use crate::archive::Opened;
use crate::money::Cents;

fn with_ready<T>(
    archive: &Mutex<Opened>,
    run: impl FnOnce(&rusqlite::Connection) -> Result<T, CommandError>,
) -> Result<T, CommandError> {
    let opened = archive.lock().map_err(|_| CommandError::internal())?;
    let Opened::Ready(ready) = &*opened else {
        return Err(CommandError::archive_not_ready());
    };
    run(ready.connection())
}

fn database(_: rusqlite::Error) -> CommandError {
    CommandError::internal()
}

fn id32(id: i64) -> Result<u32, CommandError> {
    u32::try_from(id).map_err(|_| CommandError::internal())
}

/// An account as "Konten" shows it.
#[derive(Debug, Clone, Serialize, Type, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct AccountRow {
    pub id: u32,
    pub name: String,
    pub iban: Option<String>,
    /// The balance after every booking read, as "1.234,56 €".
    pub balance: Option<String>,
    /// The bank's last printed balance and its day (ISO).
    pub bank_balance: Option<String>,
    pub bank_day: Option<String>,
    /// What does not match, in German.
    pub findings: Vec<String>,
}

#[tauri::command]
#[specta::specta]
pub fn list_accounts(
    archive: State<'_, Arc<Mutex<Opened>>>,
) -> Result<Vec<AccountRow>, CommandError> {
    with_ready(&archive, |conn| {
        // After every booking read, whatever day it is.
        let listed =
            crate::accounts::list(conn, jiff::civil::date(9999, 12, 31)).map_err(database)?;
        listed
            .into_iter()
            .map(|account| {
                Ok(AccountRow {
                    id: id32(account.id)?,
                    name: account.name,
                    iban: account.iban,
                    balance: account.balance.map(|cents| cents.to_string()),
                    bank_balance: account.bank_balance.map(|(_, cents)| cents.to_string()),
                    bank_day: account.bank_balance.map(|(day, _)| day.to_string()),
                    findings: account
                        .findings
                        .iter()
                        .map(crate::accounts::Finding::sentence)
                        .collect(),
                })
            })
            .collect()
    })
}

/// What "Umsätze" asks for: one account or all, a period, a category, a text in party or purpose.
#[derive(Debug, Clone, Deserialize, Type, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct BookingFilter {
    pub account: Option<u32>,
    /// ISO days, both included.
    pub from: Option<String>,
    pub to: Option<String>,
    pub category: Option<u32>,
    pub text: Option<String>,
    pub offset: u32,
    pub limit: u32,
}

/// A document linked to a booking, with the link's rule for "Warum?".
#[derive(Debug, Clone, Serialize, Type, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct LinkedDocument {
    pub link: u32,
    pub document: u32,
    pub name: String,
    pub why: String,
}

/// A move a booking is half of.
#[derive(Debug, Clone, Serialize, Type, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct MoveOf {
    pub pair: u32,
    pub why: String,
    /// The other account, by name.
    pub other: Option<String>,
    pub made: bool,
}

#[derive(Debug, Clone, Serialize, Type, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct BookingRow {
    pub id: u32,
    pub account: String,
    pub day: String,
    pub party: Option<String>,
    pub other_side: Option<String>,
    pub purpose: String,
    pub kind: String,
    /// Money in, not out. The amount itself is the formatted text, cents never cross as a float.
    pub incoming: bool,
    pub amount: String,
    pub category: Option<u32>,
    /// Whether the category is the party's, not the booking's own.
    pub category_by_party: bool,
    pub documents: Vec<LinkedDocument>,
    pub movement: Option<MoveOf>,
    /// The statement document it was read from, and the page of its line.
    pub statement: Option<u32>,
    pub page: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Type, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct BookingPage {
    pub rows: Vec<BookingRow>,
    pub total: u32,
    /// Money in and out of the bookings found, moves left out.
    pub income: String,
    pub spending: String,
}

/// A booking as the query reads it: id, account, day, party, other side, purpose, kind, cents, own
/// category, the party's category, statement document, page.
type RawBooking = (
    i64,
    String,
    String,
    Option<String>,
    Option<String>,
    String,
    String,
    i64,
    Option<i64>,
    Option<i64>,
    Option<i64>,
    Option<i64>,
);

/// The German sentence of a rule, for "Warum?".
fn why(rule: &str) -> String {
    let name = rule.split(':').next().unwrap_or(rule);
    match name {
        "link.number" => "Die Rechnungsnummer steht im Verwendungszweck, der Betrag stimmt.",
        "link.party-amount" => "Betrag und Partei stimmen, im Zeitraum der Rechnung.",
        "link.amount-name" => "Der Betrag stimmt, und der Name des Absenders steht in der Buchung.",
        "link.payslip" => "Das Netto der Gehaltsabrechnung kam vom Arbeitgeber in ihrem Monat.",
        "link.owner" => "Du hast diese Buchung zugeordnet.",
        "move.own-iban" => "Die Buchung nennt die IBAN eines eigenen Kontos.",
        "move.marked" => "Die Bank nennt sie eine Umbuchung, die Gegenbuchung steht auf einem eigenen Konto.",
        "move.same-day" => "Derselbe Betrag ging am selben Tag von einem eigenen Konto ab und auf einem anderen ein.",
        _ => "Eine Regel des Archivs.",
    }
    .to_owned()
}

#[tauri::command]
#[specta::specta]
pub fn list_bookings(
    archive: State<'_, Arc<Mutex<Opened>>>,
    filter: BookingFilter,
) -> Result<BookingPage, CommandError> {
    with_ready(&archive, |conn| {
        let text = filter
            .text
            .as_deref()
            .map(str::trim)
            .filter(|text| !text.is_empty())
            .map(|text| format!("%{}%", text.replace('%', "")));
        let condition = "b.replaced_by IS NULL
             AND (?1 IS NULL OR b.account_id = ?1)
             AND (?2 IS NULL OR b.day >= ?2) AND (?3 IS NULL OR b.day <= ?3)
             AND (?4 IS NULL OR coalesce(b.category_id, p.category_id) = ?4)
             AND (?5 IS NULL OR b.other_side LIKE ?5 OR b.purpose LIKE ?5 OR p.name LIKE ?5)";
        let args = params![
            filter.account,
            filter.from,
            filter.to,
            filter.category,
            text
        ];
        let (total, income, spending): (u32, i64, i64) = conn
            .query_row(
                &format!(
                    "SELECT count(*),
                        coalesce(sum(CASE WHEN b.amount_cents > 0 AND NOT EXISTS (SELECT 1 FROM booking_pairs m WHERE (m.out_booking = b.id OR m.in_booking = b.id) AND m.undone_at IS NULL) THEN b.amount_cents END), 0),
                        coalesce(sum(CASE WHEN b.amount_cents < 0 AND NOT EXISTS (SELECT 1 FROM booking_pairs m WHERE (m.out_booking = b.id OR m.in_booking = b.id) AND m.undone_at IS NULL) THEN b.amount_cents END), 0)
                     FROM bookings b LEFT JOIN parties p ON p.id = b.party_id WHERE {condition}"
                ),
                args,
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
            )
            .map_err(database)?;
        let limit = filter.limit.clamp(1, 500);
        let rows: Vec<RawBooking> = conn
            .prepare(&format!(
                "SELECT b.id, a.name, b.day, p.name, b.other_side, b.purpose, b.kind, b.amount_cents, b.category_id, p.category_id, s.document_id, b.page
                 FROM bookings b JOIN accounts a ON a.id = b.account_id LEFT JOIN parties p ON p.id = b.party_id
                 LEFT JOIN statements s ON s.id = b.statement_id
                 WHERE {condition} ORDER BY b.day DESC, b.id DESC LIMIT {limit} OFFSET {}",
                filter.offset
            ))
            .map_err(database)?
            .query_map(args, |row| {
                Ok((
                    row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?, row.get(4)?, row.get(5)?,
                    row.get(6)?, row.get(7)?, row.get(8)?, row.get(9)?, row.get(10)?, row.get(11)?,
                ))
            })
            .map_err(database)?
            .collect::<rusqlite::Result<_>>()
            .map_err(database)?;
        let mut out = Vec::new();
        for (
            id,
            account,
            day,
            party,
            other_side,
            purpose,
            kind,
            cents,
            own_category,
            party_category,
            statement,
            page,
        ) in rows
        {
            let documents = conn
                .prepare(
                    "SELECT l.id, l.document_id, coalesce(d.name, d.original_name), l.rule FROM links l JOIN documents d ON d.id = l.document_id
                     WHERE l.booking_id = ?1 AND l.undone_at IS NULL ORDER BY l.id",
                )
                .map_err(database)?
                .query_map([id], |row| Ok((row.get::<_, i64>(0)?, row.get::<_, i64>(1)?, row.get::<_, String>(2)?, row.get::<_, String>(3)?)))
                .map_err(database)?
                .collect::<rusqlite::Result<Vec<_>>>()
                .map_err(database)?
                .into_iter()
                .map(|(link, document, name, rule)| Ok(LinkedDocument { link: id32(link)?, document: id32(document)?, name, why: why(&rule) }))
                .collect::<Result<Vec<_>, CommandError>>()?;
            let movement = conn
                .query_row(
                    "SELECT m.id, m.rule, (SELECT a.name FROM bookings o JOIN accounts a ON a.id = o.account_id
                        WHERE o.id = CASE WHEN m.out_booking = ?1 THEN m.in_booking ELSE m.out_booking END),
                        (SELECT made_by IS NOT NULL FROM bookings o WHERE o.id = CASE WHEN m.out_booking = ?1 THEN m.in_booking ELSE m.out_booking END)
                     FROM booking_pairs m WHERE (m.out_booking = ?1 OR m.in_booking = ?1) AND m.undone_at IS NULL",
                    [id],
                    |row| Ok((row.get::<_, i64>(0)?, row.get::<_, String>(1)?, row.get::<_, Option<String>>(2)?, row.get::<_, Option<bool>>(3)?)),
                )
                .ok()
                .map(|(pair, rule, other, made)| Ok::<_, CommandError>(MoveOf { pair: id32(pair)?, why: why(&rule), other, made: made.unwrap_or(false) }))
                .transpose()?;
            out.push(BookingRow {
                id: id32(id)?,
                account,
                day,
                party,
                other_side,
                purpose,
                kind,
                incoming: cents > 0,
                amount: Cents::new(cents).to_string(),
                category: own_category.or(party_category).map(id32).transpose()?,
                category_by_party: own_category.is_none() && party_category.is_some(),
                documents,
                movement,
                statement: statement.map(id32).transpose()?,
                page: page.and_then(|page| u32::try_from(page).ok()),
            });
        }
        Ok(BookingPage {
            rows: out,
            total,
            income: Cents::new(income).to_string(),
            spending: Cents::new(spending).to_string(),
        })
    })
}

#[derive(Debug, Clone, Serialize, Type, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct CategoryRow {
    pub id: u32,
    pub name: String,
    /// "in" or "out".
    pub direction: String,
}

fn categories_of(conn: &rusqlite::Connection) -> Result<Vec<CategoryRow>, CommandError> {
    conn.prepare("SELECT id, name, direction FROM categories ORDER BY direction, position, id")
        .map_err(database)?
        .query_map([], |row| {
            Ok((row.get::<_, i64>(0)?, row.get(1)?, row.get(2)?))
        })
        .map_err(database)?
        .map(|row| {
            let (id, name, direction) = row.map_err(database)?;
            Ok(CategoryRow {
                id: id32(id)?,
                name,
                direction,
            })
        })
        .collect()
}

fn refused(error: crate::parties::CategoryError) -> CommandError {
    match error {
        crate::parties::CategoryError::Database(_) => CommandError::internal(),
        other => CommandError::refused(other.to_string()),
    }
}

#[tauri::command]
#[specta::specta]
pub fn list_categories(
    archive: State<'_, Arc<Mutex<Opened>>>,
) -> Result<Vec<CategoryRow>, CommandError> {
    with_ready(&archive, categories_of)
}

#[tauri::command]
#[specta::specta]
pub fn add_category(
    archive: State<'_, Arc<Mutex<Opened>>>,
    name: String,
    direction: String,
) -> Result<Vec<CategoryRow>, CommandError> {
    with_ready(&archive, |conn| {
        crate::parties::add_category(conn, &name, &direction).map_err(refused)?;
        categories_of(conn)
    })
}

#[tauri::command]
#[specta::specta]
pub fn rename_category(
    archive: State<'_, Arc<Mutex<Opened>>>,
    category: u32,
    name: String,
) -> Result<Vec<CategoryRow>, CommandError> {
    with_ready(&archive, |conn| {
        crate::parties::rename_category(conn, i64::from(category), &name).map_err(refused)?;
        categories_of(conn)
    })
}

#[tauri::command]
#[specta::specta]
pub fn remove_category(
    archive: State<'_, Arc<Mutex<Opened>>>,
    category: u32,
) -> Result<Vec<CategoryRow>, CommandError> {
    with_ready(&archive, |conn| {
        crate::parties::remove_category(conn, i64::from(category)).map_err(refused)?;
        categories_of(conn)
    })
}

/// The owner's category for a booking, or with `for_party` for all of its party ("Für alle von …").
#[tauri::command]
#[specta::specta]
pub fn choose_category(
    archive: State<'_, Arc<Mutex<Opened>>>,
    booking: u32,
    category: Option<u32>,
    for_party: bool,
) -> Result<(), CommandError> {
    with_ready(&archive, |conn| {
        crate::parties::choose_category(
            conn,
            i64::from(booking),
            category.map(i64::from),
            for_party,
            jiff::Timestamp::now(),
        )
        .map_err(database)
    })
}

/// "Rückgängig" for a link of a document to a booking.
#[tauri::command]
#[specta::specta]
pub fn undo_link(archive: State<'_, Arc<Mutex<Opened>>>, link: u32) -> Result<(), CommandError> {
    with_ready(&archive, |conn| {
        crate::links::undo(conn, i64::from(link), jiff::Timestamp::now())
            .map(|_| ())
            .map_err(database)
    })
}

/// "Rückgängig" for a move between own accounts.
#[tauri::command]
#[specta::specta]
pub fn undo_move(archive: State<'_, Arc<Mutex<Opened>>>, pair: u32) -> Result<(), CommandError> {
    with_ready(&archive, |conn| {
        crate::moves::undo(conn, i64::from(pair), jiff::Timestamp::now())
            .map(|_| ())
            .map_err(database)
    })
}
