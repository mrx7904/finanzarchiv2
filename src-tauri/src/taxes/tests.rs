use super::*;
use crate::archive::{self, Opened};
use crate::testing;

fn ready(dir: &std::path::Path) -> archive::Archive {
    match archive::open(&dir.join(archive::FILE_NAME), testing::t0()).unwrap() {
        Opened::Ready(ready) => ready,
        other => panic!("{other:?}"),
    }
}

fn invoice(conn: &Connection, number: u8, gross: i64, date: &str) -> i64 {
    conn.execute(
        "INSERT INTO documents (sha256, file_type, kind, original_name, size_bytes, source, imported_at, state)
         VALUES (?1, 'pdf', 'invoice', 'r.pdf', 1, 'dialog', 'x', 'filed')",
        [format!("{number:064x}")],
    )
    .unwrap();
    let id = conn.last_insert_rowid();
    for (field, value) in [
        ("gross", gross.to_string()),
        ("invoice_date", date.to_owned()),
    ] {
        conn.execute(
            "INSERT INTO field_values (document_id, field, printed, value, reader, page, top, read_at) VALUES (?1, ?2, ?3, ?3, 'code:5', 1, 700.0, 'x')",
            params![id, field, value],
        )
        .unwrap();
    }
    id
}

fn booking(conn: &Connection, day: &str, cents: i64) -> i64 {
    let account =
        crate::statements::account_of(conn, "DE89370400440532013000", None, testing::t0()).unwrap();
    conn.execute(
        "INSERT INTO bookings (account_id, position, day, amount_cents, kind, purpose) VALUES (?1, 0, ?2, ?3, 'Lastschrift', '')",
        params![account, day, cents],
    )
    .unwrap();
    conn.last_insert_rowid()
}

fn link(conn: &Connection, document: i64, booking: i64, cents: i64) {
    conn.execute(
        "INSERT INTO links (document_id, booking_id, cents, rule, inputs, decided_at) VALUES (?1, ?2, ?3, 'test', '{}', 'x')",
        params![document, booking, cents],
    )
    .unwrap();
}

fn assignment(category: &str) -> Assignment {
    Assignment {
        category: category.to_owned(),
        share_percent: 100,
        refunded_cents: 0,
        labour_cents: None,
    }
}

#[test]
fn what_counts_is_the_amount_less_the_refund_times_the_share() {
    let mut laptop = assignment("arbeitsmittel");
    laptop.share_percent = 60;
    assert_eq!(counted(89_999, &laptop), 53_999);
    // The owner's study: the employer pays two thirds tax free, only the own third counts.
    let mut study = assignment("fortbildung");
    study.refunded_cents = 200_000;
    assert_eq!(counted(300_000, &study), 100_000);
    let mut craftsman = assignment("handwerker");
    assert_eq!(counted(50_000, &craftsman), 0);
    craftsman.labour_cents = Some(18_000);
    assert_eq!(counted(50_000, &craftsman), 18_000);
    // Never below zero.
    let mut refunded = assignment("krankheit");
    refunded.refunded_cents = 9_000;
    assert_eq!(counted(5_000, &refunded), 0);
}

#[test]
fn the_tax_year_is_the_year_paid_else_the_invoice_date_with_a_note() {
    let dir = tempfile::tempdir().unwrap();
    let ready = ready(dir.path());
    let conn = ready.connection();
    let paid_later = invoice(conn, 1, 120_000, "2025-12-20");
    let payment = booking(conn, "2026-01-08", -120_000);
    link(conn, paid_later, payment, 120_000);
    let unpaid = invoice(conn, 2, 4_990, "2025-11-03");
    assign(
        conn,
        On::Document(paid_later),
        &assignment("fortbildung"),
        testing::t0(),
    )
    .unwrap();
    assign(
        conn,
        On::Document(unpaid),
        &assignment("fachliteratur"),
        testing::t0(),
    )
    .unwrap();
    let this = year(conn, 2026).unwrap();
    assert_eq!(this.sections.len(), 1);
    assert_eq!(this.sections[0].category.key, "fortbildung");
    assert_eq!(
        this.sections[0].entries[0].day,
        jiff::civil::date(2026, 1, 8)
    );
    assert!(this.sections[0].entries[0].notes.is_empty());
    let before = year(conn, 2025).unwrap();
    assert_eq!(
        before.sections[0].entries[0].notes,
        vec![Note::YearFromInvoice]
    );
    assert_eq!(years(conn).unwrap(), vec![2026, 2025]);
}

