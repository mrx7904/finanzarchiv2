//! The commands of the savings hints (#39): the hints the owner did not dismiss, and dismissing one.

use std::sync::{Arc, Mutex};

use serde::Serialize;
use specta::Type;
use tauri::State;

use super::CommandError;
use crate::archive::Opened;
use crate::hints::{self, Hint};
use crate::money::Cents;

/// A hint as the window takes it. `key` and `numbers` go back with "Ausblenden".
#[derive(Debug, Clone, Serialize, Type, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct HintRow {
    /// "rose", "twice", "fees", "above" or "still-runs".
    pub kind: String,
    pub key: String,
    pub numbers: String,
    pub subject: String,
    pub line: String,
    pub cents: i32,
    pub text: String,
    pub bookings: Vec<u32>,
    pub documents: Vec<u32>,
}

fn ids(ids: &[i64]) -> Result<Vec<u32>, CommandError> {
    ids.iter()
        .map(|id| u32::try_from(*id).map_err(|_| CommandError::internal()))
        .collect()
}

pub fn row(hint: &Hint) -> Result<HintRow, CommandError> {
    Ok(HintRow {
        kind: hint.kind.code().to_owned(),
        key: hint.key.clone(),
        numbers: hint.numbers.clone(),
        subject: hint.subject.clone(),
        line: hint.line.clone(),
        cents: i32::try_from(hint.cents).map_err(|_| CommandError::internal())?,
        text: Cents::new(hint.cents).to_string(),
        bookings: ids(&hint.bookings)?,
        documents: ids(&hint.documents)?,
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

/// The savings hints the owner did not dismiss.
#[tauri::command]
#[specta::specta]
pub fn list_hints(archive: State<'_, Arc<Mutex<Opened>>>) -> Result<Vec<HintRow>, CommandError> {
    let today = jiff::tz::Offset::UTC
        .to_datetime(jiff::Timestamp::now())
        .date();
    with_ready(&archive, |conn| {
        hints::found(conn, today)
            .map_err(|_| CommandError::internal())?
            .iter()
            .map(row)
            .collect()
    })
}

/// "Ausblenden": the hint stays away until its numbers change.
#[tauri::command]
#[specta::specta]
pub fn dismiss_hint(
    archive: State<'_, Arc<Mutex<Opened>>>,
    key: String,
    numbers: String,
) -> Result<(), CommandError> {
    with_ready(&archive, |conn| {
        hints::dismiss(conn, &key, &numbers, jiff::Timestamp::now())
            .map_err(|_| CommandError::internal())
    })
}
