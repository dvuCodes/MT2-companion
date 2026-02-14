use crate::database::{repository::CardData, DatabaseState};
use rusqlite::{Connection, Result as SqliteResult};
use serde::{Deserialize, Serialize};
use tauri::State;

/// Log helper for card commands
fn log_command(command: &str, details: &str) {
    log::info!("[Cards] {}: {}", command, details);
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CardResponse {
    pub id: String,
    pub name: String,
    pub clan: String,
    pub card_type: String,
    pub rarity: String,
    pub cost: Option<i32>,
    pub base_value: i32,
    pub tempo_score: i32,
    pub value_score: i32,
    pub keywords: Vec<String>,
    pub description: String,
    pub expansion: String,
}

impl From<CardData> for CardResponse {
    fn from(card: CardData) -> Self {
        Self {
            id: card.id,
            name: card.name,
            clan: card.clan,
            card_type: card.card_type,
            rarity: card.rarity,
            cost: card.cost,
            base_value: card.base_value,
            tempo_score: card.tempo_score,
            value_score: card.value_score,
            keywords: card.keywords,
            description: card.description,
            expansion: card.expansion,
        }
    }
}

/// Custom error type for card-related operations
#[derive(Debug)]
pub enum CardError {
    DatabaseError(String),
    CardNotFound(String),
    InvalidQuery(String),
}

impl std::fmt::Display for CardError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CardError::DatabaseError(msg) => write!(f, "Database error: {}", msg),
            CardError::CardNotFound(name) => write!(f, "Card '{}' not found", name),
            CardError::InvalidQuery(msg) => write!(f, "Invalid query: {}", msg),
        }
    }
}

impl From<rusqlite::Error> for CardError {
    fn from(err: rusqlite::Error) -> Self {
        CardError::DatabaseError(err.to_string())
    }
}

/// Maps a database row to a CardData struct
fn row_to_card_data(row: &rusqlite::Row) -> SqliteResult<CardData> {
    let keywords_json: String = row.get(9)?;
    let keywords: Vec<String> = serde_json::from_str(&keywords_json).unwrap_or_default();

    Ok(CardData {
        id: row.get(0)?,
        name: row.get(1)?,
        clan: row.get(2)?,
        card_type: row.get(3)?,
        rarity: row.get(4)?,
        cost: row.get(5)?,
        base_value: row.get(6)?,
        tempo_score: row.get(7)?,
        value_score: row.get(8)?,
        keywords,
        description: row.get(10)?,
        expansion: row.get(11)?,
    })
}

/// Query to select all card columns
const SELECT_CARD_SQL: &str = r#"
    SELECT 
        id, name, clan, card_type, rarity, cost,
        base_value, tempo_score, value_score, keywords,
        description, expansion
    FROM cards
"#;

/// Get a single card by exact name match
#[tauri::command]
pub fn get_card_by_name(
    name: String,
    state: State<DatabaseState>,
) -> Result<Option<CardResponse>, String> {
    if name.trim().is_empty() {
        return Err("Card name cannot be empty".to_string());
    }

    let conn = Connection::open(&state.db_path).map_err(|e| e.to_string())?;

    let mut stmt = conn
        .prepare(&format!("{} WHERE name = ?1", SELECT_CARD_SQL))
        .map_err(|e| e.to_string())?;

    let card_result = stmt
        .query_row([&name], row_to_card_data)
        .map_err(|e| match e {
            rusqlite::Error::QueryReturnedNoRows => CardError::CardNotFound(name.clone()),
            _ => CardError::DatabaseError(e.to_string()),
        });

    match card_result {
        Ok(card) => Ok(Some(card.into())),
        Err(CardError::CardNotFound(_)) => Ok(None),
        Err(e) => Err(e.to_string()),
    }
}

/// Get all cards for a specific clan
#[tauri::command]
pub fn get_cards_by_clan(
    clan: String,
    state: State<DatabaseState>,
) -> Result<Vec<CardResponse>, String> {
    if clan.trim().is_empty() {
        return Err("Clan name cannot be empty".to_string());
    }

    let conn = Connection::open(&state.db_path).map_err(|e| e.to_string())?;

    let mut stmt = conn
        .prepare(&format!("{} WHERE clan = ?1 ORDER BY name", SELECT_CARD_SQL))
        .map_err(|e| e.to_string())?;

    let cards: Result<Vec<CardData>, _> = stmt
        .query_map([&clan], row_to_card_data)
        .map_err(|e| e.to_string())?
        .collect();

    cards
        .map(|cards| cards.into_iter().map(Into::into).collect())
        .map_err(|e| format!("Failed to fetch cards: {}", e))
}

