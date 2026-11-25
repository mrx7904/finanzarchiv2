//! Snapshots of the archive database: once a day and before a migration, the newest ten kept.
//! The documents are no part of a snapshot, they never change under their hash.

use std::fs;
use std::path::{Path, PathBuf};

use jiff::tz::TimeZone;
use jiff::{SignedDuration, Timestamp};
use rusqlite::backup::NO_PROGRESS;
use rusqlite::{Connection, MAIN_DB};

pub const FOLDER: &str = "Sicherungen";
pub const KEEP: usize = 10;
const PREFIX: &str = "finanzarchiv-";
const STAMP: &str = "%Y%m%dT%H%M%SZ";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Reason {
    Daily,
    BeforeMigration,
    AfterImport,
}

impl Reason {
    fn slug(self) -> &'static str {
        match self {
            Self::Daily => "taeglich",
            Self::BeforeMigration => "vor-migration",
            Self::AfterImport => "nach-import",
        }
    }
}

/// A snapshot on disk. Its name carries time and reason and sorts as text.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Snapshot {
    pub path: PathBuf,
    pub taken_at: Timestamp,
}

pub fn folder_of(database: &Path) -> PathBuf {
    database
        .parent()
        .map_or_else(|| PathBuf::from(FOLDER), |dir| dir.join(FOLDER))
}

/// The snapshots beside a database, newest first. A file of another name is not one.
pub fn list(database: &Path) -> std::io::Result<Vec<Snapshot>> {
    let folder = folder_of(database);
    if !folder.exists() {
        return Ok(Vec::new());
    }
    let mut found: Vec<Snapshot> = fs::read_dir(folder)?
        .filter_map(Result::ok)
        .filter_map(|entry| {
            let name = entry.file_name().into_string().ok()?;
            let stamp = name.strip_prefix(PREFIX)?.get(..16)?;
            let taken_at = jiff::civil::DateTime::strptime(STAMP, stamp)
                .ok()?
                .to_zoned_datetime(TimeZone::UTC)
                .ok()?
                .timestamp();
            name.ends_with(".db").then(|| Snapshot {
                path: entry.path(),
                taken_at,
            })
        })
        .collect();
    found.sort_by(|a, b| {
        b.taken_at
            .cmp(&a.taken_at)
            .then_with(|| b.path.cmp(&a.path))
    });
    Ok(found)
}

/// Writes a whole, consistent copy through SQLite's backup API, under a temporary name first,
/// so a snapshot that exists is always complete.
pub fn take(
    conn: &Connection,
    database: &Path,
    reason: Reason,
    now: Timestamp,
) -> Result<PathBuf, SnapshotError> {
    let folder = folder_of(database);
    fs::create_dir_all(&folder)?;
    let stamp = jiff::fmt::strtime::format(STAMP, now)?;
    let target = folder.join(format!("{PREFIX}{stamp}-{}.db", reason.slug()));
    let partial = target.with_extension("db.part");
    conn.backup(MAIN_DB, &partial, NO_PROGRESS)?;
    fs::rename(&partial, &target)?;
    Ok(target)
}

/// A daily snapshot is due when the newest is older than a day, or there is none.
pub fn daily_due(snapshots: &[Snapshot], now: Timestamp) -> bool {
    snapshots
        .first()
        .is_none_or(|newest| now.duration_since(newest.taken_at) >= SignedDuration::from_hours(24))
}

/// New documents since the newest snapshot that make a snapshot due when the queue empties
/// (#19). The owner's first 132 documents had none until the next start.
pub const AFTER_IMPORT_THRESHOLD: i64 = 20;

/// Takes a snapshot when at least `AFTER_IMPORT_THRESHOLD` documents came in since the newest one.
pub fn after_import(
    conn: &Connection,
    database: &Path,
    now: Timestamp,
) -> Result<Option<PathBuf>, SnapshotError> {
    let since = list(database)?
        .first()
        .map(|newest| newest.taken_at.to_string());
    let added: i64 = match since {
        Some(since) => conn.query_row(
            "SELECT count(*) FROM documents WHERE imported_at > ?1",
            [since],
            |row| row.get(0),
        )?,
        None => conn.query_row("SELECT count(*) FROM documents", [], |row| row.get(0))?,
    };
    if added < AFTER_IMPORT_THRESHOLD {
        return Ok(None);
    }
    let path = take(conn, database, Reason::AfterImport, now)?;
    prune(database)?;
    Ok(Some(path))
}

/// Removes all but the newest `KEEP`.
pub fn prune(database: &Path) -> Result<(), SnapshotError> {
    for old in list(database)?.into_iter().skip(KEEP) {
        fs::remove_file(old.path)?;
    }
    Ok(())
}

#[derive(Debug, thiserror::Error)]
pub enum SnapshotError {
    #[error("Sicherung: Dateisystem")]
    Io(#[from] std::io::Error),
    #[error("Sicherung: Datenbank")]
    Sqlite(#[from] rusqlite::Error),
    #[error("Sicherung: Zeitstempel")]
    Time(#[from] jiff::Error),
}
