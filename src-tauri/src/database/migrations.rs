use crate::database::schema;
use rusqlite::{Connection, Result};

const CURRENT_VERSION: i32 = 1;

pub fn run_all(conn: &Connection) -> Result<()> {
    // Create migrations table if not exists
    conn.execute(
        "CREATE TABLE IF NOT EXISTS schema_migrations (
            version INTEGER PRIMARY KEY,
            applied_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
        )",
        [],
    )?;

    // Get current version
    let current: i32 = conn
        .query_row(
            "SELECT COALESCE(MAX(version), 0) FROM schema_migrations",
            [],
            |row| row.get(0),
        )
        .unwrap_or(0);

    // Run pending migrations
    if current < 1 {
        migration_001_initial_schema(conn)?;
        mark_applied(conn, 1)?;
    }

    Ok(())
}

fn mark_applied(conn: &Connection, version: i32) -> Result<()> {
    conn.execute(
        "INSERT INTO schema_migrations (version) VALUES (?1)",
        [version],
    )?;
    Ok(())
}

fn migration_001_initial_schema(conn: &Connection) -> Result<()> {
    conn.execute(schema::CREATE_CARDS_TABLE, [])?;
    conn.execute(schema::CREATE_SYNERGIES_TABLE, [])?;
    conn.execute(schema::CREATE_CONTEXT_MODIFIERS_TABLE, [])?;
    conn.execute(schema::CREATE_CHAMPION_OVERRIDES_TABLE, [])?;
    conn.execute(schema::CREATE_DECK_HISTORY_TABLE, [])?;
    conn.execute(schema::CREATE_EXPANSIONS_TABLE, [])?;
    Ok(())
}
