//! The commands the window may call. Each is written once here, the TypeScript bindings in
//! `src/bindings.ts` are generated from them (FA-06), and a test fails when they drift apart.

pub mod balances;
pub mod chat;
pub mod document_view;
pub mod documents;
pub mod evaluations;
pub mod folders;
pub mod hints;
pub mod mail;
pub mod money;
pub mod names;
pub mod recurring;
pub mod review;
pub mod salary;
pub mod settings;
pub mod taxes;

use std::sync::{Arc, Mutex};

use serde::Serialize;
use specta::Type;
use tauri::State;

use crate::archive::{Opened, Snapshot};

/// What went wrong, as a kind the window can act on and a German sentence for the owner.
/// It never carries the text of a library, which could quote a document.
#[derive(Debug, Clone, Serialize, Type, PartialEq, Eq)]
pub struct CommandError {
    pub kind: ErrorKind,
    pub message: String,
}

#[derive(Debug, Clone, Copy, Serialize, Type, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum ErrorKind {
    /// A fault of the app itself. Restarting it is the owner's way out.
    Internal,
    /// The archive is damaged or missing, so nothing can be written.
    ArchiveNotReady,
    /// What was asked for is not in the archive.
    NotFound,
    /// The owner's answer was not taken, the message says why (#17).
    Refused,
}

impl CommandError {
    pub(crate) fn refused(message: String) -> Self {
        Self {
            kind: ErrorKind::Refused,
            message,
        }
    }

    pub(crate) fn internal() -> Self {
        Self {
            kind: ErrorKind::Internal,
            message: "Ein interner Fehler ist aufgetreten. Bitte starte Finanzarchiv neu.".into(),
        }
    }

    pub(crate) fn not_found() -> Self {
        Self {
            kind: ErrorKind::NotFound,
            message: "Das Dokument oder die Seite gibt es nicht.".into(),
        }
    }

