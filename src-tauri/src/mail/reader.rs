//! The mailboxes are read in the background (#31): a round at start and every 15 minutes, one
//! mailbox after the other. A mail looked at is a row of `mail_messages`, its PDFs wait in
//! `Mail-Eingang` beside the archive until the intake took them.

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use jiff::Timestamp;
use rusqlite::{Connection, OptionalExtension, params};

use super::round::{self, Counts, Looked, Place};
use super::secrets::Secrets;
use super::{MailError, MailboxError};
use crate::archive::Opened;
use crate::documents::Source;
use crate::intake::Intake;
use crate::ollama::LocalModel;
use crate::pdf::Pdfium;

/// The folder beside the archive where the PDFs of mails wait for the intake.
pub const STAGING: &str = "Mail-Eingang";
const EVERY: Duration = Duration::from_secs(15 * 60);
const FIRST_AFTER: Duration = Duration::from_secs(60);

pub fn place_of(conn: &Connection, id: i64) -> rusqlite::Result<Place> {
    conn.query_row(
        "SELECT folder, uid_validity, last_uid FROM mailboxes WHERE id = ?1",
        [id],
        |row| {
            Ok(Place {
                folder: row.get(0)?,
                uid_validity: row.get(1)?,
                last_uid: row.get(2)?,
            })
        },
    )
}

pub fn is_known(conn: &Connection, hash: &str) -> rusqlite::Result<bool> {
    Ok(conn
        .query_row(
            "SELECT 1 FROM mail_messages WHERE message_hash = ?1",
            [hash],
            |_| Ok(()),
        )
        .optional()?
        .is_some())
}

/// Keeps a mail looked at and writes its PDFs to the staging folder. Returns their paths.
pub fn keep(
    conn: &Connection,
    staging: &Path,
    mailbox: i64,
    looked: &Looked,
    now: Timestamp,
) -> rusqlite::Result<Vec<PathBuf>> {
    let mut paths = Vec::new();
    if !looked.files.is_empty() {
        let dir = staging.join(&looked.hash[..16]);
        if std::fs::create_dir_all(&dir).is_ok() {
            for (name, bytes) in &looked.files {
                let mut path = dir.join(name);
                let mut n = 2;
                while path.exists() {
                    path = dir.join(format!("{n} {name}"));
                    n += 1;
                }
                if std::fs::write(&path, bytes).is_ok() {
                    paths.push(path);
                }
            }
        }
    }
    conn.execute(
        "INSERT INTO mail_messages (mailbox_id, message_hash, received_at, looked_at, verdict, taken)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6) ON CONFLICT (message_hash) DO NOTHING",
        params![
            mailbox,
            looked.hash,
            looked.received_at,
            now.to_string(),
            looked.verdict,
            i64::try_from(paths.len()).unwrap_or(0)
        ],
    )?;
    Ok(paths)
}

/// Removes the staged PDFs the intake took already (every job past `queued`), and empty folders.
pub fn tidy(conn: &Connection, staging: &Path) -> rusqlite::Result<()> {
    let Ok(dirs) = std::fs::read_dir(staging) else {
        return Ok(());
    };
    let mut waiting =
        conn.prepare("SELECT 1 FROM intake_jobs WHERE path = ?1 AND stage = 'queued' LIMIT 1")?;
    for dir in dirs.flatten() {
        let Ok(files) = std::fs::read_dir(dir.path()) else {
            continue;
        };
        for file in files.flatten() {
            let path = file.path();
            if waiting
                .query_row([path.to_string_lossy()], |_| Ok(()))
                .optional()?
                .is_none()
            {
                let _ = std::fs::remove_file(&path);
            }
        }
        let _ = std::fs::remove_dir(dir.path());
    }
    Ok(())
}

/// What the mailboxes gave so far: mails looked at and taken.
pub fn totals(conn: &Connection, mailbox: i64) -> rusqlite::Result<(u32, u32)> {
    conn.query_row(
        "SELECT count(*), count(*) FILTER (WHERE taken > 0) FROM mail_messages WHERE mailbox_id = ?1",
        [mailbox],
        |row| Ok((row.get(0)?, row.get(1)?)),
    )
}

