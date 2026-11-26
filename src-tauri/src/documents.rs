//! Unchanged originals under their hash (FA-07, FA-08, knowledge/decision-data-model.md).

use std::fs;
use std::path::{Path, PathBuf};

use jiff::Timestamp;
use rusqlite::{Connection, OptionalExtension};
use sha2::{Digest, Sha256};

use crate::pdf::{self, PdfCheck, Pdfium};

pub const ARCHIVE_FOLDER: &str = "Archiv";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileType {
    Pdf,
    Png,
    Jpeg,
    Webp,
}

impl FileType {
    /// Read from the first bytes, never from the name, so a renamed file is what it is.
    pub fn detect(bytes: &[u8]) -> Option<Self> {
        let head = &bytes[..bytes.len().min(1024)];
        if head.windows(5).any(|window| window == b"%PDF-") {
            Some(Self::Pdf)
        } else if bytes.starts_with(b"\x89PNG\r\n\x1a\n") {
            Some(Self::Png)
        } else if bytes.starts_with(&[0xFF, 0xD8, 0xFF]) {
            Some(Self::Jpeg)
        } else if bytes.len() >= 12 && &bytes[..4] == b"RIFF" && &bytes[8..12] == b"WEBP" {
            Some(Self::Webp)
        } else {
            None
        }
    }

    pub fn code(self) -> &'static str {
        match self {
            Self::Pdf => "pdf",
            Self::Png => "png",
            Self::Jpeg => "jpeg",
            Self::Webp => "webp",
        }
    }

    pub fn extension(self) -> &'static str {
        match self {
            Self::Jpeg => "jpg",
            other => other.code(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Source {
    Drop,
    Dialog,
}

impl Source {
    pub fn code(self) -> &'static str {
        match self {
            Self::Drop => "drop",
            Self::Dialog => "dialog",
        }
    }

    pub fn from_code(code: &str) -> Option<Self> {
        match code {
            "drop" => Some(Self::Drop),
            "dialog" => Some(Self::Dialog),
            _ => None,
        }
    }
}

/// Why a file was not taken. The German sentence is shown beside the file name.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum Refusal {
    #[error("Die Datei ist leer.")]
    Empty,
    #[error("Dateien dieser Art nimmt das Archiv nicht an. Möglich sind PDF, PNG, JPEG und WebP.")]
    UnknownType,
    #[error("Die PDF ist beschädigt und lässt sich nicht öffnen.")]
    Damaged,
    #[error(
        "Die PDF ist mit einem Passwort geschützt. Speichere sie ohne Passwort und importiere sie dann."
    )]
    PasswordProtected,
    #[error("PDFs können gerade nicht geprüft werden. Bitte starte Finanzarchiv neu.")]
    PdfEngineMissing,
    #[error("Die Datei ließ sich nicht lesen.")]
    Unreadable,
    #[error("Die Datei ließ sich nicht im Archiv ablegen.")]
    NotStored,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Outcome {
    Added {
        id: i64,
    },
    /// The same bytes are in the archive already. Nothing new is stored.
    Known {
        id: i64,
    },
    Refused(Refusal),
}

/// One file of an import with what became of it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileResult {
    pub name: String,
    pub outcome: Outcome,
}

pub fn archive_root(database: &Path) -> PathBuf {
    database.parent().map_or_else(
        || PathBuf::from(ARCHIVE_FOLDER),
        |dir| dir.join(ARCHIVE_FOLDER),
    )
}

/// Where the bytes of a hash lie: `<root>/<first two hex>/<hash>.<ext>`.
pub fn stored_path(root: &Path, sha256: &str, file_type: FileType) -> PathBuf {
    root.join(&sha256[..2])
        .join(format!("{sha256}.{}", file_type.extension()))
}

