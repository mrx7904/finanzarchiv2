use super::*;
use crate::archive::{self, Opened};
use crate::testing;

fn ready(dir: &std::path::Path) -> archive::Archive {
    match archive::open(&dir.join(archive::FILE_NAME), testing::t0()).unwrap() {
        Opened::Ready(ready) => ready,
        other => panic!("{other:?}"),
    }
}

fn document(conn: &Connection, number: u8, kind: Kind) -> i64 {
    conn.execute(
        "INSERT INTO documents (sha256, file_type, kind, original_name, size_bytes, source, imported_at, text_state)
         VALUES (?1, 'pdf', ?2, ?3, 1, 'dialog', ?4, 'text')",
        params![format!("{number:064x}"), kind.code(), format!("d{number}.pdf"), testing::t0().to_string()],
    )
    .unwrap();
    let id = conn.last_insert_rowid();
    conn.execute(
        "INSERT INTO pages (document_id, number, width, height) VALUES (?1, 1, 595.0, 842.0)",
        [id],
    )
    .unwrap();
    id
}

fn value(conn: &Connection, document: i64, reader: &str, field: &str, printed: &str, value: &str) {
    conn.execute(
        "INSERT INTO field_values (document_id, field, printed, value, reader, page, left, bottom, right, top, read_at)
         VALUES (?1, ?2, ?3, ?4, ?5, 1, 60.0, 520.0, 400.0, 531.0, 'x')",
        params![document, field, printed, value, reader],
    )
    .unwrap();
}

/// An invoice where code and the model read a different gross: it waits for `check:agree gross`.
fn disputed(conn: &Connection, number: u8, invoice_number: &str) -> i64 {
    let id = document(conn, number, Kind::Invoice);
    value(conn, id, "code:3", "gross", "49,99", "4999");
    value(conn, id, "model:beispiel", "gross", "59,99", "5999");
    value(
        conn,
        id,
        "code:3",
        "invoice_number",
        invoice_number,
        invoice_number,
    );
    value(
        conn,
        id,
        "code:3",
        "sender",
        "Beispiel Stadtwerke GmbH",
        "Beispiel Stadtwerke GmbH",
    );
    checks::decide(conn, id, testing::t0()).unwrap();
    id
}

#[test]
fn what_waits_is_grouped_by_its_first_reason_with_a_question_and_the_line() {
    let dir = tempfile::tempdir().unwrap();
    let ready = ready(dir.path());
    let conn = ready.connection();
    disputed(conn, 1, "R-1");
    disputed(conn, 2, "R-2");
    let other = document(conn, 3, Kind::Other);
    checks::decide(conn, other, testing::t0()).unwrap();
    assert_eq!(
        groups(conn).unwrap(),
        [
            ("check:kind".to_owned(), 1),
            ("check:agree gross".to_owned(), 2)
        ]
    );
    assert_eq!(
        group_label("check:agree gross"),
        "Bruttobetrag ungleich gelesen"
    );
    let asked = questions(conn, "check:agree gross").unwrap();
    assert_eq!(asked.len(), 2);
    assert_eq!(
        asked[0].text,
        "Ist 49,99 € der Bruttobetrag von „Beispiel Stadtwerke GmbH“?"
    );
    assert_eq!(
        asked[0].readings.len(),
        2,
        "both readings beside each other"
    );
    assert_eq!(asked[0].readings[1].reader, "Modell");
    assert_eq!(asked[0].readings[0].rect, Some([60.0, 520.0, 400.0, 531.0]));
    assert_eq!(asked[0].page_size, Some((595.0, 842.0)));
}

