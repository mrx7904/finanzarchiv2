//! The schema as numbered SQL files, embedded in the binary and applied in order.
//! A file is never edited after it was committed, a change is a new file.

use rusqlite::Connection;

/// Every migration, numbered from 1 without gaps. `PRAGMA user_version` holds the last applied one.
pub const MIGRATIONS: &[(u32, &str)] = &[
    (1, include_str!("../../migrations/0001_settings.sql")),
    (2, include_str!("../../migrations/0002_documents.sql")),
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
pub fn apply(conn: &mut Connection, migrations: &[(u32, &str)]) -> Result<(), MigrationError> {
    for &(number, sql) in migrations {
        let run = |conn: &mut Connection| -> rusqlite::Result<()> {
            let tx = conn.transaction()?;
            tx.execute_batch(sql)?;
            tx.pragma_update(None, "user_version", number)?;
            tx.commit()
        };
        run(conn).map_err(|source| MigrationError { number, source })?;
    }
    Ok(())
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
