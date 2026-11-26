//! Bank statements read by code (FA-25, #20): the account, the period, both balances and every
//! booking, proved by the balance (old balance plus the bookings is the new one to the cent). The
//! layouts are named by their form, never by a bank (knowledge/decision-statements.md).

pub mod layout_a;
pub mod layout_b;

use jiff::Timestamp;
use jiff::civil::Date;
use rusqlite::{Connection, OptionalExtension, params};

use crate::money::Cents;
use crate::readers::{Input, Line, Normalized, Reader, Value};

/// One booking as the statement prints it, with the line it starts on.
#[derive(Debug, Clone, PartialEq)]
pub struct Booking {
    pub day: Date,
    pub value_day: Option<Date>,
    /// Money in positive, money out negative.
    pub amount: Cents,
    pub kind: String,
    pub other_side: Option<String>,
    pub purpose: String,
    pub line: Line,
}

/// What code read of a statement.
#[derive(Debug, Clone, PartialEq)]
pub struct Statement {
    pub layout: &'static str,
    /// The first line of the first page, where the bank stands. Read at run time, never named in code.
    pub bank: Option<String>,
    pub iban: String,
    pub number: Option<String>,
    pub opening_day: Date,
    pub closing_day: Date,
    pub opening: Cents,
    pub closing: Cents,
    pub bookings: Vec<Booking>,
}

impl Statement {
    /// The sum of the bookings, `None` when it does not fit.
    pub fn bookings_sum(&self) -> Option<Cents> {
        self.bookings
            .iter()
            .try_fold(Cents::ZERO, |sum, booking| sum.checked_add(booking.amount))
    }

    /// Old balance plus the bookings minus the new balance: zero when the statement adds up.
    pub fn difference(&self) -> Option<Cents> {
        self.opening
            .checked_add(self.bookings_sum()?)?
            .checked_sub(self.closing)
    }
}

/// Reads the lines with every layout that recognizes them.
pub fn read(lines: &[Line]) -> Option<Statement> {
    layout_a::read(lines).or_else(|| layout_b::read(lines))
}

/// The values of a statement for the checks and the name: bank, IBAN, number, period, balances.
pub struct StatementCode;

impl Reader for StatementCode {
    fn name(&self) -> String {
        format!("code:{}", crate::readers::READERS_VERSION)
    }

    fn read(&self, input: &Input) -> Vec<Value> {
        let Some(statement) = read(&input.lines) else {
            return Vec::new();
        };
        let Some(first) = input.lines.first() else {
            return Vec::new();
        };
        let text = |field: &'static str, value: String| {
            Value::on(field, value.clone(), Normalized::Text(value), first)
        };
        let mut values = vec![
            text("iban", statement.iban.clone()),
            Value::on(
                "period_from",
                statement.opening_day.to_string(),
                Normalized::Date(statement.opening_day),
                first,
            ),
            Value::on(
                "period_to",
                statement.closing_day.to_string(),
                Normalized::Date(statement.closing_day),
                first,
            ),
            Value::on(
                "opening",
                statement.opening.to_string(),
                Normalized::Cents(statement.opening),
                first,
            ),
            Value::on(
                "closing",
                statement.closing.to_string(),
                Normalized::Cents(statement.closing),
                first,
            ),
        ];
        if let Some(bank) = &statement.bank {
            values.push(text("sender", bank.clone()));
        }
        if let Some(number) = &statement.number {
            values.push(text("statement_number", number.clone()));
        }
        values
    }
}

/// The account of an IBAN, made the first time a statement of it is read.
pub fn account_of(
    conn: &Connection,
    iban: &str,
    bank: Option<&str>,
    now: Timestamp,
) -> rusqlite::Result<i64> {
    if let Some(id) = conn
        .query_row("SELECT id FROM accounts WHERE iban = ?1", [iban], |row| {
            row.get(0)
        })
        .optional()?
    {
        return Ok(id);
    }
    let tail: String = iban
        .chars()
        .rev()
        .take(4)
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .collect();
    let name = match bank {
        Some(bank) => format!("{} …{tail}", crate::naming::without_legal_form(bank)),
        None => format!("Konto …{tail}"),
    };
    conn.execute(
        "INSERT INTO accounts (iban, bank, name, created_at) VALUES (?1, ?2, ?3, ?4)",
        params![iban, bank, name, now.to_string()],
    )?;
    Ok(conn.last_insert_rowid())
}

