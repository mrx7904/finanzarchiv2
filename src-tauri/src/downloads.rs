//! The downloads folder (FA-33, #28): watched only when the owner switches it on, and a file of it
//! is taken only when it looks like an invoice, a payslip, a receipt or a statement. Code decides
//! first, the local model only for a PDF with text that code did not take (knowledge/decision-folders.md).

use std::path::Path;

use jiff::Timestamp;
use rusqlite::{Connection, OptionalExtension, params};
use serde_json::json;

use crate::kinds::Kind;
use crate::ollama::{LocalModel, Question};
use crate::pdf::Pdfium;

/// Whether a file of the downloads is taken, and why, in German for the owner.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Verdict {
    pub take: bool,
    pub reason: String,
}

pub fn kind_word(kind: Kind) -> &'static str {
    match kind {
        Kind::Invoice => "eine Rechnung",
        Kind::Payslip => "eine Gehaltsabrechnung",
        Kind::Receipt => "ein Zahlungsbeleg",
        Kind::Statement => "ein Kontoauszug",
        Kind::Other | Kind::Unknown => "kein Beleg",
    }
}

pub fn money(kind: Kind) -> bool {
    matches!(
        kind,
        Kind::Invoice | Kind::Payslip | Kind::Receipt | Kind::Statement
    )
}

const SYSTEM: &str = "Du sortierst Dateien aus dem Download-Ordner. Sage nur, welche Art das Dokument hat: \
eine Rechnung (invoice), eine Gehaltsabrechnung (payslip), ein Zahlungsbeleg oder eine Quittung (receipt), \
ein Kontoauszug (statement) oder etwas anderes (other). Erfinde nichts.";

/// Asks the model for the kind of a text that code could not tell.
pub fn ask_kind(model: &dyn LocalModel, lines: &[String]) -> Option<Kind> {
    let schema = json!({
        "type": "object",
        "properties": { "kind": { "type": "string", "enum": ["invoice", "payslip", "receipt", "statement", "other"] } },
        "required": ["kind"],
    });
    let text: String = lines
        .iter()
        .take(80)
        .map(|line| format!("{line}\n"))
        .collect();
    let answer = model
        .ask(&Question {
            system: SYSTEM,
            prompt: &text,
            schema: &schema,
            images: &[],
        })
        .ok()?;
    let answer: serde_json::Value = serde_json::from_str(&answer).ok()?;
    Some(Kind::from_code(answer["kind"].as_str()?))
}

/// Looks at a file of the downloads: a statement as CSV by its layout, a PDF by the signals of its
/// text and, where they say nothing, by the model. Anything else stays out.
pub fn select(pdfium: Option<&Pdfium>, model: Option<&dyn LocalModel>, path: &Path) -> Verdict {
    let out = |reason: &str| Verdict {
        take: false,
        reason: reason.to_owned(),
    };
    let Ok(bytes) = std::fs::read(path) else {
        return out("Die Datei ließ sich nicht lesen.");
    };
    match crate::documents::FileType::detect(&bytes) {
        Some(crate::documents::FileType::Mail) => {
            out("Dateien dieser Art nimmt das Archiv nicht an.")
        }
        Some(crate::documents::FileType::Csv) => Verdict {
            take: true,
            reason: "Aus den Downloads übernommen (ein Kontoauszug als CSV).".into(),
        },
        Some(crate::documents::FileType::Pdf) => {
            let Some(pdfium) = pdfium else {
                return out("PDFs können gerade nicht gelesen werden.");
            };
            let Ok(pages) = crate::pages::read(pdfium, &bytes) else {
                return out("Die PDF ließ sich nicht lesen.");
            };
            let lines: Vec<String> = pages
                .iter()
                .flat_map(|page| page.lines.iter().map(|line| line.text.clone()))
                .collect();
            if lines.is_empty() {
                return out("Eine PDF ohne Text wird aus den Downloads nicht übernommen.");
            }
            let found = crate::kinds::classify(lines.iter().map(String::as_str)).kind;
            if money(found) {
                return Verdict {
                    take: true,
                    reason: format!(
                        "Aus den Downloads übernommen (sieht aus wie {}).",
                        kind_word(found)
                    ),
                };
            }
            match model.and_then(|model| ask_kind(model, &lines)) {
                Some(kind) if money(kind) => Verdict {
                    take: true,
                    reason: format!(
                        "Aus den Downloads übernommen (laut Modell {}).",
                        kind_word(kind)
                    ),
                },
                Some(_) => out("Laut Modell kein Beleg, nicht übernommen."),
                None => {
                    out("Kein Zeichen eines Belegs, und ohne Modell wird nicht weiter geprüft.")
                }
            }
        }
        Some(_) => out("Bilder werden aus den Downloads nicht übernommen."),
        None => out("Dateien dieser Art nimmt das Archiv nicht an."),
    }
}

