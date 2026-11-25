//! The intake queue: every way in creates jobs, workers move them through the stages, and a
//! restart resumes each job at the stage it reached (FA-09, knowledge/decision-intake.md).

pub mod jobs;

use std::collections::HashSet;
use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Condvar, Mutex, OnceLock};
use std::time::{Duration, Instant};

use jiff::Timestamp;

use crate::archive::Opened;
use crate::documents::{self, Outcome, Refusal, Source};
use crate::kinds;
use crate::ollama::LocalModel;
use crate::pages;
use crate::pdf::Pdfium;
use crate::readers;
pub use jobs::{Job, Stage};

/// The places where a test may stop a worker as if the app had been killed there.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Point {
    /// The file was read and checked, nothing was written.
    Prepared,
    /// The file lies under its hash, the transaction with its row and the job is not committed.
    BeforeCommit,
    /// The store is committed.
    AfterCommit,
    /// A pass-through stage is committed.
    Advanced,
}

#[derive(Debug, PartialEq, Eq)]
pub enum Stopped {
    /// A test killed the worker at a `Point`.
    Killed,
    /// The archive could not be written. The job keeps its stage and is tried again.
    Database,
    /// The archive is not open, nothing can be done.
    NotReady,
}

/// Moves one job through its stages until it is done, the queue is paused, or `kill` says stop.
/// `kill` is asked at every `Point`. The app passes one that never stops.
pub fn process(
    archive: &Mutex<Opened>,
    pdfium: Option<&Pdfium>,
    model: Option<&dyn LocalModel>,
    job: &Job,
    paused: &AtomicBool,
    kill: &mut dyn FnMut(Point) -> bool,
) -> Result<Stage, Stopped> {
    let mut stage = job.stage;
    while !stage.is_final() && !paused.load(Ordering::SeqCst) {
        stage = match stage {
            Stage::Queued => store(archive, pdfium, job, kill)?,
            Stage::Stored => read_text(archive, pdfium, job, kill)?,
            Stage::Text => classify(archive, job, kill)?,
            Stage::Classified => read_values(archive, job, kill)?,
            Stage::Read | Stage::Model => {
                let next = read_by_model(archive, model, job, kill)?;
                if next == Stage::Model {
                    return Ok(next);
                }
                next
            }
            _ => advance(archive, job, stage, kill)?,
        };
    }
    Ok(stage)
}

fn store(
    archive: &Mutex<Opened>,
    pdfium: Option<&Pdfium>,
    job: &Job,
    kill: &mut dyn FnMut(Point) -> bool,
) -> Result<Stage, Stopped> {
    // Reading and checking need no database, so several workers do them at once.
    let prepared = fs::read(&job.path)
        .map_err(|_| Refusal::Unreadable)
        .and_then(|bytes| documents::prepare(pdfium, &bytes).map(|prepared| (prepared, bytes)));
    if kill(Point::Prepared) {
        return Err(Stopped::Killed);
    }
    let guard = archive.lock().map_err(|_| Stopped::Database)?;
    let Opened::Ready(ready) = &*guard else {
        return Err(Stopped::NotReady);
    };
    let root = documents::archive_root(ready.path());
    let now = Timestamp::now();
    let tx = ready
        .connection()
        .unchecked_transaction()
        .map_err(|_| Stopped::Database)?;
    let outcome = match prepared {
        Ok((prepared, bytes)) => {
            documents::store(&tx, &root, &prepared, &bytes, &job.name, job.source, now)
                .unwrap_or_else(Outcome::Refused)
        }
        Err(refusal) => Outcome::Refused(refusal),
    };
    let stage = jobs::record_store(&tx, job.id, &outcome, now).map_err(|_| Stopped::Database)?;
    if kill(Point::BeforeCommit) {
        return Err(Stopped::Killed);
    }
    tx.commit().map_err(|_| Stopped::Database)?;
    if kill(Point::AfterCommit) {
        return Err(Stopped::Killed);
    }
    Ok(stage)
}

