use super::*;
use crate::archive;
use crate::readers::model::tests::Scripted;
use crate::testing;

fn opened(dir: &std::path::Path) -> Mutex<Opened> {
    Mutex::new(archive::open(&dir.join(archive::FILE_NAME), testing::t0()).unwrap())
}

fn with_conn<T>(archive: &Mutex<Opened>, run: impl FnOnce(&Connection) -> T) -> T {
    let guard = archive.lock().unwrap();
    let Opened::Ready(ready) = &*guard else {
        panic!("not ready")
    };
    run(ready.connection())
}

fn party_with_bookings(
    conn: &Connection,
    name: &str,
    cents: i64,
    days: &[&str],
    purpose: &str,
) -> i64 {
    let party = crate::parties::party_of(conn, name, None, testing::t0())
        .unwrap()
        .unwrap();
    let account = crate::statements::account_of(
        conn,
        "DE89370400440532013000",
        Some("Beispielbank eG"),
        testing::t0(),
    )
    .unwrap();
    for day in days {
        conn.execute(
            "INSERT INTO bookings (account_id, position, day, amount_cents, kind, purpose, party_id) VALUES (?1, 0, ?2, ?3, 'Lastschrift', ?4, ?5)",
            params![account, day, cents, purpose, party],
        )
        .unwrap();
    }
    party
}

