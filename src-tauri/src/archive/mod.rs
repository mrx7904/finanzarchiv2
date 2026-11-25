//! The archive database: one SQLite file with strict tables, checked before the app works on it (FA-04).

pub mod migrations;
pub mod snapshots;

use std::fs;
use std::path::{Path, PathBuf};

use jiff::Timestamp;
use rusqlite::{Connection, OpenFlags};

pub use snapshots::Snapshot;

pub const FILE_NAME: &str = "finanzarchiv.db";

/// An open, checked and migrated archive.
#[derive(Debug)]
pub struct Archive {
    conn: Connection,
    path: PathBuf,
}

impl Archive {
    pub fn connection(&self) -> &Connection {
        &self.conn
    }

    pub fn path(&self) -> &Path {
        &self.path
    }
}

/// What the app found where its archive should be.
#[derive(Debug)]
pub enum Opened {
    Ready(Archive),
    /// The file exists and SQLite finds it damaged. It stays as it is, the owner restores a snapshot.
    Damaged {
        snapshots: Vec<Snapshot>,
    },
    /// There is no file, but snapshots lie beside its place. No new, empty archive hides them.
    Missing {
        snapshots: Vec<Snapshot>,
    },
}

#[derive(Debug, thiserror::Error)]
pub enum ArchiveError {
    #[error("Archiv: Datenbank")]
    Sqlite(#[from] rusqlite::Error),
    #[error("Archiv: Dateisystem")]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Migration(#[from] migrations::MigrationError),
    #[error(transparent)]
    Snapshot(#[from] snapshots::SnapshotError),
}

/// Opens the archive at `path`: checks it, migrates it and keeps its snapshots.
pub fn open(path: &Path, now: Timestamp) -> Result<Opened, ArchiveError> {
    open_with(path, now, migrations::MIGRATIONS)
}

fn open_with(
    path: &Path,
    now: Timestamp,
    all_migrations: &[(u32, &'static str)],
) -> Result<Opened, ArchiveError> {
    let existed = path.exists();
    if !existed {
        let snapshots = snapshots::list(path)?;
        if !snapshots.is_empty() {
            return Ok(Opened::Missing { snapshots });
        }
    } else if !is_whole(path) {
        return Ok(Opened::Damaged {
            snapshots: snapshots::list(path)?,
        });
    }
    if let Some(dir) = path.parent() {
        fs::create_dir_all(dir)?;
    }
    let mut conn = Connection::open(path)?;
    conn.pragma_update_and_check(None, "journal_mode", "WAL", |row| row.get::<_, String>(0))?;
    conn.pragma_update(None, "foreign_keys", "ON")?;
    // The file stays whole on a power cut under WAL, at most the last commits are lost.
    conn.pragma_update(None, "synchronous", "NORMAL")?;

    let applied = migrations::applied(&conn)?;
    let pending: Vec<_> = all_migrations
        .iter()
        .copied()
        .filter(|(number, _)| *number > applied)
        .collect();
    if existed && applied > 0 && !pending.is_empty() {
        snapshots::take(&conn, path, snapshots::Reason::BeforeMigration, now)?;
    }
    migrations::apply(&mut conn, &pending)?;
    if existed && snapshots::daily_due(&snapshots::list(path)?, now) {
        snapshots::take(&conn, path, snapshots::Reason::Daily, now)?;
    }
    snapshots::prune(path)?;
    Ok(Opened::Ready(Archive {
        conn,
        path: path.to_path_buf(),
    }))
}

/// `PRAGMA quick_check` on the file opened read-only, so a damaged file is never written to.
fn is_whole(path: &Path) -> bool {
    let Ok(conn) = Connection::open_with_flags(path, OpenFlags::SQLITE_OPEN_READ_ONLY) else {
        return false;
    };
    conn.query_row("PRAGMA quick_check", [], |row| row.get::<_, String>(0))
        .is_ok_and(|result| result == "ok")
}

#[cfg(test)]
mod tests {
    use super::*;
    use jiff::SignedDuration;
    use std::fs::OpenOptions;

    fn t0() -> Timestamp {
        "2026-11-25T08:00:00Z".parse().unwrap()
    }

    fn later(hours: i64) -> Timestamp {
        t0() + SignedDuration::from_hours(hours)
    }

    fn ready(path: &Path, now: Timestamp) -> Archive {
        match open(path, now).unwrap() {
            Opened::Ready(archive) => archive,
            other => panic!("expected a ready archive, got {other:?}"),
        }
    }

    fn fill(archive: &Archive, rows: usize) {
        let value = "x".repeat(500);
        for index in 0..rows {
            archive
                .connection()
                .execute(
                    "INSERT INTO settings (key, value) VALUES (?1, ?2)",
                    (format!("k{index}"), &value),
                )
                .unwrap();
        }
    }

    fn schema(archive: &Archive) -> Vec<String> {
        let mut statement = archive
            .connection()
            .prepare("SELECT sql FROM sqlite_schema WHERE type = 'table' ORDER BY name")
            .unwrap();
        statement
            .query_map([], |row| row.get(0))
            .unwrap()
            .map(Result::unwrap)
            .collect()
    }

    #[test]
    fn a_new_archive_is_strict_with_foreign_keys_and_wal() {
        let dir = tempfile::tempdir().unwrap();
        let archive = ready(&dir.path().join(FILE_NAME), t0());
        let conn = archive.connection();
        assert_eq!(
            migrations::applied(conn).unwrap() as usize,
            migrations::MIGRATIONS.len()
        );
        let tables: Vec<String> = schema(&archive);
        assert!(!tables.is_empty());
        assert!(
            tables.iter().all(|sql| sql.trim_end().ends_with("STRICT")),
            "{tables:?}"
        );
        let foreign_keys: i64 = conn
            .query_row("PRAGMA foreign_keys", [], |row| row.get(0))
            .unwrap();
        assert_eq!(foreign_keys, 1);
        let mode: String = conn
            .query_row("PRAGMA journal_mode", [], |row| row.get(0))
            .unwrap();
        assert_eq!(mode, "wal");
        assert!(
            snapshots::list(archive.path()).unwrap().is_empty(),
            "a new archive needs no snapshot"
        );
    }

    #[test]
    fn opening_again_changes_nothing() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join(FILE_NAME);
        let first = schema(&ready(&path, t0()));
        let again = ready(&path, later(1));
        assert_eq!(schema(&again), first);
        assert_eq!(
            migrations::applied(again.connection()).unwrap() as usize,
            migrations::MIGRATIONS.len()
        );
    }

    #[test]
    fn a_truncated_file_is_damaged_never_empty_and_stays_untouched() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join(FILE_NAME);
        let archive = ready(&path, t0());
        fill(&archive, 400);
        drop(archive);
        let snapshot_count = snapshots::list(&path).unwrap().len();
        let length = fs::metadata(&path).unwrap().len();
        OpenOptions::new()
            .write(true)
            .open(&path)
            .unwrap()
            .set_len(length / 2)
            .unwrap();
        let before = fs::read(&path).unwrap();

        match open(&path, later(30)).unwrap() {
            Opened::Damaged { snapshots } => assert_eq!(snapshots.len(), snapshot_count),
            other => panic!("expected a damaged archive, got {other:?}"),
        }
        assert_eq!(
            fs::read(&path).unwrap(),
            before,
            "the damaged file must stay as it was"
        );
    }

    #[test]
    fn a_missing_file_beside_snapshots_is_reported_not_replaced() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join(FILE_NAME);
        fill(&ready(&path, t0()), 3);
        drop(ready(&path, later(1)));
        for suffix in ["", "-wal", "-shm"] {
            let _ = fs::remove_file(format!("{}{suffix}", path.display()));
        }
        match open(&path, later(2)).unwrap() {
            Opened::Missing { snapshots } => assert_eq!(snapshots.len(), 1),
            other => panic!("expected a missing archive, got {other:?}"),
        }
        assert!(
            !path.exists(),
            "no new archive may be created over the snapshots"
        );
    }

    #[test]
    fn one_snapshot_a_day_and_the_newest_ten_kept() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join(FILE_NAME);
        fill(&ready(&path, t0()), 3);
        drop(ready(&path, later(1)));
        drop(ready(&path, later(2)));
        assert_eq!(snapshots::list(&path).unwrap().len(), 1);
        drop(ready(&path, later(25)));
        assert_eq!(snapshots::list(&path).unwrap().len(), 2);
        for day in 2..14 {
            drop(ready(&path, later(day * 24 + 1)));
        }
        let kept = snapshots::list(&path).unwrap();
        assert_eq!(kept.len(), snapshots::KEEP);
        assert_eq!(kept[0].taken_at, later(13 * 24 + 1));
    }

