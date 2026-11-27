//! The commands of the recurring payments (#34): what comes again, and "ist keine" of the owner.
//! Amounts cross as whole cents (`i32`) and as German text, days as `YYYY-MM-DD`.

use std::sync::{Arc, Mutex};

use serde::Serialize;
use specta::Type;
use tauri::State;

use super::CommandError;
use crate::archive::Opened;
use crate::money::Cents;
use crate::recurring::{self, Recurring, Rhythm};

/// A recurring payment as the window takes it.
#[derive(Debug, Clone, Serialize, Type, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RecurringRow {
    pub party: u32,
    pub name: String,
    pub incoming: bool,
    /// "monatlich", "vierteljährlich" or "jährlich".
    pub rhythm: String,
    pub since: String,
    pub last: String,
    pub next: Option<String>,
    pub count: u32,
    pub cents: i32,
    pub text: String,
    pub per_year_cents: i32,
    pub per_year_text: String,
    /// "seit März 2026 2,00 € mehr".
    pub change: Option<String>,
    pub ended: bool,
}

fn cents32(cents: i64) -> Result<i32, CommandError> {
    i32::try_from(cents).map_err(|_| CommandError::internal())
}

pub fn row(found: &Recurring) -> Result<RecurringRow, CommandError> {
    Ok(RecurringRow {
        party: u32::try_from(found.party).map_err(|_| CommandError::internal())?,
        name: found.name.clone(),
        incoming: found.incoming,
        rhythm: found.rhythm.name().to_owned(),
        since: found.since.to_string(),
        last: found.last.to_string(),
        next: found.next.map(|day| day.to_string()),
        count: found.count,
        cents: cents32(found.cents)?,
        text: Cents::new(found.cents).to_string(),
        per_year_cents: cents32(found.per_year)?,
        per_year_text: Cents::new(found.per_year).to_string(),
        change: found.change_text(),
        ended: found.ended,
    })
}

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

fn today() -> jiff::civil::Date {
    jiff::tz::Offset::UTC
        .to_datetime(jiff::Timestamp::now())
        .date()
}

/// The recurring payments, the running first.
#[tauri::command]
#[specta::specta]
pub fn list_recurring(
    archive: State<'_, Arc<Mutex<Opened>>>,
) -> Result<Vec<RecurringRow>, CommandError> {
    with_ready(&archive, |conn| {
        recurring::found(conn, today())
            .map_err(|_| CommandError::internal())?
            .iter()
            .map(row)
            .collect()
    })
}

/// "Ist keine": the owner says a found payment is none. Named by party, direction, rhythm and the
/// day of its first payment, as `list_recurring` gave them.
#[tauri::command]
#[specta::specta]
pub fn not_recurring(
    archive: State<'_, Arc<Mutex<Opened>>>,
    party: u32,
    incoming: bool,
    rhythm: String,
    since: String,
) -> Result<(), CommandError> {
    let rhythm = Rhythm::from_name(&rhythm).ok_or_else(CommandError::not_found)?;
    let since = since
        .parse::<jiff::civil::Date>()
        .map_err(|_| CommandError::not_found())?;
    with_ready(&archive, |conn| {
        recurring::reject(
            conn,
            i64::from(party),
            incoming,
            rhythm,
            since,
            jiff::Timestamp::now(),
        )
        .map_err(|_| CommandError::internal())
    })
}
