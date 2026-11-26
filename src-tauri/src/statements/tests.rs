use super::*;
use jiff::civil::date;

/// A line of a page at the given left edge, top to bottom in the order given.
fn line(page: u32, left: f64, text: &str, index: usize) -> Line {
    let top = 800.0 - 12.0 * index as f64;
    Line {
        page,
        text: text.into(),
        left,
        bottom: top - 9.0,
        right: 560.0,
        top,
    }
}

/// An invented statement of layout A over two pages, with a December booking in January, a carry
/// between the pages, a negative closing balance, the codes of the form at the left edge and the
/// text after the table. Every value is made up.
fn sample() -> Vec<Line> {
    let page_1 = [
        (125.0, "Beispielbank eG"),
        (336.0, "Kontoauszug Nr. 1/2026"),
        (335.0, "erstellt am 31.01.2026 08:00 Blatt 1 von 2"),
        (336.0, "IBAN: DE89 3704 0044 0532 0130 00 BIC: MUSTDEXXXXX"),
        (40.0, "Bu-Tag Wert Vorgang"),
        (234.0, "alter Kontostand vom 30.12.2025 100,00 H"),
        (40.0, "31.12. 31.12. Basislastschrift 30,00 S"),
        (116.0, "Testhaus Energie AG"),
        (116.0, "Abschlag Dezember"),
        (116.0, "Kunde 4711"),
        (40.0, "15.01. 15.01. Lohn/Gehalt 1.000,00 H"),
        (116.0, "Beispiel Arbeitgeber GmbH"),
        (278.0, "Übertrag auf Blatt 2 1.070,00 H"),
        (2.0, "9999"),
    ];
    let page_2 = [
        (125.0, "Beispielbank eG"),
        (336.0, "Kontoauszug Nr. 1/2026"),
        (40.0, "Bu-Tag Wert Vorgang"),
        (278.0, "Übertrag von Blatt 1 1.070,00 H"),
        (40.0, "20.01. 20.01. Überweisungsauftrag 1.120,50 S"),
        (116.0, "Muster Vermietung"),
        (116.0, "Miete Januar"),
        (234.0, "neuer Kontostand vom 31.01.2026 50,50 S"),
        (
            56.0,
            "Bitte prüfen Sie diesen Kontoauszug, 30,00 H gelten als genehmigt.",
        ),
    ];
    let mut lines: Vec<Line> = page_1
        .iter()
        .enumerate()
        .map(|(index, (left, text))| line(1, *left, text, index))
        .collect();
    lines.extend(
        page_2
            .iter()
            .enumerate()
            .map(|(index, (left, text))| line(2, *left, text, index)),
    );
    lines
}

#[test]
fn a_statement_of_layout_a_is_read_with_every_booking() {
    let statement = read(&sample()).expect("layout A");
    assert_eq!(statement.layout, "A");
    assert_eq!(statement.iban, "DE89370400440532013000");
    assert_eq!(statement.number.as_deref(), Some("1/2026"));
    assert_eq!(statement.bank.as_deref(), Some("Beispielbank eG"));
    assert_eq!(
        (statement.opening_day, statement.closing_day),
        (date(2025, 12, 30), date(2026, 1, 31))
    );
    assert_eq!(
        (statement.opening, statement.closing),
        (Cents::new(10_000), Cents::new(-5_050))
    );
    let days: Vec<Date> = statement
        .bookings
        .iter()
        .map(|booking| booking.day)
        .collect();
    assert_eq!(
        days,
        [date(2025, 12, 31), date(2026, 1, 15), date(2026, 1, 20)],
        "December is of the year before"
    );
    let amounts: Vec<i64> = statement
        .bookings
        .iter()
        .map(|booking| booking.amount.value())
        .collect();
    assert_eq!(amounts, [-3_000, 100_000, -112_050]);
    let first = &statement.bookings[0];
    assert_eq!(first.kind, "Basislastschrift");
    assert_eq!(first.other_side.as_deref(), Some("Testhaus Energie AG"));
    assert_eq!(first.purpose, "Abschlag Dezember Kunde 4711");
    assert_eq!(
        statement.bookings[2].line.page, 2,
        "the carry over the page is no booking"
    );
    assert_eq!(statement.difference(), Some(Cents::ZERO));
}

#[test]
fn the_first_statement_of_an_account_has_an_old_balance_without_a_day() {
    let mut lines = sample();
    for line in &mut lines {
        if line.text.starts_with("alter Kontostand") {
            line.text = "alter Kontostand 100,00 H".into();
        }
    }
    let statement = read(&lines).unwrap();
    assert_eq!(
        statement.opening_day,
        date(2025, 12, 30),
        "the day before the first booking"
    );
    assert_eq!(statement.difference(), Some(Cents::ZERO));
}

#[test]
fn another_layout_or_a_missing_balance_is_no_statement_of_layout_a() {
    let mut lines = sample();
    lines.retain(|line| !line.text.contains("neuer Kontostand"));
    assert_eq!(read(&lines), None);
    let other = vec![
        line(1, 60.0, "Umsatzübersicht 2/2026", 0),
        line(1, 60.0, "Saldo alt 100,00", 1),
    ];
    assert_eq!(read(&other), None);
}

#[test]
fn a_statement_that_does_not_add_up_shows_its_difference() {
    let mut lines = sample();
    for line in &mut lines {
        if line.text.starts_with("neuer Kontostand") {
            line.text = "neuer Kontostand vom 31.01.2026 49,50 S".into();
        }
    }
    let statement = read(&lines).unwrap();
    assert_eq!(statement.difference(), Some(Cents::new(-100)));
}

#[test]
fn storing_makes_the_account_once_and_replaces_an_earlier_reading() {
    let dir = tempfile::tempdir().unwrap();
    let opened = crate::archive::open(
        &dir.path().join(crate::archive::FILE_NAME),
        crate::testing::t0(),
    )
    .unwrap();
    let crate::archive::Opened::Ready(ready) = opened else {
        panic!("not ready")
    };
    let conn = ready.connection();
    conn.execute(
        "INSERT INTO documents (sha256, file_type, kind, original_name, size_bytes, source, imported_at) VALUES (?1, 'pdf', 'statement', 'a.pdf', 1, 'dialog', 'x')",
        [format!("{:064x}", 1)],
    )
    .unwrap();
    let document = conn.last_insert_rowid();
    store(conn, document, &sample(), crate::testing::t0())
        .unwrap()
        .unwrap();
    store(conn, document, &sample(), crate::testing::t0())
        .unwrap()
        .unwrap();
    let counts: (i64, i64, i64) = conn
        .query_row("SELECT (SELECT count(*) FROM accounts), (SELECT count(*) FROM statements), (SELECT count(*) FROM bookings)", [], |row| {
            Ok((row.get(0)?, row.get(1)?, row.get(2)?))
        })
        .unwrap();
    assert_eq!(counts, (1, 1, 3), "read again, never doubled");
    let name: String = conn
        .query_row("SELECT name FROM accounts", [], |row| row.get(0))
        .unwrap();
    assert_eq!(name, "Beispielbank …3000");
    assert_eq!(
        difference_of(conn, document)
            .unwrap()
            .map(|(_, _, _, count)| count),
        Some(3)
    );
    forget(conn, document).unwrap();
    assert_eq!(difference_of(conn, document).unwrap(), None);
}
