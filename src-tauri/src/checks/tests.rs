use super::*;
use crate::archive::{self, Opened};
use crate::testing;
use proptest::prelude::*;

fn ready(dir: &std::path::Path) -> archive::Archive {
    match archive::open(&dir.join(archive::FILE_NAME), testing::t0()).unwrap() {
        Opened::Ready(ready) => ready,
        other => panic!("{other:?}"),
    }
}

/// A document row of the given kind, without a file: the checks read only the database.
fn document(conn: &Connection, number: u8, kind: Kind) -> i64 {
    conn.execute(
        "INSERT INTO documents (sha256, file_type, kind, original_name, size_bytes, source, imported_at, text_state)
         VALUES (?1, 'pdf', ?2, ?3, 1, 'dialog', ?4, 'text')",
        params![
            format!("{number:064x}"),
            kind.code(),
            format!("d{number}.pdf"),
            testing::t0().to_string()
        ],
    )
    .unwrap();
    conn.last_insert_rowid()
}

/// A value as a reader stored it. `line` false gives a value read from the page image.
fn value(
    conn: &Connection,
    document: i64,
    reader: &str,
    field: &str,
    printed: &str,
    value: &str,
    line: bool,
) {
    conn.execute(
        "INSERT INTO field_values (document_id, field, printed, value, reader, page, top, read_at) VALUES (?1, ?2, ?3, ?4, ?5, 1, ?6, 'x')",
        params![document, field, printed, value, reader, line.then_some(700.0)],
    )
    .unwrap();
}

/// An invoice that passes every check: code and the model agree, the sum holds.
fn good_invoice(conn: &Connection, number: u8, invoice_number: &str) -> i64 {
    let id = document(conn, number, Kind::Invoice);
    for reader in ["code:3", "model:beispiel"] {
        value(conn, id, reader, "gross", "11,90", "1190", true);
    }
    value(conn, id, "code:3", "net", "10,00", "1000", true);
    value(conn, id, "code:3", "vat", "1,90", "190", true);
    value(conn, id, "code:3", "vat_rate", "19 %", "19", true);
    value(
        conn,
        id,
        "code:3",
        "invoice_number",
        invoice_number,
        invoice_number,
        true,
    );
    value(
        conn,
        id,
        "model:beispiel",
        "sender",
        "Beispiel Werkstatt",
        "Beispiel Werkstatt",
        true,
    );
    id
}

fn failed(verdict: &Verdict) -> Vec<&'static str> {
    verdict
        .outcomes
        .iter()
        .filter(|outcome| !outcome.passed)
        .map(|outcome| outcome.check)
        .collect()
}

#[test]
fn an_invoice_that_passes_every_check_is_filed_with_its_decision() {
    let dir = tempfile::tempdir().unwrap();
    let ready = ready(dir.path());
    let conn = ready.connection();
    let id = good_invoice(conn, 1, "R-1");
    let verdict = decide(conn, id, testing::t0()).unwrap();
    assert_eq!(failed(&verdict), Vec::<&str>::new());
    assert_eq!(verdict.rule, Some(RULE_CHECKED));
    let (state, rule, inputs): (String, String, String) = conn
        .query_row(
            "SELECT d.state, x.rule, x.inputs FROM documents d JOIN decisions x ON x.document_id = d.id WHERE d.id = ?1",
            [id],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        )
        .unwrap();
    assert_eq!((state.as_str(), rule.as_str()), ("filed", RULE_CHECKED));
    assert!(inputs.contains("\"gross\""), "the values it saw");
    let shown = shown(conn, id).unwrap();
    assert!(shown.outcomes.iter().all(|(_, _, passed, _)| *passed));
    assert!(
        shown
            .outcomes
            .iter()
            .any(|(_, _, _, detail)| detail.as_deref()
                == Some("Den Bruttobetrag lesen der Code und das Modell gleich."))
    );
}

#[test]
fn undo_lets_the_document_wait_and_it_is_not_filed_again() {
    let dir = tempfile::tempdir().unwrap();
    let ready = ready(dir.path());
    let conn = ready.connection();
    let id = good_invoice(conn, 1, "R-1");
    decide(conn, id, testing::t0()).unwrap();
    assert!(undo(conn, id, testing::t0()).unwrap());
    assert!(
        !undo(conn, id, testing::t0()).unwrap(),
        "nothing left to undo"
    );
    let state: String = conn
        .query_row("SELECT state FROM documents WHERE id = ?1", [id], |row| {
            row.get(0)
        })
        .unwrap();
    assert_eq!(state, "waiting");
    let again = decide(conn, id, testing::t0()).unwrap();
    assert_eq!(failed(&again), [UNDONE]);
    assert_eq!(shown(conn, id).unwrap().rule, None);
}

