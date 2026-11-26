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
                let next = read_by_model(archive, pdfium, model, job, kill)?;
                if next == Stage::Model {
                    return Ok(next);
                }
                next
            }
            Stage::Checked => check(archive, job, kill)?,
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
        // A CSV statement keeps each line of its table as a line of one page (#21).
        (documents::FileType::Csv, _) => fs::read(&path).ok().map(|bytes| {
            let text = crate::statements::layout_b::decode(&bytes);
            let page = pages::PageText {
                number: 1,
                width: 0.0,
                height: 0.0,
                lines: crate::statements::layout_b::lines_of(&text),
            };
            (vec![page], "table", None)
        }),
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
            // A CSV statement is known by its layout (#21), a table has no sentences to count.
            let found = if crate::statements::layout_b::looks_like(&lines.join("\n")) {
                kinds::Classified {
                    kind: kinds::Kind::Statement,
                    signals: vec!["umsatzanzeige"],
                }
            } else {
                kinds::classify(lines.iter().map(String::as_str))
            };
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

/// What the model step needs of a document, read under the lock: the kind, the fields of the kind
/// no reader found, the lines, whether it has text, and where the file lies.
struct Asked {
    id: i64,
    kind: kinds::Kind,
    fields: Vec<readers::model::Field>,
    lines: Vec<readers::Line>,
    found: Vec<String>,
    scan: bool,
    stored: Option<(PathBuf, documents::FileType)>,
}

fn asked(conn: &rusqlite::Connection, root: &std::path::Path, id: i64) -> rusqlite::Result<Asked> {
    let (kind, state): (String, Option<String>) = conn.query_row(
        "SELECT kind, text_state FROM documents WHERE id = ?1",
        [id],
        |row| Ok((row.get(0)?, row.get(1)?)),
    )?;
    let kind = kinds::Kind::from_code(&kind);
    let found: Vec<String> = conn
        .prepare("SELECT DISTINCT field FROM field_values WHERE document_id = ?1 AND reader NOT LIKE 'model:%' AND reader NOT LIKE 'image:%'")?
        .query_map([id], |row| row.get(0))?
        .collect::<rusqlite::Result<_>>()?;
    let fields: Vec<readers::model::Field> = readers::model::fields_of(kind)
        .iter()
        .copied()
        // The decisive values are asked even when code found them: two independent readers (#16).
        .filter(|(name, _, _)| {
            !found.iter().any(|f| f == name) || readers::image::decisive(kind).contains(name)
        })
        .collect();
    Ok(Asked {
        id,
        kind,
        lines: if fields.is_empty() {
            Vec::new()
        } else {
            readers::lines_of(conn, id)?
        },
        fields,
        found,
        scan: matches!(state.as_deref(), Some("scan" | "image")),
        stored: documents::stored_of(conn, root, id)?,
    })
}

