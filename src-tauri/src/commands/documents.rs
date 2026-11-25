//! Commands of the document store (#04). The window never sends a path: the core asks the
//! system dialog itself or receives the dropped paths from the window event.

use std::path::PathBuf;
use std::sync::Mutex;

use serde::{Deserialize, Serialize};
use specta::Type;
use tauri::{AppHandle, State};
use tauri_plugin_dialog::DialogExt;
use tauri_specta::Event;

use super::CommandError;
use crate::archive::Opened;
use crate::documents::{self, FileResult, Outcome, Source};
use crate::pdf;

const EXTENSIONS: &[&str] = &["pdf", "png", "jpg", "jpeg", "webp"];

/// What one import did, for the line the window shows after it.
#[derive(Debug, Clone, Default, Serialize, Deserialize, Type, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ImportSummary {
    pub added: u32,
    pub known: u32,
    pub refused: Vec<RefusedFile>,
    /// Set when nothing could be imported at all, as a German sentence.
    pub problem: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RefusedFile {
    pub name: String,
    pub reason: String,
}

/// Sent when files dropped on the window were imported.
#[derive(Debug, Clone, Serialize, Deserialize, Type, Event)]
#[serde(rename_all = "camelCase")]
pub struct ImportFinished {
    pub summary: ImportSummary,
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
}

pub fn summarize(results: &[FileResult]) -> ImportSummary {
    let mut summary = ImportSummary::default();
    for result in results {
        match &result.outcome {
            Outcome::Added { .. } => summary.added += 1,
            Outcome::Known { .. } => summary.known += 1,
            Outcome::Refused(refusal) => summary.refused.push(RefusedFile {
                name: result.name.clone(),
                reason: refusal.to_string(),
            }),
        }
    }
    summary
}

/// Imports the paths into the open archive. Used by the dialog commands and the drop handler.
pub fn import(archive: &Mutex<Opened>, paths: &[PathBuf], source: Source) -> ImportSummary {
    let Ok(opened) = archive.lock() else {
        return ImportSummary {
            problem: Some(CommandError::internal().message),
            ..ImportSummary::default()
        };
    };
    let Opened::Ready(ready) = &*opened else {
        return ImportSummary {
            problem: Some("Das Archiv ist nicht bereit, deshalb wurde nichts importiert.".into()),
            ..ImportSummary::default()
        };
    };
    let root = documents::archive_root(ready.path());
    let results = documents::import_paths(
        ready.connection(),
        &root,
        pdf::engine().ok(),
        paths,
        source,
        jiff::Timestamp::now(),
    );
    summarize(&results)
}

/// Asks for files in the dialog of the system and imports them.
#[tauri::command]
#[specta::specta]
pub async fn import_files(
    app: AppHandle,
    archive: State<'_, Mutex<Opened>>,
) -> Result<ImportSummary, CommandError> {
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
    Ok(import(&archive, &paths, Source::Dialog))
}

/// Asks for a folder in the dialog of the system and imports everything in it.
#[tauri::command]
#[specta::specta]
pub async fn import_folder(
    app: AppHandle,
    archive: State<'_, Mutex<Opened>>,
) -> Result<ImportSummary, CommandError> {
    let picked = app
        .dialog()
        .file()
        .set_title("Ordner importieren")
        .blocking_pick_folder();
    let paths: Vec<PathBuf> = picked
        .and_then(|folder| folder.into_path().ok())
        .into_iter()
        .collect();
    Ok(import(&archive, &paths, Source::Dialog))
}

/// The documents of the archive, newest first.
#[tauri::command]
#[specta::specta]
pub fn list_documents(archive: State<'_, Mutex<Opened>>) -> Result<Vec<DocumentRow>, CommandError> {
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
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::documents::Refusal;

    #[test]
    fn a_summary_counts_and_names_each_refused_file() {
        let results = [
            FileResult {
                name: "a.pdf".into(),
                outcome: Outcome::Added { id: 1 },
            },
            FileResult {
                name: "a-kopie.pdf".into(),
                outcome: Outcome::Known { id: 1 },
            },
            FileResult {
                name: "notiz.txt".into(),
                outcome: Outcome::Refused(Refusal::UnknownType),
            },
        ];
        let summary = summarize(&results);
        assert_eq!((summary.added, summary.known), (1, 1));
        assert_eq!(summary.refused.len(), 1);
        assert_eq!(summary.refused[0].name, "notiz.txt");
        assert!(
            summary.refused[0]
                .reason
                .contains("PDF, PNG, JPEG und WebP")
        );
    }

    #[test]
    fn nothing_is_imported_into_an_archive_that_is_not_ready() {
        let archive = Mutex::new(Opened::Damaged {
            snapshots: Vec::new(),
        });
        let summary = import(&archive, &[PathBuf::from("egal.pdf")], Source::Drop);
        assert_eq!(summary.added, 0);
        assert!(summary.problem.unwrap().contains("nicht bereit"));
    }
}