fn answer(name: &str) -> Result<String, ModelError> {
    Ok(format!(r#"{{"category":"{name}"}}"#))
}

#[test]
fn the_model_learns_name_bookings_rhythm_and_purposes() {
    let dir = tempfile::tempdir().unwrap();
    let archive = opened(dir.path());
    with_conn(&archive, |conn| {
        let party = party_with_bookings(
            conn,
            "Beispiel Musik AB",
            -999,
            &["2026-01-05", "2026-02-05", "2026-03-05", "2026-04-05"],
            "Beispiel Musik   Premium   Abo",
        );
        let context = context_of(conn, party).unwrap();
        assert_eq!((context.outgoing, context.incoming), (4, 0));
        assert_eq!(context.typical, Some((Cents::new(-999), 4)));
        assert_eq!(context.rhythm, Some("monatlich"));
        assert_eq!(context.purposes, ["Beispiel Musik Premium Abo"]);
        let text = prompt(&context, &categories(conn).unwrap());
        assert!(text.contains("Partei: Beispiel Musik"), "{text}");
        assert!(text.contains("4 Ausgänge, 0 Eingänge"));
        assert!(text.contains("monatlich"));
        assert!(text.contains("- Abos und Medien (aus): Spotify"));
        assert!(text.contains("- Gehalt und Lohn (ein)"));
    });
}

#[test]
fn every_party_is_asked_once_and_the_owner_choice_stays() {
    let dir = tempfile::tempdir().unwrap();
    let archive = opened(dir.path());
    let (music, owned, unclear) = with_conn(&archive, |conn| {
        let music = party_with_bookings(conn, "Beispiel Musik AB", -999, &["2026-01-05"], "Abo");
        let owned = party_with_bookings(conn, "Beispiel Laden", -2_000, &["2026-01-06"], "Einkauf");
        let unclear = party_with_bookings(conn, "Max Muster", -5_000, &["2026-01-07"], "Danke");
        // The owner chose none for the shop: that stays.
        let booking: i64 = conn
            .query_row(
                "SELECT id FROM bookings WHERE party_id = ?1",
                [owned],
                |row| row.get(0),
            )
            .unwrap();
        crate::parties::choose_category(conn, booking, None, true, testing::t0()).unwrap();
        (music, owned, unclear)
    });
    let model = Scripted::new(vec![answer("Abos und Medien"), answer("keine")]);
    assert_eq!(
        pass(&archive, &model, 40),
        Some(2),
        "the owner's party is not asked"
    );
    assert_eq!(pass(&archive, &model, 40), Some(0), "once");
    with_conn(&archive, |conn| {
        let category = |party: i64| -> (Option<String>, Option<String>) {
            conn.query_row(
                "SELECT c.name, p.category_by FROM parties p LEFT JOIN categories c ON c.id = p.category_id WHERE p.id = ?1",
                [party],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .unwrap()
        };
        assert_eq!(
            category(music),
            (Some("Abos und Medien".into()), Some("model".into()))
        );
        assert_eq!(
            category(unclear),
            (None, None),
            "none fits: none, asked once"
        );
        assert_eq!(category(owned), (None, Some("owner".into())));
        assert!(by_model(conn, music).unwrap());
        // The owner changes the model's choice: it is the owner's now and stays.
        let booking: i64 = conn
            .query_row(
                "SELECT id FROM bookings WHERE party_id = ?1",
                [music],
                |row| row.get(0),
            )
            .unwrap();
        let other: i64 = conn
            .query_row(
                "SELECT id FROM categories WHERE name = 'Freizeit und Hobby'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        crate::parties::choose_category(conn, booking, Some(other), true, testing::t0()).unwrap();
        keep(conn, music, None, testing::t0()).unwrap();
        assert_eq!(
            category(music),
            (Some("Freizeit und Hobby".into()), Some("owner".into()))
        );
    });
}

#[test]
fn without_an_answer_the_parties_wait() {
    let dir = tempfile::tempdir().unwrap();
    let archive = opened(dir.path());
    with_conn(&archive, |conn| {
        party_with_bookings(conn, "Beispiel Musik AB", -999, &["2026-01-05"], "Abo");
    });
    let away = Scripted::new(vec![Err(ModelError::Unreachable("127.0.0.1:11434".into()))]);
    assert_eq!(pass(&archive, &away, 40), None);
    let model = Scripted::new(vec![answer("Abos und Medien")]);
    assert_eq!(pass(&archive, &model, 40), Some(1), "asked again later");
}

#[test]
fn a_booking_of_paypal_belongs_to_its_payee() {
    let purpose = "1043123456789/PP.1234.PP/. Beispiel Musik AB, Ihr Einkauf bei Beispiel Musik AB EREF: 1043 MREF: ABC CRED: DE99 IBAN: DE99 BIC: X";
    assert_eq!(
        crate::parties::payee_of("PayPal Europe S.a.r.l. et Cie S.C.A", purpose).as_deref(),
        Some("Beispiel Musik AB")
    );
    assert_eq!(
        crate::parties::payee_of(
            "PayPal Europe",
            "PP.1234.PP . , Ihr Einkauf bei Beispiel Kino EREF: 1"
        )
        .as_deref(),
        Some("Beispiel Kino"),
        "the name only after the words"
    );
    assert_eq!(
        crate::parties::payee_of("Beispiel Laden", purpose),
        None,
        "only a payment service"
    );
    assert_eq!(
        crate::parties::payee_of("PayPal Europe", "Rückzahlung"),
        None
    );

    let dir = tempfile::tempdir().unwrap();
    let archive = opened(dir.path());
    with_conn(&archive, |conn| {
        let account = crate::statements::account_of(
            conn,
            "DE89370400440532013000",
            Some("Beispielbank eG"),
            testing::t0(),
        )
        .unwrap();
        conn.execute(
            "INSERT INTO bookings (account_id, position, day, amount_cents, kind, other_side, purpose) VALUES (?1, 0, '2026-03-02', -999, 'Lastschrift', 'PayPal Europe S.a.r.l. et Cie S.C.A', ?2)",
            params![account, purpose],
        )
        .unwrap();
        crate::parties::assign(conn, testing::t0()).unwrap();
        let party: String = conn
            .query_row(
                "SELECT p.name FROM bookings b JOIN parties p ON p.id = b.party_id",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(party, "Beispiel Musik AB", "the payee");
        // A booking given PayPal as its party before is moved to its payee once.
        let paypal = crate::parties::party_of(
            conn,
            "PayPal Europe S.a.r.l. et Cie S.C.A",
            None,
            testing::t0(),
        )
        .unwrap()
        .unwrap();
        conn.execute("UPDATE bookings SET party_id = ?1", [paypal])
            .unwrap();
        assert_eq!(crate::parties::payees(conn, testing::t0()).unwrap(), 1);
        assert_eq!(crate::parties::payees(conn, testing::t0()).unwrap(), 0);
    });
}
