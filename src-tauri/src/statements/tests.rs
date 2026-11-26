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

/// An invented CSV export in layout B of the same account as `sample`, newest first.
const CSV: &str = "Umsatzanzeige;Datei erstellt am: 01.02.2026 08:00

IBAN;DE89 3704 0044 0532 0130 00
Kontoname;Girokonto
Bank;Beispielbank
Kunde;Erika Musterfrau
Zeitraum;01.01.2026 - 31.01.2026
Saldo;1.050,00;EUR

Sortierung;Datum absteigend

Buchung;Valuta;Auftraggeber/Empfänger;Buchungstext;Verwendungszweck;Referenz;Saldo;Währung;Betrag;Währung
20.01.2026;20.01.2026;Muster Bäckerei;Lastschrift;Brot \"Vollkorn\";;1.050,00;EUR;-20,00;EUR
15.01.2026;15.01.2026;Beispiel Arbeitgeber GmbH;Gutschrift;-;;1.070,00;EUR;1.000,00;EUR
";

fn csv_lines(text: &str) -> Vec<Line> {
    layout_b::lines_of(text)
        .into_iter()
        .map(|line| Line {
            page: 1,
            text: line.text,
            left: f64::from(line.left),
            bottom: f64::from(line.bottom),
            right: f64::from(line.right),
            top: f64::from(line.top),
        })
        .collect()
}

#[test]
fn a_csv_of_layout_b_is_read_oldest_first_and_proved_by_its_balances() {
    assert!(layout_b::looks_like(CSV));
    assert!(
        !layout_b::looks_like("Name;Betrag\nMiete;12,00\n"),
        "any other CSV is refused"
    );
    let statement = read(&csv_lines(CSV)).expect("layout B");
    assert_eq!(statement.layout, "B");
    assert_eq!(statement.iban, "DE89370400440532013000");
    assert_eq!(statement.bank.as_deref(), Some("Beispielbank"));
    assert_eq!(
        (statement.opening_day, statement.closing_day),
        (date(2025, 12, 31), date(2026, 1, 31))
    );
    assert_eq!(
        (statement.opening, statement.closing),
        (Cents::new(7_000), Cents::new(105_000))
    );
    let days: Vec<Date> = statement
        .bookings
        .iter()
        .map(|booking| booking.day)
        .collect();
    assert_eq!(days, [date(2026, 1, 15), date(2026, 1, 20)]);
    assert_eq!(statement.bookings[0].purpose, "", "a dash is no purpose");
    assert_eq!(statement.bookings[1].purpose, "Brot \"Vollkorn\"");
    assert_eq!(
        statement.bookings[1].other_side.as_deref(),
        Some("Muster Bäckerei")
    );
    assert_eq!(statement.difference(), Some(Cents::ZERO));
}

#[test]
fn a_csv_in_windows_1252_is_decoded() {
    let bytes: Vec<u8> =
        b"Umsatzanzeige;x\nIBAN;DE00\nBuchung;Valuta;Auftraggeber/Empf\xe4nger;Saldo;Betrag\n"
            .to_vec();
    let text = layout_b::decode(&bytes);
    assert!(text.contains("Empfänger"));
    assert!(layout_b::looks_like(&text));
    assert_eq!(
        layout_b::decode(b"\xEF\xBB\xBFUmsatz"),
        "Umsatz",
        "a BOM is dropped"
    );
    assert_eq!(
        layout_b::cells("a;\"b;c\";\"d \"\"e\"\"\""),
        ["a", "b;c", "d \"e\""]
    );
}

