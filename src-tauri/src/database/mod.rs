use rusqlite::{Connection, Result};
use std::path::Path;

pub mod migrations;
pub mod repository;
pub mod schema;

pub struct DatabaseState {
    pub db_path: std::path::PathBuf,
}

impl DatabaseState {
    pub fn new(db_path: std::path::PathBuf) -> Self {
        Self { db_path }
    }
}

pub fn init(db_path: &Path) -> Result<()> {
    let conn = Connection::open(db_path)?;
    
    // Run migrations
    migrations::run_all(&conn)?;
    
    // Seed data if needed
    if is_empty(&conn)? {
        repository::seed_data(&conn)?;
    }
    
    Ok(())
}

fn is_empty(conn: &Connection) -> Result<bool> {
    let count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM cards",
        [],
        |row| row.get(0),
    )?;
    Ok(count == 0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    
    #[test]
    fn test_database_initialization() {
        let temp_file = NamedTempFile::new().unwrap();
        let db_path = temp_file.path();
        
        init(db_path).expect("Database initialization failed");
        
        // Verify tables exist
        let conn = Connection::open(db_path).unwrap();
        let tables = [
            "cards",
            "synergies",
            "context_modifiers",
            "champion_overrides",
            "deck_history",
        ];
        
        for table in &tables {
            let count: i64 = conn
                .query_row(
                    "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name=?1",
                    [table],
                    |row| row.get(0),
                )
                .unwrap();
            assert_eq!(count, 1, "Table {} should exist", table);
        }
    }
}
