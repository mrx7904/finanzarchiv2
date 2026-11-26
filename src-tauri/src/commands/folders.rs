//! The commands of "Ordner" (#27): the watched folders, the inbox, adding, removing and opening.

use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use serde::Serialize;
use specta::Type;
use tauri::{AppHandle, State};
use tauri_plugin_dialog::DialogExt;

use super::CommandError;
use crate::archive::Opened;
use crate::folders::{self, FolderError, Watcher};

/// A watched folder as "Ordner" shows it.
#[derive(Debug, Clone, Serialize, Type, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct FolderRow {
    pub id: u32,
    pub path: String,
    pub is_inbox: bool,
    /// Why the folder is not watched right now, in German.
    pub problem: Option<String>,
}

fn rows(archive: &Mutex<Opened>) -> Result<Vec<FolderRow>, CommandError> {
    let opened = archive.lock().map_err(|_| CommandError::internal())?;
    let Opened::Ready(ready) = &*opened else {
        return Err(CommandError::archive_not_ready());
    };
    folders::list(ready.connection())
        .map_err(|_| CommandError::internal())?
        .into_iter()
        .map(|folder| {
            Ok(FolderRow {
                id: u32::try_from(folder.id).map_err(|_| CommandError::internal())?,
                path: folder.path.to_string_lossy().into_owned(),
                is_inbox: folder.is_inbox,
                problem: folder.problem,
            })
        })
        .collect()
}

fn refused(error: FolderError) -> CommandError {
    match error {
        FolderError::Database => CommandError::internal(),
        other => CommandError::refused(other.to_string()),
    }
}

fn change(
    archive: &Mutex<Opened>,
    watcher: &Arc<Watcher>,
    run: impl FnOnce(&rusqlite::Connection, &std::path::Path) -> Result<(), FolderError>,
) -> Result<Vec<FolderRow>, CommandError> {
    {
        let opened = archive.lock().map_err(|_| CommandError::internal())?;
        let Opened::Ready(ready) = &*opened else {
            return Err(CommandError::archive_not_ready());
        };
        let archive_dir = ready.path().parent().map(PathBuf::from).unwrap_or_default();
        run(ready.connection(), &archive_dir).map_err(refused)?;
    }
    watcher.refresh();
    rows(archive)
}

fn pick(app: &AppHandle, title: &str) -> Option<PathBuf> {
    app.dialog()
        .file()
        .set_title(title)
        .blocking_pick_folder()
        .and_then(|folder| folder.into_path().ok())
}

#[tauri::command]
#[specta::specta]
pub fn list_folders(
    archive: State<'_, Arc<Mutex<Opened>>>,
) -> Result<Vec<FolderRow>, CommandError> {
    rows(&archive)
}

/// Asks for a folder and watches it.
#[tauri::command]
#[specta::specta]
pub async fn add_folder(
    app: AppHandle,
    archive: State<'_, Arc<Mutex<Opened>>>,
    watcher: State<'_, Arc<Watcher>>,
) -> Result<Vec<FolderRow>, CommandError> {
    let Some(path) = pick(&app, "Ordner beobachten") else {
        return rows(&archive);
    };
    change(&archive, &watcher, |conn, archive_dir| {
        folders::add(conn, archive_dir, &path, jiff::Timestamp::now()).map(|_| ())
    })
}

#[tauri::command]
#[specta::specta]
pub fn remove_folder(
    archive: State<'_, Arc<Mutex<Opened>>>,
    watcher: State<'_, Arc<Watcher>>,
    folder: u32,
) -> Result<Vec<FolderRow>, CommandError> {
    change(&archive, &watcher, |conn, _| {
        folders::remove(conn, i64::from(folder))
    })
}

/// Asks for another place of the inbox.
#[tauri::command]
#[specta::specta]
pub async fn move_inbox(
    app: AppHandle,
    archive: State<'_, Arc<Mutex<Opened>>>,
    watcher: State<'_, Arc<Watcher>>,
) -> Result<Vec<FolderRow>, CommandError> {
    let Some(path) = pick(&app, "Anderen Ort für den Eingang wählen") else {
        return rows(&archive);
    };
    change(&archive, &watcher, |conn, archive_dir| {
        folders::move_inbox(conn, archive_dir, &path)
    })
}

/// Opens a watched folder in the file manager of the system. It takes the id, so the window can
/// open no other path.
#[tauri::command]
#[specta::specta]
pub fn open_folder(
    archive: State<'_, Arc<Mutex<Opened>>>,
    folder: u32,
) -> Result<(), CommandError> {
    let path = rows(&archive)?
        .into_iter()
        .find(|row| row.id == folder)
        .map(|row| PathBuf::from(row.path))
        .ok_or_else(CommandError::not_found)?;
    let _ = std::fs::create_dir_all(&path);
    let program = match std::env::consts::OS {
        "windows" => "explorer",
        "macos" => "open",
        _ => "xdg-open",
    };
    std::process::Command::new(program)
        .arg(&path)
        .spawn()
        .map(|_| ())
        .map_err(|_| CommandError::internal())
}
