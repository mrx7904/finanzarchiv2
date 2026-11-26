use super::*;
use crate::archive::{self, Opened};
use crate::testing;
use jiff::civil::date;
use proptest::prelude::*;

fn ready(dir: &std::path::Path) -> archive::Archive {
    match archive::open(&dir.join(archive::FILE_NAME), testing::t0()).unwrap() {
        Opened::Ready(ready) => ready,
        other => panic!("{other:?}"),
    }
}

fn account(conn: &Connection) -> i64 {
    crate::statements::account_of(
        conn,
        "DE89370400440532013000",
        Some("Beispielbank eG"),
        testing::t0(),
    )
    .unwrap()
}

/// A statement of the account with its bookings, as `statements::store` keeps them.
fn statement(
    conn: &Connection,
    number: u8,
    account: i64,
    from: Date,
    to: Date,
    opening: i64,
    bookings: &[(Date, i64)],
) -> i64 {
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
        params![document, account, from.to_string(), to.to_string(), opening, opening + sum, sum],
    )
    .unwrap();
    let id = conn.last_insert_rowid();
    for (position, (day, cents)) in bookings.iter().enumerate() {
        conn.execute(
            "INSERT INTO bookings (account_id, statement_id, position, day, amount_cents, kind, purpose) VALUES (?1, ?2, ?3, ?4, ?5, 'Buchung', '')",
            params![account, id, position as i64, day.to_string(), cents],
        )
        .unwrap();
    }
    refresh_opening(conn, account).unwrap();
    id
}

#[test]
fn the_balance_counts_from_the_earliest_statement_and_matches_the_bank() {
    let dir = tempfile::tempdir().unwrap();
    let ready = ready(dir.path());
    let conn = ready.connection();
    let id = account(conn);
    assert_eq!(findings(conn, id).unwrap(), [Finding::NoOpening]);
    statement(
        conn,
        2,
        id,
        date(2026, 1, 31),
        date(2026, 2, 28),
        50_000,
        &[(date(2026, 2, 3), -2_000)],
    );
    statement(
        conn,
        1,
        id,
        date(2025, 12, 31),
        date(2026, 1, 31),
        40_000,
        &[(date(2026, 1, 15), 10_000)],
    );
    assert_eq!(
        opening(conn, id).unwrap().map(|opening| opening.day),
        Some(date(2025, 12, 31)),
        "an earlier statement moves the opening back"
    );
    assert_eq!(balance_on(conn, id, date(2025, 12, 30)).unwrap(), None);
    assert_eq!(
        balance_on(conn, id, date(2026, 1, 14)).unwrap(),
        Some(Cents::new(40_000))
    );
    assert_eq!(
        balance_on(conn, id, date(2026, 1, 15)).unwrap(),
        Some(Cents::new(50_000))
    );
    assert_eq!(
        balance_on(conn, id, date(2026, 2, 28)).unwrap(),
        Some(Cents::new(48_000))
    );
    assert_eq!(findings(conn, id).unwrap(), []);
    let listed = list(conn, date(2026, 3, 10)).unwrap();
    assert_eq!(listed[0].balance, Some(Cents::new(48_000)));
    assert_eq!(
        listed[0].bank_balance,
        Some((date(2026, 2, 28), Cents::new(48_000)))
    );
}

#[test]
fn a_gap_and_a_difference_are_findings() {
    let dir = tempfile::tempdir().unwrap();
    let ready = ready(dir.path());
    let conn = ready.connection();
    let id = account(conn);
    statement(
        conn,
        1,
        id,
        date(2025, 12, 31),
        date(2026, 1, 31),
        40_000,
        &[(date(2026, 1, 15), 10_000)],
    );
    // March is read, February is missing: it starts later and with another balance.
    statement(
        conn,
        3,
        id,
        date(2026, 2, 28),
        date(2026, 3, 31),
        47_000,
        &[(date(2026, 3, 2), -1_000)],
    );
    let found = findings(conn, id).unwrap();
    assert!(matches!(found[0], Finding::Gap { .. }), "{found:?}");
    assert!(found[0].sentence().contains("dazwischen fehlt etwas"));
    assert!(
        found.iter().any(|finding| matches!(finding, Finding::Differs { bank, counted, .. } if *bank == Cents::new(46_000) && *counted == Cents::new(49_000))),
        "{found:?}"
    );
    assert!(found.iter().all(|finding| !finding.sentence().is_empty()));
}

