//! Moves between own accounts (FA-28, #23): money moved from one own account to another changes
//! both balances and counts as neither income nor spending. Each move is a pair of bookings found
//! by a rule with a stable id, a decision that "Warum?" explains and "Rückgängig" takes back.

use jiff::Timestamp;
use rusqlite::{Connection, OptionalExtension, params};
use serde_json::json;

/// The purpose or other side names the IBAN of another own account.
pub const RULE_OWN_IBAN: &str = "move.own-iban:1";
/// The bank marks the booking as a move ("Umbuchung"), its other half is the same amount the
/// other way on the same day on another own account.
pub const RULE_MARKED: &str = "move.marked:1";
/// Out of one own account and the same amount into another of the same bank on the same day.
pub const RULE_SAME_DAY: &str = "move.same-day:1";

/// What a booking the rules look at carries.
struct Candidate {
    id: i64,
    account: i64,
    day: String,
    amount: i64,
    kind: String,
    other_side: Option<String>,
    purpose: String,
}

fn ibans_in(text: &str) -> Vec<String> {
    // Every IBAN with a valid check sum, wherever it stands in the text.
    let mut found = Vec::new();
    let mut rest = text;
    while let Some(iban) = crate::readers::parse::iban(rest) {
        let compact: String = iban.chars().filter(|c| !c.is_whitespace()).collect();
        let Some(at) = rest.find(&iban) else { break };
        rest = &rest[at + iban.len()..];
        found.push(compact);
    }
    found
}

/// Whether the other side of a booking is the bank of its account itself: a fee, a dividend of a
/// share of the bank. Never a move, even when its purpose names an own IBAN (lesson of 1.0).
fn is_the_bank(
    conn: &Connection,
    account: i64,
    other_side: Option<&str>,
) -> rusqlite::Result<bool> {
    let bank: Option<String> = conn.query_row(
        "SELECT bank FROM accounts WHERE id = ?1",
        [account],
        |row| row.get(0),
    )?;
    let (Some(bank), Some(other)) = (bank, other_side) else {
        return Ok(false);
    };
    let (bank, other) = (crate::parties::fold(&bank), crate::parties::fold(other));
    Ok(!bank.is_empty() && !other.is_empty() && (other.contains(&bank) || bank.contains(&other)))
}

fn paired(conn: &Connection, booking: i64) -> rusqlite::Result<bool> {
    Ok(conn
        .query_row(
            "SELECT 1 FROM booking_pairs WHERE (out_booking = ?1 OR in_booking = ?1) AND undone_at IS NULL LIMIT 1",
            [booking],
            |_| Ok(true),
        )
        .optional()?
        .is_some())
}

/// A pair someone took back is not made again.
fn was_undone(conn: &Connection, booking: i64) -> rusqlite::Result<bool> {
    Ok(conn
        .query_row(
            "SELECT 1 FROM booking_pairs WHERE (out_booking = ?1 OR in_booking = ?1) AND undone_at IS NOT NULL LIMIT 1",
            [booking],
            |_| Ok(true),
        )
        .optional()?
        .is_some())
}

fn candidates(conn: &Connection) -> rusqlite::Result<Vec<Candidate>> {
    conn.prepare(
        "SELECT id, account_id, day, amount_cents, kind, other_side, purpose FROM bookings
         WHERE replaced_by IS NULL AND made_by IS NULL ORDER BY day, id",
    )?
    .query_map([], |row| {
        Ok(Candidate {
            id: row.get(0)?,
            account: row.get(1)?,
            day: row.get(2)?,
            amount: row.get(3)?,
            kind: row.get(4)?,
            other_side: row.get(5)?,
            purpose: row.get(6)?,
        })
    })?
    .collect()
}

