//! Commands of the document store (#04) and the intake queue (#05). The window never sends a
//! path: the core asks the system dialog itself or receives the dropped paths from the window event.

use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use serde::{Deserialize, Serialize};
use specta::Type;
use tauri::{AppHandle, State};
use tauri_plugin_dialog::DialogExt;
use tauri_specta::Event;

use super::CommandError;
use crate::archive::Opened;
use crate::documents::{self, Source};
use crate::intake::{Intake, jobs};
use crate::previews;

/// What the dialog offers. A CSV is taken only when it reads as a statement (#21).
const EXTENSIONS: &[&str] = &["pdf", "png", "jpg", "jpeg", "webp", "csv"];

/// Sent whenever the queue changed: jobs were added, one moved on, or the pause was switched.
#[derive(Debug, Clone, Default, Serialize, Deserialize, Type, Event)]
pub struct IntakeChanged;

/// How many files an import put into the queue.
#[derive(Debug, Clone, Serialize, Type, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Queued {
    pub files: u32,
}

#[derive(Debug, Clone, Serialize, Type, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct JobRow {
    pub id: u32,
    pub name: String,
    /// The stage as its code: queued, stored, text, classified, read, model, checked, failed.
    pub stage: String,
    /// Why a failed file was not taken, or why a job waits for the model, as a German sentence.
    pub error: Option<String>,
}

/// What the window shows under "Eingang".
#[derive(Debug, Clone, Serialize, Type, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct IntakeOverview {
    pub paused: bool,
    pub open: u32,
    /// The oldest open jobs, at most 50.
    pub running: Vec<JobRow>,
    pub failed: Vec<JobRow>,
    /// New documents of the last 24 hours.
    pub added_today: u32,
}

#[derive(Debug, Clone, Serialize, Type, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct DocumentRow {
    pub id: u32,
    pub name: String,
    pub kind: String,
    pub file_type: String,
    pub pages: Option<u32>,
    /// UTC, ISO 8601.
    pub imported_at: String,
    /// text, scan or image, null while the intake has not read it.
    pub text_state: Option<String>,
    /// open (not checked yet), filed or waiting (#16).
    pub state: String,
    /// The name of the file as it came in, which the preview keeps beside the name (#18).
    pub original_name: String,
    /// Whether the owner typed the name.
    pub name_by_owner: bool,
    /// Why the downloads folder took it (#28), null for every other way in.
    pub arrival: Option<String>,
}

/// One result of a check, as "Warum?" shows it.
#[derive(Debug, Clone, Serialize, Type, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct CheckRow {
    /// The stable id, for example `check:agree`.
    pub check: String,
    pub field: Option<String>,
    pub passed: bool,
    /// The German sentence of the result.
    pub detail: Option<String>,
}

/// The checks of a document and the decision that filed it.
#[derive(Debug, Clone, Serialize, Type, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct DocumentChecks {
    pub state: String,
    /// The rule that filed it, null when it waits or was never checked.
    pub rule: Option<String>,
    /// UTC, ISO 8601.
    pub decided_at: Option<String>,
    /// The failed checks first.
    pub checks: Vec<CheckRow>,
}

fn enqueue(intake: &Intake, paths: &[PathBuf], source: Source) -> Result<Queued, CommandError> {
    if paths.is_empty() {
        return Ok(Queued { files: 0 });
    }
    intake
        .enqueue(paths, source)
        .map(|files| Queued { files })
        .map_err(|_| CommandError::archive_not_ready())
}

/// Puts dropped files into the queue. Called by the window event, not by the window.
pub fn enqueue_dropped(intake: &Intake, paths: &[PathBuf]) {
    if enqueue(intake, paths, Source::Drop).is_err() {
        eprintln!("Gezogene Dateien konnten nicht in den Eingang");
    }
}

