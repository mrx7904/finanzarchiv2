use super::*;
use crate::archive::{self, Opened};
use crate::testing;

fn ready(dir: &std::path::Path) -> archive::Archive {
    match archive::open(&dir.join(archive::FILE_NAME), testing::t0()).unwrap() {
        Opened::Ready(ready) => ready,
        other => panic!("{other:?}"),
    }
}

/// Bookings of one invented account with the given other sides and purposes.
fn bookings(conn: &Connection, sides: &[(&str, &str)]) -> Vec<i64> {
    let account = crate::statements::account_of(
        conn,
        "DE89370400440532013000",
        Some("Beispielbank eG"),
        testing::t0(),
    )
    .unwrap();
    sides
        .iter()
        .enumerate()
        .map(|(position, (other, purpose))| {
            conn.execute(
                "INSERT INTO bookings (account_id, position, day, amount_cents, kind, other_side, purpose) VALUES (?1, ?2, '2026-01-15', -1000, 'Lastschrift', ?3, ?4)",
                params![account, position as i64, other, purpose],
            )
            .unwrap();
            conn.last_insert_rowid()
        })
        .collect()
}

fn party(conn: &Connection, booking: i64) -> Option<i64> {
    conn.query_row(
        "SELECT party_id FROM bookings WHERE id = ?1",
        [booking],
        |row| row.get(0),
    )
    .unwrap()
}

#[test]
fn the_categories_start_with_a_list_of_everyday_ones_once() {
    let dir = tempfile::tempdir().unwrap();
    let ready = ready(dir.path());
    let conn = ready.connection();
    let count: i64 = conn
        .query_row("SELECT count(*) FROM categories", [], |row| row.get(0))
        .unwrap();
    assert!(
        (20..=30).contains(&count),
        "{count} categories, the owner asked for 20 to 30"
    );
    assert_eq!(seed_categories(conn).unwrap(), 0, "never twice");
    let added = add_category(conn, "  Haustiere  ", "out").unwrap();
    assert_eq!(
        add_category(conn, "haustiere", "out"),
        Err(CategoryError::Taken("haustiere".into()))
    );
    assert_eq!(add_category(conn, " ", "out"), Err(CategoryError::Empty));
    rename_category(conn, added, "Tiere").unwrap();
    let name: String = conn
        .query_row(
            "SELECT name FROM categories WHERE id = ?1",
            [added],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(name, "Tiere");
    remove_category(conn, added).unwrap();
}

#[test]
fn spellings_of_one_party_are_one_party_and_an_iban_joins_a_new_spelling() {
    let dir = tempfile::tempdir().unwrap();
    let ready = ready(dir.path());
    let conn = ready.connection();
    let ids = bookings(
        conn,
        &[
            (
                "Testhaus Energie AG",
                "Abschlag IBAN: DE89 3704 0044 0532 0130 00",
            ),
            ("TESTHAUS ENERGIE", "Abschlag"),
            ("Testhaus-Energie AG,", "Abschlag"),
            ("Stadtwerke Muster", "Neukunde IBAN: DE89370400440532013000"),
            ("Muster Bäckerei", "Brot"),
        ],
    );
    assert_eq!(assign(conn, testing::t0()).unwrap(), 5);
    assert_eq!(assign(conn, testing::t0()).unwrap(), 0, "never twice");
    let first = party(conn, ids[0]);
    assert_eq!(party(conn, ids[1]), first, "case and legal form");
    assert_eq!(party(conn, ids[2]), first, "signs and spaces");
    assert_eq!(party(conn, ids[3]), first, "a new spelling of a known IBAN");
    assert_ne!(party(conn, ids[4]), first);
    let name: String = conn
        .query_row(
            "SELECT name FROM parties WHERE id = ?1",
            [first.unwrap()],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(name, "Testhaus Energie");
}

#[test]
fn a_booking_takes_its_partys_category_unless_the_owner_gave_it_another() {
    let dir = tempfile::tempdir().unwrap();
    let ready = ready(dir.path());
    let conn = ready.connection();
    let ids = bookings(
        conn,
        &[
            ("Muster Bäckerei", "Brot"),
            ("Muster Bäckerei", "Kuchen"),
            ("Muster Bäckerei", "Geschenk"),
        ],
    );
    assign(conn, testing::t0()).unwrap();
    let food: i64 = conn
        .query_row(
            "SELECT id FROM categories WHERE name = 'Lebensmittel'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    let gifts: i64 = conn
        .query_row(
            "SELECT id FROM categories WHERE name = 'Geschenke und Spenden'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(category_of(conn, ids[0]).unwrap(), None);
    choose_category(conn, ids[0], Some(food), true, testing::t0()).unwrap();
    assert_eq!(
        category_of(conn, ids[1]).unwrap(),
        Some(food),
        "für alle von Muster Bäckerei"
    );
    choose_category(conn, ids[2], Some(gifts), false, testing::t0()).unwrap();
    assert_eq!(
        category_of(conn, ids[2]).unwrap(),
        Some(gifts),
        "the owner's choice for one booking stays"
    );
    assert_eq!(category_of(conn, ids[1]).unwrap(), Some(food));
    let corrections: i64 = conn
        .query_row("SELECT count(*) FROM corrections", [], |row| row.get(0))
        .unwrap();
    assert_eq!(corrections, 2, "each choice is a correction the rule reads");
    remove_category(conn, food).unwrap();
    assert_eq!(category_of(conn, ids[1]).unwrap(), None);
}

#[test]
fn merging_and_splitting_parties_can_be_undone() {
    let dir = tempfile::tempdir().unwrap();
    let ready = ready(dir.path());
    let conn = ready.connection();
    let ids = bookings(
        conn,
        &[
            ("Muster Bäckerei", "Brot"),
            ("Bäckerei Muster Filiale 2", "Brot"),
            ("Muster Bäckerei", "Kuchen"),
        ],
    );
    assign(conn, testing::t0()).unwrap();
    let (a, b) = (party(conn, ids[0]).unwrap(), party(conn, ids[1]).unwrap());
    assert_ne!(a, b);
    let merged = merge(conn, b, a, testing::t0()).unwrap();
    assert_eq!(party(conn, ids[1]), Some(a));
    assert!(undo(conn, merged, testing::t0()).unwrap());
    assert!(
        !undo(conn, merged, testing::t0()).unwrap(),
        "nothing left to undo"
    );
    assert_eq!(party(conn, ids[1]), Some(b));

    let alias: i64 = conn
        .query_row(
            "SELECT id FROM party_aliases WHERE party_id = ?1",
            [a],
            |row| row.get(0),
        )
        .unwrap();
    let split_off = split(conn, alias, testing::t0()).unwrap();
    let new = party(conn, ids[0]).unwrap();
    assert_ne!(new, a);
    assert_eq!(
        party(conn, ids[2]),
        Some(new),
        "every booking of that spelling"
    );
    assert!(undo(conn, split_off, testing::t0()).unwrap());
    assert_eq!(party(conn, ids[0]), Some(a));
}

#[test]
fn folding_leaves_out_what_does_not_name_the_party() {
    assert_eq!(fold("Beispiel Stadtwerke GmbH"), "beispielstadtwerke");
    assert_eq!(fold("BEISPIEL-STADTWERKE"), "beispielstadtwerke");
    assert_eq!(fold("  "), "");
}
