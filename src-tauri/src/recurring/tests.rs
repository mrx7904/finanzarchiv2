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
    crate::statements::account_of(
        conn,
        "DE89370400440532013000",
        Some("Beispielbank eG"),
        testing::t0(),
    )
    .unwrap()
}

fn party(conn: &Connection, name: &str) -> i64 {
    crate::parties::party_of(conn, name, None, testing::t0())
        .unwrap()
        .unwrap()
}

fn book(conn: &Connection, account: i64, party: i64, day: &str, cents: i64) -> i64 {
    conn.execute(
        "INSERT INTO bookings (account_id, position, day, amount_cents, kind, purpose, party_id) VALUES (?1, 0, ?2, ?3, 'Lastschrift', '', ?4)",
        params![account, day, cents, party],
    )
    .unwrap();
    conn.last_insert_rowid()
}

fn day(text: &str) -> Date {
    text.parse().unwrap()
}

fn today() -> Date {
    day("2026-11-25")
}

#[test]
fn a_monthly_payment_is_found_with_its_next_day_and_its_amount_in_the_year() {
    let dir = tempfile::tempdir().unwrap();
    let ready = ready(dir.path());
    let conn = ready.connection();
    let giro = account(conn);
    let music = party(conn, "Beispiel Musik");
    for month in 6..=11 {
        book(conn, giro, music, &format!("2026-{month:02}-03"), -1_099);
    }
    let found = found(conn, today()).unwrap();
    assert_eq!(found.len(), 1);
    let music = &found[0];
    assert_eq!(music.rhythm, Rhythm::Monthly);
    assert!(!music.incoming);
    assert_eq!(music.since, day("2026-06-03"));
    assert_eq!(music.last, day("2026-11-03"));
    assert_eq!(music.next, Some(day("2026-12-03")));
    assert_eq!(music.count, 6);
    assert_eq!(music.cents, -1_099);
    assert_eq!(music.per_year, -13_188);
    assert_eq!(music.change, None);
    assert!(!music.ended);
}

#[test]
fn quarterly_and_yearly_payments_keep_their_rhythm_a_few_days_late() {
    let dir = tempfile::tempdir().unwrap();
    let ready = ready(dir.path());
    let conn = ready.connection();
    let giro = account(conn);
    let radio = party(conn, "Beitragsservice");
    for day in ["2026-01-15", "2026-04-17", "2026-07-15", "2026-10-16"] {
        book(conn, giro, radio, day, -5_508);
    }
    let insurance = party(conn, "Beispiel Versicherung");
    for day in ["2024-02-01", "2025-02-03", "2026-02-02"] {
        book(conn, giro, insurance, day, -6_210);
    }
    let found = found(conn, today()).unwrap();
    let rhythms: Vec<(String, Rhythm)> = found.iter().map(|r| (r.name.clone(), r.rhythm)).collect();
    assert_eq!(
        rhythms,
        vec![
            ("Beitragsservice".to_owned(), Rhythm::Quarterly),
            ("Beispiel Versicherung".to_owned(), Rhythm::Yearly),
        ]
    );
    assert_eq!(found[0].per_year, -22_032);
    assert_eq!(found[1].next, Some(day("2027-02-02")));
}

#[test]
fn a_changed_amount_is_named_with_the_month_it_changed() {
    let dir = tempfile::tempdir().unwrap();
    let ready = ready(dir.path());
    let conn = ready.connection();
    let giro = account(conn);
    let gym = party(conn, "Beispiel Fitness");
    for (day, cents) in [
        ("2026-01-02", -1_999),
        ("2026-02-02", -1_999),
        ("2026-03-02", -2_199),
        ("2026-04-02", -2_199),
        ("2026-05-04", -2_199),
    ] {
        book(conn, giro, gym, day, cents);
    }
    let found = found(conn, day("2026-05-20")).unwrap();
    assert_eq!(found.len(), 1);
    assert_eq!(
        found[0].change,
        Some(Change {
            day: day("2026-03-02"),
            more_cents: 200
        })
    );
    assert_eq!(
        found[0].change_text().as_deref(),
        Some("seit März 2026 2,00 € mehr")
    );
    assert_eq!(found[0].cents, -2_199);
}

