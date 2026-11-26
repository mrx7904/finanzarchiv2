//! Watched folders and the inbox folder (FA-32, #27): a file saved into a watched folder goes through
//! the intake as if it was dropped, once its size has stood still. Nothing in a folder is moved,
//! renamed or deleted (knowledge/decision-folders.md).

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use jiff::Timestamp;
use rusqlite::{Connection, OptionalExtension, params};

use crate::archive::Opened;
use crate::documents::Source;
use crate::intake::Intake;

/// The name of the inbox folder in the documents folder of the system (the owner, 2026-11-26).
pub const INBOX_NAME: &str = "Finanzarchiv Eingang";
/// How long a file's size and time of change must stand still before it is taken.
pub const SETTLE: Duration = Duration::from_secs(2);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Folder {
    pub id: i64,
    pub path: PathBuf,
    pub is_inbox: bool,
    pub recursive: bool,
    pub problem: Option<String>,
}

/// Why a change of the folders was not taken. German, for the owner.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum FolderError {
    #[error("Der Ordner wird schon beobachtet.")]
    Known,
    #[error("Den Eingangsordner kann man verlegen, aber nicht entfernen.")]
    Inbox,
    #[error("Der Ordner liegt im Archiv selbst und kann nicht beobachtet werden.")]
    InsideArchive,
    #[error("Die Datenbank des Archivs antwortet nicht.")]
    Database,
}

impl From<rusqlite::Error> for FolderError {
    fn from(_: rusqlite::Error) -> Self {
        Self::Database
    }
}

pub fn list(conn: &Connection) -> rusqlite::Result<Vec<Folder>> {
    conn.prepare("SELECT id, path, is_inbox, recursive, problem FROM watched_folders ORDER BY is_inbox DESC, path")?
        .query_map([], |row| {
            Ok(Folder {
                id: row.get(0)?,
                path: PathBuf::from(row.get::<_, String>(1)?),
                is_inbox: row.get(2)?,
                recursive: row.get(3)?,
                problem: row.get(4)?,
            })
        })?
        .collect()
}

/// Adds a folder to watch. A folder inside the archive's own folder is refused, it would read the
/// archive back into itself.
pub fn add(
    conn: &Connection,
    archive_dir: &Path,
    path: &Path,
    now: Timestamp,
) -> Result<i64, FolderError> {
    if path.starts_with(archive_dir) {
        return Err(FolderError::InsideArchive);
    }
    let text = path.to_string_lossy().into_owned();
    let known: Option<i64> = conn
        .query_row(
            "SELECT id FROM watched_folders WHERE path = ?1",
            [&text],
            |row| row.get(0),
        )
        .optional()?;
    if known.is_some() {
        return Err(FolderError::Known);
    }
    conn.execute(
        "INSERT INTO watched_folders (path, added_at) VALUES (?1, ?2)",
        params![text, now.to_string()],
    )?;
    Ok(conn.last_insert_rowid())
}

/// Stops watching a folder. Its files stay in the archive, the inbox cannot be removed.
pub fn remove(conn: &Connection, id: i64) -> Result<(), FolderError> {
    let inbox: Option<bool> = conn
        .query_row(
            "SELECT is_inbox FROM watched_folders WHERE id = ?1",
            [id],
            |row| row.get(0),
        )
        .optional()?;
    if inbox == Some(true) {
        return Err(FolderError::Inbox);
    }
    conn.execute("DELETE FROM seen_files WHERE folder_id = ?1", [id])?;
    conn.execute("DELETE FROM watched_folders WHERE id = ?1", [id])?;
    Ok(())
}

/// The inbox folder: made on the first start in the documents folder of the system, made again
/// when the owner deleted it. Returns its path.
pub fn ensure_inbox(
    conn: &Connection,
    documents_dir: &Path,
    now: Timestamp,
) -> rusqlite::Result<PathBuf> {
    let path: Option<String> = conn
        .query_row(
            "SELECT path FROM watched_folders WHERE is_inbox = 1",
            [],
            |row| row.get(0),
        )
        .optional()?;
    let path = match path {
        Some(path) => PathBuf::from(path),
        None => {
            let path = documents_dir.join(INBOX_NAME);
            conn.execute(
                "INSERT INTO watched_folders (path, is_inbox, added_at) VALUES (?1, 1, ?2)
                 ON CONFLICT (path) DO UPDATE SET is_inbox = 1",
                params![path.to_string_lossy(), now.to_string()],
            )?;
            path
        }
    };
    let _ = std::fs::create_dir_all(&path);
    Ok(path)
}