#[test]
fn yes_confirms_the_value_and_the_document_is_filed() {
    let dir = tempfile::tempdir().unwrap();
    let ready = ready(dir.path());
    let conn = ready.connection();
    let id = disputed(conn, 1, "R-1");
    let state = answer(conn, id, "check:agree gross", &Answer::Yes, testing::t0()).unwrap();
    assert_eq!(state, "filed");
    let (value, rule): (String, String) = conn
        .query_row(
            "SELECT v.value, x.rule FROM field_values v JOIN decisions x ON x.document_id = v.document_id
             WHERE v.document_id = ?1 AND v.field = 'gross' AND v.reader = 'owner'",
            [id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .unwrap();
    assert_eq!(value, "4999");
    assert_eq!(
        rule,
        checks::RULE_CHECKED,
        "the owner's value is a reader, no acceptance"
    );
}

#[test]
fn a_reading_or_a_typed_value_answers_no_and_a_wrong_amount_is_refused() {
    let dir = tempfile::tempdir().unwrap();
    let ready = ready(dir.path());
    let conn = ready.connection();
    let id = disputed(conn, 1, "R-1");
    let refused = answer(
        conn,
        id,
        "check:agree gross",
        &Answer::Typed("12.50".into()),
        testing::t0(),
    )
    .unwrap_err();
    assert!(
        refused.to_string().contains("Punkt"),
        "the reason of Cents::parse_de: {refused}"
    );
    assert_eq!(
        answer(
            conn,
            id,
            "check:agree gross",
            &Answer::Reading(1),
            testing::t0()
        )
        .unwrap(),
        "filed"
    );
    let owner: String = conn
        .query_row(
            "SELECT value FROM field_values WHERE document_id = ?1 AND reader = 'owner'",
            [id],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(owner, "5999");

    let other = disputed(conn, 2, "R-2");
    assert_eq!(
        answer(
            conn,
            other,
            "check:agree gross",
            &Answer::Typed("1.234,56 €".into()),
            testing::t0()
        )
        .unwrap(),
        "filed"
    );
    assert!(matches!(
        answer(
            conn,
            other,
            "check:agree gross",
            &Answer::Kind(Kind::Invoice),
            testing::t0()
        ),
        Err(AnswerError::Refused(Refused::Mismatch))
    ));
}

#[test]
fn an_unclear_kind_and_a_possible_duplicate_are_answered_too() {
    let dir = tempfile::tempdir().unwrap();
    let ready = ready(dir.path());
    let conn = ready.connection();
    let other = document(conn, 1, Kind::Unknown);
    checks::decide(conn, other, testing::t0()).unwrap();
    assert_eq!(
        questions(conn, "check:kind").unwrap()[0].text,
        "Welche Art hat das Dokument?"
    );
    assert_eq!(
        answer(
            conn,
            other,
            "check:kind",
            &Answer::Kind(Kind::Other),
            testing::t0()
        )
        .unwrap(),
        "filed"
    );
    let rule: String = conn
        .query_row(
            "SELECT rule FROM decisions WHERE document_id = ?1",
            [other],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(rule, checks::RULE_OWNER);

    let first = disputed(conn, 2, "R-7");
    let second = disputed(conn, 3, "R-7");
    answer(
        conn,
        first,
        "check:agree gross",
        &Answer::Yes,
        testing::t0(),
    )
    .unwrap();
    assert_eq!(
        answer(
            conn,
            second,
            "check:agree gross",
            &Answer::Yes,
            testing::t0()
        )
        .unwrap(),
        "waiting"
    );
    assert_eq!(
        questions(conn, "check:duplicate").unwrap().len(),
        2,
        "both wait, the owner decides"
    );
    assert!(
        matches!(
            answer(conn, second, "check:duplicate", &Answer::No, testing::t0()),
            Err(AnswerError::Refused(Refused::Mismatch))
        ),
        "a duplicate is answered no by deleting it"
    );
    assert_eq!(
        answer(conn, second, "check:duplicate", &Answer::Yes, testing::t0()).unwrap(),
        "filed"
    );
}

#[test]
fn no_to_a_duplicate_deletes_it_into_the_trash_and_its_twin_is_filed() {
    let dir = tempfile::tempdir().unwrap();
    let ready = ready(dir.path());
    let conn = ready.connection();
    let root = dir.path().join("Archiv");
    let first = disputed(conn, 1, "R-7");
    let second = disputed(conn, 2, "R-7");
    answer(
        conn,
        first,
        "check:agree gross",
        &Answer::Yes,
        testing::t0(),
    )
    .unwrap();
    answer(
        conn,
        second,
        "check:agree gross",
        &Answer::Yes,
        testing::t0(),
    )
    .unwrap();
    // The original of the second lies in the archive, as the intake stored it.
    let (sha256,): (String,) = conn
        .query_row(
            "SELECT sha256 FROM documents WHERE id = ?1",
            [second],
            |row| Ok((row.get(0)?,)),
        )
        .unwrap();
    let stored = crate::documents::stored_path(&root, &sha256, crate::documents::FileType::Pdf);
    std::fs::create_dir_all(stored.parent().unwrap()).unwrap();
    std::fs::write(&stored, b"%PDF").unwrap();
    let moved = delete_duplicate(conn, &root, second, testing::t0())
        .unwrap()
        .unwrap();
    assert!(moved.starts_with(dir.path().join(crate::documents::TRASH_FOLDER)));
    assert!(moved.to_string_lossy().ends_with("_d2.pdf"));
    assert!(!stored.exists());
    let left: i64 = conn
        .query_row(
            "SELECT count(*) FROM field_values WHERE document_id = ?1",
            [second],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(left, 0, "every row about it is gone");
    let state: String = conn
        .query_row(
            "SELECT state FROM documents WHERE id = ?1",
            [first],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(state, "filed", "its twin is no duplicate any more");
    assert!(
        matches!(
            delete_duplicate(conn, &root, first, testing::t0()),
            Err(AnswerError::Refused(Refused::Mismatch))
        ),
        "only a document that waits as a duplicate is deleted this way"
    );
}

#[test]
fn no_after_an_undone_filing_sets_the_document_aside() {
    let dir = tempfile::tempdir().unwrap();
    let ready = ready(dir.path());
    let conn = ready.connection();
    let id = disputed(conn, 1, "R-1");
    answer(conn, id, "check:agree gross", &Answer::Yes, testing::t0()).unwrap();
    checks::undo(conn, id, testing::t0()).unwrap();
    assert_eq!(questions(conn, "check:undone").unwrap().len(), 1);
    assert_eq!(
        answer(conn, id, "check:undone", &Answer::No, testing::t0()).unwrap(),
        "waiting"
    );
    assert!(groups(conn).unwrap().is_empty(), "it left Prüfen");
    let listed = crate::documents::list(conn).unwrap();
    assert_eq!(listed[0].state, "held");
}