#[test]
fn a_bill_that_differs_every_month_names_no_change() {
    let dir = tempfile::tempdir().unwrap();
    let ready = ready(dir.path());
    let conn = ready.connection();
    let giro = account(conn);
    let phone = party(conn, "Beispiel Telefon");
    for (day, cents) in [
        ("2026-07-10", -4_495),
        ("2026-08-10", -4_612),
        ("2026-09-10", -4_503),
        ("2026-10-12", -4_580),
    ] {
        book(conn, giro, phone, day, cents);
    }
    let found = found(conn, today()).unwrap();
    assert_eq!(found.len(), 1);
    assert_eq!(found[0].change, None);
}

#[test]
fn one_that_stopped_is_marked_ended_and_stays() {
    let dir = tempfile::tempdir().unwrap();
    let ready = ready(dir.path());
    let conn = ready.connection();
    let giro = account(conn);
    let film = party(conn, "Beispiel Film");
    for month in 1..=5 {
        book(conn, giro, film, &format!("2026-{month:02}-20"), -1_399);
    }
    // The statements reach November: the film paid nothing since May.
    let shop = party(conn, "Beispiel Laden");
    book(conn, giro, shop, "2026-11-20", -2_500);
    let found = found(conn, today()).unwrap();
    assert_eq!(found.len(), 1);
    assert!(found[0].ended);
    assert_eq!(found[0].next, None);
}

#[test]
fn a_salary_is_found_as_money_in_and_shopping_is_not_recurring() {
    let dir = tempfile::tempdir().unwrap();
    let ready = ready(dir.path());
    let conn = ready.connection();
    let giro = account(conn);
    let employer = party(conn, "Beispiel Software GmbH");
    for month in 7..=10 {
        book(
            conn,
            giro,
            employer,
            &format!("2026-{month:02}-28"),
            385_000,
        );
    }
    let market = party(conn, "Beispiel Markt");
    for (day, cents) in [
        ("2026-09-02", -3_412),
        ("2026-09-06", -3_150),
        ("2026-09-13", -3_390),
        ("2026-09-20", -3_300),
        ("2026-09-27", -3_280),
        ("2026-10-04", -3_402),
    ] {
        book(conn, giro, market, day, cents);
    }
    let found = found(conn, today()).unwrap();
    assert_eq!(found.len(), 1);
    assert!(found[0].incoming);
    assert_eq!(found[0].name, "Beispiel Software");
}

#[test]
fn two_payments_of_one_party_with_different_amounts_are_two() {
    let dir = tempfile::tempdir().unwrap();
    let ready = ready(dir.path());
    let conn = ready.connection();
    let giro = account(conn);
    let insurer = party(conn, "Beispiel Versicherung");
    for month in 3..=8 {
        book(conn, giro, insurer, &format!("2026-{month:02}-01"), -2_350);
        book(conn, giro, insurer, &format!("2026-{month:02}-01"), -8_900);
    }
    let found = found(conn, day("2026-08-15")).unwrap();
    let amounts: Vec<i64> = found.iter().map(|r| r.cents).collect();
    assert_eq!(amounts, vec![-8_900, -2_350]);
}