/// The stage after `read`: the local model reads the fields code left empty, from the text, and a
/// scan or a decisive value still missing from the page image (#15). Asking runs beside the
/// database. Without an answer the job waits in the stage `model` with the reason and is asked
/// again later. A document without anything to ask, or an app without a model, goes on at once.
fn read_by_model(
    archive: &Mutex<Opened>,
    pdfium: Option<&Pdfium>,
    model: Option<&dyn LocalModel>,
    job: &Job,
    kill: &mut dyn FnMut(Point) -> bool,
) -> Result<Stage, Stopped> {
    let asked = {
        let guard = archive.lock().map_err(|_| Stopped::Database)?;
        let Opened::Ready(ready) = &*guard else {
            return Err(Stopped::NotReady);
        };
        let root = documents::archive_root(ready.path());
        match jobs::document_of(ready.connection(), job.id).map_err(|_| Stopped::Database)? {
            Some(id) => Some(asked(ready.connection(), &root, id).map_err(|_| Stopped::Database)?),
            None => None,
        }
    };
    let (Some(model), Some(asked)) = (model, asked) else {
        return advance(archive, job, Stage::Read, kill);
    };
    let wait = |error: &crate::ollama::ModelError| error.should_wait().then(|| error.to_string());
    let spoiled = |error: crate::ollama::ModelError| readers::model::Finding {
        field: "",
        detail: error.to_string(),
    };

    let mut text = None;
    if !asked.scan && !asked.fields.is_empty() && !asked.lines.is_empty() {
        match readers::model::ask(model, asked.kind, &asked.fields, &asked.lines) {
            Err(error) if wait(&error).is_some() => {
                return hold_for_model(archive, job, &error.to_string());
            }
            Err(error) => {
                text = Some(readers::model::Read {
                    values: Vec::new(),
                    findings: vec![spoiled(error)],
                });
            }
            Ok(read) => text = Some(read),
        }
    }
    let mut found = asked.found.clone();
    found.extend(
        text.iter()
            .flat_map(|read| read.values.iter().map(|value| value.field.to_owned())),
    );
    let lacking = readers::image::lacks_decisive(asked.kind, &found);
    let mut image = None;
    if asked.scan || !lacking.is_empty() {
        let pngs = match (&asked.stored, pdfium) {
            (Some((path, file_type)), Some(pdfium)) => fs::read(path)
                .map(|bytes| {
                    readers::image::pages_of(pdfium, &bytes, *file_type, readers::image::WIDTH)
                })
                .unwrap_or_default(),
            _ => Vec::new(),
        };
        let kind = (!asked.scan).then_some(asked.kind);
        match readers::image::ask(model, kind, &lacking, &pngs) {
            Err(error) if wait(&error).is_some() => {
                return hold_for_model(archive, job, &error.to_string());
            }
            Err(error) => {
                image = Some(readers::image::Read {
                    findings: vec![spoiled(error)],
                    ..readers::image::Read::default()
                });
            }
            Ok(read) => image = Some(read),
        }
    }
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
    let name = model.name();
    if let Some(read) = &text {
        readers::model::save(&tx, asked.id, &name, read, now).map_err(|_| Stopped::Database)?;
    }
    if let Some(read) = &image {
        if let Some(kind) = read.kind {
            documents::set_kind(&tx, asked.id, kind, &["image"]).map_err(|_| Stopped::Database)?;
        }
        readers::image::save(&tx, asked.id, &name, read, now).map_err(|_| Stopped::Database)?;
    }
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

/// The stage `checked`: the checks of #16 decide, in one transaction with the stage, whether the
/// document is filed by itself or waits for the owner with its reasons.
fn check(
    archive: &Mutex<Opened>,
    job: &Job,
    kill: &mut dyn FnMut(Point) -> bool,
) -> Result<Stage, Stopped> {
    let next = {
        let guard = archive.lock().map_err(|_| Stopped::Database)?;
        let Opened::Ready(ready) = &*guard else {
            return Err(Stopped::NotReady);
        };
        let tx = ready
            .connection()
            .unchecked_transaction()
            .map_err(|_| Stopped::Database)?;
        let next = match jobs::document_of(&tx, job.id).map_err(|_| Stopped::Database)? {
            Some(id) => {
                let verdict = crate::checks::decide(&tx, id, Timestamp::now())
                    .map_err(|_| Stopped::Database)?;
                if verdict.filed() {
                    Stage::Filed
                } else {
                    Stage::Waiting
                }
            }
            None => Stage::Filed,
        };
        jobs::set_stage(&tx, job.id, next, Timestamp::now()).map_err(|_| Stopped::Database)?;
        if kill(Point::BeforeCommit) {
            return Err(Stopped::Killed);
        }
        tx.commit().map_err(|_| Stopped::Database)?;
        next
    };
    if kill(Point::AfterCommit) {
        return Err(Stopped::Killed);
    }
    Ok(next)
}

/// Holds the job in the stage `model` with the German reason the model gave none.
fn hold_for_model(archive: &Mutex<Opened>, job: &Job, reason: &str) -> Result<Stage, Stopped> {
    let guard = archive.lock().map_err(|_| Stopped::Database)?;
    let Opened::Ready(ready) = &*guard else {
        return Err(Stopped::NotReady);
    };
    jobs::wait_for_model(ready.connection(), job.id, reason, Timestamp::now())
        .map_err(|_| Stopped::Database)?;
    Ok(Stage::Model)
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

    /// The local model, when one was set.
    pub fn model(&self) -> Option<Arc<dyn LocalModel>> {
        self.model.get().cloned()
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