#[test]
fn a_payment_is_assigned_once_on_its_document_or_its_booking() {
    let dir = tempfile::tempdir().unwrap();
    let ready = ready(dir.path());
    let conn = ready.connection();
    let document = invoice(conn, 1, 8_900, "2026-02-01");
    let payment = booking(conn, "2026-02-03", -8_900);
    link(conn, document, payment, 8_900);
    assign(
        conn,
        On::Document(document),
        &assignment("arbeitsmittel"),
        testing::t0(),
    )
    .unwrap();
    assert!(matches!(
        assign(
            conn,
            On::Booking(payment),
            &assignment("arbeitsmittel"),
            testing::t0()
        ),
        Err(TaxError::ThroughDocument)
    ));
    // Changing the assignment of the document is no second one.
    let mut half = assignment("arbeitsmittel");
    half.share_percent = 50;
    assign(conn, On::Document(document), &half, testing::t0()).unwrap();
    let this = year(conn, 2026).unwrap();
    assert_eq!(this.sections[0].entries.len(), 1);
    assert_eq!(this.sections[0].counted, 4_450);
    unassign(conn, On::Document(document)).unwrap();
    assign(
        conn,
        On::Booking(payment),
        &assignment("arbeitsmittel"),
        testing::t0(),
    )
    .unwrap();
    assert!(matches!(
        assign(
            conn,
            On::Document(document),
            &assignment("arbeitsmittel"),
            testing::t0()
        ),
        Err(TaxError::ThroughBooking)
    ));
    assert!(matches!(
        assign(
            conn,
            On::Booking(payment),
            &assignment("nichts"),
            testing::t0()
        ),
        Err(TaxError::UnknownCategory)
    ));
}

#[test]
fn the_lump_sums_of_a_year_count_with_the_documents_against_the_pauschbetrag() {
    let dir = tempfile::tempdir().unwrap();
    let ready = ready(dir.path());
    let conn = ready.connection();
    let book = invoice(conn, 1, 8_990, "2026-03-01");
    assign(
        conn,
        On::Document(book),
        &assignment("fachliteratur"),
        testing::t0(),
    )
    .unwrap();
    set_profile(
        conn,
        2026,
        &Profile {
            commute_km: 18,
            commute_days: 150,
            own_car: true,
            employer_commute_cents: 12_000,
            homeoffice_days: 60,
        },
        testing::t0(),
    )
    .unwrap();
    let this = year(conn, 2026).unwrap();
    // 18 km × 150 days × 0,38 € less 120 € support, as checked by hand in 1.0.
    assert_eq!(this.entfernungspauschale, Some(90_600));
    assert_eq!(this.homeoffice, Some(36_000));
    assert_eq!(this.werbungskosten, 135_590);
    assert_eq!(this.above_pauschbetrag, Some(12_590));
}

#[test]
fn the_entfernungspauschale_of_2025_has_two_rates_and_without_a_car_a_cap() {
    let values = year_values(2025).unwrap();
    let far = Profile {
        commute_km: 30,
        commute_days: 200,
        own_car: true,
        ..Profile::default()
    };
    assert_eq!(
        entfernungspauschale(&far, &values),
        (20 * 30 + 10 * 38) * 200
    );
    let by_train = Profile {
        commute_km: 90,
        commute_days: 220,
        own_car: false,
        ..Profile::default()
    };
    assert_eq!(entfernungspauschale(&by_train, &values), 450_000);
    let home = Profile {
        homeoffice_days: 230,
        ..Profile::default()
    };
    assert_eq!(homeoffice(&home, &values), 126_000);
}