/// Moves the inbox to another folder. The old one is no longer watched, its files stay archived.
pub fn move_inbox(conn: &Connection, archive_dir: &Path, path: &Path) -> Result<(), FolderError> {
    if path.starts_with(archive_dir) {
        return Err(FolderError::InsideArchive);
    }
    let tx = conn.unchecked_transaction()?;
    tx.execute("DELETE FROM seen_files WHERE folder_id IN (SELECT id FROM watched_folders WHERE is_inbox = 1)", [])?;
    tx.execute("DELETE FROM watched_folders WHERE is_inbox = 1", [])?;
    tx.execute(
        "DELETE FROM watched_folders WHERE path = ?1",
        [path.to_string_lossy()],
    )?;
    tx.execute(
        "INSERT INTO watched_folders (path, is_inbox, added_at) VALUES (?1, 1, ?2)",
        params![path.to_string_lossy(), Timestamp::now().to_string()],
    )?;
    tx.commit()?;
    Ok(())
}

/// Whether a file is one to take: not hidden, not a system file, not a download still running.
pub fn wanted(path: &Path) -> bool {
    let Some(name) = path.file_name().and_then(|name| name.to_str()) else {
        return false;
    };
    let lower = name.to_lowercase();
    !crate::documents::is_hidden(path)
        && !name.starts_with("~$")
        && ![
            ".crdownload",
            ".part",
            ".partial",
            ".download",
            ".tmp",
            ".opdownload",
        ]
        .iter()
        .any(|end| lower.ends_with(end))
}

/// What identifies a version of a file: its size and its time of change.
fn stamp(path: &Path) -> Option<(u64, String)> {
    let meta = std::fs::metadata(path).ok()?;
    if !meta.is_file() {
        return None;
    }
    let modified = meta
        .modified()
        .ok()
        .and_then(|time| time.duration_since(std::time::UNIX_EPOCH).ok())
        .map_or(0, |since| since.as_millis());
    Some((meta.len(), modified.to_string()))
}

fn files_in(folder: &Folder) -> Vec<PathBuf> {
    fn walk(dir: &Path, recursive: bool, found: &mut Vec<PathBuf>) {
        for entry in std::fs::read_dir(dir).into_iter().flatten().flatten() {
            let path = entry.path();
            if path.is_dir() {
                if recursive && wanted(&path) {
                    walk(&path, recursive, found);
                }
            } else if wanted(&path) {
                found.push(path);
            }
        }
    }
    let mut found = Vec::new();
    walk(&folder.path, folder.recursive, &mut found);
    found.sort();
    found
}

/// The files of a folder not handed to the intake in this version yet.
pub fn new_files(conn: &Connection, folder: &Folder) -> rusqlite::Result<Vec<PathBuf>> {
    let mut found = Vec::new();
    for path in files_in(folder) {
        let Some((size, modified)) = stamp(&path) else {
            continue;
        };
        let seen: Option<(i64, String)> = conn
            .query_row(
                "SELECT size_bytes, modified FROM seen_files WHERE folder_id = ?1 AND path = ?2",
                params![folder.id, path.to_string_lossy()],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .optional()?;
        if seen != Some((i64::try_from(size).unwrap_or(i64::MAX), modified)) {
            found.push(path);
        }
    }
    Ok(found)
}

pub fn mark_seen(
    conn: &Connection,
    folder: i64,
    path: &Path,
    now: Timestamp,
) -> rusqlite::Result<()> {
    let Some((size, modified)) = stamp(path) else {
        return Ok(());
    };
    conn.execute(
        "INSERT INTO seen_files (folder_id, path, size_bytes, modified, seen_at) VALUES (?1, ?2, ?3, ?4, ?5)
         ON CONFLICT (folder_id, path) DO UPDATE SET size_bytes = excluded.size_bytes, modified = excluded.modified, seen_at = excluded.seen_at",
        params![folder, path.to_string_lossy(), i64::try_from(size).unwrap_or(i64::MAX), modified, now.to_string()],
    )?;
    Ok(())
}

/// Marks a folder that is gone, or answers again.
fn set_problem(conn: &Connection, folder: i64, problem: Option<&str>) -> rusqlite::Result<()> {
    conn.execute(
        "UPDATE watched_folders SET problem = ?2 WHERE id = ?1",
        params![folder, problem],
    )?;
    Ok(())
}

/// The folder a path lies in, of the watched ones.
fn folder_of<'a>(folders: &'a [Folder], path: &Path) -> Option<&'a Folder> {
    folders
        .iter()
        .filter(|folder| path.starts_with(&folder.path))
        .filter(|folder| folder.recursive || path.parent() == Some(folder.path.as_path()))
        .max_by_key(|folder| folder.path.components().count())
}

/// A file waiting for its size to stand still: what it was at the last look, and since when.
type Pending = (Option<(u64, String)>, Instant);

