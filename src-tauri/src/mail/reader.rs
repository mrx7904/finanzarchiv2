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
use crate::kinds::Kind;
use crate::ollama::LocalModel;

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
        "INSERT INTO mail_messages (mailbox_id, message_hash, received_at, looked_at, verdict, taken, kind, paypal_code)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8) ON CONFLICT (message_hash) DO NOTHING",
        params![
            mailbox,
            looked.hash,
            looked.received_at,
            now.to_string(),
            looked.verdict,
            i64::try_from(paths.len()).unwrap_or(0),
            looked.kind.map(Kind::code),
            looked.paypal_code
        ],
    )?;
    // A mail of a shop keeps its orders (#32), and every mail counts as read for them.
    crate::orders::note(conn, &looked.hash, &looked.orders, now)?;
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

/// Whether a PayPal payment with this transaction code was taken before (#32).
pub fn paid_before(conn: &Connection, code: &str) -> rusqlite::Result<bool> {
    Ok(conn
        .query_row(
            "SELECT 1 FROM mail_messages WHERE paypal_code = ?1 AND taken > 0 LIMIT 1",
            [code],
            |_| Ok(()),
        )
        .optional()?
        .is_some())
}

/// Whether the orders of a mail looked at before were read (#32).
pub fn orders_read(conn: &Connection, hash: &str) -> rusqlite::Result<bool> {
    Ok(conn
        .query_row(
            "SELECT orders_read FROM mail_messages WHERE message_hash = ?1",
            [hash],
            |row| row.get::<_, bool>(0),
        )
        .optional()?
        .unwrap_or(true))
}

/// The ledger of a round in the archive: what it knows and where a mail looked at goes.
struct ArchiveLedger<'a> {
    reader: &'a Reader,
    mailbox: i64,
    taken: Vec<PathBuf>,
}

impl round::Ledger for ArchiveLedger<'_> {
    fn known(&self, hash: &str) -> bool {
        self.reader
            .with(|conn, _| is_known(conn, hash))
            .unwrap_or(true)
    }

    fn paid_before(&self, code: &str) -> bool {
        self.reader
            .with(|conn, _| paid_before(conn, code))
            .unwrap_or(false)
    }

    fn orders_read(&self, hash: &str) -> bool {
        self.reader
            .with(|conn, _| orders_read(conn, hash))
            .unwrap_or(true)
    }

    fn note(&mut self, looked: &Looked) {
        let mailbox = self.mailbox;
        if let Some(paths) = self
            .reader
            .with(|conn, staging| keep(conn, staging, mailbox, looked, Timestamp::now()))
        {
            self.taken.extend(paths);
        }
    }

    fn note_orders(&mut self, hash: &str, orders: &[crate::orders::Order]) {
        let _ = self
            .reader
            .with(|conn, _| crate::orders::note(conn, hash, orders, Timestamp::now()));
    }
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
    model: Option<Arc<dyn LocalModel>>,
    secrets: Box<dyn Secrets>,
    busy: AtomicBool,
}

