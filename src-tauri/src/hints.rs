//! Savings hints (#39, FA-42): what the archive observed that may save money, by code, each with
//! its numbers and the bookings behind it. A hint says what it saw, never what to buy, sell or
//! invest, and none is about investments, loans or insurance cover. The owner can dismiss a hint,
//! and it stays dismissed until its numbers change (knowledge/decision-hints.md).

use std::collections::BTreeMap;

use jiff::Timestamp;
use jiff::civil::Date;
use rusqlite::{Connection, OptionalExtension, params};

use crate::money::Cents;
use crate::recurring::{self, Recurring};

/// Categories no hint speaks of: investments, loans and insurance cover are no matter of a hint,
/// cash says nothing of what it bought.
const NEVER: &[&str] = &[
    "Versicherungen",
    "Kredit und Tilgung",
    "Sparen und Anlegen",
    "Bargeld",
];

/// Categories where two running payments at once are worth a look (two streaming services).
const TWICE: &[&str] = &[
    "Abos und Medien",
    "Internet und Telefon",
    "Freizeit und Hobby",
];

/// The category of bank fees.
const FEES: &str = "Bankgebühren";

/// A category is clearly above the year before: a quarter more and at least 100 €.
const ABOVE_SHARE: i64 = 4;
const ABOVE_CENTS: i64 = 10_000;

/// A word of a letter that cancels.
static CANCELLED: std::sync::LazyLock<regex::Regex> = std::sync::LazyLock::new(|| {
    regex::Regex::new(r"(?i)\bkündigung").unwrap_or_else(|_| unreachable!("a fixed pattern"))
});

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Kind {
    /// A recurring payment whose amount rose.
    Rose,
    /// Two or more running payments of one category.
    Twice,
    /// The fees of banks in a year.
    Fees,
    /// A category clearly above the same months of the year before.
    Above,
    /// A recurring payment that still runs after a letter of its party that cancels.
    StillRuns,
}

impl Kind {
    pub fn code(self) -> &'static str {
        match self {
            Kind::Rose => "rose",
            Kind::Twice => "twice",
            Kind::Fees => "fees",
            Kind::Above => "above",
            Kind::StillRuns => "still-runs",
        }
    }
}

/// One hint: what it is about, one short line of what was seen, the figure, and its sources.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Hint {
    pub kind: Kind,
    /// Names what the hint is about, so a dismissal finds it again.
    pub key: String,
    /// The numbers it rests on: when they change, a dismissed hint comes back.
    pub numbers: String,
    /// A party or a category.
    pub subject: String,
    /// What was seen, short: "seit April 2026 7,00 € mehr".
    pub line: String,
    /// The figure of the hint, as money out is booked (negative).
    pub cents: i64,
    pub bookings: Vec<i64>,
    pub documents: Vec<i64>,
}

const MONTHS: [&str; 12] = [
    "Januar",
    "Februar",
    "März",
    "April",
    "Mai",
    "Juni",
    "Juli",
    "August",
    "September",
    "Oktober",
    "November",
    "Dezember",
];

fn month_name(day: Date) -> &'static str {
    MONTHS[usize::from(day.month().unsigned_abs()) - 1]
}

/// The category of each party, by name.
fn party_categories(conn: &Connection) -> rusqlite::Result<BTreeMap<i64, String>> {
    conn.prepare("SELECT p.id, c.name FROM parties p JOIN categories c ON c.id = p.category_id")?
        .query_map([], |row| Ok((row.get(0)?, row.get(1)?)))?
        .collect()
}

fn rose(running: &[&Recurring], categories: &BTreeMap<i64, String>) -> Vec<Hint> {
    running
        .iter()
        .filter(|one| !one.incoming)
        .filter(|one| {
            !categories
                .get(&one.party)
                .is_some_and(|name| NEVER.contains(&name.as_str()))
        })
        .filter_map(|one| {
            let change = one.change?;
            (change.more_cents > 0).then(|| Hint {
                kind: Kind::Rose,
                key: format!("rose:{}:{}", one.party, one.since),
                numbers: format!("{}:{}", change.day, change.more_cents),
                subject: one.name.clone(),
                line: format!(
                    "seit {} {} {} mehr, {} im Jahr",
                    month_name(change.day),
                    change.day.year(),
                    Cents::new(change.more_cents),
                    Cents::new(one.per_year.abs())
                ),
                cents: -change.more_cents * (12 / one.rhythm.months()),
                bookings: one.bookings.clone(),
                documents: Vec::new(),
            })
        })
        .collect()
}

