//! The commands of names by template (#18).

use std::sync::{Arc, Mutex};

use serde::Serialize;
use specta::Type;
use tauri::State;

use super::CommandError;
use crate::archive::Opened;
use crate::kinds::Kind;
use crate::naming;

/// The template of one kind, as the owner edits it.
#[derive(Debug, Clone, Serialize, Type, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct NameTemplate {
    pub kind: String,
    pub template: String,
    pub default_template: String,
}

/// A placeholder with what it stands for.
#[derive(Debug, Clone, Serialize, Type, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Placeholder {
    pub name: String,
    pub meaning: String,
}

#[derive(Debug, Clone, Serialize, Type, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct NameTemplates {
    pub templates: Vec<NameTemplate>,
    pub placeholders: Vec<Placeholder>,
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

fn templates(conn: &rusqlite::Connection) -> Result<NameTemplates, CommandError> {
    let templates = naming::NAMED_KINDS
        .iter()
        .map(|kind| {
            Ok(NameTemplate {
                kind: kind.code().to_owned(),
                template: naming::template(conn, *kind)
                    .map_err(|_| CommandError::internal())?
                    .unwrap_or_default(),
                default_template: naming::default_template(*kind)
                    .unwrap_or_default()
                    .to_owned(),
            })
        })
        .collect::<Result<_, CommandError>>()?;
    Ok(NameTemplates {
        templates,
        placeholders: naming::PLACEHOLDERS
            .iter()
            .map(|(name, meaning)| Placeholder {
                name: (*name).to_owned(),
                meaning: (*meaning).to_owned(),
            })
            .collect(),
    })
}

/// The templates of the names with their placeholders.
#[tauri::command]
#[specta::specta]
pub fn name_templates(
    archive: State<'_, Arc<Mutex<Opened>>>,
) -> Result<NameTemplates, CommandError> {
    with_ready(&archive, templates)
}

/// Stores the owner's template of a kind (empty restores the default) and names the filed
/// documents of that kind again, those the owner named excepted.
#[tauri::command]
#[specta::specta]
pub fn set_name_template(
    archive: State<'_, Arc<Mutex<Opened>>>,
    kind: String,
    template: String,
) -> Result<NameTemplates, CommandError> {
    with_ready(&archive, |conn| {
        let kind = Kind::from_code(&kind);
        naming::set_template(conn, kind, &template)
            .map_err(|error| CommandError::refused(error.to_string()))?;
        let filed: Vec<i64> = conn
            .prepare("SELECT id FROM documents WHERE kind = ?1 AND state = 'filed' AND name_by_owner = 0")
            .and_then(|mut statement| {
                statement
                    .query_map([kind.code()], |row| row.get(0))?
                    .collect()
            })
            .map_err(|_| CommandError::internal())?;
        for document in filed {
            naming::name_document(conn, document).map_err(|_| CommandError::internal())?;
        }
        templates(conn)
    })
}

/// The owner's name of a document, kept from now on. Empty gives it back to its template.
#[tauri::command]
#[specta::specta]
pub fn rename_document(
    archive: State<'_, Arc<Mutex<Opened>>>,
    document: u32,
    name: String,
) -> Result<Option<String>, CommandError> {
    with_ready(&archive, |conn| {
        naming::rename(conn, i64::from(document), &name).map_err(|_| CommandError::internal())
    })
}
