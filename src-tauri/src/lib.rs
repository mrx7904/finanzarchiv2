//! The core of Finanzarchiv 2.0. Every rule of the archive lives here, the window only shows and asks.

pub mod archive;
pub mod commands;
pub mod money;

use std::process::ExitCode;
use std::sync::Mutex;

use tauri::Manager;

/// Starts the app and returns how it ended, so `main` needs no `expect`.
pub fn run() -> ExitCode {
    let commands = commands::builder();
    let result = tauri::Builder::default()
        .invoke_handler(commands.invoke_handler())
        .setup(|app| {
            let path = app.path().app_data_dir()?.join(archive::FILE_NAME);
            let opened = archive::open(&path, jiff::Timestamp::now())?;
            app.manage(Mutex::new(opened));
            Ok(())
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