/// Imports the bytes of one file. The bytes that were hashed are the bytes that are written.
pub fn import_bytes(
    conn: &Connection,
    root: &Path,
    pdfium: Option<&Pdfium>,
    bytes: &[u8],
    original_name: &str,
    source: Source,
    now: Timestamp,
) -> Outcome {
    match import_checked(conn, root, pdfium, bytes, original_name, source, now) {
        Ok(outcome) => outcome,
        Err(refusal) => Outcome::Refused(refusal),
    }
}

fn import_checked(
    conn: &Connection,
    root: &Path,
    pdfium: Option<&Pdfium>,
    bytes: &[u8],
    original_name: &str,
    source: Source,
    now: Timestamp,
) -> Result<Outcome, Refusal> {
    let prepared = prepare(pdfium, bytes)?;
    store(conn, root, &prepared, bytes, original_name, source, now)
}

/// What is known about a file before the archive is touched: its type, its pages and its hash.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Prepared {
    pub file_type: FileType,
    pub pages: Option<u32>,
    pub sha256: String,
}

/// Checks and hashes the bytes. Needs no database, so workers do it side by side.
pub fn prepare(pdfium: Option<&Pdfium>, bytes: &[u8]) -> Result<Prepared, Refusal> {
    if bytes.is_empty() {
        return Err(Refusal::Empty);
    }
    let file_type = FileType::detect(bytes).ok_or(Refusal::UnknownType)?;
    let pages = match file_type {
        FileType::Pdf => match pdf::check(pdfium.ok_or(Refusal::PdfEngineMissing)?, bytes) {
            PdfCheck::Readable { pages } => Some(pages),
            PdfCheck::Damaged => return Err(Refusal::Damaged),
            PdfCheck::PasswordProtected => return Err(Refusal::PasswordProtected),
        },
        _ => None,
    };
    Ok(Prepared {
        file_type,
        pages,
        sha256: hex(&Sha256::digest(bytes)),
    })
}

/// Writes the file under its hash and adds its row. Run it inside the transaction that also
/// moves the job on, so a crash leaves either both or neither. A file written by a crashed
/// attempt is found under its hash and kept.
pub fn store(
    conn: &Connection,
    root: &Path,
    prepared: &Prepared,
    bytes: &[u8],
    original_name: &str,
    source: Source,
    now: Timestamp,
) -> Result<Outcome, Refusal> {
    if let Some(id) = find_by_hash(conn, &prepared.sha256).map_err(|_| Refusal::NotStored)? {
        return Ok(Outcome::Known { id });
    }
    write_once(
        &stored_path(root, &prepared.sha256, prepared.file_type),
        bytes,
    )
    .map_err(|_| Refusal::NotStored)?;
    conn.execute(
        "INSERT INTO documents (sha256, file_type, original_name, size_bytes, pages, source, imported_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        (
            &prepared.sha256,
            prepared.file_type.code(),
            original_name,
            i64::try_from(bytes.len()).map_err(|_| Refusal::NotStored)?,
            prepared.pages,
            source.code(),
            now.to_string(),
        ),
    )
    .map_err(|_| Refusal::NotStored)?;
    Ok(Outcome::Added {
        id: conn.last_insert_rowid(),
    })
}

fn find_by_hash(conn: &Connection, sha256: &str) -> rusqlite::Result<Option<i64>> {
    conn.query_row(
        "SELECT id FROM documents WHERE sha256 = ?1",
        [sha256],
        |row| row.get(0),
    )
    .optional()
}