/// The stage `text`: the lines of every page with their place, or the mark that a PDF is a scan
/// or the file an image. Reading runs beside the database, saving is one transaction with the stage.
fn read_text(
    archive: &Mutex<Opened>,
    pdfium: Option<&Pdfium>,
    job: &Job,
    kill: &mut dyn FnMut(Point) -> bool,
) -> Result<Stage, Stopped> {
    let located = {
        let guard = archive.lock().map_err(|_| Stopped::Database)?;
        let Opened::Ready(ready) = &*guard else {
            return Err(Stopped::NotReady);
        };
        let root = documents::archive_root(ready.path());
        let document =
            jobs::document_of(ready.connection(), job.id).map_err(|_| Stopped::Database)?;
        match document {
            Some(id) => documents::stored_of(ready.connection(), &root, id)
                .map_err(|_| Stopped::Database)?
                .map(|(path, file_type)| (id, path, file_type)),
            None => None,
        }
    };
    // A job without its document (it cannot happen after a store) goes on without text.
    let Some((id, path, file_type)) = located else {
        return advance(archive, job, Stage::Stored, kill);
    };
    let read = match (file_type, pdfium) {
        (documents::FileType::Pdf, Some(pdfium)) => fs::read(&path).ok().and_then(|bytes| {
            let read = pages::read(pdfium, &bytes).ok()?;
            let state = if pages::is_scan(&read) {
                "scan"
            } else {
                "text"
            };
            Some((read, state, readers::e_invoice::attachment(pdfium, &bytes)))
        }),
        (documents::FileType::Pdf, None) => None,
        _ => Some((Vec::new(), "image", None)),
    };
    if kill(Point::Prepared) {
        return Err(Stopped::Killed);
    }
    let guard = archive.lock().map_err(|_| Stopped::Database)?;
    let Opened::Ready(ready) = &*guard else {
        return Err(Stopped::NotReady);
    };
    let tx = ready
        .connection()
        .unchecked_transaction()
        .map_err(|_| Stopped::Database)?;
    if let Some((read, state, attachment)) = read {
        documents::save_pages(&tx, id, &read, state).map_err(|_| Stopped::Database)?;
        readers::e_invoice::save_attachment(&tx, id, attachment.as_ref())
            .map_err(|_| Stopped::Database)?;
    }
    jobs::set_stage(&tx, job.id, Stage::Text, Timestamp::now()).map_err(|_| Stopped::Database)?;
    if kill(Point::BeforeCommit) {
        return Err(Stopped::Killed);
    }
    tx.commit().map_err(|_| Stopped::Database)?;
    if kill(Point::AfterCommit) {
        return Err(Stopped::Killed);
    }
    Ok(Stage::Text)
}

/// The stage `classified`: the kind of the document from the lines the stage `text` stored, with
/// the signals it rests on, in one transaction with the stage.
fn classify(
    archive: &Mutex<Opened>,
    job: &Job,
    kill: &mut dyn FnMut(Point) -> bool,
) -> Result<Stage, Stopped> {
    {
        let guard = archive.lock().map_err(|_| Stopped::Database)?;
        let Opened::Ready(ready) = &*guard else {
            return Err(Stopped::NotReady);
        };
        let tx = ready
            .connection()
            .unchecked_transaction()
            .map_err(|_| Stopped::Database)?;
        if let Some(id) = jobs::document_of(&tx, job.id).map_err(|_| Stopped::Database)? {
            let lines = documents::lines_of(&tx, id).map_err(|_| Stopped::Database)?;
            let found = kinds::classify(lines.iter().map(String::as_str));
            documents::set_kind(&tx, id, found.kind, &found.signals)
                .map_err(|_| Stopped::Database)?;
        }
        jobs::set_stage(&tx, job.id, Stage::Classified, Timestamp::now())
            .map_err(|_| Stopped::Database)?;
        if kill(Point::BeforeCommit) {
            return Err(Stopped::Killed);
        }
        tx.commit().map_err(|_| Stopped::Database)?;
    }
    if kill(Point::AfterCommit) {
        return Err(Stopped::Killed);
    }
    Ok(Stage::Classified)
}