/// The booking on another own account the same day with the opposite amount, when there is exactly one.
fn counterpart(
    conn: &Connection,
    booking: &Candidate,
    account: Option<i64>,
    same_bank: bool,
) -> rusqlite::Result<Option<i64>> {
    let found: Vec<i64> = conn
        .prepare(
            "SELECT b.id FROM bookings b JOIN accounts a ON a.id = b.account_id JOIN accounts mine ON mine.id = ?2
             WHERE b.account_id <> ?2 AND b.day = ?3 AND b.amount_cents = ?4 AND b.replaced_by IS NULL AND b.made_by IS NULL
             AND (?5 IS NULL OR b.account_id = ?5) AND (?6 = 0 OR a.bank IS mine.bank)",
        )?
        .query_map(
            params![booking.id, booking.account, booking.day, -booking.amount, account, i64::from(same_bank)],
            |row| row.get(0),
        )?
        .collect::<rusqlite::Result<_>>()?;
    let mut free = Vec::new();
    for id in found {
        if !paired(conn, id)? && !was_undone(conn, id)? {
            free.push(id);
        }
    }
    let [one] = free.as_slice() else {
        return Ok(None);
    };
    // A named IBAN says which account it is. Otherwise the pair must be clear from both sides: the
    // other half sees only this booking too.
    if account.is_some() {
        return Ok(Some(*one));
    }
    let mirrored: i64 = conn.query_row(
        "SELECT count(*) FROM bookings b WHERE b.account_id <> (SELECT account_id FROM bookings WHERE id = ?1)
         AND b.day = ?2 AND b.amount_cents = ?3 AND b.replaced_by IS NULL AND b.made_by IS NULL
         AND NOT EXISTS (SELECT 1 FROM booking_pairs p WHERE (p.out_booking = b.id OR p.in_booking = b.id))",
        params![one, booking.day, booking.amount],
        |row| row.get(0),
    )?;
    Ok((mirrored == 1).then_some(*one))
}

fn record(
    conn: &Connection,
    out: i64,
    into: Option<i64>,
    rule: &str,
    inputs: serde_json::Value,
    now: Timestamp,
) -> rusqlite::Result<i64> {
    conn.execute(
        "INSERT INTO booking_pairs (out_booking, in_booking, rule, inputs, decided_at) VALUES (?1, ?2, ?3, ?4, ?5)",
        params![out, into, rule, inputs.to_string(), now.to_string()],
    )?;
    Ok(conn.last_insert_rowid())
}

/// Makes the other half of a move on an own account that has no statement for its day: the same
/// amount the other way, marked as made by the rule.
fn make_half(
    conn: &Connection,
    booking: &Candidate,
    account: i64,
    rule: &str,
) -> rusqlite::Result<Option<i64>> {
    let covered: bool = conn
        .query_row(
            "SELECT 1 FROM statements WHERE account_id = ?1 AND opening_day < ?2 AND closing_day >= ?2 LIMIT 1",
            params![account, booking.day],
            |_| Ok(true),
        )
        .optional()?
        .is_some();
    if covered {
        // A statement of that account covers the day, so its own booking is the other half.
        return Ok(None);
    }
    conn.execute(
        "INSERT INTO bookings (account_id, position, day, amount_cents, kind, other_side, purpose, made_by)
         VALUES (?1, 0, ?2, ?3, ?4, ?5, ?6, ?7)",
        params![account, booking.day, -booking.amount, booking.kind, booking.other_side, booking.purpose, rule],
    )?;
    Ok(Some(conn.last_insert_rowid()))
}