    pub(crate) fn archive_not_ready() -> Self {
        Self {
            kind: ErrorKind::ArchiveNotReady,
            message: "Das Archiv ist nicht bereit, deshalb wurde nichts importiert.".into(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Type, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct AppStatus {
    pub version: String,
    pub archive: ArchiveStatus,
}

/// The state of the archive as the window shows it.
#[derive(Debug, Clone, Serialize, Type, PartialEq, Eq)]
#[serde(tag = "state", rename_all = "camelCase")]
pub enum ArchiveStatus {
    Ready,
    Damaged { snapshots: Vec<SnapshotInfo> },
    Missing { snapshots: Vec<SnapshotInfo> },
}

#[derive(Debug, Clone, Serialize, Type, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SnapshotInfo {
    pub name: String,
    /// UTC, ISO 8601.
    pub taken_at: String,
}

impl From<&Snapshot> for SnapshotInfo {
    fn from(snapshot: &Snapshot) -> Self {
        Self {
            name: snapshot
                .path
                .file_name()
                .map(|name| name.to_string_lossy().into_owned())
                .unwrap_or_default(),
            taken_at: snapshot.taken_at.to_string(),
        }
    }
}

impl From<&Opened> for ArchiveStatus {
    fn from(opened: &Opened) -> Self {
        let infos = |snapshots: &[Snapshot]| snapshots.iter().map(SnapshotInfo::from).collect();
        match opened {
            Opened::Ready(_) => Self::Ready,
            Opened::Damaged { snapshots } => Self::Damaged {
                snapshots: infos(snapshots),
            },
            Opened::Missing { snapshots } => Self::Missing {
                snapshots: infos(snapshots),
            },
        }
    }
}

/// The version of the app and the state of its archive.
#[tauri::command]
#[specta::specta]
pub fn app_status(
    app: tauri::AppHandle,
    archive: State<'_, Arc<Mutex<Opened>>>,
) -> Result<AppStatus, CommandError> {
    let opened = archive.lock().map_err(|_| CommandError::internal())?;
    Ok(AppStatus {
        version: app.package_info().version.to_string(),
        archive: ArchiveStatus::from(&*opened),
    })
}

/// Every command and event of the app, for the invoke handler and for the bindings.
pub fn builder() -> tauri_specta::Builder<tauri::Wry> {
    tauri_specta::Builder::<tauri::Wry>::new()
        .commands(tauri_specta::collect_commands![
            app_status,
            documents::import_files,
            documents::import_folder,
            documents::intake_overview,
            documents::set_intake_paused,
            documents::dismiss_failed,
            documents::list_documents,
            documents::page_image,
            documents::mail_text,
            documents::document_checks,
            documents::undo_filing,
            review::review_groups,
            review::review_questions,
            review::answer_question,
            review::delete_duplicate,
            names::name_templates,
            names::set_name_template,
            names::rename_document,
            money::list_accounts,
            money::list_bookings,
            money::booking_detail,
            money::link_to_booking,
            money::list_orders,
            evaluations::evaluation_years,
            evaluations::evaluation_year,
            recurring::list_recurring,
            recurring::not_recurring,
            salary::salary_years,
            balances::balances_by_month,
            hints::list_hints,
            hints::dismiss_hint,
            taxes::tax_categories,
            taxes::tax_years,
            taxes::tax_year,
            taxes::assign_tax,
            taxes::unassign_tax,
            taxes::set_tax_profile,
            chat::list_chats,
            chat::chat_messages,
            chat::delete_chat,
            chat::ask_chat,
            settings::archive_info,
            settings::model_status,
            document_view::document_detail,
            document_view::delete_document,
            document_view::reveal_document,
            document_view::set_order_needs_invoice,
            document_view::order_invoice_candidates,
            document_view::assign_order_invoice,
            money::list_categories,
            money::add_category,
            money::rename_category,
            money::remove_category,
            money::choose_category,
            money::undo_link,
            money::undo_move,
            folders::list_folders,
            folders::add_folder,
            folders::remove_folder,
            folders::move_inbox,
            folders::open_folder,
            folders::downloads_state,
            folders::set_downloads,
            folders::recheck_downloads,
            mail::list_mailboxes,
            mail::add_mailbox,
            mail::test_mailbox,
            mail::remove_mailbox,
            mail::read_mail_now,
        ])
        .events(tauri_specta::collect_events![documents::IntakeChanged])
        .error_handling(tauri_specta::ErrorHandlingMode::Result)
}

/// The generated TypeScript, with the line endings of the repository.
pub fn bindings() -> Result<String, String> {
    let dir = tempfile_dir()?;
    let path = dir.join("bindings.ts");
    builder()
        .export(specta_typescript::Typescript::default(), &path)
        .map_err(|error| error.to_string())?;
    let text = std::fs::read_to_string(&path).map_err(|error| error.to_string())?;
    let _ = std::fs::remove_dir_all(&dir);
    Ok(text.replace("\r\n", "\n"))
}

fn tempfile_dir() -> Result<std::path::PathBuf, String> {
    let dir = std::env::temp_dir().join(format!("finanzarchiv2-bindings-{}", std::process::id()));
    std::fs::create_dir_all(&dir).map_err(|error| error.to_string())?;
    Ok(dir)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn committed_path() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../src/bindings.ts")
    }

    #[test]
    fn bindings_are_current() {
        let committed = std::fs::read_to_string(committed_path())
            .unwrap_or_default()
            .replace("\r\n", "\n");
        assert!(
            committed == bindings().unwrap(),
            "src/bindings.ts does not match the commands. Regenerate it with `npm run bindings`."
        );
    }

    /// Writes `src/bindings.ts`. Run through `npm run bindings`, never as part of the check.
    #[test]
    #[ignore = "writes src/bindings.ts, run by npm run bindings"]
    fn write_bindings() {
        std::fs::write(committed_path(), bindings().unwrap()).unwrap();
    }

    #[test]
    fn the_state_of_the_archive_reaches_the_window() {
        let snapshot = Snapshot {
            path: PathBuf::from("Sicherungen/finanzarchiv-20261125T080000Z-taeglich.db"),
            taken_at: "2026-11-25T08:00:00Z".parse().unwrap(),
        };
        let status = ArchiveStatus::from(&Opened::Damaged {
            snapshots: vec![snapshot],
        });
        assert_eq!(
            status,
            ArchiveStatus::Damaged {
                snapshots: vec![SnapshotInfo {
                    name: "finanzarchiv-20261125T080000Z-taeglich.db".into(),
                    taken_at: "2026-11-25T08:00:00Z".into(),
                }]
            }
        );
        let json = serde_json::to_value(&status).unwrap();
        assert_eq!(json["state"], "damaged");
        assert_eq!(json["snapshots"][0]["takenAt"], "2026-11-25T08:00:00Z");
    }

    #[test]
    fn an_error_says_its_kind_and_a_german_sentence() {
        let json = serde_json::to_value(CommandError::internal()).unwrap();
        assert_eq!(json["kind"], "internal");
        assert!(
            json["message"]
                .as_str()
                .unwrap()
                .contains("Finanzarchiv neu")
        );
    }
}