impl Reader {
    pub fn new(
        archive: Arc<Mutex<Opened>>,
        intake: Arc<Intake>,
        model: Option<Arc<dyn LocalModel>>,
        secrets: Box<dyn Secrets>,
    ) -> Arc<Self> {
        Arc::new(Self {
            archive,
            intake,
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
        let mut ledger = ArchiveLedger {
            reader: self,
            mailbox: id,
            taken: Vec::new(),
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
            self.model.as_deref(),
            &mut ledger,
        )
        .await?;
        let taken = ledger.taken;
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

/// The kind the model gave the mail a file of the intake came from (#31), as a hint for when code
/// cannot tell: always for the text of a mail, for an attachment only when it was the mail's only
/// file, so terms or a revocation sent beside an invoice do not become one.
pub fn hint_for(conn: &Connection, staged: &Path) -> rusqlite::Result<Option<Kind>> {
    let Some(prefix) = staged
        .parent()
        .and_then(Path::file_name)
        .and_then(|name| name.to_str())
        .filter(|name| name.len() == 16)
    else {
        return Ok(None);
    };
    let found: Option<(Option<String>, i64, String)> = conn
        .query_row(
            "SELECT kind, taken, verdict FROM mail_messages WHERE substr(message_hash, 1, 16) = ?1",
            [prefix],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        )
        .optional()?;
    Ok(found.and_then(|(kind, taken, verdict)| {
        let text = verdict.starts_with("Übernommen als Text der Mail");
        (text || taken == 1)
            .then(|| kind.map(|kind| Kind::from_code(&kind)))
            .flatten()
            .filter(|kind| crate::downloads::money(*kind))
    }))
}

/// The signal a kind from the mail's verdict is stored with.
pub const HINT_SIGNAL: &str = "mail:model";

/// Documents from mail whose kind code could not tell, read again with the kind of their mail.
/// Runs at start, so documents taken before the hint existed get it too. Returns how many.
pub fn apply_hints(conn: &Connection, now: Timestamp) -> rusqlite::Result<usize> {
    let candidates: Vec<(i64, String, String)> = conn
        .prepare(
            "SELECT d.id, j.path, d.original_name FROM documents d
             JOIN intake_jobs j ON j.document_id = d.id AND j.source = 'mail'
             WHERE d.source = 'mail' AND d.kind IN ('other', 'unknown') AND d.state <> 'filed'
             AND NOT EXISTS (SELECT 1 FROM intake_jobs a WHERE a.document_id = d.id AND a.stage NOT IN ('filed', 'waiting', 'failed'))
             GROUP BY d.id",
        )?
        .query_map([], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)))?
        .collect::<rusqlite::Result<_>>()?;
    let mut changed = 0;
    for (id, path, name) in candidates {
        let Some(kind) = hint_for(conn, Path::new(&path))? else {
            continue;
        };
        let tx = conn.unchecked_transaction()?;
        crate::documents::set_kind(&tx, id, kind, &[HINT_SIGNAL])?;
        tx.execute(
            "INSERT INTO intake_jobs (source, path, name, stage, document_id, created_at, updated_at) VALUES ('mail', ?1, ?2, 'classified', ?3, ?4, ?4)",
            params![path, name, id, now.to_string()],
        )?;
        tx.commit()?;
        changed += 1;
    }
    Ok(changed)
}

/// The mails that 0.4.0 took as a PDF of their text are taken again as their text (the owner's
/// word of 2026-11-26): their documents go to the Papierkorb beside the archive, their staged
/// files and jobs go, and the mail is forgotten. The mailboxes then read from the start of their
/// reach again, where every other mail is known by its hash. Returns how many mails.
pub fn replace_mail_pdfs(conn: &Connection, root: &Path) -> rusqlite::Result<usize> {
    let mails: Vec<String> = conn
        .prepare(
            "SELECT message_hash FROM mail_messages WHERE verdict LIKE 'Übernommen als PDF der Mail%'",
        )?
        .query_map([], |row| row.get(0))?
        .collect::<rusqlite::Result<_>>()?;
    let mut replaced = 0;
    for hash in &mails {
        let prefix = &hash[..16];
        let jobs: Vec<(i64, String, Option<i64>)> = conn
            .prepare("SELECT id, path, document_id FROM intake_jobs WHERE source = 'mail' AND instr(path, ?1) > 0")?
            .query_map([prefix], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)))?
            .collect::<rusqlite::Result<_>>()?;
        let mut kept = false;
        for (_, _, document) in &jobs {
            if let Some(document) = document
                && crate::documents::delete(conn, root, *document).is_err()
            {
                kept = true;
            }
        }
        if kept {
            continue;
        }
        for (job, path, _) in &jobs {
            conn.execute("DELETE FROM intake_jobs WHERE id = ?1", [job])?;
            let path = Path::new(path);
            let _ = std::fs::remove_file(path);
            if let Some(dir) = path.parent() {
                let _ = std::fs::remove_dir(dir);
            }
        }
        conn.execute("DELETE FROM mail_messages WHERE message_hash = ?1", [hash])?;
        replaced += 1;
    }
    if replaced > 0 {
        conn.execute("UPDATE mailboxes SET last_uid = 0", [])?;
    }
    Ok(replaced)
}

#[cfg(test)]
mod tests;