/// Finds every move among the bookings not paired yet and records it. Returns how many pairs.
pub fn find(conn: &Connection, now: Timestamp) -> rusqlite::Result<usize> {
    let own: Vec<(i64, String)> = conn
        .prepare("SELECT id, iban FROM accounts WHERE iban IS NOT NULL")?
        .query_map([], |row| Ok((row.get(0)?, row.get(1)?)))?
        .collect::<rusqlite::Result<_>>()?;
    let mut made = 0;
    for booking in candidates(conn)? {
        if paired(conn, booking.id)?
            || was_undone(conn, booking.id)?
            || is_the_bank(conn, booking.account, booking.other_side.as_deref())?
        {
            continue;
        }
        let text = format!(
            "{} {}",
            booking.other_side.as_deref().unwrap_or(""),
            booking.purpose
        );
        let named = ibans_in(&text).into_iter().find_map(|iban| {
            own.iter()
                .find(|(id, own)| *own == iban && *id != booking.account)
                .map(|(id, _)| *id)
        });
        let found = if let Some(account) = named {
            let other = match counterpart(conn, &booking, Some(account), false)? {
                Some(other) => Some(other),
                None => make_half(conn, &booking, account, RULE_OWN_IBAN)?,
            };
            other.map(|other| {
                (
                    RULE_OWN_IBAN,
                    other,
                    json!({ "iban": "own", "account": account }),
                )
            })
        } else if booking.kind.to_lowercase().contains("umbuchung")
            || booking.purpose.to_lowercase().starts_with("umbuchung")
        {
            counterpart(conn, &booking, None, false)?
                .map(|other| (RULE_MARKED, other, json!({ "marked": booking.kind })))
        } else {
            counterpart(conn, &booking, None, true)?
                .filter(|_| booking.amount != 0)
                .map(|other| (RULE_SAME_DAY, other, json!({ "same_day": booking.day })))
        };
        let Some((rule, other, mut inputs)) = found else {
            continue;
        };
        if paired(conn, other)? {
            continue;
        }
        inputs["day"] = json!(booking.day);
        inputs["cents"] = json!(booking.amount.abs());
        let (out_booking, in_booking) = if booking.amount < 0 {
            (booking.id, other)
        } else {
            (other, booking.id)
        };
        record(conn, out_booking, Some(in_booking), rule, inputs, now)?;
        made += 1;
    }
    Ok(made)
}

/// A statement read for an account covers its days: a half the app made there before gives way
/// to the statement's own booking, and its move is found again (#23). Returns how many went.
pub fn give_way(
    conn: &Connection,
    account: i64,
    opening_day: &str,
    closing_day: &str,
) -> rusqlite::Result<usize> {
    let made: Vec<i64> = conn
        .prepare("SELECT id FROM bookings WHERE account_id = ?1 AND made_by IS NOT NULL AND day > ?2 AND day <= ?3")?
        .query_map(params![account, opening_day, closing_day], |row| row.get(0))?
        .collect::<rusqlite::Result<_>>()?;
    for id in &made {
        conn.execute(
            "DELETE FROM booking_pairs WHERE out_booking = ?1 OR in_booking = ?1",
            [id],
        )?;
        conn.execute("DELETE FROM bookings WHERE id = ?1", [id])?;
    }
    Ok(made.len())
}

/// Whether a booking is half of a move, so it counts as neither income nor spending.
pub fn is_move(conn: &Connection, booking: i64) -> rusqlite::Result<bool> {
    paired(conn, booking)
}

/// "Rückgängig": the pair is no move any more, and the rules do not pair those bookings again. A
/// half the app made stays for the record but counts no more (`replaced_by` itself).
pub fn undo(conn: &Connection, pair: i64, now: Timestamp) -> rusqlite::Result<bool> {
    let tx = conn.unchecked_transaction()?;
    let row: Option<(i64, Option<i64>)> = tx
        .query_row(
            "SELECT out_booking, in_booking FROM booking_pairs WHERE id = ?1 AND undone_at IS NULL",
            [pair],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .optional()?;
    let Some((out, into)) = row else {
        return Ok(false);
    };
    tx.execute(
        "UPDATE booking_pairs SET undone_at = ?2 WHERE id = ?1",
        params![pair, now.to_string()],
    )?;
    for booking in [Some(out), into].into_iter().flatten() {
        let made: Option<String> = tx.query_row(
            "SELECT made_by FROM bookings WHERE id = ?1",
            [booking],
            |row| row.get(0),
        )?;
        if made.is_some() {
            tx.execute(
                "UPDATE bookings SET replaced_by = id WHERE id = ?1",
                [booking],
            )?;
        }
    }
    tx.commit()?;
    Ok(true)
}

#[cfg(test)]
mod tests;
