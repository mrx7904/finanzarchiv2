use rusqlite::{Connection, params};

use super::*;
use crate::archive;
use crate::kinds::Kind;
use crate::testing;

fn party(conn: &Connection, name: &str) -> i64 {
    crate::parties::party_of(conn, name, None, testing::t0())
        .unwrap()
        .unwrap()
}

/// A filed document of the kind with its values as code read them.
fn document(conn: &Connection, number: u8, kind: Kind, party: i64, values: &[(&str, &str)]) -> i64 {
    conn.execute(
        "INSERT INTO documents (sha256, file_type, kind, original_name, size_bytes, source, imported_at, state, party_id)
         VALUES (?1, 'pdf', ?2, ?3, 1, 'dialog', 'x', 'filed', ?4)",
        params![format!("{number:064x}"), kind.code(), format!("d{number}.pdf"), party],
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
        "INSERT INTO bookings (account_id, position, day, amount_cents, kind, other_side, purpose, party_id) VALUES (?1, 0, ?2, ?3, 'Lastschrift', 'Testhaus Energie AG', ?4, ?5)",
        params![account, day, cents, purpose, party],
    )
    .unwrap();
    conn.last_insert_rowid()
}

#[test]
fn a_booking_shows_its_document_its_party_over_the_year_and_what_might_be_its_document() {
    let dir = tempfile::tempdir().unwrap();
    let archive::Opened::Ready(ready) =
        archive::open(&dir.path().join(archive::FILE_NAME), testing::t0()).unwrap()
    else {
        panic!("not ready")
    };
    let conn = ready.connection();
    let energy = party(conn, "Testhaus Energie AG");
    let invoice = document(
        conn,
        1,
        Kind::Invoice,
        energy,
        &[
            ("gross", "8500"),
            ("invoice_date", "2026-02-01"),
            ("invoice_number", "R-2026-7"),
            ("sender", "Testhaus Energie AG"),
        ],
    );
    let paid = booking(conn, "2026-02-14", -8_500, Some(energy), "Abschlag Februar");
    let earlier = booking(conn, "2026-01-14", -8_500, Some(energy), "Abschlag Januar");
    booking(
        conn,
        "2025-12-14",
        -8_000,
        Some(energy),
        "Abschlag Dezember",
    );
    let twice = document(
        conn,
        2,
        Kind::Invoice,
        energy,
        &[("gross", "4200"), ("invoice_date", "2026-03-01")],
    );
    let first = booking(conn, "2026-03-03", -4_200, Some(energy), "");
    booking(conn, "2026-03-20", -4_200, Some(energy), "");
    crate::links::find(conn, testing::t0()).unwrap();

    let detail = detail_of(conn, paid).unwrap();
    assert_eq!(detail.purpose, "Abschlag Februar");
    assert!(!detail.incoming && detail.amount.contains("85,00"));
    assert_eq!(detail.documents.len(), 1);
    let card = &detail.documents[0];
    assert_eq!(
        (
            card.document,
            card.sender.as_deref(),
            card.number.as_deref(),
            card.day.as_deref()
        ),
        (
            u32::try_from(invoice).unwrap(),
            Some("Testhaus Energie AG"),
            Some("R-2026-7"),
            Some("2026-02-01")
        )
    );
    assert!(
        card.amount
            .as_deref()
            .is_some_and(|amount| amount.contains("85,00"))
    );
    assert!(!card.why.is_empty());
    let year = detail.party_year.as_ref().unwrap();
    assert_eq!(
        (year.party.as_str(), year.year, year.count),
        ("Testhaus Energie", 2026, 4)
    );
    assert!(year.sum.contains("254,00"), "{}", year.sum);
    assert!(
        !year
            .latest
            .iter()
            .any(|brief| brief.id == u32::try_from(paid).unwrap()),
        "not itself"
    );
    assert_eq!(year.latest.len(), 4, "the year before too");
    assert!(detail.candidates.is_empty() && detail.movement.is_none());
    assert!(detail_of(conn, earlier).unwrap().documents.is_empty());

    let asked = detail_of(conn, first).unwrap();
    assert_eq!(asked.candidates.len(), 1, "a question in Prüfen names it");
    assert_eq!(asked.candidates[0].document, u32::try_from(twice).unwrap());
    assert_eq!(asked.candidates[0].link, 0);
    crate::links::answer(conn, twice, Some(first), testing::t0()).unwrap();
    let answered = detail_of(conn, first).unwrap();
    assert!(answered.candidates.is_empty());
    assert_eq!(
        answered.documents[0].document,
        u32::try_from(twice).unwrap()
    );
}