/// Stores what code read of a statement document, in place of an earlier reading: its account,
/// the statement and its bookings. A document that reads as no statement loses an earlier reading.
pub fn store(
    conn: &Connection,
    document: i64,
    lines: &[Line],
    now: Timestamp,
) -> rusqlite::Result<Option<i64>> {
    let earlier: Option<i64> = conn
        .query_row(
            "SELECT id FROM statements WHERE document_id = ?1",
            [document],
            |row| row.get(0),
        )
        .optional()?;
    if let Some(earlier) = earlier {
        release(conn, earlier)?;
        conn.execute("DELETE FROM bookings WHERE statement_id = ?1", [earlier])?;
        conn.execute("DELETE FROM statements WHERE id = ?1", [earlier])?;
    }
    let Some(statement) = read(lines) else {
        return Ok(None);
    };
    let Some(sum) = statement.bookings_sum() else {
        return Ok(None);
    };
    let account = account_of(conn, &statement.iban, statement.bank.as_deref(), now)?;
    conn.execute(
        "INSERT INTO statements (document_id, account_id, layout, number, opening_day, closing_day, opening_cents, closing_cents, bookings_cents, read_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
        params![
            document,
            account,
            statement.layout,
            statement.number,
            statement.opening_day.to_string(),
            statement.closing_day.to_string(),
            statement.opening.value(),
            statement.closing.value(),
            sum.value(),
            now.to_string()
        ],
    )?;
    let id = conn.last_insert_rowid();
    let mut insert = conn.prepare(
        "INSERT INTO bookings (account_id, statement_id, position, day, value_day, amount_cents, kind, other_side, purpose, page, left, bottom, right, top)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14)",
    )?;
    for (position, booking) in statement.bookings.iter().enumerate() {
        insert.execute(params![
            account,
            id,
            i64::try_from(position).unwrap_or(i64::MAX),
            booking.day.to_string(),
            booking.value_day.map(|day| day.to_string()),
            booking.amount.value(),
            booking.kind,
            booking.other_side,
            booking.purpose,
            booking.line.page,
            booking.line.left,
            booking.line.bottom,
            booking.line.right,
            booking.line.top
        ])?;
    }
    join_twins(conn, account, id, statement.layout == layout_b::NAME)?;
    Ok(Some(id))
}

/// The bookings another statement replaced by one of this statement count again, before this
/// statement's bookings go.
fn release(conn: &Connection, statement: i64) -> rusqlite::Result<()> {
    conn.execute(
        "UPDATE bookings SET replaced_by = NULL WHERE replaced_by IN (SELECT id FROM bookings WHERE statement_id = ?1)",
        [statement],
    )?;
    Ok(())
}

/// CSV before PDF (#21): a booking of another statement of the same account with the same day and
/// amount is the same booking when it is the only such one of the day on both sides. The CSV's
/// booking counts, the PDF's is kept as replaced by it. Returns how many were joined.
fn join_twins(
    conn: &Connection,
    account: i64,
    statement: i64,
    csv: bool,
) -> rusqlite::Result<usize> {
    let mine: Vec<(i64, String, i64)> = conn
        .prepare("SELECT id, day, amount_cents FROM bookings WHERE statement_id = ?1 AND replaced_by IS NULL")?
        .query_map([statement], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)))?
        .collect::<rusqlite::Result<_>>()?;
    let mut joined = 0;
    for (id, day, amount) in &mine {
        if mine
            .iter()
            .filter(|(_, d, a)| d == day && a == amount)
            .count()
            != 1
        {
            continue;
        }
        let others: Vec<(i64, String)> = conn
            .prepare(
                "SELECT b.id, s.layout FROM bookings b JOIN statements s ON s.id = b.statement_id
                 WHERE b.account_id = ?1 AND b.statement_id <> ?2 AND b.day = ?3 AND b.amount_cents = ?4 AND b.replaced_by IS NULL",
            )?
            .query_map(params![account, statement, day, amount], |row| Ok((row.get(0)?, row.get(1)?)))?
            .collect::<rusqlite::Result<_>>()?;
        let [(other, layout)] = others.as_slice() else {
            continue;
        };
        let other_csv = layout == layout_b::NAME;
        let (replaced, by) = match (csv, other_csv) {
            (true, false) => (*other, *id),
            (false, true) => (*id, *other),
            _ => continue,
        };
        conn.execute(
            "UPDATE bookings SET replaced_by = ?2 WHERE id = ?1",
            params![replaced, by],
        )?;
        joined += 1;
    }
    Ok(joined)
}

/// Removes what was read of a document as a statement, when it is no statement any more.
pub fn forget(conn: &Connection, document: i64) -> rusqlite::Result<()> {
    let earlier: Option<i64> = conn
        .query_row(
            "SELECT id FROM statements WHERE document_id = ?1",
            [document],
            |row| row.get(0),
        )
        .optional()?;
    if let Some(earlier) = earlier {
        release(conn, earlier)?;
    }
    conn.execute(
        "DELETE FROM bookings WHERE statement_id IN (SELECT id FROM statements WHERE document_id = ?1)",
        [document],
    )?;
    conn.execute("DELETE FROM statements WHERE document_id = ?1", [document])?;
    Ok(())
}

/// The balance check of a statement as stored: `None` when no statement was read of the document,
/// else the difference of old balance plus bookings and the new balance.
pub fn difference_of(
    conn: &Connection,
    document: i64,
) -> rusqlite::Result<Option<(Cents, Cents, Cents, i64)>> {
    conn.query_row(
        "SELECT opening_cents, bookings_cents, closing_cents, (SELECT count(*) FROM bookings b WHERE b.statement_id = s.id)
         FROM statements s WHERE document_id = ?1",
        [document],
        |row| {
            Ok((
                Cents::new(row.get(0)?),
                Cents::new(row.get(1)?),
                Cents::new(row.get(2)?),
                row.get(3)?,
            ))
        },
    )
    .optional()
}

#[cfg(test)]
mod tests;