fn twice(running: &[&Recurring], categories: &BTreeMap<i64, String>) -> Vec<Hint> {
    let mut by_category: BTreeMap<&str, Vec<&Recurring>> = BTreeMap::new();
    for one in running.iter().filter(|one| !one.incoming) {
        if let Some(name) = categories.get(&one.party)
            && TWICE.contains(&name.as_str())
        {
            by_category.entry(name.as_str()).or_default().push(one);
        }
    }
    by_category
        .into_iter()
        .filter_map(|(category, ones)| {
            let parties: std::collections::BTreeSet<i64> =
                ones.iter().map(|one| one.party).collect();
            if parties.len() < 2 {
                return None;
            }
            let per_year: i64 = ones.iter().map(|one| one.per_year).sum();
            let names: Vec<&str> = ones.iter().map(|one| one.name.as_str()).collect();
            Some(Hint {
                kind: Kind::Twice,
                key: format!("twice:{category}"),
                numbers: parties
                    .iter()
                    .map(i64::to_string)
                    .collect::<Vec<_>>()
                    .join(","),
                subject: category.to_owned(),
                line: format!(
                    "{} laufend: {}, zusammen {} im Jahr",
                    ones.len(),
                    names.join(", "),
                    Cents::new(per_year.abs())
                ),
                cents: per_year,
                bookings: ones
                    .iter()
                    .flat_map(|one| one.bookings.iter().copied())
                    .collect(),
                documents: Vec::new(),
            })
        })
        .collect()
}

/// The bookings that count with their effective category, between two days.
fn spending(
    conn: &Connection,
    from: Date,
    to: Date,
) -> rusqlite::Result<Vec<(i64, i64, Option<String>)>> {
    conn.prepare(
        "SELECT b.id, b.amount_cents, c.name FROM bookings b
         LEFT JOIN parties p ON p.id = b.party_id
         LEFT JOIN categories c ON c.id = coalesce(b.category_id, p.category_id)
         WHERE b.replaced_by IS NULL AND b.day >= ?1 AND b.day <= ?2
           AND NOT EXISTS (SELECT 1 FROM booking_pairs m WHERE (m.out_booking = b.id OR m.in_booking = b.id) AND m.undone_at IS NULL)
           AND (c.direction IS NULL OR c.direction = 'out')",
    )?
    .query_map(params![from.to_string(), to.to_string()], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)))?
    .collect()
}

fn fees(conn: &Connection, until: Date) -> rusqlite::Result<Vec<Hint>> {
    let from = jiff::civil::date(until.year(), 1, 1);
    let rows: Vec<(i64, i64)> = spending(conn, from, until)?
        .into_iter()
        .filter(|(_, _, category)| category.as_deref() == Some(FEES))
        .map(|(id, cents, _)| (id, cents))
        .collect();
    let sum: i64 = rows.iter().map(|(_, cents)| cents).sum();
    if sum >= 0 {
        return Ok(Vec::new());
    }
    Ok(vec![Hint {
        kind: Kind::Fees,
        key: format!("fees:{}", until.year()),
        numbers: format!("{sum}:{}", rows.len()),
        subject: FEES.to_owned(),
        line: format!(
            "{}: {} in {} Buchungen",
            until.year(),
            Cents::new(-sum),
            rows.len()
        ),
        cents: sum,
        bookings: rows.into_iter().map(|(id, _)| id).collect(),
        documents: Vec::new(),
    }])
}

fn above(conn: &Connection, until: Date) -> rusqlite::Result<Vec<Hint>> {
    let from = jiff::civil::date(until.year(), 1, 1);
    let (Ok(before_from), Ok(before_until)) = (
        from.checked_sub(jiff::Span::new().years(1)),
        until.checked_sub(jiff::Span::new().years(1)),
    ) else {
        return Ok(Vec::new());
    };
    let sums = |rows: Vec<(i64, i64, Option<String>)>| {
        let mut by: BTreeMap<String, (i64, Vec<i64>)> = BTreeMap::new();
        for (id, cents, category) in rows {
            if let Some(category) = category
                && !NEVER.contains(&category.as_str())
            {
                let entry = by.entry(category).or_default();
                entry.0 -= cents;
                entry.1.push(id);
            }
        }
        by
    };
    let this = sums(spending(conn, from, until)?);
    let before = sums(spending(conn, before_from, before_until)?);
    Ok(this
        .into_iter()
        .filter_map(|(category, (cents, ids))| {
            let (was, _) = before.get(&category)?;
            let more = cents - was;
            (*was > 0 && more >= ABOVE_CENTS && more * ABOVE_SHARE >= *was).then(|| Hint {
                kind: Kind::Above,
                key: format!("above:{}:{category}", until.year()),
                numbers: format!("{cents}:{was}"),
                subject: category.clone(),
                line: format!(
                    "{} bis {}: {} statt {}",
                    until.year(),
                    month_name(until),
                    Cents::new(cents),
                    Cents::new(*was)
                ),
                cents: -more,
                bookings: ids,
                documents: Vec::new(),
            })
        })
        .collect())
}

