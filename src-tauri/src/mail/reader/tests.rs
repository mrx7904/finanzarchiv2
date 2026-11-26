use std::sync::Mutex;
use std::sync::atomic::AtomicBool;

use super::*;
use crate::archive;
use crate::intake::{self, jobs};
use crate::testing;

fn open(dir: &Path) -> Mutex<Opened> {
    Mutex::new(archive::open(&dir.join(archive::FILE_NAME), testing::t0()).unwrap())
}

fn with<T>(archive: &Mutex<Opened>, run: impl FnOnce(&Connection) -> T) -> T {
    let guard = archive.lock().unwrap();
    let Opened::Ready(ready) = &*guard else {
        panic!("not ready")
    };
    run(ready.connection())
}

/// Runs every open job to its end in this thread, without a model.
fn drain(archive: &Mutex<Opened>) {
    let running = AtomicBool::new(false);
    loop {
        let open = with(archive, |conn| jobs::open_jobs(conn).unwrap());
        let Some(job) = open.into_iter().next() else {
            return;
        };
        intake::process(
            archive,
            Some(testing::pdfium()),
            None,
            &job,
            &running,
            &mut |_| false,
        )
        .unwrap();
    }
}

fn mailbox(conn: &Connection) -> i64 {
    conn.execute(
        "INSERT INTO mailboxes (address, host, added_at, last_uid) VALUES ('test@example.com', 'imap.gmail.com', '2026-11-26T00:00:00Z', 40)",
        [],
    )
    .unwrap();
    conn.last_insert_rowid()
}

fn looked(hash: char, verdict: &str, kind: Option<Kind>, files: Vec<(String, Vec<u8>)>) -> Looked {
    Looked {
        hash: hash.to_string().repeat(64),
        received_at: None,
        verdict: verdict.into(),
        kind,
        files,
        paypal_code: None,
        orders: Vec::new(),
    }
}

/// Keeps the mail, hands its files to the intake as a round would, and runs the intake.
fn take(archive: &Mutex<Opened>, staging: &Path, looked: &Looked) {
    with(archive, |conn| {
        let id = conn
            .query_row("SELECT id FROM mailboxes", [], |row| row.get(0))
            .unwrap_or_else(|_| mailbox(conn));
        let paths = keep(conn, staging, id, looked, testing::t0()).unwrap();
        jobs::enqueue(conn, &paths, Source::Mail, testing::t0()).unwrap();
    });
    drain(archive);
}

fn document(archive: &Mutex<Opened>, name: &str) -> (String, String, String, Option<String>) {
    with(archive, |conn| {
        conn.query_row(
            "SELECT file_type, kind, coalesce(text_state, ''), kind_signals FROM documents WHERE original_name = ?1",
            [name],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
        )
        .unwrap()
    })
}

#[test]
fn the_text_of_a_mail_is_a_document_and_takes_the_kind_of_its_mail() {
    let dir = tempfile::tempdir().unwrap();
    let archive = open(dir.path());
    let staging = dir.path().join(STAGING);
    // Nothing in it that code takes for a receipt: the mail's kind decides.
    let text = crate::mail::text::mail_file(
        &[
            "Von: Beispiel Verein <info@example.com>".into(),
            "Betreff: Ihr Monat".into(),
        ],
        "Danke für Ihren Beitrag von 12,00 EUR im November.",
    );
    take(
        &archive,
        &staging,
        &looked(
            'a',
            "Übernommen als Text der Mail (laut Modell ein Zahlungsbeleg).",
            Some(Kind::Receipt),
            vec![("Ihr Monat.txt".into(), text.into_bytes())],
        ),
    );
    let (file_type, kind, state, signals) = document(&archive, "Ihr Monat.txt");
    assert_eq!(
        (file_type.as_str(), kind.as_str(), state.as_str()),
        ("mail", "receipt", "mail")
    );
    assert_eq!(signals, Some(format!("[\"{HINT_SIGNAL}\"]")));
    let lines = with(&archive, |conn| {
        let id: i64 = conn
            .query_row("SELECT id FROM documents", [], |row| row.get(0))
            .unwrap();
        crate::documents::lines_of(conn, id).unwrap()
    });
    assert_eq!(
        lines,
        [
            "Von: Beispiel Verein <info@example.com>",
            "Betreff: Ihr Monat",
            "",
            "Danke für Ihren Beitrag von 12,00 EUR im November."
        ],
        "the mark is not a line"
    );
    let listed = with(&archive, |conn| crate::documents::list(conn).unwrap());
    assert_eq!(
        listed[0].arrival.as_deref(),
        Some("Aus einem Postfach. Übernommen als Text der Mail (laut Modell ein Zahlungsbeleg).")
    );
}