/// Writes beside the target first and renames, so a file under a hash is always whole.
/// A file that is there already holds the same bytes, its name is their hash.
fn write_once(target: &Path, bytes: &[u8]) -> std::io::Result<()> {
    if target.exists() {
        return Ok(());
    }
    if let Some(dir) = target.parent() {
        fs::create_dir_all(dir)?;
    }
    let partial = target.with_extension("part");
    fs::write(&partial, bytes)?;
    fs::rename(&partial, target)
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

/// Every file below the given paths, folders walked through, in a stable order.
pub fn expand(paths: &[PathBuf]) -> Vec<PathBuf> {
    let mut files = Vec::new();
    for path in paths {
        collect(path, &mut files);
    }
    files
}

fn collect(path: &Path, files: &mut Vec<PathBuf>) {
    if path.is_dir() {
        let Ok(entries) = fs::read_dir(path) else {
            return;
        };
        let mut children: Vec<PathBuf> = entries
            .filter_map(Result::ok)
            .map(|entry| entry.path())
            .collect();
        children.sort();
        for child in children {
            collect(&child, files);
        }
    } else if path.is_file() {
        files.push(path.to_path_buf());
    }
}

/// Imports files and folders from the disk, one result per file.
pub fn import_paths(
    conn: &Connection,
    root: &Path,
    pdfium: Option<&Pdfium>,
    paths: &[PathBuf],
    source: Source,
    now: Timestamp,
) -> Vec<FileResult> {
    expand(paths)
        .into_iter()
        .filter(|path| !is_hidden(path))
        .map(|path| {
            let name = path
                .file_name()
                .map(|n| n.to_string_lossy().into_owned())
                .unwrap_or_default();
            let outcome = match fs::read(&path) {
                Ok(bytes) => import_bytes(conn, root, pdfium, &bytes, &name, source, now),
                Err(_) => Outcome::Refused(Refusal::Unreadable),
            };
            FileResult { name, outcome }
        })
        .collect()
}

/// Files the system keeps beside the owner's, such as `.DS_Store` or `Thumbs.db`.
pub fn is_hidden(path: &Path) -> bool {
    path.file_name()
        .and_then(|name| name.to_str())
        .is_some_and(|name| {
            name.starts_with('.')
                || name.eq_ignore_ascii_case("thumbs.db")
                || name.eq_ignore_ascii_case("desktop.ini")
        })
}

/// A row of the list of documents.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Listed {
    pub id: i64,
    pub name: String,
    pub kind: String,
    pub file_type: String,
    pub pages: Option<i64>,
    pub imported_at: String,
    /// text, scan or image, None until the stage text ran.
    pub text_state: Option<String>,
    pub state: String,
    pub original_name: String,
    pub name_by_owner: bool,
}

pub fn list(conn: &Connection) -> rusqlite::Result<Vec<Listed>> {
    let mut statement = conn.prepare(
        "SELECT id, coalesce(name, original_name), kind, file_type, pages, imported_at, text_state,
                CASE WHEN state = 'waiting' AND EXISTS (SELECT 1 FROM field_values v WHERE v.document_id = documents.id AND v.reader = 'owner' AND v.field = 'hold' AND v.value = '1') THEN 'held' ELSE state END,
                original_name, name_by_owner
         FROM documents ORDER BY imported_at DESC, id DESC",
    )?;
    statement
        .query_map([], |row| {
            Ok(Listed {
                id: row.get(0)?,
                name: row.get(1)?,
                kind: row.get(2)?,
                file_type: row.get(3)?,
                pages: row.get(4)?,
                imported_at: row.get(5)?,
                text_state: row.get(6)?,
                state: row.get(7)?,
                original_name: row.get(8)?,
                name_by_owner: row.get(9)?,
            })
        })?
        .collect()
}

/// The folder beside the archive where the original of a deleted document goes, so a wrong click
/// can be undone by hand until the owner empties it (#17).
pub const TRASH_FOLDER: &str = "Papierkorb";