/// A letter of the party that cancels, with its day: the invoice date read, else the day it came.
fn cancellations(conn: &Connection, party: i64) -> rusqlite::Result<Vec<(i64, Date)>> {
    let documents: Vec<(i64, String)> = conn
        .prepare("SELECT id, imported_at FROM documents WHERE party_id = ?1 AND kind NOT IN ('statement', 'payslip')")?
        .query_map([party], |row| Ok((row.get(0)?, row.get(1)?)))?
        .collect::<rusqlite::Result<_>>()?;
    let mut found = Vec::new();
    for (id, imported_at) in documents {
        let cancels = conn
            .prepare("SELECT text FROM page_lines WHERE document_id = ?1")?
            .query_map([id], |row| row.get::<_, String>(0))?
            .filter_map(Result::ok)
            .any(|text| CANCELLED.is_match(&text));
        if !cancels {
            continue;
        }
        let dated: Option<String> = conn
            .query_row(
                "SELECT value FROM field_values WHERE document_id = ?1 AND field = 'invoice_date' ORDER BY id LIMIT 1",
                [id],
                |row| row.get(0),
            )
            .optional()?;
        let day = dated
            .and_then(|day| day.parse::<Date>().ok())
            .or_else(|| imported_at.get(..10).and_then(|day| day.parse().ok()));
        if let Some(day) = day {
            found.push((id, day));
        }
    }
    Ok(found)
}

fn still_runs(conn: &Connection, running: &[&Recurring]) -> rusqlite::Result<Vec<Hint>> {
    let mut hints = Vec::new();
    for one in running.iter().filter(|one| !one.incoming) {
        for (document, day) in cancellations(conn, one.party)? {
            // A cancellation ends with the period: one more payment is no hint.
            let after: Vec<i64> = days_after(conn, &one.bookings, one.rhythm.months(), day)?;
            if after.is_empty() {
                continue;
            }
            hints.push(Hint {
                kind: Kind::StillRuns,
                key: format!("still-runs:{}:{document}", one.party),
                numbers: after.len().to_string(),
                subject: one.name.clone(),
                line: format!(
                    "nach der Kündigung vom {} noch {}× gezahlt",
                    day.strftime("%d.%m.%Y"),
                    after.len()
                ),
                cents: one.cents * i64::try_from(after.len()).unwrap_or(0),
                bookings: after,
                documents: vec![document],
            });
        }
    }
    Ok(hints)
}

/// The bookings later than a rhythm after a day.
fn days_after(
    conn: &Connection,
    bookings: &[i64],
    months: i64,
    day: Date,
) -> rusqlite::Result<Vec<i64>> {
    let limit = day
        .checked_add(jiff::Span::new().months(months))
        .unwrap_or(day);
    let mut later = Vec::new();
    for id in bookings {
        let booked: String =
            conn.query_row("SELECT day FROM bookings WHERE id = ?1", [id], |row| {
                row.get(0)
            })?;
        if booked.parse::<Date>().is_ok_and(|booked| booked > limit) {
            later.push(*id);
        }
    }
    Ok(later)
}

/// Whether the owner dismissed a hint with these numbers.
fn dismissed(conn: &Connection, hint: &Hint) -> rusqlite::Result<bool> {
    let numbers: Option<String> = conn
        .query_row(
            "SELECT numbers FROM hints_dismissed WHERE key = ?1",
            [&hint.key],
            |row| row.get(0),
        )
        .optional()?;
    Ok(numbers.as_deref() == Some(hint.numbers.as_str()))
}

/// Every hint of the archive as of `today` that the owner did not dismiss, by kind.
pub fn found(conn: &Connection, today: Date) -> rusqlite::Result<Vec<Hint>> {
    let until = recurring::known_until(conn, today)?;
    let all = recurring::found(conn, today)?;
    let running: Vec<&Recurring> = all.iter().filter(|one| !one.ended).collect();
    let categories = party_categories(conn)?;
    let mut hints = rose(&running, &categories);
    hints.extend(twice(&running, &categories));
    hints.extend(fees(conn, until)?);
    hints.extend(above(conn, until)?);
    hints.extend(still_runs(conn, &running)?);
    let mut kept = Vec::new();
    for hint in hints {
        if !dismissed(conn, &hint)? {
            kept.push(hint);
        }
    }
    Ok(kept)
}

/// The owner dismisses a hint with the numbers it showed.
pub fn dismiss(
    conn: &Connection,
    key: &str,
    numbers: &str,
    now: Timestamp,
) -> rusqlite::Result<()> {
    conn.execute(
        "INSERT INTO hints_dismissed (key, numbers, dismissed_at) VALUES (?1, ?2, ?3)
         ON CONFLICT (key) DO UPDATE SET numbers = excluded.numbers, dismissed_at = excluded.dismissed_at",
        params![key, numbers, now.to_string()],
    )?;
    Ok(())
}

#[cfg(test)]
mod tests;
