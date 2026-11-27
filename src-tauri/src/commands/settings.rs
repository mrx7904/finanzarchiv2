//! The commands of the pages of "Einstellungen" (#49) that no other view had: where the archive
//! lies with its counts and snapshots, and whether the local model is reached.

use std::sync::{Arc, Mutex};

use serde::Serialize;
use specta::Type;
use tauri::State;

use super::CommandError;
use crate::archive::{Opened, snapshots};

#[derive(Debug, Clone, Serialize, Type, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SnapshotRow {
    pub name: String,
    /// UTC, ISO 8601.
    pub taken_at: String,
    /// "täglich", "vor einer Umstellung" or "nach einem Import".
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Type, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ArchiveInfo {
    pub folder: String,
    pub documents: u32,
    pub bookings: u32,
    pub accounts: u32,
    pub snapshots: Vec<SnapshotRow>,
}

/// The reason of a snapshot, from its name.
pub fn reason_of(name: &str) -> &'static str {
    if name.contains("vor-migration") {
        "vor einer Umstellung"
    } else if name.contains("nach-import") {
        "nach einem Import"
    } else {
        "täglich"
    }
}

#[tauri::command]
#[specta::specta]
pub fn archive_info(archive: State<'_, Arc<Mutex<Opened>>>) -> Result<ArchiveInfo, CommandError> {
    let opened = archive.lock().map_err(|_| CommandError::internal())?;
    let Opened::Ready(ready) = &*opened else {
        return Err(CommandError::archive_not_ready());
    };
    let conn = ready.connection();
    let count = |sql: &str| -> Result<u32, CommandError> {
        conn.query_row(sql, [], |row| row.get::<_, u32>(0))
            .map_err(|_| CommandError::internal())
    };
    let snapshots = snapshots::list(ready.path())
        .map_err(|_| CommandError::internal())?
        .into_iter()
        .map(|snapshot| {
            let name = snapshot
                .path
                .file_name()
                .map(|name| name.to_string_lossy().into_owned())
                .unwrap_or_default();
            SnapshotRow {
                reason: reason_of(&name).to_owned(),
                taken_at: snapshot.taken_at.to_string(),
                name,
            }
        })
        .collect();
    Ok(ArchiveInfo {
        folder: ready
            .path()
            .parent()
            .map(|folder| folder.display().to_string())
            .unwrap_or_default(),
        documents: count("SELECT count(*) FROM documents")?,
        bookings: count("SELECT count(*) FROM bookings WHERE replaced_by IS NULL")?,
        accounts: count("SELECT count(*) FROM accounts")?,
        snapshots,
    })
}

#[derive(Debug, Clone, Serialize, Type, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ModelStatus {
    pub name: String,
    pub address: String,
    pub reached: bool,
    pub installed: bool,
    /// Why it is not ready, German, when it is not.
    pub problem: Option<String>,
}

/// Whether the local model answers: "Verbindung testen" calls it again.
#[tauri::command]
#[specta::specta]
pub async fn model_status(
    archive: State<'_, Arc<Mutex<Opened>>>,
) -> Result<ModelStatus, CommandError> {
    let name = {
        let opened = archive.lock().map_err(|_| CommandError::internal())?;
        let Opened::Ready(ready) = &*opened else {
            return Err(CommandError::archive_not_ready());
        };
        crate::readers::model::configured(ready.connection())
    };
    tauri::async_runtime::spawn_blocking(move || {
        let model = match crate::ollama::Ollama::from_env(&name) {
            Ok(model) => model,
            Err(error) => {
                return ModelStatus {
                    name,
                    address: String::new(),
                    reached: false,
                    installed: false,
                    problem: Some(error.to_string()),
                };
            }
        };
        let address = model.base().to_owned();
        match model.installed() {
            Ok(true) => ModelStatus {
                name,
                address,
                reached: true,
                installed: true,
                problem: None,
            },
            Ok(false) => ModelStatus {
                problem: Some(crate::ollama::ModelError::NotInstalled(name.clone()).to_string()),
                name,
                address,
                reached: true,
                installed: false,
            },
            Err(error) => ModelStatus {
                name,
                address,
                reached: false,
                installed: false,
                problem: Some(error.to_string()),
            },
        }
    })
    .await
    .map_err(|_| CommandError::internal())
}

#[cfg(test)]
mod tests {
    #[test]
    fn a_snapshot_says_why_it_was_taken() {
        assert_eq!(
            super::reason_of("finanzarchiv-20261127T080000Z-vor-migration.db"),
            "vor einer Umstellung"
        );
        assert_eq!(
            super::reason_of("finanzarchiv-20261127T080000Z-nach-import.db"),
            "nach einem Import"
        );
        assert_eq!(
            super::reason_of("finanzarchiv-20261127T080000Z-taeglich.db"),
            "täglich"
        );
    }
}
