//! The core of Finanzarchiv 2.0. Every rule of the archive lives here, the window only shows and asks.

use std::process::ExitCode;

/// Starts the app and returns how it ended, so `main` needs no `expect`.
pub fn run() -> ExitCode {
    match tauri::Builder::default().run(tauri::generate_context!()) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            // Only the kind of the failure, never a value of the archive (knowledge/conventions.md).
            eprintln!("Finanzarchiv konnte nicht starten: {error}");
            ExitCode::FAILURE
        }
    }
}