pub const SETTING: &str = "downloads.watch";

/// Whether the owner switched the downloads on.
pub fn is_on(conn: &Connection) -> rusqlite::Result<bool> {
    Ok(conn
        .query_row(
            "SELECT id FROM watched_folders WHERE is_downloads = 1",
            [],
            |row| row.get::<_, i64>(0),
        )
        .optional()?
        .is_some())
}

/// Switches the downloads on: the folder is watched at its top level, and what lies there now is
/// noted as seen and left alone. Returns how many files were left.
pub fn switch_on(conn: &Connection, dir: &Path, now: Timestamp) -> rusqlite::Result<usize> {
    if is_on(conn)? {
        return Ok(0);
    }
    let tx = conn.unchecked_transaction()?;
    tx.execute(
        "INSERT INTO watched_folders (path, is_downloads, recursive, added_at) VALUES (?1, 1, 0, ?2)
         ON CONFLICT (path) DO UPDATE SET is_downloads = 1, recursive = 0",
        params![dir.to_string_lossy(), now.to_string()],
    )?;
    let folder: i64 = tx.query_row(
        "SELECT id FROM watched_folders WHERE is_downloads = 1",
        [],
        |row| row.get(0),
    )?;
    let folder_row = crate::folders::Folder {
        id: folder,
        path: dir.to_path_buf(),
        is_inbox: false,
        recursive: false,
        problem: None,
        is_downloads: true,
    };
    let existing = crate::folders::new_files(&tx, &folder_row)?;
    for path in &existing {
        crate::folders::mark_seen(&tx, folder, path, now)?;
        tx.execute(
            "UPDATE seen_files SET verdict = 'lag schon da' WHERE folder_id = ?1 AND path = ?2",
            params![folder, path.to_string_lossy()],
        )?;
    }
    tx.commit()?;
    Ok(existing.len())
}

pub fn switch_off(conn: &Connection) -> rusqlite::Result<()> {
    conn.execute("DELETE FROM seen_files WHERE folder_id IN (SELECT id FROM watched_folders WHERE is_downloads = 1)", [])?;
    conn.execute("DELETE FROM watched_folders WHERE is_downloads = 1", [])?;
    Ok(())
}

/// "Vorhandene prüfen": the files left when the downloads were switched on are looked at again.
pub fn recheck(conn: &Connection) -> rusqlite::Result<usize> {
    conn.execute(
        "DELETE FROM seen_files WHERE verdict = 'lag schon da' AND folder_id IN (SELECT id FROM watched_folders WHERE is_downloads = 1)",
        [],
    )
}

/// What the downloads gave: files taken and left out.
pub fn counts(conn: &Connection) -> rusqlite::Result<(u32, u32, u32)> {
    conn.query_row(
        "SELECT count(*) FILTER (WHERE verdict LIKE 'Aus den Downloads übernommen%'),
                count(*) FILTER (WHERE verdict IS NOT NULL AND verdict NOT LIKE 'Aus den Downloads übernommen%' AND verdict <> 'lag schon da'),
                count(*) FILTER (WHERE verdict = 'lag schon da')
         FROM seen_files WHERE folder_id IN (SELECT id FROM watched_folders WHERE is_downloads = 1)",
        [],
        |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
    )
}

#[cfg(test)]
mod tests;
