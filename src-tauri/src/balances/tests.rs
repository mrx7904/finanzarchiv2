use super::*;
use crate::archive::{self, Opened};
use crate::testing;
use jiff::civil::date;

fn ready(dir: &std::path::Path) -> archive::Archive {
    match archive::open(&dir.join(archive::FILE_NAME), testing::t0()).unwrap() {
        Opened::Ready(ready) => ready,
        other => panic!("{other:?}"),
    }
}

fn account(conn: &Connection, iban: &str) -> i64 {
    crate::statements::account_of(conn, iban, Some("Beispielbank eG"), testing::t0()).unwrap()
}

/// A statement with its bookings; `closing` differs from opening plus bookings when given.
#[allow(clippy::too_many_arguments)]
fn statement(
    conn: &Connection,
    number: u8,
    account: i64,
    from: Date,
    to: Date,
    opening: i64,
    bookings: &[(Date, i64)],
    closing: Option<i64>,
) {
    conn.execute(
        "INSERT INTO documents (sha256, file_type, kind, original_name, size_bytes, source, imported_at) VALUES (?1, 'pdf', 'statement', 'a', 1, 'dialog', 'x')",
        [format!("{number:064x}")],
    )
    .unwrap();
    let document = conn.last_insert_rowid();
    let sum: i64 = bookings.iter().map(|(_, cents)| cents).sum();
    conn.execute(
        "INSERT INTO statements (document_id, account_id, layout, opening_day, closing_day, opening_cents, closing_cents, bookings_cents, read_at)
         VALUES (?1, ?2, 'A', ?3, ?4, ?5, ?6, ?7, 'x')",
        params![document, account, from.to_string(), to.to_string(), opening, closing.unwrap_or(opening + sum), sum],
    )
    .unwrap();
    let id = conn.last_insert_rowid();
    for (position, (day, cents)) in bookings.iter().enumerate() {
        conn.execute(
            "INSERT INTO bookings (account_id, statement_id, position, day, amount_cents, kind, purpose) VALUES (?1, ?2, ?3, ?4, ?5, 'Buchung', '')",
            params![account, id, i64::try_from(position).unwrap(), day.to_string(), cents],
        )
        .unwrap();
    }
    accounts::refresh_opening(conn, account).unwrap();
}

#[test]
fn every_day_has_the_balance_of_balance_on() {
    let dir = tempfile::tempdir().unwrap();
    let ready = ready(dir.path());
    let conn = ready.connection();
    let giro = account(conn, "DE89370400440532013000");
    statement(
        conn,
        1,
        giro,
        date(2025, 12, 31),
        date(2026, 1, 31),
        100_000,
        // The bank dated the first booking with the day of the statement before.
        &[
            (date(2025, 12, 31), -500),
            (date(2026, 1, 5), -2_000),
            (date(2026, 1, 5), 300),
            (date(2026, 1, 20), 259_440),
        ],
        None,
    );
    statement(
        conn,
        2,
        giro,
        date(2026, 1, 31),
        date(2026, 2, 28),
        357_240,
        &[(date(2026, 2, 3), -78_000)],
        None,
    );
    let days = of_account(conn, giro).unwrap();
    assert_eq!(days.first().map(|d| d.day), Some(date(2025, 12, 31)));
    assert_eq!(days.last().map(|d| d.day), Some(date(2026, 2, 28)));
    assert_eq!(days.len(), 60);
    for known in &days {
        assert_eq!(
            Some(known.counted),
            accounts::balance_on(conn, giro, known.day).unwrap(),
            "{}",
            known.day
        );
    }
    let end = on(&days, date(2026, 1, 31)).unwrap();
    assert_eq!(end.bank, Some(Cents::new(357_240)));
    assert_eq!(end.difference(), None);
}

