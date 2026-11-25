use std::path::Path;
use std::sync::atomic::AtomicBool;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use super::*;
use crate::archive::{self, Opened};
use crate::testing;

fn open(path: &Path) -> Mutex<Opened> {
    let opened = archive::open(path, testing::t0()).unwrap();
    assert!(matches!(opened, Opened::Ready(_)));
    Mutex::new(opened)
}

/// `count` distinct files in a folder: every tenth a PDF, the rest small PNGs.
fn files(dir: &Path, count: usize) -> Vec<PathBuf> {
    (0..count)
        .map(|index| {
            let (name, bytes) = if index % 10 == 0 {
                (
                    format!("rechnung-{index:03}.pdf"),
                    testing::pdf(&[&format!("Rechnung {index}")]),
                )
            } else {
                let mut png = testing::PNG.to_vec();
                png.extend_from_slice(format!("bild {index}").as_bytes());
                (format!("bild-{index:03}.png"), png)
            };
            let path = dir.join(name);
            std::fs::write(&path, bytes).unwrap();
            path
        })
        .collect()
}

fn with_ready<T>(archive: &Mutex<Opened>, read: impl FnOnce(&rusqlite::Connection) -> T) -> T {
    let guard = archive.lock().unwrap();
    let Opened::Ready(ready) = &*guard else {
        panic!("not ready")
    };
    read(ready.connection())
}

fn count(archive: &Mutex<Opened>, sql: &str) -> i64 {
    with_ready(archive, |conn| {
        conn.query_row(sql, [], |row| row.get(0)).unwrap()
    })
}

/// Runs every open job to its end in this thread, asking `kill` at each point.
fn drain(archive: &Mutex<Opened>, kill: &mut dyn FnMut(Point) -> bool) -> Result<(), Stopped> {
    let running = AtomicBool::new(false);
    loop {
        let open = with_ready(archive, |conn| jobs::open_jobs(conn).unwrap());
        let Some(job) = open.into_iter().next() else {
            return Ok(());
        };
        process(archive, Some(testing::pdfium()), &job, &running, kill)?;
    }
}

#[test]
fn a_folder_becomes_one_job_per_file_without_system_files() {
    let dir = tempfile::tempdir().unwrap();
    let archive = open(&dir.path().join(archive::FILE_NAME));
    let source = tempfile::tempdir().unwrap();
    std::fs::create_dir(source.path().join("unter")).unwrap();
    files(&source.path().join("unter"), 3);
    std::fs::write(source.path().join("Thumbs.db"), b"x").unwrap();
    let queued = with_ready(&archive, |conn| {
        jobs::enqueue(
            conn,
            &[source.path().to_path_buf()],
            Source::Drop,
            testing::t0(),
        )
        .unwrap()
    });
    assert_eq!(queued, 3);
    let open = with_ready(&archive, |conn| jobs::open_jobs(conn).unwrap());
    assert!(
        open.iter()
            .all(|job| job.stage == Stage::Queued && job.source == Source::Drop)
    );
}

#[test]
fn every_job_ends_filed_or_failed_with_its_reason() {
    let dir = tempfile::tempdir().unwrap();
    let archive = open(&dir.path().join(archive::FILE_NAME));
    let source = tempfile::tempdir().unwrap();
    let mut paths = files(source.path(), 5);
    let copy = source.path().join("kopie.png");
    std::fs::copy(&paths[1], &copy).unwrap();
    let text = source.path().join("notiz.txt");
    std::fs::write(&text, b"Notiz").unwrap();
    paths.extend([copy, text]);
    with_ready(&archive, |conn| {
        jobs::enqueue(conn, &paths, Source::Dialog, testing::t0()).unwrap()
    });
    drain(&archive, &mut |_| false).unwrap();

    assert_eq!(count(&archive, "SELECT count(*) FROM documents"), 5);
    assert_eq!(
        count(
            &archive,
            "SELECT count(*) FROM intake_jobs WHERE stage = 'filed' AND known = 0"
        ),
        5
    );
    assert_eq!(
        count(
            &archive,
            "SELECT count(*) FROM intake_jobs WHERE stage = 'filed' AND known = 1"
        ),
        1
    );
    let failed = with_ready(&archive, |conn| jobs::listed(conn, false, 10).unwrap());
    assert_eq!(failed.len(), 1);
    assert_eq!(failed[0].name, "notiz.txt");
    assert!(
        failed[0]
            .error
            .as_deref()
            .unwrap()
            .contains("PDF, PNG, JPEG und WebP")
    );
}

