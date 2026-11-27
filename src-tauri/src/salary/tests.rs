use super::*;
use crate::archive::{self, Opened};
use crate::testing;

fn ready(dir: &std::path::Path) -> archive::Archive {
    match archive::open(&dir.join(archive::FILE_NAME), testing::t0()).unwrap() {
        Opened::Ready(ready) => ready,
        other => panic!("{other:?}"),
    }
}

fn payslip(conn: &Connection, number: u8, state: &str, values: &[(&str, &str)]) -> i64 {
    conn.execute(
        "INSERT INTO documents (sha256, file_type, kind, original_name, size_bytes, source, imported_at, state)
         VALUES (?1, 'pdf', 'payslip', 'd.pdf', 1, 'dialog', 'x', ?2)",
        params![format!("{number:064x}"), state],
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

fn month_values(period: &str, brutto: &str, netto: &str) -> Vec<(&'static str, String)> {
    vec![
        ("period", period.to_owned()),
        ("brutto", brutto.to_owned()),
        ("netto", netto.to_owned()),
        ("lohnsteuer", "52000".to_owned()),
        ("rentenversicherung", "37200".to_owned()),
    ]
}

fn add(conn: &Connection, number: u8, period: &str, brutto: &str, netto: &str) -> i64 {
    let values = month_values(period, brutto, netto);
    let values: Vec<(&str, &str)> = values.iter().map(|(f, v)| (*f, v.as_str())).collect();
    payslip(conn, number, "filed", &values)
}

fn paid(conn: &Connection, payslip: i64, day: &str, cents: i64) {
    let account =
        crate::statements::account_of(conn, "DE89370400440532013000", None, testing::t0()).unwrap();
    conn.execute(
        "INSERT INTO bookings (account_id, position, day, amount_cents, kind, purpose) VALUES (?1, 0, ?2, ?3, 'Gutschrift', 'Lohn')",
        params![account, day, cents],
    )
    .unwrap();
    let booking = conn.last_insert_rowid();
    conn.execute(
        "INSERT INTO links (document_id, booking_id, cents, rule, inputs, decided_at) VALUES (?1, ?2, ?3, 'test', '{}', 'x')",
        params![payslip, booking, cents],
    )
    .unwrap();
}

#[test]
fn every_month_of_the_range_has_its_amounts_and_one_without_a_payslip_is_missing() {
    let dir = tempfile::tempdir().unwrap();
    let ready = ready(dir.path());
    let conn = ready.connection();
    add(conn, 1, "2025-11", "400000", "259440");
    add(conn, 2, "2025-12", "400000", "259440");
    add(conn, 3, "2026-02", "410000", "264000");
    let months = months(conn).unwrap();
    let shown: Vec<(i32, i8, bool)> = months
        .iter()
        .map(|m| (m.year, m.month, m.missing()))
        .collect();
    assert_eq!(
        shown,
        vec![
            (2025, 11, false),
            (2025, 12, false),
            (2026, 1, true),
            (2026, 2, false)
        ]
    );
    // A missing month is no zero: it has no amounts at all.
    assert!(months[2].amounts.is_empty());
    assert_eq!(months[3].amounts.get("brutto"), Some(&410_000));
    assert_eq!(months[3].amounts.get("kirchensteuer"), None);
}

#[test]
fn a_year_sums_its_months_and_counts_what_is_missing() {
    let dir = tempfile::tempdir().unwrap();
    let ready = ready(dir.path());
    let conn = ready.connection();
    add(conn, 1, "2025-11", "400000", "259440");
    add(conn, 2, "2026-01", "400000", "259440");
    add(conn, 3, "2026-03", "410000", "264000");
    let years = years(conn).unwrap();
    assert_eq!(
        years.iter().map(|y| y.year).collect::<Vec<_>>(),
        vec![2026, 2025]
    );
    let this = &years[0];
    assert_eq!(this.months.len(), 3);
    assert_eq!(this.missing(), 1);
    assert_eq!(this.totals.get("brutto"), Some(&810_000));
    assert_eq!(this.totals.get("lohnsteuer"), Some(&104_000));
    // Months after the last payslip show nothing.
    assert_eq!(this.months.last().map(|m| m.month), Some(3));
}

#[test]
fn the_netto_stands_against_the_booking_that_paid_it() {
    let dir = tempfile::tempdir().unwrap();
    let ready = ready(dir.path());
    let conn = ready.connection();
    let november = add(conn, 1, "2025-11", "400000", "259440");
    let december = add(conn, 2, "2025-12", "400000", "259440");
    add(conn, 3, "2026-01", "400000", "259440");
    paid(conn, november, "2025-11-28", 259_440);
    paid(conn, december, "2025-12-29", 254_440);
    let months = months(conn).unwrap();
    assert_eq!(months[0].paid, Some(259_440));
    assert_eq!(months[0].difference(), None);
    assert_eq!(months[1].difference(), Some(-5_000));
    assert_eq!(months[2].paid, None);
    assert_eq!(months[2].difference(), None);
}

#[test]
fn two_employers_in_a_month_are_summed_and_a_payslip_not_filed_does_not_count() {
    let dir = tempfile::tempdir().unwrap();
    let ready = ready(dir.path());
    let conn = ready.connection();
    add(conn, 1, "2026-01", "400000", "259440");
    add(conn, 2, "2026-01", "52000", "52000");
    payslip(
        conn,
        3,
        "waiting",
        &[("period", "2026-02"), ("brutto", "1")],
    );
    let months = months(conn).unwrap();
    assert_eq!(months.len(), 1);
    assert_eq!(months[0].payslips.len(), 2);
    assert_eq!(months[0].amounts.get("netto"), Some(&311_440));
}

#[test]
fn the_owners_value_counts_over_the_readers() {
    let dir = tempfile::tempdir().unwrap();
    let ready = ready(dir.path());
    let conn = ready.connection();
    let id = add(conn, 1, "2026-01", "400000", "259440");
    conn.execute(
        "INSERT INTO field_values (document_id, field, printed, value, reader, read_at) VALUES (?1, 'netto', '2.600,00', '260000', 'owner', 'x')",
        params![id],
    )
    .unwrap();
    let months = months(conn).unwrap();
    assert_eq!(months[0].amounts.get("netto"), Some(&260_000));
}

#[test]
fn no_payslip_no_months() {
    let dir = tempfile::tempdir().unwrap();
    let ready = ready(dir.path());
    assert!(months(ready.connection()).unwrap().is_empty());
}
