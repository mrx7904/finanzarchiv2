//! Balances over time (#36, FA-40): the balance of each account at the end of every day the
//! archive covers, in one pass over its bookings and the same as `accounts::balance_on`, and the
//! sum of all accounts. Where the bank printed a balance it is the one shown, and a difference to
//! the bookings is kept (knowledge/decision-balances.md).

use std::collections::BTreeMap;

use jiff::civil::Date;
use jiff::{Span, ToSpan};
use rusqlite::{Connection, params};

use crate::accounts;
use crate::money::Cents;

/// The end of one day of an account.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Day {
    pub day: Date,
    /// The opening plus the bookings up to this day.
    pub counted: Cents,
    /// What the bank printed at the end of a statement closing on this day.
    pub bank: Option<Cents>,
}

impl Day {
    /// The bank's balance where it printed one, else the one counted.
    pub fn shown(&self) -> Cents {
        self.bank.unwrap_or(self.counted)
    }

    /// The bank's balance minus the one counted, when they differ.
    pub fn difference(&self) -> Option<Cents> {
        let bank = self.bank?;
        (bank != self.counted).then(|| Cents::new(bank.value() - self.counted.value()))
    }
}

fn next(day: Date) -> Date {
    day.checked_add(1.day()).unwrap_or(day)
}

/// Every day of an account from its opening to the last day it knows (its last booking or the end
/// of its last statement). Empty without an opening.
pub fn of_account(conn: &Connection, account: i64) -> rusqlite::Result<Vec<Day>> {
    let Some(opening) = accounts::opening(conn, account)? else {
        return Ok(Vec::new());
    };
    // The bookings `balance_on` counts: after the opening's day, and those of the earliest statement
    // a bank dated with the day of the statement before.
    let bookings: Vec<(Date, i64)> = conn
        .prepare(
            "SELECT day, amount_cents FROM bookings WHERE account_id = ?1 AND replaced_by IS NULL
             AND (day > ?2 OR statement_id IN (SELECT id FROM statements WHERE account_id = ?1 AND opening_day = ?2))
             ORDER BY day, id",
        )?
        .query_map(params![account, opening.day.to_string()], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?))
        })?
        .filter_map(|row| match row {
            Ok((day, cents)) => day.parse().ok().map(|day| Ok((day, cents))),
            Err(error) => Some(Err(error)),
        })
        .collect::<rusqlite::Result<_>>()?;
    let mut bank: BTreeMap<Date, Cents> = BTreeMap::new();
    let closings = conn
        .prepare("SELECT closing_day, closing_cents FROM statements WHERE account_id = ?1 ORDER BY closing_day, id")?
        .query_map([account], |row| Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?)))?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    for (day, cents) in closings {
        if let Ok(day) = day.parse::<Date>() {
            bank.insert(day, Cents::new(cents));
        }
    }
    let last = bookings
        .last()
        .map(|(day, _)| *day)
        .into_iter()
        .chain(bank.keys().next_back().copied())
        .fold(opening.day, Date::max);
    let mut days = Vec::new();
    let mut running = opening.balance.value();
    let mut at = 0;
    let mut day = opening.day;
    loop {
        while let Some((booked, cents)) = bookings.get(at)
            && *booked <= day
        {
            running += cents;
            at += 1;
        }
        days.push(Day {
            day,
            counted: Cents::new(running),
            bank: bank.get(&day).copied(),
        });
        if day >= last {
            break;
        }
        day = next(day);
    }
    Ok(days)
}

/// The balance of an account at the end of a day, from its series: the last known one after its
/// last day, `None` before its opening.
pub fn on(days: &[Day], day: Date) -> Option<Day> {
    let at = days.partition_point(|known| known.day <= day);
    at.checked_sub(1).map(|at| days[at])
}

/// The sum of all accounts at the end of each day.
pub type Total = Vec<(Date, Cents)>;

/// One account and its days.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Account {
    pub id: i64,
    pub name: String,
    pub days: Vec<Day>,
}

/// Every account with an opening and its days, and the sum of all for every day from the first
/// opening to the last day any account knows. An account counts in the sum from its opening, with
/// its last balance after its last day.
pub fn all(conn: &Connection) -> rusqlite::Result<(Vec<Account>, Total)> {
    let ids: Vec<(i64, String)> = conn
        .prepare("SELECT id, name FROM accounts ORDER BY name, id")?
        .query_map([], |row| Ok((row.get(0)?, row.get(1)?)))?
        .collect::<rusqlite::Result<_>>()?;
    let mut accounts = Vec::new();
    for (id, name) in ids {
        let days = of_account(conn, id)?;
        if !days.is_empty() {
            accounts.push(Account { id, name, days });
        }
    }
    let first = accounts
        .iter()
        .filter_map(|a| a.days.first())
        .map(|d| d.day)
        .min();
    let last = accounts
        .iter()
        .filter_map(|a| a.days.last())
        .map(|d| d.day)
        .max();
    let mut total = Vec::new();
    if let (Some(first), Some(last)) = (first, last) {
        let mut day = first;
        loop {
            let sum: i64 = accounts
                .iter()
                .filter_map(|account| on(&account.days, day))
                .map(|known| known.shown().value())
                .sum();
            total.push((day, Cents::new(sum)));
            if day >= last {
                break;
            }
            day = next(day);
        }
    }
    Ok((accounts, total))
}

/// The last day of each month a series covers, and the last day it knows.
pub fn month_ends(first: Date, last: Date) -> Vec<Date> {
    let mut ends = Vec::new();
    let mut month = first.first_of_month();
    while month <= last {
        let end = month.last_of_month().min(last);
        ends.push(end);
        let Ok(following) = month.checked_add(Span::new().months(1)) else {
            break;
        };
        month = following;
    }
    ends
}

#[cfg(test)]
mod tests;