#[test]
fn the_stage_text_stores_the_lines_and_marks_scans_and_images() {
    let dir = tempfile::tempdir().unwrap();
    let archive = open(&dir.path().join(archive::FILE_NAME));
    let source = tempfile::tempdir().unwrap();
    let invoice = source.path().join("rechnung.pdf");
    std::fs::write(
        &invoice,
        testing::pdf(&["Rechnung Nr. 7", "Brutto 49,99 EUR"]),
    )
    .unwrap();
    let scan = source.path().join("scan.pdf");
    std::fs::write(&scan, testing::pdf_without_text()).unwrap();
    let photo = source.path().join("foto.png");
    std::fs::write(&photo, testing::PNG).unwrap();
    with_ready(&archive, |conn| {
        jobs::enqueue(conn, &[invoice, scan, photo], Source::Dialog, testing::t0()).unwrap()
    });
    drain(&archive, &mut |_| false).unwrap();

    let state = |name: &str| -> String {
        with_ready(&archive, |conn| {
            conn.query_row(
                "SELECT text_state FROM documents WHERE original_name = ?1",
                [name],
                |row| row.get(0),
            )
            .unwrap()
        })
    };
    assert_eq!(state("rechnung.pdf"), "text");
    assert_eq!(state("scan.pdf"), "scan");
    assert_eq!(state("foto.png"), "image");
    let lines: Vec<String> = with_ready(&archive, |conn| {
        let mut statement = conn
            .prepare("SELECT l.text FROM page_lines l JOIN documents d ON d.id = l.document_id WHERE d.original_name = 'rechnung.pdf' ORDER BY l.line")
            .unwrap();
        statement
            .query_map([], |row| row.get(0))
            .unwrap()
            .map(Result::unwrap)
            .collect()
    });
    assert_eq!(lines, ["Rechnung Nr. 7", "Brutto 49,99 EUR"]);
    assert_eq!(
        count(&archive, "SELECT count(*) FROM pages"),
        2,
        "one page each for the two PDFs, none for the image"
    );
}

#[test]
fn the_stage_classified_gives_each_document_its_kind_and_signals() {
    let dir = tempfile::tempdir().unwrap();
    let archive = open(&dir.path().join(archive::FILE_NAME));
    let source = tempfile::tempdir().unwrap();
    let invoice = source.path().join("rechnung.pdf");
    std::fs::write(
        &invoice,
        testing::pdf(&[
            "Rechnungsnummer R-7",
            "Umsatzsteuer 19 % 7,98 EUR",
            "Rechnungsbetrag 49,99 EUR",
        ]),
    )
    .unwrap();
    let payslip = source.path().join("gehalt.pdf");
    std::fs::write(
        &payslip,
        testing::pdf(&[
            "Verdienstabrechnung März 2026",
            "Gesamtbrutto 3.800,00",
            "Auszahlungsbetrag 2.512,10",
        ]),
    )
    .unwrap();
    let scan = source.path().join("scan.pdf");
    std::fs::write(&scan, testing::pdf_without_text()).unwrap();
    with_ready(&archive, |conn| {
        jobs::enqueue(
            conn,
            &[invoice, payslip, scan],
            Source::Dialog,
            testing::t0(),
        )
        .unwrap()
    });
    drain(&archive, &mut |_| false).unwrap();

    let row = |name: &str| -> (String, String) {
        with_ready(&archive, |conn| {
            conn.query_row(
                "SELECT kind, kind_signals FROM documents WHERE original_name = ?1",
                [name],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .unwrap()
        })
    };
    assert_eq!(
        row("rechnung.pdf"),
        (
            "invoice".into(),
            r#"["rechnungsnummer","rechnungsbetrag","umsatzsteuer"]"#.into()
        )
    );
    assert_eq!(row("gehalt.pdf").0, "payslip");
    assert_eq!(row("scan.pdf"), ("unknown".into(), "[]".into()));
}

#[test]
fn a_large_import_is_saved_once_the_queue_is_empty() {
    use crate::archive::snapshots::{self, AFTER_IMPORT_THRESHOLD};
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join(archive::FILE_NAME);
    let archive = open(&path);
    let source = tempfile::tempdir().unwrap();
    let threshold = usize::try_from(AFTER_IMPORT_THRESHOLD).unwrap();
    let paths = files(source.path(), threshold);
    let now = Timestamp::now();

    with_ready(&archive, |conn| {
        jobs::enqueue(conn, &paths[..threshold - 1], Source::Drop, testing::t0()).unwrap()
    });
    drain(&archive, &mut |_| false).unwrap();
    with_ready(&archive, |conn| {
        assert_eq!(
            snapshots::after_import(conn, &path, now).unwrap(),
            None,
            "below the threshold"
        )
    });

    with_ready(&archive, |conn| {
        jobs::enqueue(conn, &paths[threshold - 1..], Source::Drop, testing::t0()).unwrap()
    });
    drain(&archive, &mut |_| false).unwrap();
    let taken = with_ready(&archive, |conn| {
        snapshots::after_import(conn, &path, now).unwrap()
    });
    assert!(taken.is_some_and(|file| file.to_string_lossy().ends_with("-nach-import.db")));
    let later = now + jiff::SignedDuration::from_secs(5);
    with_ready(&archive, |conn| {
        assert_eq!(
            snapshots::after_import(conn, &path, later).unwrap(),
            None,
            "nothing new since"
        )
    });
    assert_eq!(snapshots::list(&path).unwrap().len(), 1);
}

#[test]
fn the_workers_save_a_large_import_by_themselves() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join(archive::FILE_NAME);
    let archive = Arc::new(open(&path));
    let intake = Intake::new(Arc::clone(&archive), Box::new(|| {}));
    let source = tempfile::tempdir().unwrap();
    let paths = files(source.path(), 25);
    intake.start(2, Some(testing::pdfium()));
    intake.enqueue(&paths, Source::Dialog).unwrap();
    let deadline = Instant::now() + Duration::from_secs(60);
    while crate::archive::snapshots::list(&path).unwrap().is_empty() {
        assert!(Instant::now() < deadline, "no snapshot after the import");
        std::thread::sleep(Duration::from_millis(20));
    }
    assert_eq!(count(&archive, "SELECT count(*) FROM documents"), 25);
}

