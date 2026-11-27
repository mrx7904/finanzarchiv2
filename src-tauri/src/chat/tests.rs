use std::sync::Mutex;

use super::*;
use crate::archive::{self, Opened};
use crate::testing;

/// A model that answers with the steps it was given, in turn, and keeps the prompts it saw.
struct Scripted {
    steps: Mutex<Vec<String>>,
    prompts: Mutex<Vec<String>>,
}

impl Scripted {
    fn new(steps: &[Value]) -> Self {
        Self {
            steps: Mutex::new(steps.iter().rev().map(Value::to_string).collect()),
            prompts: Mutex::new(Vec::new()),
        }
    }

    fn prompts(&self) -> Vec<String> {
        self.prompts.lock().unwrap().clone()
    }
}

impl LocalModel for Scripted {
    fn name(&self) -> String {
        "scripted".into()
    }

    fn ask(&self, question: &Question) -> Result<String, ModelError> {
        self.prompts
            .lock()
            .unwrap()
            .push(question.prompt.to_owned());
        self.steps
            .lock()
            .unwrap()
            .pop()
            .ok_or_else(|| ModelError::BadAnswer("no step left".into()))
    }
}

fn ready(dir: &std::path::Path) -> archive::Archive {
    match archive::open(&dir.join(archive::FILE_NAME), testing::t0()).unwrap() {
        Opened::Ready(ready) => ready,
        other => panic!("{other:?}"),
    }
}

fn today() -> Date {
    jiff::civil::date(2026, 11, 25)
}

/// An archive with a salary, a rent and a purchase, and its read-only connection.
fn archive_with_bookings(dir: &std::path::Path) -> (archive::Archive, Connection) {
    let ready = ready(dir);
    let conn = ready.connection();
    let account =
        crate::statements::account_of(conn, "DE89370400440532013000", None, testing::t0()).unwrap();
    for (day, cents, other, purpose) in [
        (
            "2026-03-28",
            259_440,
            "Beispiel Software GmbH",
            "Gehalt März",
        ),
        (
            "2026-03-01",
            -78_000,
            "Wohnbau Musterstadt eG",
            "Miete März",
        ),
        (
            "2026-03-12",
            -4_995,
            "Beispiel Musikhaus",
            "Gitarrensaiten Bestellung 42",
        ),
    ] {
        let party = crate::parties::party_of(conn, other, None, testing::t0()).unwrap();
        conn.execute(
            "INSERT INTO bookings (account_id, position, day, amount_cents, kind, other_side, purpose, party_id) VALUES (?1, 0, ?2, ?3, 'Buchung', ?4, ?5, ?6)",
            params![account, day, cents, other, purpose, party],
        )
        .unwrap();
    }
    let path = ready.path().to_path_buf();
    (ready, read_only(&path).unwrap())
}

#[test]
fn the_lookups_run_on_a_connection_that_cannot_write() {
    let dir = tempfile::tempdir().unwrap();
    let (_ready, conn) = archive_with_bookings(dir.path());
    let refused = conn.execute("DELETE FROM bookings", []);
    assert!(refused.is_err());
}

#[test]
fn an_answer_copies_the_amounts_of_its_lookup_and_names_its_sources() {
    let dir = tempfile::tempdir().unwrap();
    let (_ready, conn) = archive_with_bookings(dir.path());
    let model = Scripted::new(&[
        json!({ "action": "lookup", "tool": "buchungen", "query": "Gitarrensaiten" }),
        json!({ "action": "answer", "text": "Für die Gitarrensaiten gingen 49,95 € weg." }),
    ]);
    let answer = answer(
        &conn,
        &model,
        "Was haben die Gitarrensaiten gekostet?",
        &[],
        today(),
    )
    .unwrap();
    assert_eq!(answer.text, "Für die Gitarrensaiten gingen 49,95 € weg.");
    assert_eq!(answer.sources.len(), 1);
    assert!(matches!(answer.sources[0], Source::Booking(_)));
    // The model saw the amount as code wrote it.
    assert!(model.prompts()[1].contains("-49,95 €"));
}

#[test]
fn a_miscopied_amount_gets_one_correction_then_the_answer_goes_without_numbers() {
    let dir = tempfile::tempdir().unwrap();
    let (_ready, conn) = archive_with_bookings(dir.path());
    let corrected = Scripted::new(&[
        json!({ "action": "lookup", "tool": "einnahmen_ausgaben", "year": 2026, "month": 3 }),
        json!({ "action": "answer", "text": "Im März kamen 2.594,00 € herein." }),
        json!({ "action": "answer", "text": "Im März kamen 2.594,40 € herein." }),
    ]);
    let answer_one = answer(&conn, &corrected, "Was kam im März herein?", &[], today()).unwrap();
    assert_eq!(answer_one.text, "Im März kamen 2.594,40 € herein.");
    assert!(corrected.prompts()[2].contains("2.594,00 €"));
    let stubborn = Scripted::new(&[
        json!({ "action": "lookup", "tool": "einnahmen_ausgaben", "year": 2026, "month": 3 }),
        json!({ "action": "answer", "text": "Im März kamen 2.594,00 € herein." }),
        json!({ "action": "answer", "text": "Es waren 2.600,00 €." }),
    ]);
    let answer_two = answer(&conn, &stubborn, "Was kam im März herein?", &[], today()).unwrap();
    assert_eq!(answer_two.text, NO_SURE_NUMBERS);
}