/// Asks for files in the dialog of the system and puts them into the queue.
#[tauri::command]
#[specta::specta]
pub async fn import_files(
    app: AppHandle,
    intake: State<'_, Arc<Intake>>,
) -> Result<Queued, CommandError> {
    let picked = app
        .dialog()
        .file()
        .set_title("Dokumente importieren")
        .add_filter("Dokumente", EXTENSIONS)
        .blocking_pick_files();
    let paths: Vec<PathBuf> = picked
        .unwrap_or_default()
        .into_iter()
        .filter_map(|file| file.into_path().ok())
        .collect();
    enqueue(&intake, &paths, Source::Dialog)
}

/// Asks for a folder in the dialog of the system and puts everything in it into the queue.
#[tauri::command]
#[specta::specta]
pub async fn import_folder(
    app: AppHandle,
    intake: State<'_, Arc<Intake>>,
) -> Result<Queued, CommandError> {
    let picked = app
        .dialog()
        .file()
        .set_title("Ordner importieren")
        .blocking_pick_folder();
    let paths: Vec<PathBuf> = picked
        .and_then(|folder| folder.into_path().ok())
        .into_iter()
        .collect();
    enqueue(&intake, &paths, Source::Dialog)
}

fn job_row(job: jobs::Listed) -> Result<JobRow, CommandError> {
    Ok(JobRow {
        id: u32::try_from(job.id).map_err(|_| CommandError::internal())?,
        name: job.name,
        stage: job.stage.code().to_owned(),
        error: job.error,
    })
}

/// The state of the queue for "Eingang".
#[tauri::command]
#[specta::specta]
pub fn intake_overview(intake: State<'_, Arc<Intake>>) -> Result<IntakeOverview, CommandError> {
    let guard = intake
        .archive()
        .lock()
        .map_err(|_| CommandError::internal())?;
    let Opened::Ready(ready) = &*guard else {
        return Err(CommandError::archive_not_ready());
    };
    let conn = ready.connection();
    let database = |_| CommandError::internal();
    let day_ago = jiff::Timestamp::now() - jiff::SignedDuration::from_hours(24);
    Ok(IntakeOverview {
        paused: intake.is_paused(),
        open: jobs::count_open(conn).map_err(database)?,
        running: jobs::listed(conn, true, 50)
            .map_err(database)?
            .into_iter()
            .map(job_row)
            .collect::<Result<_, _>>()?,
        failed: jobs::listed(conn, false, 200)
            .map_err(database)?
            .into_iter()
            .map(job_row)
            .collect::<Result<_, _>>()?,
        added_today: jobs::count_added_since(conn, day_ago).map_err(database)?,
    })
}

/// Pauses the queue or lets it go on. A running job finishes its stage and keeps it.
#[tauri::command]
#[specta::specta]
pub fn set_intake_paused(intake: State<'_, Arc<Intake>>, paused: bool) {
    intake.set_paused(paused);
}

/// Takes the failed files off the list under "Eingang".
#[tauri::command]
#[specta::specta]
pub fn dismiss_failed(intake: State<'_, Arc<Intake>>) -> Result<(), CommandError> {
    {
        let guard = intake
            .archive()
            .lock()
            .map_err(|_| CommandError::internal())?;
        let Opened::Ready(ready) = &*guard else {
            return Err(CommandError::archive_not_ready());
        };
        jobs::dismiss_failed(ready.connection()).map_err(|_| CommandError::internal())?;
    }
    intake.set_paused(intake.is_paused());
    Ok(())
}

/// A page of a document as an image for the window, as a data URL. The width is rounded up to
/// one of `previews::WIDTHS`.
#[tauri::command]
#[specta::specta]
pub fn page_image(
    archive: State<'_, Arc<Mutex<Opened>>>,
    document: u32,
    page: u32,
    width: u32,
) -> Result<String, CommandError> {
    use base64::Engine;
    let (original, file_type, cache) = {
        let opened = archive.lock().map_err(|_| CommandError::internal())?;
        let Opened::Ready(ready) = &*opened else {
            return Err(CommandError::archive_not_ready());
        };
        let root = documents::archive_root(ready.path());
        let stored = documents::stored_of(ready.connection(), &root, i64::from(document))
            .map_err(|_| CommandError::internal())?
            .ok_or_else(CommandError::not_found)?;
        (stored.0, stored.1, previews::folder_of(ready.path()))
    };
    let png = previews::page_png(
        &cache,
        &original,
        file_type,
        crate::pdf::engine().ok(),
        page,
        width,
    )
    .map_err(|_| CommandError::not_found())?;
    Ok(format!(
        "data:image/png;base64,{}",
        base64::engine::general_purpose::STANDARD.encode(png)
    ))
}