    #[test]
    fn a_snapshot_restores_the_data() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join(FILE_NAME);
        fill(&ready(&path, t0()), 5);
        drop(ready(&path, later(1)));
        let snapshot = snapshots::list(&path).unwrap().remove(0);
        let copy = Connection::open(&snapshot.path).unwrap();
        let rows: i64 = copy
            .query_row("SELECT count(*) FROM settings", [], |row| row.get(0))
            .unwrap();
        assert_eq!(rows, 5);
    }

    #[test]
    fn a_migration_of_an_archive_with_data_takes_a_snapshot_first() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join(FILE_NAME);
        let first: &[(u32, &str)] = &[(1, "CREATE TABLE a (x INTEGER) STRICT;")];
        let both: &[(u32, &str)] = &[first[0], (2, "CREATE TABLE b (y INTEGER) STRICT;")];
        assert!(matches!(
            open_with(&path, t0(), first).unwrap(),
            Opened::Ready(_)
        ));
        // Within the same day, so only the migration can be the reason for a snapshot.
        let opened = open_with(&path, later(1), both).unwrap();
        assert!(matches!(opened, Opened::Ready(_)));
        let names: Vec<String> = snapshots::list(&path)
            .unwrap()
            .iter()
            .filter_map(|s| {
                s.path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .map(str::to_owned)
            })
            .collect();
        assert!(
            names.iter().any(|n| n.ends_with("-vor-migration.db")),
            "{names:?}"
        );
    }
}
