//! The commands of "Postfächer" (#29): add a mailbox after its login worked, test it, remove it.
//! The app password goes from the window straight to the login and the keychain, never further.

use std::sync::{Arc, Mutex};

use serde::Serialize;
use specta::Type;
use tauri::State;

use super::CommandError;
use crate::archive::Opened;
use crate::mail::{self, MailboxError, reader::Reader, secrets::Keychain};

/// A mailbox as "Postfächer" shows it.
#[derive(Debug, Clone, Serialize, Type, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct MailboxRow {
    pub id: u32,
    pub address: String,
    /// When the login was last tried, UTC, ISO 8601.
    pub checked_at: Option<String>,
    /// Why the last login failed, in German, null when it worked.
    pub problem: Option<String>,
    /// When the last round through it ended, UTC, ISO 8601.
    pub read_at: Option<String>,
    /// Mails looked at so far (#31).
    pub looked: u32,
    /// Mails that gave the archive a document.
    pub taken: u32,
}

fn rows(archive: &Mutex<Opened>) -> Result<Vec<MailboxRow>, CommandError> {
    let opened = archive.lock().map_err(|_| CommandError::internal())?;
    let Opened::Ready(ready) = &*opened else {
        return Err(CommandError::archive_not_ready());
    };
    mail::list(ready.connection())
        .map_err(|_| CommandError::internal())?
        .into_iter()
        .map(|mailbox| {
            let (looked, taken) = mail::reader::totals(ready.connection(), mailbox.id)
                .map_err(|_| CommandError::internal())?;
            Ok(MailboxRow {
                id: u32::try_from(mailbox.id).map_err(|_| CommandError::internal())?,
                address: mailbox.address,
                checked_at: mailbox.checked_at,
                problem: mailbox.problem,
                read_at: mailbox.read_at,
                looked,
                taken,
            })
        })
        .collect()
}

fn refused(error: MailboxError) -> CommandError {
    match error {
        MailboxError::Database => CommandError::internal(),
        other => CommandError::refused(other.to_string()),
    }
}

#[tauri::command]
#[specta::specta]
pub fn list_mailboxes(
    archive: State<'_, Arc<Mutex<Opened>>>,
) -> Result<Vec<MailboxRow>, CommandError> {
    rows(&archive)
}

/// Logs in once and adds the mailbox only when that worked. Its first round starts at once.
#[tauri::command]
#[specta::specta]
pub async fn add_mailbox(
    archive: State<'_, Arc<Mutex<Opened>>>,
    reader: State<'_, Arc<Reader>>,
    address: String,
    password: String,
) -> Result<Vec<MailboxRow>, CommandError> {
    mail::add(
        &archive,
        &Keychain,
        &address,
        &password,
        jiff::Timestamp::now(),
        mail::login_gmail,
    )
    .await
    .map_err(refused)?;
    let reader = Arc::clone(&reader);
    std::thread::spawn(move || reader.round());
    rows(&archive)
}

/// Reads every mailbox now, as the round every 15 minutes would.
#[tauri::command]
#[specta::specta]
pub async fn read_mail_now(
    archive: State<'_, Arc<Mutex<Opened>>>,
    reader: State<'_, Arc<Reader>>,
) -> Result<Vec<MailboxRow>, CommandError> {
    let reader = Arc::clone(&reader);
    let ran = tauri::async_runtime::spawn_blocking(move || reader.round())
        .await
        .map_err(|_| CommandError::internal())?;
    if !ran {
        return Err(CommandError::refused(
            "Die Postfächer werden gerade gelesen.".into(),
        ));
    }
    rows(&archive)
}

/// Tries the login again. A failed login is shown with the mailbox, not as a failure.
#[tauri::command]
#[specta::specta]
pub async fn test_mailbox(
    archive: State<'_, Arc<Mutex<Opened>>>,
    id: u32,
) -> Result<Vec<MailboxRow>, CommandError> {
    mail::test(
        &archive,
        &Keychain,
        i64::from(id),
        jiff::Timestamp::now(),
        mail::login_gmail,
    )
    .await
    .map_err(refused)?;
    rows(&archive)
}

#[tauri::command]
#[specta::specta]
pub fn remove_mailbox(
    archive: State<'_, Arc<Mutex<Opened>>>,
    id: u32,
) -> Result<Vec<MailboxRow>, CommandError> {
    mail::remove(&archive, &Keychain, i64::from(id)).map_err(refused)?;
    rows(&archive)
}