/// The documents of the archive, newest first.
#[tauri::command]
#[specta::specta]
pub fn list_documents(
    archive: State<'_, Arc<Mutex<Opened>>>,
) -> Result<Vec<DocumentRow>, CommandError> {
    let opened = archive.lock().map_err(|_| CommandError::internal())?;
    let Opened::Ready(ready) = &*opened else {
        return Ok(Vec::new());
    };
    let rows = documents::list(ready.connection()).map_err(|_| CommandError::internal())?;
    rows.into_iter()
        .map(|row| {
            Ok(DocumentRow {
                id: u32::try_from(row.id).map_err(|_| CommandError::internal())?,
                name: row.name,
                kind: row.kind,
                file_type: row.file_type,
                pages: row.pages.and_then(|pages| u32::try_from(pages).ok()),
                imported_at: row.imported_at,
                text_state: row.text_state,
                state: row.state,
                original_name: row.original_name,
                name_by_owner: row.name_by_owner,
                arrival: row.arrival,
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nothing_is_queued_into_an_archive_that_is_not_ready() {
        let archive = Arc::new(Mutex::new(Opened::Damaged {
            snapshots: Vec::new(),
        }));
        let intake = Intake::new(archive, Box::new(|| {}));
        let error = enqueue(&intake, &[PathBuf::from("egal.pdf")], Source::Drop).unwrap_err();
        assert!(error.message.contains("nicht bereit"));
    }

    #[test]
    fn an_empty_pick_queues_nothing() {
        let archive = Arc::new(Mutex::new(Opened::Damaged {
            snapshots: Vec::new(),
        }));
        let intake = Intake::new(archive, Box::new(|| {}));
        assert_eq!(
            enqueue(&intake, &[], Source::Dialog).unwrap(),
            Queued { files: 0 }
        );
    }
}

fn checks_of(conn: &rusqlite::Connection, document: i64) -> Result<DocumentChecks, CommandError> {
    let shown = crate::checks::shown(conn, document).map_err(|error| match error {
        rusqlite::Error::QueryReturnedNoRows => CommandError::not_found(),
        _ => CommandError::internal(),
    })?;
    Ok(DocumentChecks {
        state: shown.state,
        rule: shown.rule,
        decided_at: shown.decided_at,
        checks: shown
            .outcomes
            .into_iter()
            .map(|(check, field, passed, detail)| CheckRow {
                check,
                field,
                passed,
                detail,
            })
            .collect(),
    })
}

/// The checks of a document for "Warum?" (#16).
#[tauri::command]
#[specta::specta]
pub fn document_checks(
    archive: State<'_, Arc<Mutex<Opened>>>,
    document: u32,
) -> Result<DocumentChecks, CommandError> {
    let opened = archive.lock().map_err(|_| CommandError::internal())?;
    let Opened::Ready(ready) = &*opened else {
        return Err(CommandError::archive_not_ready());
    };
    checks_of(ready.connection(), i64::from(document))
}

/// "Rückgängig": the document that was filed by itself waits for the owner again.
#[tauri::command]
#[specta::specta]
pub fn undo_filing(
    archive: State<'_, Arc<Mutex<Opened>>>,
    document: u32,
) -> Result<DocumentChecks, CommandError> {
    let opened = archive.lock().map_err(|_| CommandError::internal())?;
    let Opened::Ready(ready) = &*opened else {
        return Err(CommandError::archive_not_ready());
    };
    crate::checks::undo(
        ready.connection(),
        i64::from(document),
        jiff::Timestamp::now(),
    )
    .map_err(|_| CommandError::internal())?;
    checks_of(ready.connection(), i64::from(document))
}