#[test]
fn an_opening_the_owner_typed_is_never_written_over() {
    let dir = tempfile::tempdir().unwrap();
    let ready = ready(dir.path());
    let conn = ready.connection();
    let id = account(conn);
    set_opening(conn, id, date(2025, 12, 1), Cents::new(1_000)).unwrap();
    statement(
        conn,
        1,
        id,
        date(2025, 12, 31),
        date(2026, 1, 31),
        40_000,
        &[(date(2026, 1, 15), 10_000)],
    );
    let kept = opening(conn, id).unwrap().unwrap();
    assert_eq!(
        (kept.day, kept.balance, kept.by_owner),
        (date(2025, 12, 1), Cents::new(1_000), true)
    );
}

proptest! {
    /// Bookings in any order give the same balance, and a statement in two parts gives the same as one.
    #[test]
    fn the_balance_does_not_depend_on_order_or_on_how_statements_are_cut(
        amounts in prop::collection::vec(-100_000_i64..100_000, 1..30),
        cut in 0_usize..30,
        seed in any::<u64>(),
    ) {
        let days: Vec<Date> = (0..amounts.len()).map(|index| date(2026, 1, 1 + i8::try_from(index % 28).unwrap())).collect();
        let mut bookings: Vec<(Date, i64)> = days.iter().copied().zip(amounts.iter().copied()).collect();
        bookings.sort_by_key(|(day, _)| *day);
        let total: i64 = amounts.iter().sum();

        let whole = tempfile::tempdir().unwrap();
        let ready_whole = ready(whole.path());
        let conn = ready_whole.connection();
        let one = account(conn);
        let mut shuffled = bookings.clone();
        let mut state = seed | 1;
        for index in (1..shuffled.len()).rev() {
            state ^= state << 13;
            state ^= state >> 7;
            state ^= state << 17;
            shuffled.swap(index, usize::try_from(state % (index as u64 + 1)).unwrap());
        }
        statement(conn, 1, one, date(2025, 12, 31), date(2026, 1, 31), 10_000, &shuffled);
        prop_assert_eq!(balance_on(conn, one, date(2026, 1, 31)).unwrap(), Some(Cents::new(10_000 + total)));
        prop_assert!(findings(conn, one).unwrap().is_empty());

        let parts = tempfile::tempdir().unwrap();
        let ready_parts = ready(parts.path());
        let conn = ready_parts.connection();
        let two = account(conn);
        // Cut after the day of the cut-th booking, so both parts hold whole days.
        let middle = bookings.get(cut.min(bookings.len()).saturating_sub(1)).map_or(date(2025, 12, 31), |(day, _)| *day);
        let middle = if cut == 0 { date(2025, 12, 31) } else { middle };
        let (first, second): (Vec<_>, Vec<_>) = bookings.iter().partition(|(day, _)| *day <= middle);
        let first_sum: i64 = first.iter().map(|(_, cents)| cents).sum();
        statement(conn, 1, two, date(2025, 12, 31), middle, 10_000, &first);
        statement(conn, 2, two, middle, date(2026, 1, 31), 10_000 + first_sum, &second);
        prop_assert_eq!(balance_on(conn, two, date(2026, 1, 31)).unwrap(), Some(Cents::new(10_000 + total)));
        prop_assert!(findings(conn, two).unwrap().is_empty(), "{:?}", findings(conn, two).unwrap());
    }
}

#[test]
fn a_booking_dated_with_the_statement_before_counts_with_its_own() {
    let dir = tempfile::tempdir().unwrap();
    let ready = ready(dir.path());
    let conn = ready.connection();
    let id = account(conn);
    // The second statement holds a booking the bank dated with the closing day of the first.
    statement(
        conn,
        1,
        id,
        date(2025, 12, 31),
        date(2026, 1, 31),
        40_000,
        &[(date(2026, 1, 15), 10_000)],
    );
    statement(
        conn,
        2,
        id,
        date(2026, 1, 31),
        date(2026, 2, 28),
        50_000,
        &[(date(2026, 1, 31), -500), (date(2026, 2, 3), -2_000)],
    );
    assert_eq!(
        findings(conn, id).unwrap(),
        [],
        "each statement is held against its own bookings"
    );
    // The first statement of an account may hold a booking of the day of its old balance too.
    let other =
        crate::statements::account_of(conn, "GB82WEST12345698765432", None, testing::t0()).unwrap();
    statement(
        conn,
        3,
        other,
        date(2025, 12, 31),
        date(2026, 1, 31),
        1_000,
        &[(date(2025, 12, 31), 200)],
    );
    assert_eq!(
        balance_on(conn, other, date(2026, 1, 31)).unwrap(),
        Some(Cents::new(1_200))
    );
    assert_eq!(findings(conn, other).unwrap(), []);
}
