//! Layout A: a table with the column heads "Bu-Tag Wert Vorgang". A booking starts with its day
//! and its day of value ("dd.mm." each), the kind beside them and the amount at the right with
//! S (money out) or H (money in). The lines below it, indented, are the other side and the purpose.
//! "alter Kontostand vom" and "neuer Kontostand vom" frame the table, "Übertrag auf/von Blatt"
//! carries it from page to page. Named by its layout, never by a bank (decision-public-repository.md).

use std::sync::LazyLock;

use jiff::civil::Date;
use regex::Regex;

use super::{Booking, Statement};
use crate::money::Cents;
use crate::readers::Line;

pub const NAME: &str = "A";

fn pattern(text: &str) -> Regex {
    Regex::new(text).unwrap_or_else(|_| unreachable!("a fixed pattern"))
}

static BOOKING: LazyLock<Regex> = LazyLock::new(|| {
    pattern(
        r"^(\d{2})\.(\d{2})\.\s+(\d{2})\.(\d{2})\.\s+(.*?)\s*((?:\d{1,3}(?:\.\d{3})+|\d+),\d{2})\s+([SH])$",
    )
});
static BALANCE: LazyLock<Regex> = LazyLock::new(|| {
    pattern(
        r"(alter|neuer) Kontostand(?: vom (\d{2})\.(\d{2})\.(\d{4}))?\s+((?:\d{1,3}(?:\.\d{3})+|\d+),\d{2})\s+([SH])",
    )
});
static CARRY: LazyLock<Regex> = LazyLock::new(|| pattern(r"^Übertrag (auf|von) Blatt \d+"));
static NUMBER: LazyLock<Regex> = LazyLock::new(|| pattern(r"Kontoauszug Nr\.\s*(\d+/\d{4})"));
static IBAN: LazyLock<Regex> = LazyLock::new(|| {
    pattern(r"IBAN:?\s*([A-Z]{2}\d{2}(?:\s?[A-Z0-9]{4}){3,7}(?:\s?[A-Z0-9]{1,3})?)")
});

/// The table's columns start at the left of the page; the codes of the form at its very edge do not.
const TABLE_LEFT: f64 = 30.0;
/// The lines of the other side and the purpose are indented under the kind.
const INDENT: f64 = 90.0;

/// Whether the lines are a statement of layout A: the column heads and both balances.
pub fn recognizes(lines: &[Line]) -> bool {
    lines
        .iter()
        .any(|line| line.text.contains("Bu-Tag") && line.text.contains("Vorgang"))
        && lines
            .iter()
            .any(|line| line.text.contains("alter Kontostand"))
        && lines
            .iter()
            .any(|line| line.text.contains("neuer Kontostand vom"))
}

fn signed(printed: &str, side: &str) -> Option<Cents> {
    let cents = Cents::parse_de(printed).ok()?;
    Some(if side == "S" {
        Cents::new(-cents.value())
    } else {
        cents
    })
}

fn day(day: &str, month: &str, year: &str) -> Option<Date> {
    Date::new(year.parse().ok()?, month.parse().ok()?, day.parse().ok()?).ok()
}

/// The year of a "dd.mm." in a statement that closes on `closing`: a month after the closing month
/// is of the year before (a December booking in a January statement).
fn in_year(day_text: &str, month_text: &str, closing: Date) -> Option<Date> {
    let month: i8 = month_text.parse().ok()?;
    let year = if month > closing.month() {
        closing.year() - 1
    } else {
        closing.year()
    };
    Date::new(year, month, day_text.parse().ok()?).ok()
}

/// Reads a statement of layout A. `None` when the lines are not one, or a part is missing.
pub fn read(lines: &[Line]) -> Option<Statement> {
    if !recognizes(lines) {
        return None;
    }
    let mut opening = None;
    let mut closing = None;
    for line in lines {
        if let Some(found) = BALANCE.captures(&line.text) {
            // The first statement of a new account prints its old balance without a day.
            let on = match (found.get(2), found.get(3), found.get(4)) {
                (Some(d), Some(m), Some(y)) => Some(day(d.as_str(), m.as_str(), y.as_str())?),
                _ => None,
            };
            let amount = signed(&found[5], &found[6])?;
            if &found[1] == "alter" {
                opening = opening.or(Some((on, amount)));
            } else {
                closing = Some((on?, amount));
            }
        }
    }
    let (printed_opening_day, opening_cents) = opening?;
    let (closing_day, closing_cents) = closing?;
    let iban = lines.iter().find_map(|line| {
        IBAN.captures(&line.text)
            .map(|found| found[1].replace(' ', ""))
    })?;
    let number = lines
        .iter()
        .find_map(|line| NUMBER.captures(&line.text).map(|found| found[1].to_owned()));
    let bank = lines
        .iter()
        .find(|line| line.page == 1)
        .map(|line| line.text.trim().to_owned());

    let mut bookings: Vec<Booking> = Vec::new();
    let mut in_table = false;
    let mut ended = false;
    let mut page = 0;
    for line in lines {
        if line.page != page {
            page = line.page;
            in_table = false;
        }
        let text = line.text.trim();
        if ended || line.left < TABLE_LEFT {
            continue;
        }
        if text.starts_with("Bu-Tag") {
            in_table = true;
            continue;
        }
        if !in_table {
            continue;
        }
        if text.contains("neuer Kontostand vom") {
            ended = true;
            continue;
        }
        if CARRY.is_match(text) {
            in_table = !text.starts_with("Übertrag auf");
            continue;
        }
        if text.contains("alter Kontostand") {
            continue;
        }
        if let Some(found) = BOOKING.captures(text) {
            let on = in_year(&found[1], &found[2], closing_day)?;
            let value = in_year(&found[3], &found[4], closing_day);
            bookings.push(Booking {
                day: on,
                value_day: value,
                amount: signed(&found[6], &found[7])?,
                kind: found[5].trim().to_owned(),
                other_side: None,
                purpose: String::new(),
                line: line.clone(),
            });
            continue;
        }
        // A line under a booking: the first is the other side, the rest the purpose.
        if let Some(booking) = bookings.last_mut().filter(|_| line.left >= INDENT) {
            match &booking.other_side {
                None => booking.other_side = Some(text.to_owned()),
                Some(_) if booking.purpose.is_empty() => booking.purpose = text.to_owned(),
                Some(_) => {
                    booking.purpose.push(' ');
                    booking.purpose.push_str(text);
                }
            }
        }
    }
    // Without its day, the old balance holds up to the day before the first booking.
    let opening_day = match printed_opening_day {
        Some(day) => day,
        None => bookings
            .first()
            .map_or(closing_day, |booking| booking.day)
            .yesterday()
            .ok()?,
    };
    Some(Statement {
        layout: NAME,
        bank,
        iban,
        number,
        opening_day,
        closing_day,
        opening: opening_cents,
        closing: closing_cents,
        bookings,
    })
}