/// The stage `read`: the readers that fit the kind fill `field_values`, in one transaction with
/// the stage. The model and the image reader join in #14 and #15.
fn read_values(
    archive: &Mutex<Opened>,
    job: &Job,
    kill: &mut dyn FnMut(Point) -> bool,
) -> Result<Stage, Stopped> {
    {
        let guard = archive.lock().map_err(|_| Stopped::Database)?;
        let Opened::Ready(ready) = &*guard else {
            return Err(Stopped::NotReady);
        };
        let tx = ready
            .connection()
            .unchecked_transaction()
            .map_err(|_| Stopped::Database)?;
        if let Some(id) = jobs::document_of(&tx, job.id).map_err(|_| Stopped::Database)? {
            let kind: String = tx
                .query_row("SELECT kind FROM documents WHERE id = ?1", [id], |row| {
                    row.get(0)
                })
                .map_err(|_| Stopped::Database)?;
            readers::read_document(&tx, id, kinds::Kind::from_code(&kind), Timestamp::now())
                .map_err(|_| Stopped::Database)?;
        }
        jobs::set_stage(&tx, job.id, Stage::Read, Timestamp::now())
            .map_err(|_| Stopped::Database)?;
        if kill(Point::BeforeCommit) {
            return Err(Stopped::Killed);
        }
        tx.commit().map_err(|_| Stopped::Database)?;
    }
    if kill(Point::AfterCommit) {
        return Err(Stopped::Killed);
    }
    Ok(Stage::Read)
}

fn advance(
    archive: &Mutex<Opened>,
    job: &Job,
    stage: Stage,
    kill: &mut dyn FnMut(Point) -> bool,
) -> Result<Stage, Stopped> {
    let next = stage.next();
    {
        let guard = archive.lock().map_err(|_| Stopped::Database)?;
        let Opened::Ready(ready) = &*guard else {
            return Err(Stopped::NotReady);
        };
        jobs::set_stage(ready.connection(), job.id, next, Timestamp::now())
            .map_err(|_| Stopped::Database)?;
    }
    if kill(Point::Advanced) {
        return Err(Stopped::Killed);
    }
    Ok(next)
}

/// The fields the model is asked for: those of the kind no reader found. Read under the lock.
fn missing_fields(
    conn: &rusqlite::Connection,
    document: i64,
) -> rusqlite::Result<(kinds::Kind, Vec<readers::model::Field>, Vec<readers::Line>)> {
    let kind: String = conn.query_row(
        "SELECT kind FROM documents WHERE id = ?1",
        [document],
        |row| row.get(0),
    )?;
    let kind = kinds::Kind::from_code(&kind);
    let found: HashSet<String> = conn
        .prepare("SELECT DISTINCT field FROM field_values WHERE document_id = ?1 AND reader NOT LIKE 'model:%'")?
        .query_map([document], |row| row.get(0))?
        .collect::<rusqlite::Result<_>>()?;
    let fields: Vec<readers::model::Field> = readers::model::fields_of(kind)
        .iter()
        .copied()
        .filter(|(name, _, _)| !found.contains(*name))
        .collect();
    let lines = if fields.is_empty() {
        Vec::new()
    } else {
        readers::lines_of(conn, document)?
    };
    Ok((kind, fields, lines))
}