/// Search cards by partial name match (case-insensitive)
#[tauri::command]
pub fn search_cards(
    query: String,
    state: State<DatabaseState>,
) -> Result<Vec<CardResponse>, String> {
    if query.trim().is_empty() {
        return Ok(vec![]);
    }

    let conn = Connection::open(&state.db_path).map_err(|e| e.to_string())?;

    // Use LIKE for case-insensitive partial matching
    let search_pattern = format!("%{}%", query.trim());

    let mut stmt = conn
        .prepare(&format!(
            "{} WHERE name LIKE ?1 ORDER BY name LIMIT 50",
            SELECT_CARD_SQL
        ))
        .map_err(|e| e.to_string())?;

    let cards: Result<Vec<CardData>, _> = stmt
        .query_map([&search_pattern], row_to_card_data)
        .map_err(|e| e.to_string())?
        .collect();

    cards
        .map(|cards| cards.into_iter().map(Into::into).collect())
        .map_err(|e| format!("Failed to search cards: {}", e))
}

/// Get all cards from the database
#[tauri::command]
pub fn get_all_cards(state: State<DatabaseState>) -> Result<Vec<CardResponse>, String> {
    let conn = Connection::open(&state.db_path).map_err(|e| e.to_string())?;

    let mut stmt = conn
        .prepare(&format!("{} ORDER BY clan, name", SELECT_CARD_SQL))
        .map_err(|e| e.to_string())?;

    let cards: Result<Vec<CardData>, _> = stmt
        .query_map([], row_to_card_data)
        .map_err(|e| e.to_string())?
        .collect();

    cards
        .map(|cards| cards.into_iter().map(Into::into).collect())
        .map_err(|e| format!("Failed to fetch all cards: {}", e))
}

/// Helper function to get a card by name directly from a connection (for testing)
fn get_card_by_name_direct(conn: &Connection, name: &str) -> Result<Option<CardData>, CardError> {
    let mut stmt = conn
        .prepare(&format!("{} WHERE name = ?1", SELECT_CARD_SQL))?;

    let card_result = stmt
        .query_row([name], row_to_card_data)
        .map_err(|e| match e {
            rusqlite::Error::QueryReturnedNoRows => CardError::CardNotFound(name.to_string()),
            _ => CardError::DatabaseError(e.to_string()),
        });

    match card_result {
        Ok(card) => Ok(Some(card)),
        Err(CardError::CardNotFound(_)) => Ok(None),
        Err(e) => Err(e),
    }
}

/// Helper function to get cards by clan directly from a connection (for testing)
fn get_cards_by_clan_direct(conn: &Connection, clan: &str) -> Result<Vec<CardData>, CardError> {
    let mut stmt = conn
        .prepare(&format!("{} WHERE clan = ?1 ORDER BY name", SELECT_CARD_SQL))?;

    let cards: Result<Vec<CardData>, _> = stmt
        .query_map([clan], row_to_card_data)?
        .collect();

    cards.map_err(|e| CardError::DatabaseError(e.to_string()))
}

/// Helper function to search cards directly from a connection (for testing)
fn search_cards_direct(conn: &Connection, query: &str) -> Result<Vec<CardData>, CardError> {
    if query.trim().is_empty() {
        return Ok(vec![]);
    }

    let search_pattern = format!("%{}%", query.trim());

    let mut stmt = conn
        .prepare(&format!(
            "{} WHERE name LIKE ?1 ORDER BY name LIMIT 50",
            SELECT_CARD_SQL
        ))?;

    let cards: Result<Vec<CardData>, _> = stmt
        .query_map([&search_pattern], row_to_card_data)?
        .collect();

    cards.map_err(|e| CardError::DatabaseError(e.to_string()))
}

