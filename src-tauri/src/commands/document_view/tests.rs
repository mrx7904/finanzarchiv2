use rusqlite::{Connection, params};

use super::*;
use crate::archive::{self, Opened};
use crate::testing;

fn ready(dir: &std::path::Path) -> archive::Archive {
    match archive::open(&dir.join(archive::FILE_NAME), testing::t0()).unwrap() {
        Opened::Ready(ready) => ready,
        other => panic!("{other:?}"),
    }
}

fn invoice(conn: &Connection, number: u8, values: &[(&str, &str, &str)]) -> i64 {
    conn.execute(
        "INSERT INTO documents (sha256, file_type, kind, original_name, size_bytes, source, imported_at, state)
         VALUES (?1, 'pdf', 'invoice', 'r.pdf', 1, 'dialog', 'x', 'filed')",
        [format!("{number:064x}")],
    )
    .unwrap();
    let id = conn.last_insert_rowid();
    for (field, value, reader) in values {
        conn.execute(
            "INSERT INTO field_values (document_id, field, printed, value, reader, page, top, read_at) VALUES (?1, ?2, ?3, ?3, ?4, 1, 700.0, 'x')",
            params![id, field, value, reader],
        )
        .unwrap();
    }
    id
}

#[test]
fn a_value_says_where_it_stands_and_the_booking_that_paid_it_stands_beside() {
    let dir = tempfile::tempdir().unwrap();
    let ready = ready(dir.path());
    let conn = ready.connection();
    let document = invoice(
        conn,
        1,
        &[
            ("sender", "Beispiel Musik", "code:5"),
            ("invoice_date", "2026-03-02", "code:5"),
            ("gross", "11900", "code:5"),
            ("net", "10000", "e-invoice"),
        ],
    );
    conn.execute(
        "INSERT INTO field_checks (document_id, field, check_id, passed, checked_at) VALUES (?1, 'gross', ?2, 1, 'x')",
        params![document, checks::SUM],
    )
    .unwrap();
    conn.execute(
        "INSERT INTO field_values (document_id, field, printed, value, reader, read_at) VALUES (?1, 'sender', 'Beispiel Musikhaus', 'Beispiel Musikhaus', 'owner', 'x')",
        [document],
    )
    .unwrap();
    let account = crate::statements::account_of(
        conn,
        "DE89370400440532013000",
        Some("Beispielbank eG"),
        testing::t0(),
    )
    .unwrap();
    conn.execute(
        "INSERT INTO bookings (account_id, position, day, amount_cents, kind, purpose) VALUES (?1, 0, '2026-03-05', -11900, 'Lastschrift', '')",
        [account],
    )
    .unwrap();
    let booking = conn.last_insert_rowid();
    conn.execute(
        "INSERT INTO links (document_id, booking_id, cents, rule, inputs, decided_at) VALUES (?1, ?2, 11900, 'test', '{}', 'x')",
        params![document, booking],
    )
    .unwrap();
    let found = detail(conn, document).unwrap();
    let shown: Vec<(&str, &str, &str)> = found
        .values
        .iter()
        .map(|v| (v.name.as_str(), v.value.as_str(), v.source.as_str()))
        .collect();
    assert_eq!(
        shown,
        vec![
            ("Absender", "Beispiel Musikhaus", "owner"),
            ("Datum", "02.03.2026", "read"),
            ("Netto", "100,00 €", "sender"),
            ("Brutto", "119,00 €", "checked"),
        ]
    );
    assert_eq!(found.paid_by.len(), 1);
    assert_eq!(found.paid_by[0].amount, "119,00 €");
    assert_eq!(found.paid_by[0].day, "2026-03-05");
}

#[test]
fn a_document_with_a_tax_assignment_can_be_deleted() {
    let dir = tempfile::tempdir().unwrap();
    let ready = ready(dir.path());
    let conn = ready.connection();
    let document = invoice(
        conn,
        1,
        &[
            ("gross", "8900", "code:5"),
            ("invoice_date", "2026-02-01", "code:5"),
        ],
    );
    crate::taxes::assign(
        conn,
        crate::taxes::On::Document(document),
        &crate::taxes::Assignment {
            category: "arbeitsmittel".into(),
            share_percent: 100,
            refunded_cents: 0,
            labour_cents: None,
        },
        testing::t0(),
    )
    .unwrap();
    let root = crate::documents::archive_root(ready.path());
    crate::documents::delete(conn, &root, document).unwrap();
    let left: i64 = conn
        .query_row("SELECT count(*) FROM tax_assignments", [], |row| row.get(0))
        .unwrap();
    assert_eq!(left, 0);
}

#[test]
fn an_order_needs_no_invoice_until_the_owner_says_otherwise_or_one_is_assigned() {
    let dir = tempfile::tempdir().unwrap();
    let ready = ready(dir.path());
    let conn = ready.connection();
    conn.execute(
        "INSERT INTO orders (shop, order_number, ordered_at, total_cents, found_at) VALUES ('Amazon', '123-1234567-1234567', '2026-03-01', 4995, 'x')",
        [],
    )
    .unwrap();
    let order = conn.last_insert_rowid();
    let near = invoice(
        conn,
        1,
        &[
            ("gross", "1999", "code:5"),
            ("invoice_date", "2026-03-02", "code:5"),
        ],
    );
    let same = invoice(
        conn,
        2,
        &[
            ("gross", "4995", "code:5"),
            ("invoice_date", "2026-05-20", "code:5"),
        ],
    );
    let far = invoice(
        conn,
        3,
        &[
            ("gross", "1999", "code:5"),
            ("invoice_date", "2025-01-02", "code:5"),
        ],
    );
    let order_of = |conn: &Connection| crate::orders::list(conn).unwrap().remove(0);
    assert!(!order_of(conn).no_invoice);
    assert!(crate::orders::set_needs_invoice(conn, order, false, testing::t0()).unwrap());
    assert!(order_of(conn).no_invoice);
    assert!(crate::orders::set_needs_invoice(conn, order, true, testing::t0()).unwrap());
    assert!(!order_of(conn).no_invoice);
    // The same total first, then the nearest day.
    let candidates: Vec<i64> = crate::orders::candidates(conn, order)
        .unwrap()
        .iter()
        .map(|c| c.document)
        .collect();
    assert_eq!(candidates, vec![same, near, far]);
    crate::orders::set_needs_invoice(conn, order, false, testing::t0()).unwrap();
    assert!(crate::orders::assign(conn, order, same).unwrap());
    let assigned = order_of(conn);
    assert_eq!(assigned.invoice.map(|(id, _)| id), Some(same));
    assert!(!assigned.no_invoice);
    // An invoice an order holds is no candidate of another.
    assert!(
        !crate::orders::candidates(conn, order)
            .unwrap()
            .iter()
            .any(|c| c.document == same)
    );
}