#[test]
fn a_statement_is_filed_without_checks_and_an_unclear_kind_waits() {
    let dir = tempfile::tempdir().unwrap();
    let ready = ready(dir.path());
    let conn = ready.connection();
    let statement = document(conn, 1, Kind::Statement);
    assert_eq!(
        decide(conn, statement, testing::t0()).unwrap().rule,
        Some(RULE_STATEMENT)
    );
    let other = document(conn, 2, Kind::Other);
    let verdict = decide(conn, other, testing::t0()).unwrap();
    assert_eq!(failed(&verdict), [KIND]);
}

#[test]
fn every_check_can_fail() {
    let dir = tempfile::tempdir().unwrap();
    let ready = ready(dir.path());
    let conn = ready.connection();

    let missing = document(conn, 1, Kind::Invoice);
    assert!(failed(&judge(conn, missing).unwrap()).contains(&PRESENT));

    let alone = document(conn, 2, Kind::Invoice);
    value(conn, alone, "code:3", "gross", "11,90", "1190", true);
    assert!(
        failed(&judge(conn, alone).unwrap()).contains(&AGREE),
        "one reader is not enough"
    );

    let differ = good_invoice(conn, 3, "R-3");
    value(
        conn,
        differ,
        "image:beispiel",
        "gross",
        "17,90",
        "1790",
        false,
    );
    assert!(failed(&judge(conn, differ).unwrap()).contains(&AGREE));

    let image = document(conn, 4, Kind::Invoice);
    for reader in ["image:beispiel", "image:other"] {
        value(conn, image, reader, "gross", "11,90", "1190", false);
    }
    assert!(failed(&judge(conn, image).unwrap()).contains(&SOURCE));

    let sum = good_invoice(conn, 5, "R-5");
    conn.execute(
        "UPDATE field_values SET value = '200' WHERE document_id = ?1 AND field = 'vat'",
        [sum],
    )
    .unwrap();
    assert!(failed(&judge(conn, sum).unwrap()).contains(&SUM));

    let first = good_invoice(conn, 6, "R-6");
    let second = good_invoice(conn, 7, "R-6");
    assert!(failed(&judge(conn, second).unwrap()).contains(&DUPLICATE));
    assert!(
        failed(&judge(conn, first).unwrap()).contains(&DUPLICATE),
        "both wait, the owner decides"
    );

    let payslip = document(conn, 8, Kind::Payslip);
    for reader in ["code:3", "model:beispiel"] {
        value(conn, payslip, reader, "brutto", "4.000,00", "400000", true);
        value(conn, payslip, reader, "netto", "2.600,00", "260000", true);
    }
    value(
        conn,
        payslip,
        "code:3",
        "lohnsteuer",
        "520,00",
        "52000",
        true,
    );
    value(
        conn,
        payslip,
        "code:3",
        "employer",
        "Muster AG",
        "Muster AG",
        true,
    );
    value(
        conn,
        payslip,
        "code:3",
        "period",
        "März 2026",
        "2026-03",
        true,
    );
    assert_eq!(failed(&judge(conn, payslip).unwrap()), [SUM]);
}

proptest! {
    #[test]
    fn an_invoice_adds_up_exactly_when_net_plus_vat_is_gross(net in 0_i64..10_000_000, rate in prop::sample::select(vec![0_u32, 7, 19]), off in -3_i64..=3) {
        let vat = (net * i64::from(rate) + 50) / 100;
        let gross = Cents::new(net + vat + off);
        let outcome = invoice_sum(Some(Cents::new(net)), Some(Cents::new(vat)), gross, Some(rate));
        prop_assert_eq!(outcome.passed, off == 0, "{}", outcome.detail);
    }

    #[test]
    fn a_vat_off_by_more_than_a_cent_fails_the_rate(net in 100_i64..10_000_000, off in 2_i64..1000) {
        let vat = (net * 19 + 50) / 100 + off;
        let outcome = invoice_sum(Some(Cents::new(net)), Some(Cents::new(vat)), Cents::new(net + vat), Some(19));
        prop_assert!(!outcome.passed);
    }

    #[test]
    fn a_payslip_adds_up_exactly_when_brutto_minus_deductions_is_netto(
        brutto in 0_i64..100_000_000,
        deductions in prop::collection::vec(0_i64..1_000_000, 0..7),
        off in -2_i64..=2,
    ) {
        let total: i64 = deductions.iter().sum();
        let netto = Cents::new(brutto - total + off);
        let cents: Vec<Cents> = deductions.into_iter().map(Cents::new).collect();
        prop_assert_eq!(payslip_sum(Cents::new(brutto), &cents, netto).passed, off == 0);
    }
}