#[test]
fn a_move_between_own_accounts_is_no_recurring_payment() {
    let dir = tempfile::tempdir().unwrap();
    let ready = ready(dir.path());
    let conn = ready.connection();
    let giro = account(conn);
    let savings =
        crate::statements::account_of(conn, "GB82WEST12345698765432", None, testing::t0()).unwrap();
    let own = party(conn, "Max Muster");
    for month in 6..=10 {
        let out = book(conn, giro, own, &format!("2026-{month:02}-01"), -20_000);
        let r#in = book(conn, savings, own, &format!("2026-{month:02}-01"), 20_000);
        conn.execute(
            "INSERT INTO booking_pairs (out_booking, in_booking, rule, inputs, decided_at) VALUES (?1, ?2, 'test', '{}', ?3)",
            params![out, r#in, testing::t0().to_string()],
        )
        .unwrap();
    }
    assert!(found(conn, today()).unwrap().is_empty());
}

#[test]
fn one_the_owner_said_is_none_is_not_found_again() {
    let dir = tempfile::tempdir().unwrap();
    let ready = ready(dir.path());
    let conn = ready.connection();
    let giro = account(conn);
    let friend = party(conn, "Erika Beispiel");
    for month in 6..=10 {
        book(conn, giro, friend, &format!("2026-{month:02}-15"), -5_000);
    }
    let found_now = found(conn, today()).unwrap();
    assert_eq!(found_now.len(), 1);
    let one = &found_now[0];
    reject(
        conn,
        one.party,
        one.incoming,
        one.rhythm,
        one.since,
        testing::t0(),
    )
    .unwrap();
    reject(
        conn,
        one.party,
        one.incoming,
        one.rhythm,
        one.since,
        testing::t0(),
    )
    .unwrap();
    book(conn, giro, friend, "2026-11-15", -5_000);
    assert!(found(conn, today()).unwrap().is_empty());
}

#[test]
fn ended_is_judged_by_the_last_statement_not_by_today() {
    let dir = tempfile::tempdir().unwrap();
    let ready = ready(dir.path());
    let conn = ready.connection();
    let giro = account(conn);
    let film = party(conn, "Beispiel Film");
    for month in 3..=7 {
        book(conn, giro, film, &format!("2026-{month:02}-20"), -1_399);
    }
    // The last statement read ends in July, today is November.
    let found = found(conn, today()).unwrap();
    assert_eq!(found.len(), 1);
    assert!(!found[0].ended);
    assert_eq!(found[0].next, Some(day("2026-08-20")));
}

#[test]
fn a_price_that_jumps_and_stays_is_one_payment_with_a_change() {
    let dir = tempfile::tempdir().unwrap();
    let ready = ready(dir.path());
    let conn = ready.connection();
    let giro = account(conn);
    let music = party(conn, "Beispiel Musik");
    for (day, cents) in [
        ("2026-01-05", -999),
        ("2026-02-05", -999),
        ("2026-03-05", -999),
        ("2026-04-06", -1_699),
        ("2026-05-05", -1_699),
        ("2026-06-05", -1_699),
    ] {
        book(conn, giro, music, day, cents);
    }
    let found = found(conn, day("2026-06-20")).unwrap();
    assert_eq!(found.len(), 1);
    assert_eq!(found[0].count, 6);
    assert_eq!(
        found[0].change_text().as_deref(),
        Some("seit April 2026 7,00 € mehr")
    );
}

#[test]
fn a_missed_month_keeps_the_payment_and_a_one_off_between_is_left_out() {
    let dir = tempfile::tempdir().unwrap();
    let ready = ready(dir.path());
    let conn = ready.connection();
    let giro = account(conn);
    let landlord = party(conn, "Beispiel Wohnbau");
    for day in [
        "2026-01-01",
        "2026-02-01",
        "2026-04-01",
        "2026-05-01",
        "2026-06-01",
    ] {
        book(conn, giro, landlord, day, -78_000);
    }
    // The bill of the heating costs, once.
    book(conn, giro, landlord, "2026-04-15", -31_250);
    let found = found(conn, day("2026-06-10")).unwrap();
    assert_eq!(found.len(), 1);
    assert_eq!(found[0].count, 5);
    assert_eq!(found[0].per_year, -936_000);
}

#[test]
fn purchases_at_a_shop_every_few_days_are_no_recurring_payment() {
    let dir = tempfile::tempdir().unwrap();
    let ready = ready(dir.path());
    let conn = ready.connection();
    let giro = account(conn);
    let market = party(conn, "Beispiel Markt");
    let mut day = jiff::civil::date(2026, 1, 3);
    for at in 0..40_i64 {
        book(
            conn,
            giro,
            market,
            &day.to_string(),
            -3_000 - (at * 37) % 900,
        );
        day = day.checked_add(jiff::Span::new().days(4 + at % 4)).unwrap();
    }
    assert!(
        found(conn, jiff::civil::date(2026, 7, 1))
            .unwrap()
            .is_empty()
    );
}