/// Helper function to get all cards directly from a connection (for testing)
fn get_all_cards_direct(conn: &Connection) -> Result<Vec<CardData>, CardError> {
    let mut stmt = conn
        .prepare(&format!("{} ORDER BY clan, name", SELECT_CARD_SQL))?;

    let cards: Result<Vec<CardData>, _> = stmt
        .query_map([], row_to_card_data)?
        .collect();

    cards.map_err(|e| CardError::DatabaseError(e.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::database;
    use tempfile::NamedTempFile;

    fn setup_test_db() -> (DatabaseState, NamedTempFile) {
        let temp_file = NamedTempFile::new().unwrap();
        let db_path = temp_file.path().to_path_buf();

        // Initialize database with schema and seed data
        database::init(&db_path).unwrap();

        (DatabaseState::new(db_path), temp_file)
    }

    #[test]
    fn test_get_card_by_name_found() {
        let (state, _temp) = setup_test_db();
        let conn = Connection::open(&state.db_path).unwrap();

        // Test with a known seeded card
        let result = get_card_by_name_direct(&conn, "Fel");
        assert!(result.is_ok());

        let card = result.unwrap();
        assert!(card.is_some());
        assert_eq!(card.unwrap().name, "Fel");
    }

    #[test]
    fn test_get_card_by_name_not_found() {
        let (state, _temp) = setup_test_db();
        let conn = Connection::open(&state.db_path).unwrap();

        let result = get_card_by_name_direct(&conn, "NonExistentCard");
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    #[test]
    fn test_get_card_by_name_empty() {
        let (state, _temp) = setup_test_db();
        let conn = Connection::open(&state.db_path).unwrap();

        // Direct helper returns empty result for empty string (not an error)
        // The command wrapper handles the validation
        let result = get_card_by_name_direct(&conn, "");
        assert!(result.is_ok());
        // Empty string won't match any card name
        assert!(result.unwrap().is_none());
    }

    #[test]
    fn test_get_cards_by_clan() {
        let (state, _temp) = setup_test_db();
        let conn = Connection::open(&state.db_path).unwrap();

        let result = get_cards_by_clan_direct(&conn, "Banished");
        assert!(result.is_ok());

        let cards = result.unwrap();
        assert!(!cards.is_empty());
        assert!(cards.iter().all(|c| c.clan == "Banished"));
    }

    #[test]
    fn test_get_cards_by_clan_empty() {
        let (state, _temp) = setup_test_db();
        let conn = Connection::open(&state.db_path).unwrap();

        // Direct helper returns empty vec for empty clan
        let result = get_cards_by_clan_direct(&conn, "");
        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }

    #[test]
    fn test_search_cards() {
        let (state, _temp) = setup_test_db();
        let conn = Connection::open(&state.db_path).unwrap();

        // Search for "Fel" should find "Fel" and potentially others
        let result = search_cards_direct(&conn, "Fel");
        assert!(result.is_ok());

        let cards = result.unwrap();
        assert!(!cards.is_empty());
        assert!(cards.iter().any(|c| c.name.contains("Fel")));
    }

    #[test]
    fn test_search_cards_empty_query() {
        let (state, _temp) = setup_test_db();
        let conn = Connection::open(&state.db_path).unwrap();

        let result = search_cards_direct(&conn, "");
        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }

    #[test]
    fn test_search_cards_case_insensitive() {
        let (state, _temp) = setup_test_db();
        let conn = Connection::open(&state.db_path).unwrap();

        // Search with lowercase
        let result = search_cards_direct(&conn, "fel");
        assert!(result.is_ok());

        let cards = result.unwrap();
        assert!(!cards.is_empty());
        assert!(cards.iter().any(|c| c.name.to_lowercase().contains("fel")));
    }

    #[test]
    fn test_search_cards_partial_match() {
        let (state, _temp) = setup_test_db();
        let conn = Connection::open(&state.db_path).unwrap();

        // Search for partial match
        let result = search_cards_direct(&conn, "ust");
        assert!(result.is_ok());

        let cards = result.unwrap();
        // Should find "Just Cause" (contains "ust")
        assert!(cards.iter().any(|c| c.name.contains("Just")));
    }

    #[test]
    fn test_get_all_cards() {
        let (state, _temp) = setup_test_db();
        let conn = Connection::open(&state.db_path).unwrap();

        let result = get_all_cards_direct(&conn);
        assert!(result.is_ok());

        let cards = result.unwrap();
        assert!(!cards.is_empty());

        // Verify ordering (by clan, then name)
        for i in 1..cards.len() {
            let prev = &cards[i - 1];
            let curr = &cards[i];
            assert!(
                prev.clan <= curr.clan,
                "Cards should be sorted by clan"
            );
            if prev.clan == curr.clan {
                assert!(
                    prev.name <= curr.name,
                    "Cards in same clan should be sorted by name"
                );
            }
        }
    }
}