#[test]
fn a_paused_job_keeps_its_stage_and_goes_on_later() {
    let dir = tempfile::tempdir().unwrap();
    let archive = open(&dir.path().join(archive::FILE_NAME));
    let source = tempfile::tempdir().unwrap();
    let paths = files(source.path(), 1);
    with_ready(&archive, |conn| {
        jobs::enqueue(conn, &paths, Source::Dialog, testing::t0()).unwrap()
    });
    let job = with_ready(&archive, |conn| jobs::open_jobs(conn).unwrap()).remove(0);
    let paused = AtomicBool::new(false);
    // Pause as soon as the store is committed, as the owner's click would between two stages.
    let stage = process(
        &archive,
        Some(testing::pdfium()),
        &job,
        &paused,
        &mut |point| {
            if point == Point::AfterCommit {
                paused.store(true, std::sync::atomic::Ordering::SeqCst);
            }
            false
        },
    )
    .unwrap();
    assert_eq!(stage, Stage::Stored);
    let again = with_ready(&archive, |conn| jobs::open_jobs(conn).unwrap()).remove(0);
    assert_eq!(again.stage, Stage::Stored);
    paused.store(false, std::sync::atomic::Ordering::SeqCst);
    assert_eq!(
        process(
            &archive,
            Some(testing::pdfium()),
            &again,
            &paused,
            &mut |_| false
        )
        .unwrap(),
        Stage::Filed
    );
}

/// A small, seeded random source, so a failing round can be run again.
struct Rng(u64);

impl Rng {
    fn below(&mut self, bound: u64) -> u64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        self.0 % bound
    }
}

#[test]
fn killed_at_random_points_and_started_again_ends_with_every_file_once() {
    const FILES: usize = 200;
    let source = tempfile::tempdir().unwrap();
    let paths = files(source.path(), FILES);
    for seed in [7_u64, 1_234, 99_991] {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join(archive::FILE_NAME);
        let first = open(&path);
        with_ready(&first, |conn| {
            jobs::enqueue(conn, &paths, Source::Drop, testing::t0()).unwrap()
        });
        drop(first);

        let mut rng = Rng(seed);
        let mut kills = 0;
        loop {
            // Every start opens the archive afresh from the disk, as the app does.
            let archive = open(&path);
            let mut budget = rng.below(120) + 1;
            let result = drain(&archive, &mut |_| {
                budget -= 1;
                budget == 0
            });
            match result {
                Ok(()) => {
                    assert_eq!(
                        count(&archive, "SELECT count(*) FROM documents"),
                        FILES as i64,
                        "seed {seed}"
                    );
                    assert_eq!(
                        count(&archive, "SELECT count(DISTINCT sha256) FROM documents"),
                        FILES as i64
                    );
                    assert_eq!(
                        count(
                            &archive,
                            "SELECT count(*) FROM intake_jobs WHERE stage = 'filed' AND known = 0"
                        ),
                        FILES as i64
                    );
                    let root = with_ready(&archive, |_| crate::documents::archive_root(&path));
                    let stored = walk_files(&root)
                        .into_iter()
                        .filter(|f| f.extension().is_some_and(|e| e != "part"))
                        .count();
                    assert_eq!(stored, FILES, "one file under its hash per document");
                    break;
                }
                Err(Stopped::Killed) => kills += 1,
                Err(other) => panic!("seed {seed}: {other:?}"),
            }
        }
        assert!(kills > 10, "seed {seed} was killed only {kills} times");
    }
}