/// Removes a document from the archive: its row and every row about it, in one transaction. The
/// original moves to `Papierkorb` beside the archive folder under its hash and original name.
/// Returns where the original went, `None` when it was already missing.
pub fn delete(conn: &Connection, root: &Path, id: i64) -> rusqlite::Result<Option<PathBuf>> {
    let (sha256, original): (String, String) = conn.query_row(
        "SELECT sha256, original_name FROM documents WHERE id = ?1",
        [id],
        |row| Ok((row.get(0)?, row.get(1)?)),
    )?;
    let stored = stored_of(conn, root, id)?;
    let tx = conn.unchecked_transaction()?;
    for table in [
        "field_values",
        "field_checks",
        "page_lines",
        "pages",
        "decisions",
        "e_invoices",
        "intake_jobs",
    ] {
        tx.execute(&format!("DELETE FROM {table} WHERE document_id = ?1"), [id])?;
    }
    tx.execute("DELETE FROM documents WHERE id = ?1", [id])?;
    tx.commit()?;
    let Some((path, _)) = stored.filter(|(path, _)| path.exists()) else {
        return Ok(None);
    };
    let trash = root
        .parent()
        .map_or_else(|| PathBuf::from(TRASH_FOLDER), |dir| dir.join(TRASH_FOLDER));
    let safe_name: String = original
        .chars()
        .map(|c| {
            if "/\\:*?\"<>|".contains(c) || c.is_control() {
                '_'
            } else {
                c
            }
        })
        .collect();
    let target = trash.join(format!("{}_{safe_name}", &sha256[..sha256.len().min(12)]));
    // The rows are gone. A file that cannot be moved stays where it was, unused.
    if std::fs::create_dir_all(&trash).is_ok() && std::fs::rename(&path, &target).is_ok() {
        return Ok(Some(target));
    }
    Ok(None)
}