pub struct Reader {
    archive: Arc<Mutex<Opened>>,
    intake: Arc<Intake>,
    pdfium: Option<&'static Pdfium>,
    model: Option<Arc<dyn LocalModel>>,
    secrets: Box<dyn Secrets>,
    busy: AtomicBool,
}

impl Reader {
    pub fn new(
        archive: Arc<Mutex<Opened>>,
        intake: Arc<Intake>,
        pdfium: Option<&'static Pdfium>,
        model: Option<Arc<dyn LocalModel>>,
        secrets: Box<dyn Secrets>,
    ) -> Arc<Self> {
        Arc::new(Self {
            archive,
            intake,
            pdfium,
            model,
            secrets,
            busy: AtomicBool::new(false),
        })
    }

    fn with<T>(&self, run: impl FnOnce(&Connection, &Path) -> rusqlite::Result<T>) -> Option<T> {
        let guard = self.archive.lock().ok()?;
        let Opened::Ready(ready) = &*guard else {
            return None;
        };
        let staging = ready
            .path()
            .parent()
            .map_or_else(|| PathBuf::from(STAGING), |dir| dir.join(STAGING));
        run(ready.connection(), &staging).ok()
    }

    /// Reads every mailbox once. Returns `false` when a round was running already.
    pub fn round(&self) -> bool {
        if self.busy.swap(true, Ordering::SeqCst) {
            return false;
        }
        let _ = self.with(tidy);
        let mailboxes = self.with(|conn, _| super::list(conn)).unwrap_or_default();
        if let Ok(runtime) = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
        {
            for mailbox in mailboxes {
                let outcome = runtime.block_on(self.read_one(mailbox.id, &mailbox.address));
                let problem = outcome.err().map(|error| error.to_string());
                let _ = self.with(|conn, _| {
                    conn.execute(
                        "UPDATE mailboxes SET problem = ?2, checked_at = ?3, read_at = CASE WHEN ?2 IS NULL THEN ?3 ELSE read_at END WHERE id = ?1",
                        params![mailbox.id, problem, Timestamp::now().to_string()],
                    )
                });
            }
        }
        self.busy.store(false, Ordering::SeqCst);
        true
    }

    async fn read_one(&self, id: i64, address: &str) -> Result<Counts, MailboxError> {
        let password = self.secrets.get(address)?.ok_or(MailboxError::NoPassword)?;
        let place = self
            .with(|conn, _| place_of(conn, id))
            .ok_or(MailboxError::Database)?;
        let since = Timestamp::now()
            .checked_sub(jiff::SignedDuration::from_hours(24 * round::REACH_DAYS))
            .unwrap_or(Timestamp::UNIX_EPOCH)
            .to_zoned_datetime(jiff::tz::TimeZone::UTC)
            .date();
        let known = |hash: &str| self.with(|conn, _| is_known(conn, hash)).unwrap_or(true);
        let mut taken: Vec<PathBuf> = Vec::new();
        let mut note = |looked: &Looked| {
            if let Some(paths) =
                self.with(|conn, staging| keep(conn, staging, id, looked, Timestamp::now()))
            {
                taken.extend(paths);
            }
        };
        let stream = tokio::time::timeout(Duration::from_secs(30), super::connect_gmail())
            .await
            .map_err(|_| MailError::Timeout)??;
        let (left, counts) = round::read(
            stream,
            address,
            &password,
            &place,
            since,
            self.pdfium,
            self.model.as_deref(),
            &known,
            &mut note,
        )
        .await?;
        let _ = self.with(|conn, _| {
            conn.execute(
                "UPDATE mailboxes SET folder = ?2, uid_validity = ?3, last_uid = ?4 WHERE id = ?1",
                params![id, left.folder, left.uid_validity, left.last_uid],
            )
        });
        if !taken.is_empty() {
            let _ = self.intake.enqueue(&taken, Source::Mail);
        }
        Ok(counts)
    }

    /// A round at start, after a minute, and every 15 minutes after.
    pub fn start(self: &Arc<Self>) {
        let me = Arc::clone(self);
        std::thread::spawn(move || {
            std::thread::sleep(FIRST_AFTER);
            loop {
                me.round();
                std::thread::sleep(EVERY);
            }
        });
    }
}