fn walk_files(dir: &Path) -> Vec<PathBuf> {
    crate::documents::expand(&[dir.to_path_buf()])
}

#[test]
fn several_workers_drain_the_queue_and_report_each_change() {
    let dir = tempfile::tempdir().unwrap();
    let archive = Arc::new(open(&dir.path().join(archive::FILE_NAME)));
    let changes = Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let heard = Arc::clone(&changes);
    let intake = Intake::new(
        Arc::clone(&archive),
        Box::new(move || {
            heard.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        }),
    );
    let source = tempfile::tempdir().unwrap();
    let paths = files(source.path(), 40);
    intake.start(4, Some(testing::pdfium()));
    assert_eq!(intake.enqueue(&paths, Source::Dialog).unwrap(), 40);
    let deadline = Instant::now() + Duration::from_secs(60);
    while count(
        &archive,
        "SELECT count(*) FROM intake_jobs WHERE stage = 'filed'",
    ) < 40
    {
        assert!(Instant::now() < deadline, "the workers did not finish");
        std::thread::sleep(Duration::from_millis(20));
    }
    assert_eq!(count(&archive, "SELECT count(*) FROM documents"), 40);
    // A worker reports after the job is filed (and after the snapshot check of #19), so the last
    // report may come a moment after the last row.
    while changes.load(std::sync::atomic::Ordering::SeqCst) < 40 {
        assert!(
            Instant::now() < deadline,
            "the workers did not report every change"
        );
        std::thread::sleep(Duration::from_millis(5));
    }
}

const CHILD_ARCHIVE: &str = "FINANZARCHIV2_KILL_TEST_ARCHIVE";

/// The worker process of `the_process_killed_at_random_moments_loses_nothing`. It does nothing
/// unless that test starts it with the path of an archive.
#[test]
#[ignore = "started as a child process by the kill test"]
fn kill_test_child() {
    let Some(path) = std::env::var_os(CHILD_ARCHIVE) else {
        return;
    };
    let archive = Arc::new(open(Path::new(&path)));
    let intake = Intake::new(Arc::clone(&archive), Box::new(|| {}));
    intake.start(2, Some(testing::pdfium()));
    while count(
        &archive,
        "SELECT count(*) FROM intake_jobs WHERE stage NOT IN ('filed', 'failed')",
    ) > 0
    {
        std::thread::sleep(Duration::from_millis(5));
    }
}

/// A real process is killed, not a thread asked to stop: the parent starts this test binary as a
/// child that works the queue, kills it after a random moment and starts it again until the
/// queue is empty.
#[test]
fn the_process_killed_at_random_moments_loses_nothing() {
    const FILES: usize = 200;
    let source = tempfile::tempdir().unwrap();
    let paths = files(source.path(), FILES);
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join(archive::FILE_NAME);
    {
        let archive = open(&path);
        with_ready(&archive, |conn| {
            jobs::enqueue(conn, &paths, Source::Drop, testing::t0()).unwrap()
        });
    }
    let mut rng = Rng(2026);
    let (mut kills, mut starts) = (0, 0);
    loop {
        starts += 1;
        assert!(starts < 200, "the queue never emptied");
        let mut child = std::process::Command::new(std::env::current_exe().unwrap())
            .args([
                "--exact",
                "intake::tests::kill_test_child",
                "--ignored",
                "--test-threads=1",
            ])
            .env(CHILD_ARCHIVE, &path)
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .spawn()
            .unwrap();
        let deadline = Instant::now() + Duration::from_millis(rng.below(250) + 20);
        let finished = loop {
            if let Some(status) = child.try_wait().unwrap() {
                break Some(status);
            }
            if Instant::now() >= deadline {
                break None;
            }
            std::thread::sleep(Duration::from_millis(2));
        };
        match finished {
            Some(status) => {
                assert!(status.success(), "the child failed");
                break;
            }
            None => {
                child.kill().unwrap();
                child.wait().unwrap();
                kills += 1;
            }
        }
    }
    let archive = open(&path);
    assert_eq!(
        count(&archive, "SELECT count(*) FROM documents"),
        FILES as i64
    );
    assert_eq!(
        count(
            &archive,
            "SELECT count(*) FROM intake_jobs WHERE stage = 'filed' AND known = 0"
        ),
        FILES as i64
    );
    let stored = walk_files(&crate::documents::archive_root(&path))
        .into_iter()
        .filter(|file| {
            file.extension()
                .is_some_and(|extension| extension != "part")
        })
        .count();
    assert_eq!(stored, FILES);
    assert!(
        kills >= 1,
        "the child was never killed, the test proved nothing"
    );
    println!("{kills} kills, {starts} starts");
}