#[test]
fn the_bank_balance_is_shown_and_a_difference_kept() {
    let dir = tempfile::tempdir().unwrap();
    let ready = ready(dir.path());
    let conn = ready.connection();
    let giro = account(conn, "DE89370400440532013000");
    statement(
        conn,
        1,
        giro,
        date(2026, 1, 1),
        date(2026, 1, 31),
        100_000,
        &[(date(2026, 1, 10), -2_000)],
        Some(97_500),
    );
    let days = of_account(conn, giro).unwrap();
    let end = on(&days, date(2026, 1, 31)).unwrap();
    assert_eq!(end.counted, Cents::new(98_000));
    assert_eq!(end.shown(), Cents::new(97_500));
    assert_eq!(end.difference(), Some(Cents::new(-500)));
    // A day without a printed balance shows the one counted.
    assert_eq!(
        on(&days, date(2026, 1, 15)).unwrap().shown(),
        Cents::new(98_000)
    );
}

#[test]
fn the_sum_of_all_counts_each_account_from_its_opening_and_keeps_its_last_balance() {
    let dir = tempfile::tempdir().unwrap();
    let ready = ready(dir.path());
    let conn = ready.connection();
    let giro = account(conn, "DE89370400440532013000");
    let savings = account(conn, "GB82WEST12345698765432");
    statement(
        conn,
        1,
        giro,
        date(2026, 1, 1),
        date(2026, 1, 31),
        100_000,
        &[(date(2026, 1, 10), -2_000)],
        None,
    );
    statement(
        conn,
        2,
        savings,
        date(2026, 1, 15),
        date(2026, 2, 15),
        500_000,
        &[(date(2026, 2, 1), 1_000)],
        None,
    );
    let (accounts, total) = all(conn).unwrap();
    assert_eq!(accounts.len(), 2);
    let at = |day: Date| total.iter().find(|(d, _)| *d == day).map(|(_, c)| *c);
    assert_eq!(at(date(2026, 1, 1)), Some(Cents::new(100_000)));
    assert_eq!(at(date(2026, 1, 15)), Some(Cents::new(598_000)));
    // After the giro's last day it keeps its last balance.
    assert_eq!(at(date(2026, 2, 15)), Some(Cents::new(599_000)));
    assert_eq!(total.last().map(|(d, _)| *d), Some(date(2026, 2, 15)));
}

#[test]
fn an_account_without_an_opening_has_no_days() {
    let dir = tempfile::tempdir().unwrap();
    let ready = ready(dir.path());
    let conn = ready.connection();
    let giro = account(conn, "DE89370400440532013000");
    assert!(of_account(conn, giro).unwrap().is_empty());
    assert_eq!(on(&[], date(2026, 1, 1)), None);
}

#[test]
fn month_ends_run_to_the_last_known_day() {
    assert_eq!(
        month_ends(date(2025, 12, 31), date(2026, 3, 10)),
        vec![
            date(2025, 12, 31),
            date(2026, 1, 31),
            date(2026, 2, 28),
            date(2026, 3, 10)
        ]
    );
    assert_eq!(
        month_ends(date(2026, 1, 5), date(2026, 1, 20)),
        vec![date(2026, 1, 20)]
    );
}

#[test]
fn ten_years_of_days_take_one_pass() {
    let dir = tempfile::tempdir().unwrap();
    let ready = ready(dir.path());
    let conn = ready.connection();
    let giro = account(conn, "DE89370400440532013000");
    let mut bookings = Vec::new();
    let mut day = date(2016, 1, 2);
    while day < date(2025, 12, 31) {
        bookings.push((day, -1_000));
        day = day.checked_add(3.days()).unwrap();
    }
    statement(
        conn,
        1,
        giro,
        date(2016, 1, 1),
        date(2025, 12, 31),
        10_000_000,
        &bookings,
        None,
    );
    let started = std::time::Instant::now();
    let days = of_account(conn, giro).unwrap();
    assert_eq!(days.len(), 3653);
    assert!(started.elapsed() < std::time::Duration::from_secs(2));
}