#[test]
fn a_year_without_values_sums_without_lump_sums() {
    let dir = tempfile::tempdir().unwrap();
    let ready = ready(dir.path());
    let conn = ready.connection();
    let fee = invoice(conn, 1, 50_000, "2020-04-01");
    assign(
        conn,
        On::Document(fee),
        &assignment("fortbildung"),
        testing::t0(),
    )
    .unwrap();
    let old = year(conn, 2020).unwrap();
    assert_eq!(old.values, None);
    assert_eq!(old.entfernungspauschale, None);
    assert_eq!(old.above_pauschbetrag, None);
    assert_eq!(old.werbungskosten, 50_000);
    assert_eq!(year_values(2023).unwrap().taken_from, Some(2025));
}

#[test]
fn the_reductions_of_35a_take_their_rate_and_cap_and_a_donation_above_300_needs_a_receipt() {
    let dir = tempfile::tempdir().unwrap();
    let ready = ready(dir.path());
    let conn = ready.connection();
    let cleaning = booking(conn, "2026-05-02", -60_000);
    assign(
        conn,
        On::Booking(cleaning),
        &assignment("haushaltsnah"),
        testing::t0(),
    )
    .unwrap();
    let painter = invoice(conn, 1, 900_000, "2026-06-10");
    let mut labour = assignment("handwerker");
    labour.labour_cents = Some(800_000);
    assign(conn, On::Document(painter), &labour, testing::t0()).unwrap();
    let roofer = invoice(conn, 2, 30_000, "2026-07-10");
    assign(
        conn,
        On::Document(roofer),
        &assignment("handwerker"),
        testing::t0(),
    )
    .unwrap();
    let small = booking(conn, "2026-12-01", -5_000);
    let large = booking(conn, "2026-12-02", -50_000);
    assign(
        conn,
        On::Booking(small),
        &assignment("spende"),
        testing::t0(),
    )
    .unwrap();
    assign(
        conn,
        On::Booking(large),
        &assignment("spende"),
        testing::t0(),
    )
    .unwrap();
    let this = year(conn, 2026).unwrap();
    assert_eq!(this.haushalt_reduction, Some(12_000));
    // 20 % of 8.000 € is 1.600 €, capped at 1.200 €.
    assert_eq!(this.handwerker_reduction, Some(120_000));
    let craftsmen = this
        .sections
        .iter()
        .find(|s| s.category.key == "handwerker")
        .unwrap();
    assert_eq!(
        craftsmen.entries[1].notes,
        vec![Note::YearFromInvoice, Note::NoLabour]
    );
    let donations = this
        .sections
        .iter()
        .find(|s| s.category.key == "spende")
        .unwrap();
    let notes: Vec<&Vec<Note>> = donations.entries.iter().map(|e| &e.notes).collect();
    assert_eq!(notes, vec![&vec![], &vec![Note::NeedsReceipt]]);
}

#[test]
fn erstausbildung_is_capped_and_every_category_names_form_and_section_without_line_numbers() {
    let dir = tempfile::tempdir().unwrap();
    let ready = ready(dir.path());
    let conn = ready.connection();
    let fee = booking(conn, "2026-04-01", -700_000);
    assign(
        conn,
        On::Booking(fee),
        &assignment("erstausbildung"),
        testing::t0(),
    )
    .unwrap();
    assert_eq!(year(conn, 2026).unwrap().erstausbildung, 600_000);
    for category in CATEGORIES {
        assert!(
            category.elster.contains(" · ") || category.elster.starts_with("Anlage"),
            "{}",
            category.key
        );
        assert!(!category.elster.contains("Zeile"), "{}", category.key);
    }
}