/// Prints how long 200 files take with one and with four workers. Run by hand:
/// `cargo test --release --lib measure_workers -- --ignored --nocapture`.
#[test]
#[ignore = "a measurement, not a check"]
fn measure_workers() {
    let source = tempfile::tempdir().unwrap();
    let paths = files(source.path(), 200);
    for workers in [1, 2, 4] {
        let dir = tempfile::tempdir().unwrap();
        let archive = Arc::new(open(&dir.path().join(archive::FILE_NAME)));
        let intake = Intake::new(Arc::clone(&archive), Box::new(|| {}));
        let started = Instant::now();
        intake.start(workers, Some(testing::pdfium()));
        intake.enqueue(&paths, Source::Dialog).unwrap();
        while count(
            &archive,
            "SELECT count(*) FROM intake_jobs WHERE stage = 'filed'",
        ) < 200
        {
            std::thread::sleep(Duration::from_millis(5));
        }
        println!(
            "{workers} worker(s): 200 files in {} ms",
            started.elapsed().as_millis()
        );
    }
}

#[test]
fn an_e_invoice_is_read_from_its_xml_and_a_broken_one_falls_back() {
    let dir = tempfile::tempdir().unwrap();
    let archive = open(&dir.path().join(archive::FILE_NAME));
    let source = tempfile::tempdir().unwrap();
    let printed = [
        "Beispiel Stadtwerke GmbH",
        "Rechnungsnummer R-2026-0042",
        "Umsatzsteuer 19 % 234,57",
        "Rechnungsbetrag 1.469,13",
    ];
    let exact = source.path().join("e-rechnung.pdf");
    std::fs::write(
        &exact,
        testing::pdf_with_xml(&printed, "factur-x.xml", testing::E_INVOICE_XML),
    )
    .unwrap();
    let broken = source.path().join("kaputt.pdf");
    let cut = &testing::E_INVOICE_XML[..testing::E_INVOICE_XML.len() / 2];
    std::fs::write(
        &broken,
        testing::pdf_with_xml(&printed[1..], "factur-x.xml", cut),
    )
    .unwrap();
    with_ready(&archive, |conn| {
        jobs::enqueue(conn, &[exact, broken], Source::Dialog, testing::t0()).unwrap()
    });
    drain(&archive, &mut |_| false).unwrap();

    let gross = |name: &str, reader: &str| -> Option<(String, Option<String>)> {
        with_ready(&archive, |conn| {
            conn.query_row(
                "SELECT v.value, v.element FROM field_values v JOIN documents d ON d.id = v.document_id
                 WHERE d.original_name = ?1 AND v.field = 'gross' AND v.reader LIKE ?2",
                rusqlite::params![name, format!("{reader}:%")],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .ok()
        })
    };
    let (value, element) = gross("e-rechnung.pdf", "e-invoice").expect("read from the XML");
    assert_eq!(value, "146913");
    assert!(element.unwrap().ends_with("/GrandTotalAmount"));
    assert!(
        gross("e-rechnung.pdf", "code").is_some(),
        "the code reader reads beside it"
    );
    assert_eq!(gross("kaputt.pdf", "e-invoice"), None);
    assert_eq!(
        gross("kaputt.pdf", "code").map(|(value, _)| value),
        Some("146913".into()),
        "a broken XML falls back to the other readers"
    );
    let check = |name: &str| -> (i64, Option<String>) {
        with_ready(&archive, |conn| {
            conn.query_row(
                "SELECT c.passed, c.detail FROM field_checks c JOIN documents d ON d.id = c.document_id
                 WHERE d.original_name = ?1 AND c.check_id = 'e-invoice:readable'",
                [name],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .unwrap()
        })
    };
    assert_eq!(check("e-rechnung.pdf"), (1, None));
    let (passed, detail) = check("kaputt.pdf");
    assert_eq!(passed, 0);
    assert!(detail.unwrap().contains("E-Rechnung"), "it says so");
}
