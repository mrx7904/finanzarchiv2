use super::*;
use crate::archive::{self, Opened};
use crate::testing;

fn ready(dir: &std::path::Path) -> archive::Archive {
    match archive::open(&dir.join(archive::FILE_NAME), testing::t0()).unwrap() {
        Opened::Ready(ready) => ready,
        other => panic!("{other:?}"),
    }
}

fn party(conn: &Connection, name: &str) -> i64 {
    crate::parties::party_of(conn, name, None, testing::t0())
        .unwrap()
        .unwrap()
}

/// A filed document of the kind with its values as code read them.
fn document(conn: &Connection, number: u8, kind: Kind, party: i64, values: &[(&str, &str)]) -> i64 {
    conn.execute(
        "INSERT INTO documents (sha256, file_type, kind, original_name, size_bytes, source, imported_at, state, party_id)
         VALUES (?1, 'pdf', ?2, 'd.pdf', 1, 'dialog', 'x', 'filed', ?3)",
        params![format!("{number:064x}"), kind.code(), party],
    )
    .unwrap();
    let id = conn.last_insert_rowid();
    for (field, value) in values {
        conn.execute(
            "INSERT INTO field_values (document_id, field, printed, value, reader, page, top, read_at) VALUES (?1, ?2, ?3, ?3, 'code:5', 1, 700.0, 'x')",
            params![id, field, value],
        )
        .unwrap();
    }
    id
}

fn booking(conn: &Connection, day: &str, cents: i64, party: Option<i64>, purpose: &str) -> i64 {
    let account = crate::statements::account_of(
        conn,
        "DE89370400440532013000",
        Some("Beispielbank eG"),
        testing::t0(),
    )
    .unwrap();
    conn.execute(
        "INSERT INTO bookings (account_id, position, day, amount_cents, kind, purpose, party_id) VALUES (?1, 0, ?2, ?3, 'Lastschrift', ?4, ?5)",
        params![account, day, cents, purpose, party],
    )
    .unwrap();
    conn.last_insert_rowid()
}

fn links_of(conn: &Connection, document: i64) -> Vec<(i64, String)> {
    conn.prepare("SELECT booking_id, rule FROM links WHERE document_id = ?1 AND undone_at IS NULL")
        .unwrap()
        .query_map([document], |row| Ok((row.get(0)?, row.get(1)?)))
        .unwrap()
        .collect::<rusqlite::Result<_>>()
        .unwrap()
}

#[test]
fn the_number_in_the_purpose_links_one_or_several_invoices() {
    let dir = tempfile::tempdir().unwrap();
    let ready = ready(dir.path());
    let conn = ready.connection();
    let shop = party(conn, "Muster Shop");
    let one = document(
        conn,
        1,
        Kind::Invoice,
        shop,
        &[
            ("gross", "1190"),
            ("invoice_date", "2026-01-05"),
            ("invoice_number", "R-2026-0042"),
        ],
    );
    let a = document(
        conn,
        2,
        Kind::Invoice,
        shop,
        &[
            ("gross", "1000"),
            ("invoice_date", "2026-01-06"),
            ("invoice_number", "R-2026-0050"),
        ],
    );
    let b = document(
        conn,
        3,
        Kind::Invoice,
        shop,
        &[
            ("gross", "500"),
            ("invoice_date", "2026-01-06"),
            ("invoice_number", "R-2026-0051"),
        ],
    );
    let paid = booking(
        conn,
        "2026-01-12",
        -1_190,
        None,
        "Rechnung R2026 0042 Danke",
    );
    let both = booking(
        conn,
        "2026-01-20",
        -1_500,
        None,
        "R-2026-0050 und R-2026-0051",
    );
    assert_eq!(find(conn, testing::t0()).unwrap(), (3, 0));
    assert_eq!(links_of(conn, one), [(paid, RULE_NUMBER.to_owned())]);
    assert_eq!(links_of(conn, a), [(both, RULE_NUMBER.to_owned())]);
    assert_eq!(
        links_of(conn, b),
        [(both, RULE_NUMBER.to_owned())],
        "one booking pays both, the sum to the cent"
    );
    assert_eq!(find(conn, testing::t0()).unwrap(), (0, 0), "never twice");
}

