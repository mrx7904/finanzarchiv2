//! The core of Finanzarchiv 2.0. Every rule of the archive lives here, the window only shows and asks.

pub mod archive;
pub mod commands;
pub mod documents;
pub mod money;
pub mod pdf;
#[cfg(test)]
mod testing;

use std::path::PathBuf;
use std::process::ExitCode;
use std::sync::Mutex;

use tauri::{DragDropEvent, Manager, WindowEvent};
use tauri_specta::Event;

use commands::documents::ImportFinished;

/// Starts the app and returns how it ended, so `main` needs no `expect`.
pub fn run() -> ExitCode {
    let commands = commands::builder();
    let result = tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(commands.invoke_handler())
        .setup(move |app| {
            commands.mount_events(app);
            if pdf::init(&pdfium_folders(app.handle())).is_err() {
                eprintln!("PDFium fehlt, PDFs werden nicht angenommen");
            }
            let path = app.path().app_data_dir()?.join(archive::FILE_NAME);
            let opened = archive::open(&path, jiff::Timestamp::now())?;
            app.manage(Mutex::new(opened));
            Ok(())
        })
        .on_window_event(|window, event| {
            if let WindowEvent::DragDrop(DragDropEvent::Drop { paths, .. }) = event {
                import_dropped(window.app_handle().clone(), paths.clone());
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

/// Dropped files are imported beside the window's thread, and the window hears the result.
fn import_dropped(app: tauri::AppHandle, paths: Vec<PathBuf>) {
    std::thread::spawn(move || {
        let state = app.state::<Mutex<archive::Opened>>();
        let summary = commands::documents::import(&state, &paths, documents::Source::Drop);
        let finished = ImportFinished { summary };
        if finished.emit(&app).is_err() {
            eprintln!("Das Ergebnis eines Imports erreichte das Fenster nicht");
        }
    });
}

/// The library ships as a resource of the app. A debug build also finds it where
/// `scripts/fetch-pdfium.mjs` unpacked it.
fn pdfium_folders(app: &tauri::AppHandle) -> Vec<PathBuf> {
    let mut folders: Vec<PathBuf> = app
        .path()
        .resource_dir()
        .map(|dir| dir.join("pdfium"))
        .into_iter()
        .collect();
    // Only in a debug build, so a release carries no path of the build machine.
    #[cfg(debug_assertions)]
    folders.push(pdf::platform_folder(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("pdfium"),
    ));
    folders
}
