use super::*;
use crate::archive::{self, Opened};
use crate::testing;

fn ready(dir: &std::path::Path) -> archive::Archive {
    match archive::open(&dir.join(archive::FILE_NAME), testing::t0()).unwrap() {
        Opened::Ready(ready) => ready,
        other => panic!("{other:?}"),
    }
}

const GIRO: &str = "DE89370400440532013000";
const SAVINGS: &str = "GB82WEST12345698765432";

fn booking(
    conn: &Connection,
    account: i64,
    day: &str,
    cents: i64,
    kind: &str,
    other: &str,
    purpose: &str,
) -> i64 {
    conn.execute(
        "INSERT INTO bookings (account_id, position, day, amount_cents, kind, other_side, purpose) VALUES (?1, 0, ?2, ?3, ?4, ?5, ?6)",
        params![account, day, cents, kind, other, purpose],
    )
    .unwrap();
    conn.last_insert_rowid()
}

fn accounts(conn: &Connection) -> (i64, i64) {
    (
        crate::statements::account_of(conn, GIRO, Some("Beispielbank eG"), testing::t0()).unwrap(),
        crate::statements::account_of(conn, SAVINGS, Some("Beispielbank eG"), testing::t0())
            .unwrap(),
    )
}

fn pairs(conn: &Connection) -> Vec<(i64, Option<i64>, String)> {
    conn.prepare("SELECT out_booking, in_booking, rule FROM booking_pairs WHERE undone_at IS NULL ORDER BY id")
        .unwrap()
        .query_map([], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)))
        .unwrap()
        .collect::<rusqlite::Result<_>>()
        .unwrap()
}

#[test]
fn a_booking_naming_an_own_iban_is_paired_with_the_other_half() {
    let dir = tempfile::tempdir().unwrap();
    let ready = ready(dir.path());
    let conn = ready.connection();
    let (giro, savings) = accounts(conn);
    let out = booking(
        conn,
        giro,
        "2026-01-10",
        -50_000,
        "Überweisung",
        "Erika Musterfrau",
        "Sparen IBAN: GB82 WEST 1234 5698 7654 32",
    );
    let into = booking(
        conn,
        savings,
        "2026-01-10",
        50_000,
        "Gutschrift",
        "Erika Musterfrau",
        "Sparen",
    );
    booking(
        conn,
        giro,
        "2026-01-10",
        -50_000,
        "Lastschrift",
        "Muster Vermietung",
        "Miete",
    );
    assert_eq!(find(conn, testing::t0()).unwrap(), 1);
    assert_eq!(pairs(conn), [(out, Some(into), RULE_OWN_IBAN.to_owned())]);
    assert_eq!(find(conn, testing::t0()).unwrap(), 0, "never twice");
    assert!(is_move(conn, out).unwrap() && is_move(conn, into).unwrap());
}