#[test]
fn an_attachment_takes_the_kind_of_its_mail_only_when_it_was_the_only_file() {
    let dir = tempfile::tempdir().unwrap();
    let archive = open(dir.path());
    let staging = dir.path().join(STAGING);
    let terms = testing::pdf(&["Allgemeine Geschäftsbedingungen", "Paragraf 1 Geltung"]);
    let notice = testing::pdf(&["Ihre Unterlagen zum Vertrag"]);
    take(
        &archive,
        &staging,
        &looked(
            'b',
            "Übernommen: 2 PDF-Anhänge (laut Modell eine Rechnung).",
            Some(Kind::Invoice),
            vec![
                ("agb.pdf".into(), terms),
                ("unterlagen.pdf".into(), notice.clone()),
            ],
        ),
    );
    assert_eq!(
        document(&archive, "agb.pdf").1,
        "other",
        "two files: no hint"
    );
    let alone = testing::pdf(&["Ihre Unterlagen zum Monat"]);
    take(
        &archive,
        &staging,
        &looked(
            'c',
            "Übernommen: der PDF-Anhang (laut Modell eine Rechnung).",
            Some(Kind::Invoice),
            vec![("monat.pdf".into(), alone)],
        ),
    );
    assert_eq!(
        document(&archive, "monat.pdf").1,
        "invoice",
        "the only file"
    );
}

#[test]
fn documents_taken_before_the_hint_get_it_at_start() {
    let dir = tempfile::tempdir().unwrap();
    let archive = open(dir.path());
    let staging = dir.path().join(STAGING);
    // As 0.4.0 kept it: the kind of the mail only in the words of its verdict.
    take(
        &archive,
        &staging,
        &looked(
            'd',
            "Übernommen: der PDF-Anhang (laut Modell eine Rechnung).",
            None,
            vec![(
                "monat.pdf".into(),
                testing::pdf(&["Ihre Unterlagen zum Monat"]),
            )],
        ),
    );
    assert_eq!(document(&archive, "monat.pdf").1, "other");
    // Migration 0020 reads the kind from the verdict.
    with(&archive, |conn| {
        conn.execute(
            "UPDATE mail_messages SET kind = 'invoice' WHERE verdict LIKE '%laut Modell eine Rechnung%'",
            [],
        )
        .unwrap();
        assert_eq!(apply_hints(conn, testing::t0()).unwrap(), 1);
        assert_eq!(apply_hints(conn, testing::t0()).unwrap(), 0, "once");
    });
    drain(&archive);
    let (_, kind, _, signals) = document(&archive, "monat.pdf");
    assert_eq!(
        (kind.as_str(), signals),
        ("invoice", Some(format!("[\"{HINT_SIGNAL}\"]")))
    );
}

#[test]
fn a_mail_taken_as_a_pdf_is_forgotten_and_its_document_goes_to_the_trash() {
    let dir = tempfile::tempdir().unwrap();
    let archive = open(dir.path());
    let staging = dir.path().join(STAGING);
    let made = testing::pdf(&[
        "Von: Beispiel GmbH",
        "Betreff: Ihre Rechnung",
        "Rechnungsbetrag 23,80",
    ]);
    take(
        &archive,
        &staging,
        &looked(
            'e',
            "Übernommen als PDF der Mail (laut Modell eine Rechnung).",
            Some(Kind::Invoice),
            vec![("Ihre Rechnung.pdf".into(), made)],
        ),
    );
    // A second one the intake has not taken yet.
    with(&archive, |conn| {
        let id: i64 = conn
            .query_row("SELECT id FROM mailboxes", [], |row| row.get(0))
            .unwrap();
        let paths = keep(
            conn,
            &staging,
            id,
            &looked(
                'f',
                "Übernommen als PDF der Mail (laut Modell eine Rechnung).",
                Some(Kind::Invoice),
                vec![("Noch eine.pdf".into(), testing::pdf(&["Rechnung R-9"]))],
            ),
            testing::t0(),
        )
        .unwrap();
        jobs::enqueue(conn, &paths, Source::Mail, testing::t0()).unwrap();
    });
    let root = dir.path().join("Archiv");
    with(&archive, |conn| {
        assert_eq!(replace_mail_pdfs(conn, &root).unwrap(), 2);
        let left: (i64, i64, i64, i64) = conn
            .query_row(
                "SELECT (SELECT count(*) FROM documents), (SELECT count(*) FROM mail_messages),
                        (SELECT count(*) FROM intake_jobs), (SELECT last_uid FROM mailboxes)",
                [],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
            )
            .unwrap();
        assert_eq!(
            left,
            (0, 0, 0, 0),
            "forgotten, and read again from the start"
        );
        assert_eq!(replace_mail_pdfs(conn, &root).unwrap(), 0, "once");
    });
    let trash = dir.path().join(crate::documents::TRASH_FOLDER);
    assert_eq!(
        std::fs::read_dir(&trash).unwrap().count(),
        1,
        "the document in the trash"
    );
    assert!(
        std::fs::read_dir(&staging).map_or(true, |mut dir| dir.next().is_none()),
        "no staged file is left"
    );
}