#[test]
fn the_amount_with_the_party_links_and_two_that_fit_are_a_question() {
    let dir = tempfile::tempdir().unwrap();
    let ready = ready(dir.path());
    let conn = ready.connection();
    let energy = party(conn, "Testhaus Energie AG");
    let other = party(conn, "Muster Bäckerei");
    let invoice = document(
        conn,
        1,
        Kind::Invoice,
        energy,
        &[
            ("gross", "8500"),
            ("invoice_date", "2026-02-01"),
            ("due_date", "2026-02-15"),
        ],
    );
    booking(conn, "2026-02-10", -8_500, Some(other), "");
    let right = booking(conn, "2026-02-14", -8_500, Some(energy), "Abschlag");
    booking(
        conn,
        "2026-05-14",
        -8_500,
        Some(energy),
        "Abschlag, too late",
    );
    assert_eq!(find(conn, testing::t0()).unwrap(), (1, 0));
    assert_eq!(
        links_of(conn, invoice),
        [(right, RULE_PARTY_AMOUNT.to_owned())]
    );

    let twice = document(
        conn,
        2,
        Kind::Invoice,
        energy,
        &[("gross", "4200"), ("invoice_date", "2026-03-01")],
    );
    let first = booking(conn, "2026-03-03", -4_200, Some(energy), "");
    let second = booking(conn, "2026-03-20", -4_200, Some(energy), "");
    assert_eq!(find(conn, testing::t0()).unwrap(), (0, 1));
    assert_eq!(asked(conn).unwrap(), [twice]);
    let candidates: Vec<i64> = question(conn, twice)
        .unwrap()
        .unwrap()
        .iter()
        .map(|(id, ..)| *id)
        .collect();
    assert_eq!(candidates, [first, second]);
    answer(conn, twice, Some(second), testing::t0()).unwrap();
    assert_eq!(links_of(conn, twice), [(second, RULE_OWNER.to_owned())]);
    assert!(asked(conn).unwrap().is_empty());
}

#[test]
fn a_payslip_finds_its_netto_and_an_undone_link_is_not_made_again() {
    let dir = tempfile::tempdir().unwrap();
    let ready = ready(dir.path());
    let conn = ready.connection();
    let employer = party(conn, "Beispiel Maschinenbau GmbH");
    let payslip = document(
        conn,
        1,
        Kind::Payslip,
        employer,
        &[("netto", "259440"), ("period", "2026-03")],
    );
    let salary = booking(conn, "2026-03-31", 259_440, Some(employer), "Lohn März");
    booking(
        conn,
        "2026-03-31",
        -259_440,
        Some(employer),
        "a refund the other way",
    );
    assert_eq!(find(conn, testing::t0()).unwrap(), (1, 0));
    assert_eq!(links_of(conn, payslip), [(salary, RULE_PAYSLIP.to_owned())]);
    let link: i64 = conn
        .query_row("SELECT id FROM links", [], |row| row.get(0))
        .unwrap();
    assert!(undo(conn, link, testing::t0()).unwrap());
    assert!(!undo(conn, link, testing::t0()).unwrap());
    assert_eq!(
        find(conn, testing::t0()).unwrap(),
        (0, 0),
        "left to the owner"
    );
    assert!(links_of(conn, payslip).is_empty());
}

#[test]
fn a_word_of_the_sender_links_and_an_amount_alone_is_only_a_question() {
    let dir = tempfile::tempdir().unwrap();
    let ready = ready(dir.path());
    let conn = ready.connection();
    // The document and the bank spell the party otherwise, so they are two parties.
    let practice = party(conn, "Zahnarztpraxis Beispiel");
    let by_bank = party(conn, "DR. BEISPIEL ZAHNARZTPRAXIS");
    let named = document(
        conn,
        1,
        Kind::Invoice,
        practice,
        &[
            ("gross", "12000"),
            ("invoice_date", "2026-04-01"),
            ("sender", "Zahnarztpraxis Beispiel"),
        ],
    );
    let paid = booking(
        conn,
        "2026-04-10",
        -12_000,
        Some(by_bank),
        "DR. BEISPIEL ZAHNARZTPRAXIS Rechnung",
    );
    booking(conn, "2026-04-11", -12_000, None, "Kaufhaus Muster");
    assert_eq!(find(conn, testing::t0()).unwrap(), (1, 0));
    assert_eq!(links_of(conn, named), [(paid, RULE_AMOUNT_NAME.to_owned())]);

    let anonymous = document(
        conn,
        2,
        Kind::Invoice,
        practice,
        &[
            ("gross", "3300"),
            ("invoice_date", "2026-04-01"),
            ("sender", "Werkstatt Ohnename"),
        ],
    );
    booking(conn, "2026-04-12", -3_300, None, "Kartenzahlung");
    assert_eq!(
        find(conn, testing::t0()).unwrap(),
        (0, 1),
        "one booking of the amount is no proof"
    );
    assert_eq!(asked(conn).unwrap(), [anonymous]);
    assert!(links_of(conn, anonymous).is_empty());
}
