use super::*;
use crate::archive::{self, Opened};
use crate::testing;

fn ready(dir: &std::path::Path) -> archive::Archive {
    match archive::open(&dir.join(archive::FILE_NAME), testing::t0()).unwrap() {
        Opened::Ready(ready) => ready,
        other => panic!("{other:?}"),
    }
}

fn account(conn: &Connection) -> i64 {
    crate::statements::account_of(conn, "DE89370400440532013000", None, testing::t0()).unwrap()
}

fn party(conn: &Connection, name: &str, category: Option<&str>) -> i64 {
    let id = crate::parties::party_of(conn, name, None, testing::t0())
        .unwrap()
        .unwrap();
    if let Some(category) = category {
        conn.execute(
            "UPDATE parties SET category_id = (SELECT id FROM categories WHERE name = ?2) WHERE id = ?1",
            params![id, category],
        )
        .unwrap();
    }
    id
}

fn book(conn: &Connection, account: i64, party: i64, day: &str, cents: i64) -> i64 {
    conn.execute(
        "INSERT INTO bookings (account_id, position, day, amount_cents, kind, purpose, party_id) VALUES (?1, 0, ?2, ?3, 'Lastschrift', '', ?4)",
        params![account, day, cents, party],
    )
    .unwrap();
    conn.last_insert_rowid()
}

fn monthly(conn: &Connection, account: i64, party: i64, from: i8, to: i8, cents: i64) {
    for month in from..=to {
        book(conn, account, party, &format!("2026-{month:02}-05"), cents);
    }
}

fn today() -> Date {
    jiff::civil::date(2026, 11, 25)
}

fn kinds(hints: &[Hint]) -> Vec<Kind> {
    hints.iter().map(|hint| hint.kind).collect()
}

#[test]
fn a_recurring_payment_whose_amount_rose_is_a_hint_with_its_bookings() {
    let dir = tempfile::tempdir().unwrap();
    let ready = ready(dir.path());
    let conn = ready.connection();
    let giro = account(conn);
    let music = party(conn, "Beispiel Musik", Some("Abos und Medien"));
    monthly(conn, giro, music, 5, 7, -999);
    monthly(conn, giro, music, 8, 11, -1_199);
    let hints = found(conn, today()).unwrap();
    assert_eq!(kinds(&hints), vec![Kind::Rose]);
    let rose = &hints[0];
    assert_eq!(rose.subject, "Beispiel Musik");
    assert_eq!(rose.line, "seit August 2026 2,00 € mehr, 143,88 € im Jahr");
    assert_eq!(rose.cents, -2_400);
    assert_eq!(rose.bookings.len(), 7);
}

#[test]
fn a_rise_of_an_insurance_is_no_hint() {
    let dir = tempfile::tempdir().unwrap();
    let ready = ready(dir.path());
    let conn = ready.connection();
    let giro = account(conn);
    let insurer = party(conn, "Beispiel Versicherung", Some("Versicherungen"));
    monthly(conn, giro, insurer, 5, 7, -2_350);
    monthly(conn, giro, insurer, 8, 11, -2_590);
    assert!(found(conn, today()).unwrap().is_empty());
}

#[test]
fn two_running_payments_of_one_category_are_a_hint() {
    let dir = tempfile::tempdir().unwrap();
    let ready = ready(dir.path());
    let conn = ready.connection();
    let giro = account(conn);
    let films = party(conn, "Beispiel Film", Some("Abos und Medien"));
    let series = party(conn, "Beispiel Serien", Some("Abos und Medien"));
    monthly(conn, giro, films, 6, 11, -1_399);
    monthly(conn, giro, series, 6, 11, -999);
    let hints = found(conn, today()).unwrap();
    assert_eq!(kinds(&hints), vec![Kind::Twice]);
    assert_eq!(hints[0].subject, "Abos und Medien");
    assert_eq!(hints[0].cents, -28_776);
    assert_eq!(hints[0].bookings.len(), 12);
}

#[test]
fn the_fees_of_the_year_are_a_hint() {
    let dir = tempfile::tempdir().unwrap();
    let ready = ready(dir.path());
    let conn = ready.connection();
    let giro = account(conn);
    let bank = party(conn, "Beispielbank", Some("Bankgebühren"));
    book(conn, giro, bank, "2025-12-31", -695);
    book(conn, giro, bank, "2026-01-30", -695);
    book(conn, giro, bank, "2026-03-12", -150);
    let hints = found(conn, today()).unwrap();
    let fees: Vec<&Hint> = hints
        .iter()
        .filter(|hint| hint.kind == Kind::Fees)
        .collect();
    assert_eq!(fees.len(), 1);
    assert_eq!(fees[0].line, "2026: 8,45 € in 2 Buchungen");
}