#[test]
fn when_every_lookup_finds_nothing_the_answer_is_the_fixed_sentence() {
    let dir = tempfile::tempdir().unwrap();
    let (_ready, conn) = archive_with_bookings(dir.path());
    let model = Scripted::new(&[
        json!({ "action": "lookup", "tool": "buchungen", "query": "Segelboot" }),
        json!({ "action": "lookup", "tool": "dokumente", "query": "Segelboot" }),
        json!({ "action": "answer", "text": "Das Segelboot kostete sicher viel." }),
    ]);
    let answer = answer(&conn, &model, "Was kostete das Segelboot?", &[], today()).unwrap();
    assert_eq!(answer.text, NOTHING_FOUND);
    assert!(answer.sources.is_empty());
}

#[test]
fn a_number_without_a_lookup_is_refused() {
    let dir = tempfile::tempdir().unwrap();
    let (_ready, conn) = archive_with_bookings(dir.path());
    let model = Scripted::new(&[
        json!({ "action": "answer", "text": "Du hast 100,00 € ausgegeben." }),
        json!({ "action": "answer", "text": "Es waren 100,00 €." }),
    ]);
    let answer = answer(&conn, &model, "Wie viel habe ich ausgegeben?", &[], today()).unwrap();
    assert_eq!(answer.text, NO_SURE_NUMBERS);
    assert!(model.prompts()[1].contains("Schlage zuerst nach"));
}

#[test]
fn at_most_four_lookups_then_the_model_must_answer() {
    let dir = tempfile::tempdir().unwrap();
    let (_ready, conn) = archive_with_bookings(dir.path());
    let lookup = json!({ "action": "lookup", "tool": "partei", "query": "Wohnbau", "year": 2026 });
    let model = Scripted::new(&[
        lookup.clone(),
        lookup.clone(),
        lookup.clone(),
        lookup.clone(),
        lookup,
        json!({ "action": "answer", "text": "Die Miete ging an Wohnbau Musterstadt." }),
    ]);
    let answer = answer(&conn, &model, "Wohin ging die Miete?", &[], today()).unwrap();
    assert_eq!(answer.lookups.len(), MAX_LOOKUPS);
    assert_eq!(answer.text, "Die Miete ging an Wohnbau Musterstadt.");
}

#[test]
fn an_answer_on_taxes_carries_the_notice_and_the_history_goes_with_the_question() {
    let dir = tempfile::tempdir().unwrap();
    let (ready, conn) = archive_with_bookings(dir.path());
    let booking: i64 = ready
        .connection()
        .query_row(
            "SELECT id FROM bookings WHERE purpose LIKE 'Gitarren%'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    crate::taxes::assign(
        ready.connection(),
        crate::taxes::On::Booking(booking),
        &crate::taxes::Assignment {
            category: "arbeitsmittel".into(),
            share_percent: 100,
            refunded_cents: 0,
            labour_cents: None,
        },
        testing::t0(),
    )
    .unwrap();
    let model = Scripted::new(&[
        json!({ "action": "lookup", "tool": "steuern", "year": 2026 }),
        json!({ "action": "answer", "text": "Für Arbeitsmittel sind 49,95 € gesammelt." }),
    ]);
    let history = vec![(
        "Was kam im März herein?".to_owned(),
        "2.594,40 €.".to_owned(),
    )];
    let answer = answer(&conn, &model, "Und für die Steuer?", &history, today()).unwrap();
    assert!(
        answer.text.ends_with(crate::taxes::CHECK_NOTICE),
        "{}",
        answer.text
    );
    assert!(model.prompts()[0].contains("Frage: Was kam im März herein?"));
}

#[test]
fn an_unknown_category_returns_the_known_ones() {
    let dir = tempfile::tempdir().unwrap();
    let (_ready, conn) = archive_with_bookings(dir.path());
    let step = Step {
        action: "lookup".into(),
        tool: Some("kategorie".into()),
        query: Some("Raumfahrt".into()),
        year: Some(2026),
        ..Step::default()
    };
    let found = lookup(&conn, &step, today()).unwrap();
    assert!(found.empty);
    assert!(found.lines[0].contains("Lebensmittel"));
}

#[test]
fn chats_keep_their_history_and_are_deleted_whole() {
    let dir = tempfile::tempdir().unwrap();
    let ready = ready(dir.path());
    let conn = ready.connection();
    let chat = new_chat(conn, "Was kam im März herein?", testing::t0()).unwrap();
    add(
        conn,
        chat,
        "owner",
        "Was kam im März herein?",
        &[],
        testing::t0(),
    )
    .unwrap();
    add(
        conn,
        chat,
        "archive",
        "2.594,40 €.",
        &[Source::Booking(7), Source::Document(3)],
        testing::t0(),
    )
    .unwrap();
    assert_eq!(chats(conn).unwrap()[0].title, "Was kam im März herein?");
    let kept = messages(conn, chat).unwrap();
    assert_eq!(
        kept[1].sources,
        vec![Source::Booking(7), Source::Document(3)]
    );
    assert_eq!(history(conn, chat).unwrap().len(), 1);
    delete(conn, chat).unwrap();
    assert!(chats(conn).unwrap().is_empty());
    assert!(messages(conn, chat).unwrap().is_empty());
}

#[test]
fn the_amount_check_compares_without_sign() {
    let amounts: BTreeSet<String> = ["1.234,56€".to_owned()].into();
    assert!(unchecked("Es waren 1.234,56 € und -1.234,56 €.", &amounts).is_empty());
    assert_eq!(
        unchecked("Es waren 1.234,50 €.", &amounts),
        vec!["1.234,50€".to_owned()]
    );
}
