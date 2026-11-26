//! The balance of each account on each day, held against the bank (FA-27, #22). The balance is the
//! opening balance plus the bookings up to the day, and every balance a statement printed is a
//! proof of it. A difference or a gap between two statements is a finding, never hidden.

use jiff::civil::Date;
use rusqlite::{Connection, OptionalExtension, params};

use crate::money::Cents;

/// The opening of an account: the balance at the end of a day.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Opening {
    pub day: Date,
    pub balance: Cents,
    pub by_owner: bool,
}

fn parse_day(text: &str) -> Option<Date> {
    text.parse().ok()
}

pub fn opening(conn: &Connection, account: i64) -> rusqlite::Result<Option<Opening>> {
    let row: Option<(Option<i64>, Option<String>, Option<String>)> = conn
        .query_row(
            "SELECT opening_cents, opening_day, opening_source FROM accounts WHERE id = ?1",
            [account],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        )
        .optional()?;
    Ok(row.and_then(|(cents, day, source)| {
        Some(Opening {
            day: parse_day(&day?)?,
            balance: Cents::new(cents?),
            by_owner: source.as_deref() == Some("owner"),
        })
    }))
}

/// Takes the opening from the earliest statement of the account, so an earlier statement read
/// later moves it back. An opening the owner typed is never written over.
pub fn refresh_opening(conn: &Connection, account: i64) -> rusqlite::Result<()> {
    if opening(conn, account)?.is_some_and(|opening| opening.by_owner) {
        return Ok(());
    }
    let earliest: Option<(String, i64)> = conn
        .query_row(
            "SELECT opening_day, opening_cents FROM statements WHERE account_id = ?1 ORDER BY opening_day, id LIMIT 1",
            [account],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .optional()?;
    match earliest {
        Some((day, cents)) => conn.execute(
            "UPDATE accounts SET opening_day = ?2, opening_cents = ?3, opening_source = 'statement' WHERE id = ?1",
            params![account, day, cents],
        )?,
        None => conn.execute(
            "UPDATE accounts SET opening_day = NULL, opening_cents = NULL, opening_source = NULL WHERE id = ?1",
            [account],
        )?,
    };
    Ok(())
}

/// The owner's opening balance of an account, kept over every statement.
pub fn set_opening(
    conn: &Connection,
    account: i64,
    day: Date,
    balance: Cents,
) -> rusqlite::Result<()> {
    conn.execute(
        "UPDATE accounts SET opening_day = ?2, opening_cents = ?3, opening_source = 'owner' WHERE id = ?1",
        params![account, day.to_string(), balance.value()],
    )?;
    Ok(())
}

/// The balance at the end of a day: the opening plus the bookings after its day up to this one,
/// those another booking replaced left out. `None` before the opening or without one.
pub fn balance_on(conn: &Connection, account: i64, day: Date) -> rusqlite::Result<Option<Cents>> {
    let Some(opening) = opening(conn, account)? else {
        return Ok(None);
    };
    if day < opening.day {
        return Ok(None);
    }
    // A booking of the earliest statement may carry the day of its old balance (a bank dates it
    // with the statement before): its old balance does not hold it, so it counts.
    let amounts: Vec<i64> = conn
        .prepare(
            "SELECT amount_cents FROM bookings WHERE account_id = ?1 AND replaced_by IS NULL AND day <= ?3
             AND (day > ?2 OR statement_id IN (SELECT id FROM statements WHERE account_id = ?1 AND opening_day = ?2))",
        )?
        .query_map(params![account, opening.day.to_string(), day.to_string()], |row| row.get(0))?
        .collect::<rusqlite::Result<_>>()?;
    Ok(amounts.into_iter().try_fold(opening.balance, |sum, cents| {
        sum.checked_add(Cents::new(cents))
    }))
}

/// The balance the archive counts at the end of a statement: the opening plus the bookings of
/// every statement of the account up to this one, each counted with its own statement. A bank
/// that dates a booking with the day of the statement before so does not move it into that one.
pub fn counted_through(
    conn: &Connection,
    account: i64,
    statement: i64,
) -> rusqlite::Result<Option<Cents>> {
    let Some(opening) = opening(conn, account)? else {
        return Ok(None);
    };
    let amounts: Vec<i64> = conn
        .prepare(
            "SELECT b.amount_cents FROM bookings b JOIN statements s ON s.id = b.statement_id
             JOIN statements this ON this.id = ?2
             WHERE b.account_id = ?1 AND b.replaced_by IS NULL AND s.opening_day >= ?3
             AND (s.closing_day < this.closing_day OR (s.closing_day = this.closing_day AND s.id <= this.id))",
        )?
        .query_map(params![account, statement, opening.day.to_string()], |row| row.get(0))?
        .collect::<rusqlite::Result<_>>()?;
    Ok(amounts.into_iter().try_fold(opening.balance, |sum, cents| {
        sum.checked_add(Cents::new(cents))
    }))
}

/// What does not match between the archive and the bank, in German for the owner.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Finding {
    /// The balance the archive counts differs from the one the bank printed on that day.
    Differs {
        day: Date,
        bank: Cents,
        counted: Cents,
    },
    /// The next statement does not start where the one before ended.
    Gap {
        after: Date,
        until: Date,
        closing: Cents,
        opening: Cents,
    },
    /// There is no opening balance to count from.
    NoOpening,
}