#[test]
fn the_same_booking_of_a_csv_and_a_pdf_counts_once_the_csv_first() {
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
    let mut documents = Vec::new();
    for (number, file_type) in [(1, "pdf"), (2, "csv")] {
        conn.execute(
            "INSERT INTO documents (sha256, file_type, kind, original_name, size_bytes, source, imported_at) VALUES (?1, ?2, 'statement', 'a', 1, 'dialog', 'x')",
            params![format!("{number:064x}"), file_type],
        )
        .unwrap();
        documents.push(conn.last_insert_rowid());
    }
    store(conn, documents[0], &sample(), crate::testing::t0())
        .unwrap()
        .unwrap();
    store(conn, documents[1], &csv_lines(CSV), crate::testing::t0())
        .unwrap()
        .unwrap();
    let counting = |conn: &Connection| -> (i64, i64) {
        conn.query_row(
            "SELECT count(*) FILTER (WHERE replaced_by IS NULL), count(*) FILTER (WHERE replaced_by IS NOT NULL) FROM bookings",
            [],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .unwrap()
    };
    assert_eq!(
        counting(conn),
        (4, 1),
        "3 of the PDF and 2 of the CSV, the salary once"
    );
    let replaced_layout: String = conn
        .query_row("SELECT s.layout FROM bookings b JOIN statements s ON s.id = b.statement_id WHERE b.replaced_by IS NOT NULL", [], |row| row.get(0))
        .unwrap();
    assert_eq!(replaced_layout, "A", "the PDF's booking gives way");
    store(conn, documents[1], &csv_lines(CSV), crate::testing::t0())
        .unwrap()
        .unwrap();
    assert_eq!(
        counting(conn),
        (4, 1),
        "read again, joined again, never doubled"
    );
    forget(conn, documents[1]).unwrap();
    assert_eq!(
        counting(conn),
        (3, 0),
        "without the CSV the PDF's booking counts again"
    );
}

#[test]
fn a_statement_read_again_keeps_its_bookings_and_what_hangs_on_them() {
    let dir = tempfile::tempdir().unwrap();
    let crate::archive::Opened::Ready(ready) = crate::archive::open(
        &dir.path().join(crate::archive::FILE_NAME),
        crate::testing::t0(),
    )
    .unwrap() else {
        panic!("not ready")
    };
    let conn = ready.connection();
    let document_of = |number: u8, kind: &str| {
        conn.execute(
            "INSERT INTO documents (sha256, file_type, kind, original_name, size_bytes, source, imported_at) VALUES (?1, 'pdf', ?2, 'a.pdf', 1, 'dialog', 'x')",
            rusqlite::params![format!("{number:064x}"), kind],
        )
        .unwrap();
        conn.last_insert_rowid()
    };
    let document = document_of(1, "statement");
    let invoice = document_of(2, "invoice");
    store(conn, document, &sample(), crate::testing::t0())
        .unwrap()
        .unwrap();
    let ids = |conn: &Connection| -> Vec<i64> {
        conn.prepare("SELECT id FROM bookings ORDER BY id")
            .unwrap()
            .query_map([], |row| row.get(0))
            .unwrap()
            .collect::<rusqlite::Result<_>>()
            .unwrap()
    };
    let before = ids(conn);
    conn.execute(
        "INSERT INTO links (document_id, booking_id, cents, rule, inputs, decided_at) VALUES (?1, ?2, -100, 'link.owner:1', '{}', 'x')",
        rusqlite::params![invoice, before[0]],
    )
    .unwrap();
    conn.execute(
        "INSERT INTO corrections (kind, booking_id, made_at) VALUES ('booking-category', ?1, 'x')",
        [before[1]],
    )
    .unwrap();
    // A reader of another kind changed: read again, nothing of it changes.
    store(conn, document, &sample(), crate::testing::t0())
        .unwrap()
        .unwrap();
    assert_eq!(ids(conn), before, "the same bookings");
    let links: i64 = conn
        .query_row("SELECT count(*) FROM links", [], |row| row.get(0))
        .unwrap();
    assert_eq!(links, 1, "the link stays");

    // When the statement reads otherwise, what hung on its bookings goes before they do.
    let shorter: Vec<Line> = sample()
        .into_iter()
        .filter(|line| !line.text.contains("Miete"))
        .collect();
    let _ = store(conn, document, &shorter, crate::testing::t0()).unwrap();
    forget(conn, document).unwrap();
    let left: (i64, i64) = conn
        .query_row("SELECT (SELECT count(*) FROM links), (SELECT count(*) FROM corrections WHERE booking_id IS NOT NULL)", [], |row| Ok((row.get(0)?, row.get(1)?)))
        .unwrap();
    assert_eq!(left, (0, 0));
    let broken: i64 = conn
        .query_row("SELECT count(*) FROM pragma_foreign_key_check", [], |row| {
            row.get(0)
        })
        .unwrap();
    assert_eq!(broken, 0);
}
