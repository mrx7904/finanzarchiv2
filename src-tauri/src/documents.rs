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
    fn code(self) -> &'static str {
        match self {
            Self::Drop => "drop",
            Self::Dialog => "dialog",
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
    let sha256 = hex(&Sha256::digest(bytes));
    if let Some(id) = find_by_hash(conn, &sha256).map_err(|_| Refusal::NotStored)? {
        return Ok(Outcome::Known { id });
    }
    write_once(&stored_path(root, &sha256, file_type), bytes).map_err(|_| Refusal::NotStored)?;
    conn.execute(
        "INSERT INTO documents (sha256, file_type, original_name, size_bytes, pages, source, imported_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        (
            &sha256,
            file_type.code(),
            original_name,
            i64::try_from(bytes.len()).map_err(|_| Refusal::NotStored)?,
            pages,
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
fn is_hidden(path: &Path) -> bool {
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
}

pub fn list(conn: &Connection) -> rusqlite::Result<Vec<Listed>> {
    let mut statement = conn.prepare(
        "SELECT id, coalesce(name, original_name), kind, file_type, pages, imported_at
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
            })
        })?
        .collect()
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