/// Where the original of a document lies and what it is.
pub fn stored_of(
    conn: &Connection,
    root: &Path,
    id: i64,
) -> rusqlite::Result<Option<(PathBuf, FileType)>> {
    let row: Option<(String, String)> = conn
        .query_row(
            "SELECT sha256, file_type FROM documents WHERE id = ?1",
            [id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .optional()?;
    Ok(row.and_then(|(sha256, code)| {
        let file_type = [FileType::Pdf, FileType::Png, FileType::Jpeg, FileType::Webp]
            .into_iter()
            .find(|t| t.code() == code)?;
        Some((stored_path(root, &sha256, file_type), file_type))
    }))
}

/// Replaces the pages and lines of a document and sets its text state.
pub fn save_pages(
    conn: &Connection,
    id: i64,
    pages: &[crate::pages::PageText],
    state: &str,
) -> rusqlite::Result<()> {
    conn.execute("DELETE FROM page_lines WHERE document_id = ?1", [id])?;
    conn.execute("DELETE FROM pages WHERE document_id = ?1", [id])?;
    let mut page_insert = conn.prepare(
        "INSERT INTO pages (document_id, number, width, height) VALUES (?1, ?2, ?3, ?4)",
    )?;
    let mut line_insert = conn.prepare(
        "INSERT INTO page_lines (document_id, page, line, text, left, bottom, right, top) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
    )?;
    for page in pages {
        page_insert.execute(rusqlite::params![
            id,
            page.number,
            f64::from(page.width),
            f64::from(page.height)
        ])?;
        for (index, line) in page.lines.iter().enumerate() {
            line_insert.execute(rusqlite::params![
                id,
                page.number,
                i64::try_from(index + 1).unwrap_or(i64::MAX),
                line.text,
                f64::from(line.left),
                f64::from(line.bottom),
                f64::from(line.right),
                f64::from(line.top)
            ])?;
        }
    }
    conn.execute(
        "UPDATE documents SET text_state = ?2 WHERE id = ?1",
        rusqlite::params![id, state],
    )?;
    Ok(())
}

/// The lines of every page of a document, top to bottom, page after page.
pub fn lines_of(conn: &Connection, id: i64) -> rusqlite::Result<Vec<String>> {
    let mut statement =
        conn.prepare("SELECT text FROM page_lines WHERE document_id = ?1 ORDER BY page, line")?;
    let lines = statement.query_map([id], |row| row.get(0))?;
    lines.collect()
}

/// Sets the kind of a document and the signals it rests on.
pub fn set_kind(
    conn: &Connection,
    id: i64,
    kind: crate::kinds::Kind,
    signals: &[&str],
) -> rusqlite::Result<()> {
    let signals = serde_json::to_string(signals).unwrap_or_else(|_| "[]".into());
    conn.execute(
        "UPDATE documents SET kind = ?2, kind_signals = ?3 WHERE id = ?1",
        rusqlite::params![id, kind.code(), signals],
    )?;
    Ok(())
}

/// Gives every document read by an older version its kind (#10): those with text but no signals.
/// Returns how many it classified.
pub fn classify_missing(conn: &Connection) -> rusqlite::Result<usize> {
    let ids: Vec<i64> = conn
        .prepare("SELECT id FROM documents WHERE text_state IS NOT NULL AND kind_signals IS NULL")?
        .query_map([], |row| row.get(0))?
        .collect::<rusqlite::Result<_>>()?;
    let tx = conn.unchecked_transaction()?;
    for id in &ids {
        let lines = lines_of(&tx, *id)?;
        let found = crate::kinds::classify(lines.iter().map(String::as_str));
        set_kind(&tx, *id, found.kind, &found.signals)?;
    }
    tx.commit()?;
    Ok(ids.len())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::archive::{self, Opened};
    use crate::testing;

    struct Fixture {
        _dir: tempfile::TempDir,
        archive: archive::Archive,
        root: PathBuf,
    }

    fn fixture() -> Fixture {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join(archive::FILE_NAME);
        let Opened::Ready(archive) = archive::open(&path, testing::t0()).unwrap() else {
            panic!("archive not ready")
        };
        let root = archive_root(&path);
        Fixture {
            _dir: dir,
            archive,
            root,
        }
    }

    fn import(fixture: &Fixture, bytes: &[u8], name: &str) -> Outcome {
        import_bytes(
            fixture.archive.connection(),
            &fixture.root,
            Some(testing::pdfium()),
            bytes,
            name,
            Source::Dialog,
            testing::t0(),
        )
    }

    #[test]
    fn documents_read_before_the_kind_existed_get_it_once() {
        let fixture = fixture();
        let conn = fixture.archive.connection();
        let Outcome::Added { id } = import(
            &fixture,
            &testing::pdf(&["Rechnungsnummer 1", "Umsatzsteuer 1,00"]),
            "r.pdf",
        ) else {
            panic!("not added")
        };
        let read = crate::pages::read(
            testing::pdfium(),
            &testing::pdf(&["Rechnungsnummer 1", "Umsatzsteuer 1,00"]),
        )
        .unwrap();
        save_pages(conn, id, &read, "text").unwrap();
        assert_eq!(classify_missing(conn).unwrap(), 1);
        let kind: String = conn
            .query_row("SELECT kind FROM documents WHERE id = ?1", [id], |row| {
                row.get(0)
            })
            .unwrap();
        assert_eq!(kind, "invoice");
        assert_eq!(
            classify_missing(conn).unwrap(),
            0,
            "a second start classifies nothing again"
        );
    }

    #[test]
    fn detects_the_type_from_the_bytes() {
        assert_eq!(
            FileType::detect(&testing::pdf(&["Rechnung"])),
            Some(FileType::Pdf)
        );
        assert_eq!(FileType::detect(testing::PNG), Some(FileType::Png));
        assert_eq!(
            FileType::detect(&[0xFF, 0xD8, 0xFF, 0xE0, 0, 0]),
            Some(FileType::Jpeg)
        );
        assert_eq!(
            FileType::detect(b"RIFF\x24\0\0\0WEBPVP8 "),
            Some(FileType::Webp)
        );
        assert_eq!(FileType::detect(b"Hallo"), None);
    }

    #[test]
    fn stores_the_bytes_unchanged_under_their_hash() {
        let fixture = fixture();
        let bytes = testing::pdf(&["Rechnung 2026-001", "Summe 49,99 EUR"]);
        let Outcome::Added { id } = import(&fixture, &bytes, "rechnung.pdf") else {
            panic!("not added")
        };
        let sha256 = hex(&Sha256::digest(&bytes));
        let stored = stored_path(&fixture.root, &sha256, FileType::Pdf);
        assert_eq!(
            stored
                .parent()
                .unwrap()
                .file_name()
                .unwrap()
                .to_str()
                .unwrap(),
            &sha256[..2]
        );
        assert_eq!(hex(&Sha256::digest(fs::read(&stored).unwrap())), sha256);
        let listed = list(fixture.archive.connection()).unwrap();
        assert_eq!(listed.len(), 1);
        assert_eq!(listed[0].id, id);
        assert_eq!(listed[0].name, "rechnung.pdf");
        assert_eq!(listed[0].kind, "unknown");
        assert_eq!(listed[0].pages, Some(1));
    }

    #[test]
    fn the_same_file_twice_is_stored_once() {
        let fixture = fixture();
        let bytes = testing::pdf(&["Gehaltsabrechnung"]);
        let Outcome::Added { id } = import(&fixture, &bytes, "a.pdf") else {
            panic!("not added")
        };
        assert_eq!(import(&fixture, &bytes, "kopie.pdf"), Outcome::Known { id });
        assert_eq!(list(fixture.archive.connection()).unwrap().len(), 1);
    }

    #[test]
    fn refuses_empty_unknown_damaged_and_protected_files() {
        let fixture = fixture();
        assert_eq!(
            import(&fixture, b"", "leer.pdf"),
            Outcome::Refused(Refusal::Empty)
        );
        assert_eq!(
            import(&fixture, b"nur Text", "notiz.txt"),
            Outcome::Refused(Refusal::UnknownType)
        );
        let whole = testing::pdf(&["Rechnung"]);
        let damaged = &whole[..whole.len() / 3];
        assert_eq!(
            import(&fixture, damaged, "kaputt.pdf"),
            Outcome::Refused(Refusal::Damaged)
        );
        let protected = testing::protected_pdf(&["Geheim"], "passwort");
        assert_eq!(
            import(&fixture, &protected, "geschuetzt.pdf"),
            Outcome::Refused(Refusal::PasswordProtected)
        );
        assert!(list(fixture.archive.connection()).unwrap().is_empty());
        assert!(
            !fixture.root.exists(),
            "a refused file leaves nothing in the archive folder"
        );
    }

    #[test]
    fn a_pdf_cannot_be_taken_without_the_engine() {
        let fixture = fixture();
        let outcome = import_bytes(
            fixture.archive.connection(),
            &fixture.root,
            None,
            &testing::pdf(&["Rechnung"]),
            "r.pdf",
            Source::Drop,
            testing::t0(),
        );
        assert_eq!(outcome, Outcome::Refused(Refusal::PdfEngineMissing));
    }

    #[test]
    fn a_folder_is_walked_through_and_system_files_are_left_out() {
        let fixture = fixture();
        let source = tempfile::tempdir().unwrap();
        let nested = source.path().join("2026").join("März");
        fs::create_dir_all(&nested).unwrap();
        fs::write(source.path().join("a.pdf"), testing::pdf(&["A"])).unwrap();
        fs::write(nested.join("b.png"), testing::PNG).unwrap();
        fs::write(nested.join("notiz.txt"), b"Notiz").unwrap();
        fs::write(nested.join(".DS_Store"), b"x").unwrap();
        let results = import_paths(
            fixture.archive.connection(),
            &fixture.root,
            Some(testing::pdfium()),
            &[source.path().to_path_buf()],
            Source::Drop,
            testing::t0(),
        );
        let names: Vec<&str> = results.iter().map(|r| r.name.as_str()).collect();
        assert_eq!(names, ["b.png", "notiz.txt", "a.pdf"]);
        let added = results
            .iter()
            .filter(|r| matches!(r.outcome, Outcome::Added { .. }))
            .count();
        let refused = results
            .iter()
            .filter(|r| r.outcome == Outcome::Refused(Refusal::UnknownType))
            .count();
        assert_eq!((added, refused), (2, 1));
    }
}