/// The stage after `read`: the local model reads the fields code left empty. Asking runs beside
/// the database. Without an answer the job waits in the stage `model` with the reason and is asked
/// again later. A document without missing fields, or an app without a model, goes on at once.
fn read_by_model(
    archive: &Mutex<Opened>,
    model: Option<&dyn LocalModel>,
    job: &Job,
    kill: &mut dyn FnMut(Point) -> bool,
) -> Result<Stage, Stopped> {
    let asked = {
        let guard = archive.lock().map_err(|_| Stopped::Database)?;
        let Opened::Ready(ready) = &*guard else {
            return Err(Stopped::NotReady);
        };
        match jobs::document_of(ready.connection(), job.id).map_err(|_| Stopped::Database)? {
            Some(id) => Some((
                id,
                missing_fields(ready.connection(), id).map_err(|_| Stopped::Database)?,
            )),
            None => None,
        }
    };
    let (Some(model), Some((id, (kind, fields, lines)))) = (model, asked) else {
        return advance(archive, job, Stage::Read, kill);
    };
    if fields.is_empty() || lines.is_empty() {
        return advance(archive, job, Stage::Read, kill);
    }
    let answer = readers::model::ask(model, kind, &fields, &lines);
    if kill(Point::Prepared) {
        return Err(Stopped::Killed);
    }
    let guard = archive.lock().map_err(|_| Stopped::Database)?;
    let Opened::Ready(ready) = &*guard else {
        return Err(Stopped::NotReady);
    };
    let tx = ready
        .connection()
        .unchecked_transaction()
        .map_err(|_| Stopped::Database)?;
    let now = Timestamp::now();
    let read = match answer {
        Err(error) if error.should_wait() => {
            jobs::wait_for_model(&tx, job.id, &error.to_string(), now)
                .map_err(|_| Stopped::Database)?;
            tx.commit().map_err(|_| Stopped::Database)?;
            return Ok(Stage::Model);
        }
        // An answer the model spoiled is a finding, the document goes on without its values.
        Err(error) => readers::model::Read {
            values: Vec::new(),
            findings: vec![readers::model::Finding {
                field: "",
                detail: error.to_string(),
            }],
        },
        Ok(read) => read,
    };
    readers::model::save(&tx, id, &model.name(), &read, now).map_err(|_| Stopped::Database)?;
    jobs::set_stage(&tx, job.id, Stage::Checked, now).map_err(|_| Stopped::Database)?;
    if kill(Point::BeforeCommit) {
        return Err(Stopped::Killed);
    }
    tx.commit().map_err(|_| Stopped::Database)?;
    if kill(Point::AfterCommit) {
        return Err(Stopped::Killed);
    }
    Ok(Stage::Checked)
}

/// How long the jobs in the stage `model` rest after Ollama gave no answer.
pub const MODEL_REST: Duration = Duration::from_secs(30);

/// The queue of the running app: its workers, the pause and who hears about changes.
pub struct Intake {
    archive: Arc<Mutex<Opened>>,
    paused: AtomicBool,
    claimed: Mutex<HashSet<i64>>,
    wake: (Mutex<()>, Condvar),
    on_change: Box<dyn Fn() + Send + Sync>,
    model: OnceLock<Arc<dyn LocalModel>>,
    model_rests_until: Mutex<Option<Instant>>,
}

#[derive(Debug, thiserror::Error)]
#[error("Eingang: Datenbank")]
pub struct IntakeError;

impl Intake {
    pub fn new(archive: Arc<Mutex<Opened>>, on_change: Box<dyn Fn() + Send + Sync>) -> Arc<Self> {
        Arc::new(Self {
            archive,
            paused: AtomicBool::new(false),
            claimed: Mutex::new(HashSet::new()),
            wake: (Mutex::new(()), Condvar::new()),
            on_change,
            model: OnceLock::new(),
            model_rests_until: Mutex::new(None),
        })
    }

    /// The local model the workers ask for the fields code left empty. Set once, before `start`.
    pub fn set_model(&self, model: Arc<dyn LocalModel>) {
        let _ = self.model.set(model);
    }

