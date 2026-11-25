//! PDFium, the PDF engine of Chrome, bound once per process (knowledge/decision-pdf.md).

use std::path::{Path, PathBuf};
use std::sync::OnceLock;

pub use pdfium_render::prelude::Pdfium;
use pdfium_render::prelude::{PdfiumError, PdfiumInternalError};

static ENGINE: OnceLock<Result<Pdfium, String>> = OnceLock::new();

/// What a PDF turned out to be, before anything is stored.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PdfCheck {
    Readable { pages: u32 },
    Damaged,
    PasswordProtected,
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("PDFium ist nicht verfügbar")]
pub struct EngineMissing;

/// The folder of the library for this system, inside `src-tauri/pdfium/` as `scripts/fetch-pdfium.mjs` unpacks it.
pub fn platform_folder(root: &Path) -> PathBuf {
    let platform = match (std::env::consts::OS, std::env::consts::ARCH) {
        ("windows", _) => "win-x64/bin",
        ("macos", "aarch64") => "mac-arm64/lib",
        ("macos", _) => "mac-x64/lib",
        _ => "linux-x64/lib",
    };
    root.join(platform)
}

/// Binds the library from the first folder that holds it. Later calls return the first result.
pub fn init(folders: &[PathBuf]) -> Result<&'static Pdfium, EngineMissing> {
    ENGINE
        .get_or_init(|| {
            let mut reasons = Vec::new();
            for folder in folders {
                let library = Pdfium::pdfium_platform_library_name_at_path(folder);
                if !library.exists() {
                    continue;
                }
                match Pdfium::bind_to_library(&library) {
                    Ok(bindings) => return Ok(Pdfium::new(bindings)),
                    Err(error) => reasons.push(format!("{error:?}")),
                }
            }
            Err(reasons.join("; "))
        })
        .as_ref()
        .map_err(|_| EngineMissing)
}

pub fn engine() -> Result<&'static Pdfium, EngineMissing> {
    ENGINE
        .get()
        .and_then(|result| result.as_ref().ok())
        .ok_or(EngineMissing)
}

/// Opens the bytes as a PDF and says whether the archive can take them.
pub fn check(pdfium: &Pdfium, bytes: &[u8]) -> PdfCheck {
    match pdfium.load_pdf_from_byte_slice(bytes, None) {
        Ok(document) => PdfCheck::Readable {
            pages: u32::try_from(document.pages().len()).unwrap_or(0),
        },
        Err(PdfiumError::PdfiumLibraryInternalError(PdfiumInternalError::PasswordError)) => {
            PdfCheck::PasswordProtected
        }
        Err(_) => PdfCheck::Damaged,
    }
}
