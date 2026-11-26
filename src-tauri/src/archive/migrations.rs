//! The schema as numbered SQL files, embedded in the binary and applied in order.
//! A file is never edited after it was committed, a change is a new file.

use rusqlite::{Connection, OptionalExtension};

/// Every migration, numbered from 1 without gaps. `PRAGMA user_version` holds the last applied one.
pub const MIGRATIONS: &[(u32, &str)] = &[
    (1, include_str!("../../migrations/0001_settings.sql")),
    (2, include_str!("../../migrations/0002_documents.sql")),
    (3, include_str!("../../migrations/0003_intake_jobs.sql")),
    (4, include_str!("../../migrations/0004_pages.sql")),
    (5, include_str!("../../migrations/0005_kind_signals.sql")),
    (6, include_str!("../../migrations/0006_field_values.sql")),
    (7, include_str!("../../migrations/0007_e_invoices.sql")),
    (8, include_str!("../../migrations/0008_model_stage.sql")),
    (9, include_str!("../../migrations/0009_decisions.sql")),
    (10, include_str!("../../migrations/0010_names.sql")),
    (
        11,
        include_str!("../../migrations/0011_accounts_and_bookings.sql"),
    ),
    (12, include_str!("../../migrations/0012_csv_statements.sql")),
    (
        13,
        include_str!("../../migrations/0013_parties_and_categories.sql"),
    ),
    (14, include_str!("../../migrations/0014_moves.sql")),
    (15, include_str!("../../migrations/0015_links.sql")),
    (
        16,
        include_str!("../../migrations/0016_watched_folders.sql"),
    ),
    (17, include_str!("../../migrations/0017_downloads.sql")),
    (18, include_str!("../../migrations/0018_mailboxes.sql")),
    (19, include_str!("../../migrations/0019_mail_messages.sql")),
    (20, include_str!("../../migrations/0020_mail_text.sql")),
];

#[derive(Debug, thiserror::Error)]
#[error("Migration {number} schlug fehl")]
pub struct MigrationError {
    pub number: u32,
    #[source]
    pub source: rusqlite::Error,
}

pub fn applied(conn: &Connection) -> rusqlite::Result<u32> {
    conn.query_row("PRAGMA user_version", [], |row| row.get(0))
}

pub fn pending(conn: &Connection) -> rusqlite::Result<Vec<(u32, &'static str)>> {
    let current = applied(conn)?;
    Ok(MIGRATIONS
        .iter()
        .copied()
        .filter(|(number, _)| *number > current)
        .collect())
}

/// Applies each pending migration in a transaction of its own, so a failed one leaves nothing half done.
/// The foreign keys are off while the migrations run, so a migration can build a table anew that
/// others refer to (as SQLite recommends for a changed CHECK), and every migration must leave them
/// whole before it commits (`PRAGMA foreign_key_check`).
pub fn apply(conn: &mut Connection, migrations: &[(u32, &str)]) -> Result<(), MigrationError> {
    let first = migrations.first().map_or(0, |(number, _)| *number);
    conn.pragma_update(None, "foreign_keys", "OFF")
        .map_err(|source| MigrationError {
            number: first,
            source,
        })?;
    let result = (|| {
        for &(number, sql) in migrations {
            let run = |conn: &mut Connection| -> rusqlite::Result<()> {
                let tx = conn.transaction()?;
                tx.execute_batch(sql)?;
                let broken: Option<String> = tx
                    .query_row("PRAGMA foreign_key_check", [], |row| row.get(0))
                    .optional()?;
                if let Some(table) = broken {
                    return Err(rusqlite::Error::SqliteFailure(
                        rusqlite::ffi::Error::new(rusqlite::ffi::SQLITE_CONSTRAINT_FOREIGNKEY),
                        Some(format!("foreign key broken in {table}")),
                    ));
                }
                tx.pragma_update(None, "user_version", number)?;
                tx.commit()
            };
            run(conn).map_err(|source| MigrationError { number, source })?;
        }
        Ok(())
    })();
    conn.pragma_update(None, "foreign_keys", "ON")
        .map_err(|source| MigrationError {
            number: first,
            source,
        })?;
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn migrations_are_numbered_from_one_without_gaps() {
        for (index, (number, _)) in MIGRATIONS.iter().enumerate() {
            assert_eq!(*number as usize, index + 1);
        }
    }

    #[test]
    fn a_failing_migration_leaves_the_schema_as_it_was() {
        let mut conn = Connection::open_in_memory().unwrap();
        let broken = [(
            1,
            "CREATE TABLE a (x INTEGER) STRICT; CREATE TABLE broken (;",
        )];
        let error = apply(&mut conn, &broken).unwrap_err();
        assert_eq!(error.number, 1);
        assert_eq!(applied(&conn).unwrap(), 0);
        let tables: i64 = conn
            .query_row(
                "SELECT count(*) FROM sqlite_schema WHERE type = 'table'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(tables, 0);
    }
}