#[test]
fn without_a_statement_on_the_other_side_the_app_makes_the_other_half() {
    let dir = tempfile::tempdir().unwrap();
    let ready = ready(dir.path());
    let conn = ready.connection();
    let (giro, savings) = accounts(conn);
    let out = booking(
        conn,
        giro,
        "2026-01-10",
        -50_000,
        "Überweisung",
        "Erika Musterfrau",
        "IBAN: GB82WEST12345698765432",
    );
    assert_eq!(find(conn, testing::t0()).unwrap(), 1);
    let found = pairs(conn);
    let [(found_out, Some(made), _)] = found.as_slice() else {
        panic!("one pair with a half: {found:?}")
    };
    let (found_out, made) = (*found_out, *made);
    assert_eq!(found_out, out);
    let (account, cents, by): (i64, i64, Option<String>) = conn
        .query_row(
            "SELECT account_id, amount_cents, made_by FROM bookings WHERE id = ?1",
            [made],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        )
        .unwrap();
    assert_eq!(
        (account, cents, by.as_deref()),
        (savings, 50_000, Some(RULE_OWN_IBAN))
    );
    let pair: i64 = conn
        .query_row("SELECT id FROM booking_pairs", [], |row| row.get(0))
        .unwrap();
    assert!(undo(conn, pair, testing::t0()).unwrap());
    assert!(!is_move(conn, out).unwrap());
    let counts: i64 = conn
        .query_row(
            "SELECT count(*) FROM bookings WHERE id = ?1 AND replaced_by IS NULL",
            [made],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(counts, 0, "the made half counts no more");
    assert_eq!(
        find(conn, testing::t0()).unwrap(),
        0,
        "an undone pair is not made again"
    );
}

#[test]
fn a_marked_move_and_a_same_day_pair_of_one_bank_are_moves() {
    let dir = tempfile::tempdir().unwrap();
    let ready = ready(dir.path());
    let conn = ready.connection();
    let (giro, savings) = accounts(conn);
    let marked = booking(
        conn,
        giro,
        "2026-02-01",
        -10_000,
        "Umbuchung",
        "Erika Musterfrau",
        "Tagesgeld",
    );
    let marked_in = booking(
        conn,
        savings,
        "2026-02-01",
        10_000,
        "Gutschrift",
        "Erika Musterfrau",
        "",
    );
    let out = booking(
        conn,
        savings,
        "2026-02-03",
        -7_500,
        "Überweisung",
        "Erika Musterfrau",
        "",
    );
    let into = booking(
        conn,
        giro,
        "2026-02-03",
        7_500,
        "Gutschrift",
        "Erika Musterfrau",
        "",
    );
    assert_eq!(find(conn, testing::t0()).unwrap(), 2);
    let found = pairs(conn);
    assert!(
        found.contains(&(marked, Some(marked_in), RULE_MARKED.to_owned())),
        "{found:?}"
    );
    assert!(
        found.contains(&(out, Some(into), RULE_SAME_DAY.to_owned())),
        "{found:?}"
    );
}

#[test]
fn the_bank_itself_is_never_an_own_account() {
    let dir = tempfile::tempdir().unwrap();
    let ready = ready(dir.path());
    let conn = ready.connection();
    let (giro, savings) = accounts(conn);
    // A dividend of the share of the bank names an own IBAN in its purpose (lesson of 1.0).
    booking(
        conn,
        giro,
        "2026-03-01",
        1_200,
        "Dividende",
        "Beispielbank eG",
        "Dividende Anteil IBAN: GB82WEST12345698765432",
    );
    booking(
        conn,
        savings,
        "2026-03-01",
        -1_200,
        "Überweisung",
        "Beispielbank eG",
        "",
    );
    assert_eq!(find(conn, testing::t0()).unwrap(), 0);
    // Two equal bookings the same day leave the pair unclear: none is made.
    booking(
        conn,
        giro,
        "2026-03-05",
        -900,
        "Überweisung",
        "Erika Musterfrau",
        "",
    );
    booking(
        conn,
        savings,
        "2026-03-05",
        900,
        "Gutschrift",
        "Erika Musterfrau",
        "",
    );
    booking(
        conn,
        savings,
        "2026-03-05",
        900,
        "Gutschrift",
        "Erika Musterfrau",
        "",
    );
    assert_eq!(find(conn, testing::t0()).unwrap(), 0);
}

#[test]
fn a_made_half_gives_way_when_the_statement_of_its_account_arrives() {
    let dir = tempfile::tempdir().unwrap();
    let ready = ready(dir.path());
    let conn = ready.connection();
    let (giro, savings) = accounts(conn);
    let out = booking(
        conn,
        giro,
        "2026-01-10",
        -50_000,
        "Überweisung",
        "Erika Musterfrau",
        "IBAN: GB82WEST12345698765432",
    );
    find(conn, testing::t0()).unwrap();
    assert_eq!(
        conn.query_row(
            "SELECT count(*) FROM bookings WHERE made_by IS NOT NULL",
            [],
            |row| row.get::<_, i64>(0)
        )
        .unwrap(),
        1
    );
    // The savings statement arrives later with its own booking of the move.
    let into = booking(
        conn,
        savings,
        "2026-01-10",
        50_000,
        "Gutschrift",
        "Erika Musterfrau",
        "",
    );
    assert_eq!(
        give_way(conn, savings, "2025-12-31", "2026-01-31").unwrap(),
        1
    );
    find(conn, testing::t0()).unwrap();
    assert_eq!(pairs(conn), [(out, Some(into), RULE_OWN_IBAN.to_owned())]);
    assert_eq!(
        conn.query_row(
            "SELECT count(*) FROM bookings WHERE made_by IS NOT NULL",
            [],
            |row| row.get::<_, i64>(0)
        )
        .unwrap(),
        0
    );
}
