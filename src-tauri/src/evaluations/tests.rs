use super::*;
use crate::archive::{self, Opened};
use crate::testing;

fn ready(dir: &std::path::Path) -> archive::Archive {
    match archive::open(&dir.join(archive::FILE_NAME), testing::t0()).unwrap() {
        Opened::Ready(ready) => ready,
        other => panic!("{other:?}"),
    }
}

fn account(conn: &Connection, iban: &str) -> i64 {
    crate::statements::account_of(conn, iban, Some("Beispielbank eG"), testing::t0()).unwrap()
}

fn booking(
    conn: &Connection,
    account: i64,
    day: &str,
    cents: i64,
    party: Option<i64>,
    category: Option<i64>,
) -> i64 {
    conn.execute(
        "INSERT INTO bookings (account_id, position, day, amount_cents, kind, purpose, party_id, category_id) VALUES (?1, 0, ?2, ?3, 'Buchung', '', ?4, ?5)",
        params![account, day, cents, party, category],
    )
    .unwrap();
    conn.last_insert_rowid()
}

fn category(conn: &Connection, name: &str) -> i64 {
    conn.query_row("SELECT id FROM categories WHERE name = ?1", [name], |row| {
        row.get(0)
    })
    .unwrap_or_else(|_| panic!("no category {name}"))
}

fn party(conn: &Connection, name: &str, category: Option<i64>) -> i64 {
    let id = crate::parties::party_of(conn, name, None, testing::t0())
        .unwrap()
        .unwrap();
    conn.execute(
        "UPDATE parties SET category_id = ?2 WHERE id = ?1",
        params![id, category],
    )
    .unwrap();
    id
}

#[test]
fn a_year_counts_each_booking_once_by_its_side_and_a_refund_lowers_its_category() {
    let dir = tempfile::tempdir().unwrap();
    let ready = ready(dir.path());
    let conn = ready.connection();
    let giro = account(conn, "DE89370400440532013000");
    let savings = account(conn, "GB82WEST12345698765432");
    let directions: Vec<(String, String)> = conn
        .prepare("SELECT name, direction FROM categories ORDER BY position")
        .unwrap()
        .query_map([], |row| Ok((row.get(0)?, row.get(1)?)))
        .unwrap()
        .collect::<rusqlite::Result<_>>()
        .unwrap();
    let out_name = &directions.iter().find(|(_, d)| d == "out").unwrap().0;
    let in_name = &directions.iter().find(|(_, d)| d == "in").unwrap().0;
    let spend = category(conn, out_name);
    let earn = category(conn, in_name);
    let shop = party(conn, "Beispiel Laden", Some(spend));
    let employer = party(conn, "Beispiel Arbeitgeber", Some(earn));

    booking(conn, giro, "2026-01-31", 300_000, Some(employer), None);
    booking(conn, giro, "2026-02-03", -5_000, Some(shop), None);
    // The shop pays 10,00 back in March: a refund, it lowers the spending.
    booking(conn, giro, "2026-03-04", 1_000, Some(shop), None);
    // An unknown party, no category: money out is spending.
    booking(conn, giro, "2026-03-10", -2_500, None, None);
    // A move between own accounts counts nowhere.
    let out = booking(conn, giro, "2026-04-01", -50_000, None, None);
    let inn = booking(conn, savings, "2026-04-01", 50_000, None, None);
    conn.execute(
        "INSERT INTO booking_pairs (out_booking, in_booking, rule, inputs, decided_at) VALUES (?1, ?2, 'move.same-day:1', '{}', 'x')",
        params![out, inn],
    )
    .unwrap();
    // A booking replaced by its CSV twin counts once.
    let twin = booking(conn, giro, "2026-05-02", -1_200, Some(shop), None);
    let pdf = booking(conn, giro, "2026-05-02", -1_200, Some(shop), None);
    conn.execute(
        "UPDATE bookings SET replaced_by = ?1 WHERE id = ?2",
        params![twin, pdf],
    )
    .unwrap();
    // The year before is not this year.
    booking(conn, giro, "2025-12-31", -9_999, Some(shop), None);

    let year = year(conn, 2026).unwrap();
    assert_eq!(
        year.total(Side::Income),
        Sum {
            cents: 300_000,
            count: 1
        }
    );
    assert_eq!(
        year.total(Side::Spending),
        Sum {
            cents: 5_000 - 1_000 + 2_500 + 1_200,
            count: 4
        }
    );
    assert_eq!(
        year.spending[2].cents,
        -1_000 + 2_500,
        "March: the refund and the unknown"
    );
    assert_eq!(
        year.income[3].cents + year.spending[3].cents,
        0,
        "the move counts nowhere"
    );
    let shop_category = year
        .categories
        .iter()
        .find(|part| part.id == Some(spend))
        .unwrap();
    assert_eq!(
        shop_category.sum,
        Sum {
            cents: 5_000 - 1_000 + 1_200,
            count: 3
        }
    );
    assert_eq!(
        year.categories.last().unwrap().id,
        None,
        "Ohne Kategorie last"
    );
    let shop_party = year
        .parties
        .iter()
        .find(|part| part.id == Some(shop))
        .unwrap();
    assert_eq!(shop_party.side, Side::Spending);
    assert_eq!(years(conn).unwrap(), [2026, 2025]);
}

#[test]
fn the_side_of_a_booking() {
    assert_eq!(side_of(1_000, None), (Side::Income, 1_000));
    assert_eq!(side_of(-1_000, None), (Side::Spending, 1_000));
    assert_eq!(
        side_of(1_000, Some("out")),
        (Side::Spending, -1_000),
        "a refund"
    );
    assert_eq!(
        side_of(-1_000, Some("in")),
        (Side::Income, -1_000),
        "income paid back"
    );
    assert_eq!(side_of(-1_000, Some("out")), (Side::Spending, 1_000));
}