/// The watcher of the running app: the system's notifications of the folders, a list of files
/// waiting for their size to stand still, and a round that hands the settled ones to the intake.
pub struct Watcher {
    archive: Arc<Mutex<Opened>>,
    intake: Arc<Intake>,
    pending: Mutex<HashMap<PathBuf, Pending>>,
    system: Mutex<Option<notify::RecommendedWatcher>>,
}

impl Watcher {
    pub fn new(archive: Arc<Mutex<Opened>>, intake: Arc<Intake>) -> Arc<Self> {
        Arc::new(Self {
            archive,
            intake,
            pending: Mutex::new(HashMap::new()),
            system: Mutex::new(None),
        })
    }

    fn folders(&self) -> Vec<Folder> {
        let Ok(guard) = self.archive.lock() else {
            return Vec::new();
        };
        let Opened::Ready(ready) = &*guard else {
            return Vec::new();
        };
        list(ready.connection()).unwrap_or_default()
    }

    /// Watches the folders as they are now: at start and after every change of the list. Every
    /// folder is read once, so what came while the app was closed is found too.
    pub fn refresh(self: &Arc<Self>) {
        let folders = self.folders();
        let me = Arc::clone(self);
        let system = notify::recommended_watcher(move |event: notify::Result<notify::Event>| {
            if let Ok(event) = event
                && (event.kind.is_create() || event.kind.is_modify())
                && let Ok(mut pending) = me.pending.lock()
            {
                for path in event.paths.into_iter().filter(|path| wanted(path)) {
                    pending.insert(path, (None, Instant::now()));
                }
            }
        });
        let mut watcher = system.ok();
        if let Ok(guard) = self.archive.lock()
            && let Opened::Ready(ready) = &*guard
        {
            for folder in &folders {
                let reachable = folder.path.is_dir();
                let watched = reachable
                    && watcher.as_mut().is_some_and(|watcher| {
                        use notify::Watcher as _;
                        let mode = if folder.recursive {
                            notify::RecursiveMode::Recursive
                        } else {
                            notify::RecursiveMode::NonRecursive
                        };
                        watcher.watch(&folder.path, mode).is_ok()
                    });
                let problem = (!watched).then_some("Der Ordner ist nicht erreichbar.");
                let _ = set_problem(ready.connection(), folder.id, problem);
                if reachable
                    && let Ok(files) = new_files(ready.connection(), folder)
                    && let Ok(mut pending) = self.pending.lock()
                {
                    for path in files {
                        pending.insert(path, (None, Instant::now()));
                    }
                }
            }
        }
        if let Ok(mut system) = self.system.lock() {
            *system = watcher;
        }
    }

    /// One round: the files whose size and time stood still since the last round go to the intake.
    pub fn tick(&self) -> usize {
        let settled: Vec<PathBuf> = {
            let Ok(mut pending) = self.pending.lock() else {
                return 0;
            };
            let mut settled = Vec::new();
            pending.retain(|path, (last, since)| {
                let now = stamp(path);
                if now.is_none() {
                    return false;
                }
                if *last == now && since.elapsed() >= SETTLE {
                    settled.push(path.clone());
                    return false;
                }
                if *last != now {
                    *last = now;
                    *since = Instant::now();
                }
                true
            });
            settled
        };
        if settled.is_empty() {
            return 0;
        }
        let folders = self.folders();
        let mut taken = Vec::new();
        if let Ok(guard) = self.archive.lock()
            && let Opened::Ready(ready) = &*guard
        {
            for path in settled {
                let Some(folder) = folder_of(&folders, &path) else {
                    continue;
                };
                // A file handed in before and unchanged since is not handed again.
                if new_files_contains(ready.connection(), folder, &path) {
                    let _ = mark_seen(ready.connection(), folder.id, &path, Timestamp::now());
                    taken.push(path);
                }
            }
        }
        if !taken.is_empty() {
            let _ = self.intake.enqueue(&taken, Source::Folder);
        }
        taken.len()
    }

    /// Runs the rounds until the app ends.
    pub fn start(self: &Arc<Self>) {
        self.refresh();
        let me = Arc::clone(self);
        std::thread::spawn(move || {
            loop {
                std::thread::sleep(Duration::from_millis(700));
                me.tick();
            }
        });
    }
}

fn new_files_contains(conn: &Connection, folder: &Folder, path: &Path) -> bool {
    let Some((size, modified)) = stamp(path) else {
        return false;
    };
    let seen: Option<(i64, String)> = conn
        .query_row(
            "SELECT size_bytes, modified FROM seen_files WHERE folder_id = ?1 AND path = ?2",
            params![folder.id, path.to_string_lossy()],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .ok();
    seen != Some((i64::try_from(size).unwrap_or(i64::MAX), modified))
}

#[cfg(test)]
mod tests;
