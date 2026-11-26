use super::*;
use crate::archive::{self, Opened};
use crate::testing;

fn ready(dir: &Path) -> archive::Archive {
    match archive::open(&dir.join(archive::FILE_NAME), testing::t0()).unwrap() {
        Opened::Ready(ready) => ready,
        other => panic!("{other:?}"),
    }
}

/// A document of the kind with its values as code read them.
fn document(
    conn: &Connection,
    number: u8,
    kind: &str,
    state: &str,
    values: &[(&str, &str)],
) -> i64 {
    conn.execute(
        "INSERT INTO documents (sha256, file_type, kind, original_name, size_bytes, source, imported_at, state)
         VALUES (?1, 'mail', ?2, ?3, 1, 'mail', 'x', ?4)",
        params![format!("{number:064x}"), kind, format!("d{number}.txt"), state],
    )
    .unwrap();
    let id = conn.last_insert_rowid();
    for (field, value) in values {
        conn.execute(
            "INSERT INTO field_values (document_id, field, printed, value, reader, page, top, read_at) VALUES (?1, ?2, ?3, ?3, 'code:6', 1, 700.0, 'x')",
            params![id, field, value],
        )
        .unwrap();
    }
    id
}

fn paypal(conn: &Connection, number: u8, payee: &str, cents: &str, day: &str, code: &str) -> i64 {
    document(
        conn,
        number,
        "receipt",
        "filed",
        &[
            ("sender", payee),
            ("gross", cents),
            ("invoice_date", day),
            ("invoice_number", code),
            (VIA, "PayPal"),
        ],
    )
}

#[test]
fn the_seller_invoice_makes_the_paypal_mail_its_receipt_and_takes_its_booking() {
    let dir = tempfile::tempdir().unwrap();
    let ready = ready(dir.path());
    let conn = ready.connection();
    let receipt = paypal(
        conn,
        1,
        "Beispiel Musik AB",
        "999",
        "2026-11-20",
        "1AB23456CD7890123",
    );
    let invoice = document(
        conn,
        2,
        "invoice",
        "filed",
        &[
            ("sender", "Beispiel Musik"),
            ("gross", "999"),
            ("invoice_date", "2026-11-19"),
        ],
    );
    // Same amount, but too late and by another seller: no pair.
    document(
        conn,
        3,
        "invoice",
        "filed",
        &[
            ("sender", "Beispiel Musik"),
            ("gross", "999"),
            ("invoice_date", "2026-12-30"),
        ],
    );
    document(
        conn,
        4,
        "invoice",
        "filed",
        &[
            ("sender", "Anderer Laden"),
            ("gross", "999"),
            ("invoice_date", "2026-11-21"),
        ],
    );
    let account = crate::statements::account_of(
        conn,
        "DE89370400440532013000",
        Some("Beispielbank eG"),
        testing::t0(),
    )
    .unwrap();
    conn.execute(
        "INSERT INTO bookings (account_id, position, day, amount_cents, kind, purpose) VALUES (?1, 0, '2026-11-22', -999, 'Lastschrift', 'PayPal Europe Ihr Einkauf bei Beispiel Musik')",
        [account],
    )
    .unwrap();
    let booking = conn.last_insert_rowid();
    conn.execute(
        "INSERT INTO links (document_id, booking_id, cents, rule, inputs, decided_at) VALUES (?1, ?2, -999, 'link.amount-name:1', '{}', 'x')",
        params![receipt, booking],
    )
    .unwrap();

    assert_eq!(pair(conn, testing::t0()).unwrap(), 1);
    assert_eq!(pair_of(conn, receipt).unwrap(), Some((invoice, true)));
    assert_eq!(pair_of(conn, invoice).unwrap(), Some((receipt, false)));
    let linked: (i64, String) = conn
        .query_row(
            "SELECT booking_id, rule FROM links WHERE document_id = ?1",
            [invoice],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .unwrap();
    assert_eq!(linked, (booking, RULE_LINK.to_owned()));
    assert_eq!(pair(conn, testing::t0()).unwrap(), 0, "once");
}

#[test]
fn the_order_number_pairs_when_the_names_differ_and_two_fitting_invoices_pair_none() {
    let dir = tempfile::tempdir().unwrap();
    let ready = ready(dir.path());
    let conn = ready.connection();
    let receipt = document(
        conn,
        1,
        "receipt",
        "filed",
        &[
            ("sender", "Zahlungsdienst XY"),
            ("gross", "2380"),
            ("invoice_date", "2026-11-20"),
            ("order_number", "B-2026-0815"),
            (VIA, "PayPal"),
        ],
    );
    let invoice = document(
        conn,
        2,
        "invoice",
        "filed",
        &[
            ("sender", "Beispiel Laden"),
            ("gross", "2380"),
            ("invoice_date", "2026-11-18"),
            ("order_number", "B-2026-0815"),
        ],
    );
    assert_eq!(pair(conn, testing::t0()).unwrap(), 1);
    assert_eq!(pair_of(conn, receipt).unwrap(), Some((invoice, true)));

    let lone = paypal(
        conn,
        3,
        "Beispiel Kino",
        "1250",
        "2026-11-20",
        "9ZY87654XW3210987",
    );
    for number in [4, 5] {
        document(
            conn,
            number,
            "invoice",
            "filed",
            &[
                ("sender", "Beispiel Kino"),
                ("gross", "1250"),
                ("invoice_date", "2026-11-19"),
            ],
        );
    }
    assert_eq!(
        pair(conn, testing::t0()).unwrap(),
        0,
        "two fit: it stays the payment itself"
    );
    assert_eq!(pair_of(conn, lone).unwrap(), None);
}

#[test]
fn two_mails_of_one_payment_are_one() {
    let dir = tempfile::tempdir().unwrap();
    let ready = ready(dir.path());
    let conn = ready.connection();
    let root = dir.path().join("Archiv");
    let first = paypal(
        conn,
        1,
        "Beispiel Musik AB",
        "999",
        "2026-11-20",
        "1AB23456CD7890123",
    );
    let second = paypal(
        conn,
        2,
        "Beispiel Musik AB",
        "999",
        "2026-11-20",
        "1AB23456CD7890123",
    );
    let held = paypal(
        conn,
        3,
        "Beispiel Musik AB",
        "999",
        "2026-11-20",
        "1AB23456CD7890123",
    );
    conn.execute(
        "INSERT INTO field_values (document_id, field, printed, value, reader, read_at) VALUES (?1, 'hold', '1', '1', 'owner', 'x')",
        [held],
    )
    .unwrap();
    let other = paypal(
        conn,
        4,
        "Beispiel Musik AB",
        "999",
        "2026-11-21",
        "2CD34567EF8901234",
    );
    assert_eq!(merge_twins(conn, &root, testing::t0()).unwrap(), 1);
    let left: Vec<i64> = conn
        .prepare("SELECT id FROM documents ORDER BY id")
        .unwrap()
        .query_map([], |row| row.get(0))
        .unwrap()
        .collect::<rusqlite::Result<_>>()
        .unwrap();
    assert_eq!(
        left,
        [first, held, other],
        "the second went, the one the owner held stays"
    );
    assert!(!left.contains(&second));
    assert_eq!(merge_twins(conn, &root, testing::t0()).unwrap(), 0);
}
