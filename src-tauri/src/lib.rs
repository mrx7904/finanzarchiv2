//! The core of Finanzarchiv 2.0. Every rule of the archive lives here, the window only shows and asks.

pub mod accounts;
pub mod archive;
pub mod checks;
pub mod commands;
pub mod documents;
pub mod downloads;
pub mod folders;
pub mod intake;
pub mod kinds;
pub mod links;
pub mod mail;
pub mod money;
pub mod moves;
pub mod naming;
pub mod ollama;
pub mod pages;
pub mod parties;
pub mod pdf;
pub mod previews;
pub mod readers;
pub mod review;
pub mod statements;
#[cfg(test)]
mod testing;

use std::path::PathBuf;
use std::process::ExitCode;
use std::sync::{Arc, Mutex};

use tauri::{DragDropEvent, Manager, WindowEvent};
use tauri_specta::Event;

use commands::documents::IntakeChanged;
use intake::Intake;

/// Starts the app and returns how it ended, so `main` needs no `expect`.
pub fn run() -> ExitCode {
    let commands = commands::builder();
    let result = tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(commands.invoke_handler())
        .setup(move |app| {
            commands.mount_events(app);
            let pdfium = pdf::init(&pdfium_folders(app.handle())).ok();
            if pdfium.is_none() {
                eprintln!("PDFium fehlt, PDFs werden nicht angenommen");
            }
            let path = app.path().app_data_dir()?.join(archive::FILE_NAME);
            let opened = archive::open(&path, jiff::Timestamp::now())?;
            let (workers, model) = match &opened {
                archive::Opened::Ready(ready) => {
                    if documents::classify_missing(ready.connection()).is_err() {
                        eprintln!("Die Art älterer Dokumente ließ sich nicht bestimmen");
                    }
                    if let Some(pdfium) = pdfium {
                        let root = documents::archive_root(ready.path());
                        if readers::e_invoice::extract_missing(ready.connection(), &root, pdfium)
                            .is_err()
                        {
                            eprintln!("E-Rechnungen älterer Dokumente ließen sich nicht suchen");
                        }
                    }
                    if readers::read_outdated(ready.connection(), jiff::Timestamp::now()).is_err() {
                        eprintln!("Ältere Dokumente ließen sich nicht neu lesen");
                    }
                    if checks::supersede_statement_rule(ready.connection(), jiff::Timestamp::now())
                        .is_err()
                    {
                        eprintln!("Die Kontoauszüge ließen sich nicht neu prüfen");
                    }
                    let root = documents::archive_root(ready.path());
                    if checks::requeue_unchecked(ready.connection(), &root, jiff::Timestamp::now())
                        .is_err()
                    {
                        eprintln!("Ungeprüfte Dokumente ließen sich nicht in den Eingang legen");
                    }
                    // #31: mails taken as a PDF are taken again as their text, files of mails get
                    // the kind of their mail where code could not tell.
                    if mail::reader::replace_mail_pdfs(ready.connection(), &root).is_err() {
                        eprintln!("Die PDFs von Mails ließen sich nicht ersetzen");
                    }
                    if mail::reader::apply_hints(ready.connection(), jiff::Timestamp::now())
                        .is_err()
                    {
                        eprintln!("Dokumente aus Mails ließen sich nicht neu einordnen");
                    }
                    (
                        intake::jobs::workers(ready.connection()),
                        readers::model::configured(ready.connection()),
                    )
                }
                _ => (0, String::new()),
            };
            let archive = Arc::new(Mutex::new(opened));
            let handle = app.handle().clone();
            let intake = Intake::new(
                Arc::clone(&archive),
                Box::new(move || {
                    // The window reloads what it shows. A closed window misses nothing it needs.
                    let _ = IntakeChanged.emit(&handle);
                }),
            );
            match ollama::Ollama::from_env(&model) {
                Ok(ollama) => intake.set_model(Arc::new(ollama)),
                Err(error) => eprintln!("{error}"),
            }
            if workers > 0 {
                intake.start(workers, pdfium);
            }
            // The inbox folder and every watched folder (#27): made at start, read once, then watched.
            if let Ok(documents_dir) = app.path().document_dir()
                && let Ok(guard) = archive.lock()
                && let archive::Opened::Ready(ready) = &*guard
                && folders::ensure_inbox(ready.connection(), &documents_dir, jiff::Timestamp::now())
                    .is_err()
            {
                eprintln!("Der Eingangsordner ließ sich nicht anlegen");
            }
            let watcher = folders::Watcher::new(
                Arc::clone(&archive),
                Arc::clone(&intake),
                pdfium,
                intake.model(),
            );
            if workers > 0 {
                watcher.start();
            }
            // The mailboxes (#31): a round a minute after start, then every 15 minutes.
            let mail_reader = mail::reader::Reader::new(
                Arc::clone(&archive),
                Arc::clone(&intake),
                intake.model(),
                Box::new(mail::secrets::Keychain),
            );
            if workers > 0 {
                mail_reader.start();
            }
            app.manage(archive);
            app.manage(intake);
            app.manage(watcher);
            app.manage(mail_reader);
            Ok(())
        })
        .on_window_event(|window, event| {
            if let WindowEvent::DragDrop(DragDropEvent::Drop { paths, .. }) = event {
                let intake = window.app_handle().state::<Arc<Intake>>();
                commands::documents::enqueue_dropped(&intake, paths);
            }
        })
        .run(tauri::generate_context!());
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            // Only the kind of the failure, never a value of the archive (knowledge/conventions.md).
            eprintln!("Finanzarchiv konnte nicht starten: {error}");
            ExitCode::FAILURE
        }
    }
}

/// The library ships as a resource of the app. A debug build also finds it where
/// `scripts/fetch-pdfium.mjs` unpacked it.
fn pdfium_folders(app: &tauri::AppHandle) -> Vec<PathBuf> {
    let bundled = app
        .path()
        .resource_dir()
        .map(|dir| dir.join("pdfium"))
        .into_iter();
    // Only in a debug build, so a release carries no path of the build machine.
    #[cfg(debug_assertions)]
    let bundled = bundled.chain(std::iter::once(pdf::platform_folder(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("pdfium"),
    )));
    bundled.collect()
}
