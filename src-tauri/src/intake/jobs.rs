//! The jobs of the intake queue in the database (FA-09). Every function runs on the connection
//! of the archive, the worker decides when.

use std::path::{Path, PathBuf};

use jiff::Timestamp;
use rusqlite::{Connection, OptionalExtension, params};

use crate::documents::{self, Outcome, Source};

/// The fixed stages of a job, in order. `Queued` waits for the store, `Filed`, `Waiting` and
/// `Failed` end a job. `Model` holds a job while the local model does not answer (#14).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Stage {
    Queued,
    Stored,
    Text,
    Classified,
    Read,
    Model,
    Checked,
    Filed,
    Waiting,
    Failed,
}

impl Stage {
    pub fn code(self) -> &'static str {
        match self {
            Self::Queued => "queued",
            Self::Stored => "stored",
            Self::Text => "text",
            Self::Classified => "classified",
            Self::Read => "read",
            Self::Model => "model",
            Self::Checked => "checked",
            Self::Filed => "filed",
            Self::Waiting => "waiting",
            Self::Failed => "failed",
        }
    }

    pub fn from_code(code: &str) -> Option<Self> {
        [
            Self::Queued,
            Self::Stored,
            Self::Text,
            Self::Classified,
            Self::Read,
            Self::Model,
            Self::Checked,
            Self::Filed,
            Self::Waiting,
            Self::Failed,
        ]
        .into_iter()
        .find(|stage| stage.code() == code)
    }

    pub fn is_final(self) -> bool {
        matches!(self, Self::Filed | Self::Waiting | Self::Failed)
    }

    /// The stage after a pass-through stage.
    pub fn next(self) -> Self {
        match self {
            Self::Queued => Self::Stored,
            Self::Stored => Self::Text,
            Self::Text => Self::Classified,
            Self::Classified => Self::Read,
            Self::Read | Self::Model => Self::Checked,
            Self::Checked | Self::Filed => Self::Filed,
            Self::Waiting => Self::Waiting,
            Self::Failed => Self::Failed,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Job {
    pub id: i64,
    pub source: Source,
    pub path: PathBuf,
    pub name: String,
    pub stage: Stage,
    /// The document the job stored, once it did.
    pub document: Option<i64>,
}

/// One job per file below the paths, folders walked through, system files left out.
pub fn enqueue(
    conn: &Connection,
    paths: &[PathBuf],
    source: Source,
    now: Timestamp,
) -> rusqlite::Result<u32> {
    let files: Vec<PathBuf> = documents::expand(paths)
        .into_iter()
        .filter(|path| !documents::is_hidden(path))
        .collect();
    let tx = conn.unchecked_transaction()?;
    {
        let mut insert = tx.prepare(
            "INSERT INTO intake_jobs (source, path, name, stage, created_at, updated_at) VALUES (?1, ?2, ?3, 'queued', ?4, ?4)",
        )?;
        for file in &files {
            insert.execute(params![
                source.code(),
                file.to_string_lossy(),
                file_name(file),
                now.to_string()
            ])?;
        }
    }
    tx.commit()?;
    Ok(u32::try_from(files.len()).unwrap_or(u32::MAX))
}

fn file_name(path: &Path) -> String {
    path.file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_default()
}

/// The jobs that are not done, oldest first.
pub fn open_jobs(conn: &Connection) -> rusqlite::Result<Vec<Job>> {
    let mut statement = conn.prepare(
        "SELECT id, source, path, name, stage, document_id FROM intake_jobs
         WHERE stage NOT IN ('filed', 'waiting', 'failed') ORDER BY id",
    )?;
    let rows = statement.query_map([], |row| {
        let source: String = row.get(1)?;
        let path: String = row.get(2)?;
        let stage: String = row.get(4)?;
        Ok(Job {
            id: row.get(0)?,
            source: Source::from_code(&source).unwrap_or(Source::Dialog),
            path: PathBuf::from(path),
            name: row.get(3)?,
            stage: Stage::from_code(&stage).unwrap_or(Stage::Queued),
            document: row.get(5)?,
        })
    })?;
    rows.collect()
}

/// Moves a job on. A reason it waited for is cleared, it is no longer true.
pub fn set_stage(conn: &Connection, id: i64, stage: Stage, now: Timestamp) -> rusqlite::Result<()> {
    conn.execute(
        "UPDATE intake_jobs SET stage = ?2, error = NULL, updated_at = ?3 WHERE id = ?1",
        params![id, stage.code(), now.to_string()],
    )?;
    Ok(())
}

/// Holds a job in the stage `model` with the German reason the model gave none.
pub fn wait_for_model(
    conn: &Connection,
    id: i64,
    reason: &str,
    now: Timestamp,
) -> rusqlite::Result<()> {
    conn.execute(
        "UPDATE intake_jobs SET stage = 'model', error = ?2, updated_at = ?3 WHERE id = ?1",
        params![id, reason, now.to_string()],
    )?;
    Ok(())
}

/// The open jobs that are not waiting for the model: when none is left, an import is done.
pub fn count_busy(conn: &Connection) -> rusqlite::Result<u32> {
    conn.query_row(
        "SELECT count(*) FROM intake_jobs WHERE stage NOT IN ('filed', 'waiting', 'failed', 'model')",
        [],
        |row| row.get(0),
    )
}

/// The document a job stored, once it did.
pub fn document_of(conn: &Connection, id: i64) -> rusqlite::Result<Option<i64>> {
    conn.query_row(
        "SELECT document_id FROM intake_jobs WHERE id = ?1",
        [id],
        |row| row.get(0),
    )
}

/// Records what the store did and returns the stage the job goes on with.
pub fn record_store(
    conn: &Connection,
    id: i64,
    outcome: &Outcome,
    now: Timestamp,
) -> rusqlite::Result<Stage> {
    let (stage, document, known, error) = match outcome {
        Outcome::Added { id } => (Stage::Stored, Some(*id), 0, None),
        Outcome::Known { id } => (Stage::Filed, Some(*id), 1, None),
        Outcome::Refused(refusal) => (Stage::Failed, None, 0, Some(refusal.to_string())),
    };
    conn.execute(
        "UPDATE intake_jobs SET stage = ?2, document_id = ?3, known = ?4, error = ?5, updated_at = ?6 WHERE id = ?1",
        params![id, stage.code(), document, known, error, now.to_string()],
    )?;
    Ok(stage)
}

/// A job as the window shows it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Listed {
    pub id: i64,
    pub name: String,
    pub stage: Stage,
    pub error: Option<String>,
}

pub fn listed(conn: &Connection, open: bool, limit: u32) -> rusqlite::Result<Vec<Listed>> {
    let filter = if open {
        "stage NOT IN ('filed', 'waiting', 'failed')"
    } else {
        "stage = 'failed'"
    };
    let mut statement = conn.prepare(&format!(
        "SELECT id, name, stage, error FROM intake_jobs WHERE {filter} ORDER BY id LIMIT ?1"
    ))?;
    let rows = statement.query_map([limit], |row| {
        let stage: String = row.get(2)?;
        Ok(Listed {
            id: row.get(0)?,
            name: row.get(1)?,
            stage: Stage::from_code(&stage).unwrap_or(Stage::Queued),
            error: row.get(3)?,
        })
    })?;
    rows.collect()
}

pub fn count_open(conn: &Connection) -> rusqlite::Result<u32> {
    conn.query_row(
        "SELECT count(*) FROM intake_jobs WHERE stage NOT IN ('filed', 'waiting', 'failed')",
        [],
        |row| row.get(0),
    )
}

/// Jobs that took a new document since the given moment.
pub fn count_added_since(conn: &Connection, since: Timestamp) -> rusqlite::Result<u32> {
    conn.query_row(
        "SELECT count(*) FROM intake_jobs WHERE stage IN ('filed', 'waiting') AND known = 0 AND updated_at >= ?1",
        [since.to_string()],
        |row| row.get(0),
    )
}

/// Takes the failed jobs off the list. Their files were never stored.
pub fn dismiss_failed(conn: &Connection) -> rusqlite::Result<usize> {
    conn.execute("DELETE FROM intake_jobs WHERE stage = 'failed'", [])
}

pub const WORKERS_SETTING: &str = "intake.workers";
/// Two workers: measured on 2026-11-25, see knowledge/decision-intake.md.
pub const DEFAULT_WORKERS: usize = 2;

/// The number of workers: the setting when the owner set one, else the default.
pub fn workers(conn: &Connection) -> usize {
    conn.query_row(
        "SELECT value FROM settings WHERE key = ?1",
        [WORKERS_SETTING],
        |row| row.get::<_, String>(0),
    )
    .optional()
    .ok()
    .flatten()
    .and_then(|value| value.parse().ok())
    .filter(|count| (1..=16).contains(count))
    .unwrap_or(DEFAULT_WORKERS)
}
