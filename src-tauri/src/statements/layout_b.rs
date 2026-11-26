//! Layout B: a CSV export of a bank, cells split by ";". Above the table, lines of key and value:
//! "Umsatzanzeige", "IBAN", "Kontoname", "Bank", "Zeitraum" ("dd.mm.yyyy - dd.mm.yyyy") and "Saldo".
//! The table is headed "Buchung;…;Saldo;Währung;Betrag;Währung" and lists the bookings newest first,
//! each with the balance after it. Named by its layout, never by a bank.

use jiff::civil::Date;

use super::{Booking, Statement};
use crate::money::Cents;
use crate::readers::Line;

pub const NAME: &str = "B";

/// The bytes as text: UTF-8 (a BOM dropped), else Windows-1252, as the owner's bank writes it.
pub fn decode(bytes: &[u8]) -> String {
    let bytes = bytes.strip_prefix(b"\xEF\xBB\xBF").unwrap_or(bytes);
    if let Ok(text) = std::str::from_utf8(bytes) {
        return text.to_owned();
    }
    // The 32 places where Windows-1252 differs from Latin-1, the rest map one to one.
    const HIGH: [char; 32] = [
        '€', '\u{81}', '‚', 'ƒ', '„', '…', '†', '‡', 'ˆ', '‰', 'Š', '‹', 'Œ', '\u{8D}', 'Ž',
        '\u{8F}', '\u{90}', '‘', '’', '“', '”', '•', '–', '—', '˜', '™', 'š', '›', 'œ', '\u{9D}',
        'ž', 'Ÿ',
    ];
    bytes
        .iter()
        .map(|&byte| match byte {
            0x80..=0x9F => HIGH[usize::from(byte - 0x80)],
            _ => char::from(byte),
        })
        .collect()
}

/// The cells of a line, with quotes ("a;b") and doubled quotes inside them.
pub fn cells(line: &str) -> Vec<String> {
    let mut cells = Vec::new();
    let mut cell = String::new();
    let mut quoted = false;
    let mut chars = line.chars().peekable();
    while let Some(c) = chars.next() {
        match c {
            '"' if quoted && chars.peek() == Some(&'"') => {
                cell.push('"');
                chars.next();
            }
            '"' if quoted => quoted = false,
            // A quote opens a quoted cell only at its start, inside a plain cell it is a character.
            '"' if cell.is_empty() => quoted = true,
            ';' if !quoted => cells.push(std::mem::take(&mut cell)),
            c => cell.push(c),
        }
    }
    cells.push(cell);
    cells
        .into_iter()
        .map(|cell| cell.trim().to_owned())
        .collect()
}

fn is_table_head(cells: &[String]) -> bool {
    cells.first().is_some_and(|cell| cell == "Buchung")
        && cells.iter().any(|cell| cell == "Saldo")
        && cells.iter().any(|cell| cell == "Betrag")
}

/// Whether a text is a statement of layout B: its first line, an IBAN and the head of the table.
pub fn looks_like(text: &str) -> bool {
    let mut lines = text.lines();
    lines
        .next()
        .is_some_and(|first| first.starts_with("Umsatzanzeige"))
        && text.lines().any(|line| line.starts_with("IBAN;"))
        && text.lines().any(|line| is_table_head(&cells(line)))
}

fn date(text: &str) -> Option<Date> {
    let mut parts = text.trim().split('.');
    let day = parts.next()?.parse().ok()?;
    let month = parts.next()?.parse().ok()?;
    let year = parts.next()?.parse().ok()?;
    Date::new(year, month, day).ok()
}

fn amount(text: &str) -> Option<Cents> {
    Cents::parse_de(text.trim()).ok()
}

/// Reads a statement of layout B from its lines, one CSV line each. `None` when it is none.
pub fn read(lines: &[Line]) -> Option<Statement> {
    let texts: Vec<&str> = lines.iter().map(|line| line.text.as_str()).collect();
    if !texts
        .first()
        .is_some_and(|first| first.starts_with("Umsatzanzeige"))
    {
        return None;
    }
    let value = |key: &str| {
        texts.iter().find_map(|text| {
            let cells = cells(text);
            (cells.first().map(String::as_str) == Some(key))
                .then(|| cells.get(1).cloned())
                .flatten()
        })
    };
    let iban = value("IBAN")?.replace(' ', "");
    let (from, to) = {
        let period = value("Zeitraum")?;
        let (from, to) = period.split_once('-')?;
        (date(from)?, date(to)?)
    };
    let closing = amount(&value("Saldo")?)?;
    let bank = value("Bank");
    let head_at = lines
        .iter()
        .position(|line| is_table_head(&cells(&line.text)))?;
    let head = cells(&lines[head_at].text);
    let column = |name: &str| head.iter().position(|cell| cell == name);
    let (saldo, betrag) = (column("Saldo")?, column("Betrag")?);
    let (other, text_column, purpose) = (
        column("Auftraggeber/Empfänger"),
        column("Buchungstext"),
        column("Verwendungszweck"),
    );
    let mut newest_first: Vec<(Booking, Cents)> = Vec::new();
    for line in &lines[head_at + 1..] {
        let row = cells(&line.text);
        if row.len() < head.len() {
            continue;
        }
        let Some(day) = date(&row[0]) else { continue };
        let pick = |index: Option<usize>| {
            index
                .and_then(|index| row.get(index))
                .cloned()
                .unwrap_or_default()
        };
        newest_first.push((
            Booking {
                day,
                value_day: row.get(1).and_then(|cell| date(cell)),
                amount: amount(&row[betrag])?,
                kind: pick(text_column),
                other_side: Some(pick(other)).filter(|other| !other.is_empty()),
                purpose: Some(pick(purpose))
                    .filter(|purpose| purpose != "-")
                    .unwrap_or_default(),
                line: line.clone(),
            },
            amount(&row[saldo])?,
        ));
    }
    let oldest = newest_first.last();
    // The balance before the oldest booking: its balance after it, minus its amount.
    let opening = match oldest {
        Some((booking, after)) => after.checked_sub(booking.amount)?,
        None => closing,
    };
    let bookings: Vec<Booking> = newest_first
        .into_iter()
        .rev()
        .map(|(booking, _)| booking)
        .collect();
    Some(Statement {
        layout: NAME,
        bank,
        iban,
        number: None,
        opening_day: from.yesterday().ok()?,
        closing_day: to,
        opening,
        closing,
        bookings,
    })
}

/// The lines of a CSV file for the archive: one per line of the file, top to bottom, in page 1.
pub fn lines_of(text: &str) -> Vec<crate::pages::Line> {
    text.lines()
        .enumerate()
        .map(|(index, line)| {
            let top = -(index as f32);
            crate::pages::Line {
                text: line.to_owned(),
                left: 0.0,
                bottom: top - 1.0,
                right: 0.0,
                top,
            }
        })
        .collect()
}