    fn rest_model(&self) {
        if let Ok(mut until) = self.model_rests_until.lock() {
            *until = Some(Instant::now() + MODEL_REST);
        }
    }

    fn model_rests(&self) -> bool {
        self.model_rests_until
            .lock()
            .ok()
            .and_then(|until| *until)
            .is_some_and(|until| Instant::now() < until)
    }

    pub fn archive(&self) -> &Mutex<Opened> {
        &self.archive
    }

    /// Starts the workers. Each runs until the app ends.
    pub fn start(self: &Arc<Self>, workers: usize, pdfium: Option<&'static Pdfium>) {
        for _ in 0..workers.max(1) {
            let intake = Arc::clone(self);
            std::thread::spawn(move || intake.work(pdfium));
        }
    }

    pub fn enqueue(&self, paths: &[PathBuf], source: Source) -> Result<u32, IntakeError> {
        let count = {
            let guard = self.archive.lock().map_err(|_| IntakeError)?;
            let Opened::Ready(ready) = &*guard else {
                return Err(IntakeError);
            };
            jobs::enqueue(ready.connection(), paths, source, Timestamp::now())
                .map_err(|_| IntakeError)?
        };
        self.wake.1.notify_all();
        (self.on_change)();
        Ok(count)
    }

    pub fn set_paused(&self, paused: bool) {
        self.paused.store(paused, Ordering::SeqCst);
        self.wake.1.notify_all();
        (self.on_change)();
    }

    pub fn is_paused(&self) -> bool {
        self.paused.load(Ordering::SeqCst)
    }

    fn work(&self, pdfium: Option<&'static Pdfium>) {
        loop {
            match self.claim() {
                Some(job) if !self.is_paused() => {
                    // A stop for the database or a closed archive leaves the job where it was.
                    let model = self.model.get().map(|model| model.as_ref());
                    let ended = process(
                        &self.archive,
                        pdfium,
                        model,
                        &job,
                        &self.paused,
                        &mut |_| false,
                    );
                    if ended == Ok(Stage::Model) {
                        self.rest_model();
                    }
                    if let Ok(mut claimed) = self.claimed.lock() {
                        claimed.remove(&job.id);
                    }
                    self.snapshot_when_done();
                    (self.on_change)();
                }
                claimed => {
                    if let (Some(job), Ok(mut set)) = (claimed, self.claimed.lock()) {
                        set.remove(&job.id);
                    }
                    self.sleep();
                }
            }
        }
    }

    /// Once the queue is empty, a snapshot when many documents came in (#19). Under the lock of
    /// the archive, so two workers that finish together take one snapshot.
    fn snapshot_when_done(&self) {
        let Ok(guard) = self.archive.lock() else {
            return;
        };
        let Opened::Ready(ready) = &*guard else {
            return;
        };
        if jobs::count_busy(ready.connection()).ok() != Some(0) {
            return;
        }
        if crate::archive::snapshots::after_import(
            ready.connection(),
            ready.path(),
            Timestamp::now(),
        )
        .is_err()
        {
            eprintln!("Der Sicherungspunkt nach dem Import schlug fehl");
        }
    }

    /// The oldest open job no other worker holds.
    fn claim(&self) -> Option<Job> {
        let open = {
            let guard = self.archive.lock().ok()?;
            let Opened::Ready(ready) = &*guard else {
                return None;
            };
            jobs::open_jobs(ready.connection()).ok()?
        };
        let rests = self.model_rests();
        let mut claimed = self.claimed.lock().ok()?;
        let job = open
            .into_iter()
            .filter(|job| !(rests && job.stage == Stage::Model))
            .find(|job| !claimed.contains(&job.id))?;
        claimed.insert(job.id);
        Some(job)
    }

    fn sleep(&self) {
        if let Ok(guard) = self.wake.0.lock() {
            let _ = self.wake.1.wait_timeout(guard, Duration::from_secs(2));
        }
    }
}

#[cfg(test)]
mod tests;