impl Finding {
    pub fn sentence(&self) -> String {
        match self {
            Self::Differs { day, bank, counted } => format!(
                "Am {} zeigt die Bank {bank}, das Archiv zählt {counted}.",
                day.strftime("%d.%m.%Y")
            ),
            Self::Gap {
                after,
                until,
                closing,
                opening,
            } if closing == opening => format!(
                "Zwischen dem {} und dem {} fehlt ein Kontoauszug.",
                after.strftime("%d.%m.%Y"),
                until.strftime("%d.%m.%Y")
            ),
            Self::Gap {
                after,
                until,
                closing,
                opening,
            } => format!(
                "Der Auszug bis {} endet mit {closing}, der nächste ab {} beginnt mit {opening}: dazwischen fehlt etwas.",
                after.strftime("%d.%m.%Y"),
                until.strftime("%d.%m.%Y")
            ),
            Self::NoOpening => "Für dieses Konto gibt es keinen Startsaldo.".into(),
        }
    }
}

/// Every finding of an account, oldest first: the balance on each closing day of a statement
/// against the bank's, and each gap between two statements.
pub fn findings(conn: &Connection, account: i64) -> rusqlite::Result<Vec<Finding>> {
    if opening(conn, account)?.is_none() {
        return Ok(vec![Finding::NoOpening]);
    }
    let statements: Vec<(i64, String, String, i64, i64)> = conn
        .prepare("SELECT id, opening_day, closing_day, opening_cents, closing_cents FROM statements WHERE account_id = ?1 ORDER BY closing_day, id")?
        .query_map([account], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?, row.get(4)?)))?
        .collect::<rusqlite::Result<_>>()?;
    let mut found = Vec::new();
    let mut before: Option<(Date, Cents)> = None;
    for (statement, opening_day, closing_day, opening_cents, closing_cents) in statements {
        let (Some(opening_day), Some(closing_day)) =
            (parse_day(&opening_day), parse_day(&closing_day))
        else {
            continue;
        };
        let (opening_cents, closing_cents) = (Cents::new(opening_cents), Cents::new(closing_cents));
        if let Some((after, closing)) = before {
            // A statement may start on the day the one before ended, or cover days of it (a CSV
            // over PDF months). Only a later start or another balance is a gap.
            if opening_day > after || (opening_day == after && opening_cents != closing) {
                found.push(Finding::Gap {
                    after,
                    until: opening_day,
                    closing,
                    opening: opening_cents,
                });
            }
        }
        if let Some(counted) = counted_through(conn, account, statement)?
            && counted != closing_cents
        {
            found.push(Finding::Differs {
                day: closing_day,
                bank: closing_cents,
                counted,
            });
        }
        before = Some(match before {
            Some((after, closing)) if after > closing_day => (after, closing),
            _ => (closing_day, closing_cents),
        });
    }
    Ok(found)
}

/// An account as the window lists it: its balance today and the bank's last one.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Listed {
    pub id: i64,
    pub name: String,
    pub iban: Option<String>,
    pub balance: Option<Cents>,
    pub bank_balance: Option<(Date, Cents)>,
    pub findings: Vec<Finding>,
}

pub fn list(conn: &Connection, today: Date) -> rusqlite::Result<Vec<Listed>> {
    let accounts: Vec<(i64, String, Option<String>)> = conn
        .prepare("SELECT id, name, iban FROM accounts ORDER BY name, id")?
        .query_map([], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)))?
        .collect::<rusqlite::Result<_>>()?;
    accounts
        .into_iter()
        .map(|(id, name, iban)| {
            let bank: Option<(String, i64)> = conn
                .query_row(
                    "SELECT closing_day, closing_cents FROM statements WHERE account_id = ?1 ORDER BY closing_day DESC, id DESC LIMIT 1",
                    [id],
                    |row| Ok((row.get(0)?, row.get(1)?)),
                )
                .optional()?;
            Ok(Listed {
                id,
                name,
                iban,
                balance: balance_on(conn, id, today)?,
                bank_balance: bank.and_then(|(day, cents)| Some((parse_day(&day)?, Cents::new(cents)))),
                findings: findings(conn, id)?,
            })
        })
        .collect()
}

#[cfg(test)]
mod tests;