#[test]
fn a_category_clearly_above_the_same_months_of_the_year_before_is_a_hint() {
    let dir = tempfile::tempdir().unwrap();
    let ready = ready(dir.path());
    let conn = ready.connection();
    let giro = account(conn);
    let restaurant = party(conn, "Beispiel Restaurant", Some("Restaurant und Café"));
    let market = party(conn, "Beispiel Markt", Some("Lebensmittel"));
    // The year before until March: 100 € in restaurants, 300 € food.
    book(conn, giro, restaurant, "2025-02-10", -10_000);
    book(conn, giro, market, "2025-02-11", -30_000);
    // A booking after March of the year before does not count against this year until March.
    book(conn, giro, restaurant, "2025-06-10", -50_000);
    // This year until March: 250 € in restaurants, 320 € food.
    book(conn, giro, restaurant, "2026-01-10", -15_000);
    book(conn, giro, restaurant, "2026-03-10", -10_000);
    book(conn, giro, market, "2026-03-11", -32_000);
    let hints = found(conn, jiff::civil::date(2026, 3, 31)).unwrap();
    let above: Vec<&Hint> = hints
        .iter()
        .filter(|hint| hint.kind == Kind::Above)
        .collect();
    assert_eq!(above.len(), 1);
    assert_eq!(above[0].subject, "Restaurant und Café");
    assert_eq!(above[0].line, "2026 bis März: 250,00 € statt 100,00 €");
    assert_eq!(above[0].bookings.len(), 2);
}

#[test]
fn a_payment_that_runs_after_a_cancellation_of_its_party_is_a_hint() {
    let dir = tempfile::tempdir().unwrap();
    let ready = ready(dir.path());
    let conn = ready.connection();
    let giro = account(conn);
    let gym = party(conn, "Beispiel Fitness", Some("Freizeit und Hobby"));
    monthly(conn, giro, gym, 3, 11, -1_999);
    conn.execute(
        "INSERT INTO documents (sha256, file_type, kind, original_name, size_bytes, source, imported_at, state, party_id)
         VALUES (?1, 'pdf', 'other', 'k.pdf', 1, 'dialog', '2026-07-01T08:00:00Z', 'filed', ?2)",
        params![format!("{:064x}", 7), gym],
    )
    .unwrap();
    let letter = conn.last_insert_rowid();
    conn.execute(
        "INSERT INTO pages (document_id, number, width, height) VALUES (?1, 1, 595, 842)",
        [letter],
    )
    .unwrap();
    conn.execute(
        "INSERT INTO page_lines (document_id, page, line, text, left, bottom, right, top) VALUES (?1, 1, 1, 'Bestätigung Ihrer Kündigung zum 31.07.2026', 0, 0, 1, 1)",
        [letter],
    )
    .unwrap();
    conn.execute(
        "INSERT INTO field_values (document_id, field, printed, value, reader, read_at) VALUES (?1, 'invoice_date', '15.06.2026', '2026-06-15', 'code:5', 'x')",
        [letter],
    )
    .unwrap();
    let hints = found(conn, today()).unwrap();
    let still: Vec<&Hint> = hints
        .iter()
        .filter(|hint| hint.kind == Kind::StillRuns)
        .collect();
    assert_eq!(still.len(), 1);
    // June 15 plus a month: the payments of August to November are after it.
    assert_eq!(
        still[0].line,
        "nach der Kündigung vom 15.06.2026 noch 4× gezahlt"
    );
    assert_eq!(still[0].documents, vec![letter]);
    assert_eq!(still[0].cents, -7_996);
}

#[test]
fn a_dismissed_hint_stays_away_until_its_numbers_change() {
    let dir = tempfile::tempdir().unwrap();
    let ready = ready(dir.path());
    let conn = ready.connection();
    let giro = account(conn);
    let bank = party(conn, "Beispielbank", Some("Bankgebühren"));
    book(conn, giro, bank, "2026-01-30", -695);
    let hint = found(conn, today()).unwrap().remove(0);
    dismiss(conn, &hint.key, &hint.numbers, testing::t0()).unwrap();
    assert!(found(conn, today()).unwrap().is_empty());
    book(conn, giro, bank, "2026-02-27", -695);
    let back = found(conn, today()).unwrap();
    assert_eq!(back.len(), 1);
    assert_eq!(back[0].line, "2026: 13,90 € in 2 Buchungen");
}

#[test]
fn no_hint_is_advice() {
    let dir = tempfile::tempdir().unwrap();
    let ready = ready(dir.path());
    let conn = ready.connection();
    let giro = account(conn);
    let music = party(conn, "Beispiel Musik", Some("Abos und Medien"));
    let other = party(conn, "Beispiel Hörbuch", Some("Abos und Medien"));
    let bank = party(conn, "Beispielbank", Some("Bankgebühren"));
    monthly(conn, giro, music, 5, 7, -999);
    monthly(conn, giro, music, 8, 11, -1_199);
    monthly(conn, giro, other, 5, 11, -995);
    book(conn, giro, bank, "2026-01-30", -695);
    let hints = found(conn, today()).unwrap();
    assert!(hints.len() >= 3);
    for hint in hints {
        let text = format!("{} {}", hint.subject, hint.line).to_lowercase();
        for advice in [
            "kündige",
            "wechsle",
            "solltest",
            "empfehl",
            "investier",
            "spare ",
        ] {
            assert!(!text.contains(advice), "{text}");
        }
    }
}
