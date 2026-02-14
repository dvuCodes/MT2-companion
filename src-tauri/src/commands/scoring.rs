use crate::database::repository::CardData;
use crate::database::DatabaseState;
use crate::scoring::{
    calculator::{ScoreCalculator, ScoringResult},
    context::ContextModifier,
    synergies::Synergy,
};
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use tauri::State;

#[derive(Serialize, Deserialize, Debug)]
pub struct DraftScoreRequest {
    pub card_id: String,
    pub current_deck: Vec<String>,
    pub champion: String,
    pub ring_number: i32,
    pub covenant: i32,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct DraftScoreResponse {
    pub score: i32,
    pub tier: String,
    pub reasons: Vec<String>,
}

impl From<ScoringResult> for DraftScoreResponse {
    fn from(result: ScoringResult) -> Self {
        Self {
            score: result.score,
            tier: result.tier,
            reasons: result.reasons,
        }
    }
}

/// Error type for scoring operations
#[derive(Debug)]
pub enum ScoringError {
    DatabaseError(String),
    CardNotFound(String),
    InvalidInput(String),
}

impl std::fmt::Display for ScoringError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ScoringError::DatabaseError(msg) => write!(f, "Database error: {}", msg),
            ScoringError::CardNotFound(id) => write!(f, "Card with ID '{}' not found", id),
            ScoringError::InvalidInput(msg) => write!(f, "Invalid input: {}", msg),
        }
    }
}

impl From<rusqlite::Error> for ScoringError {
    fn from(err: rusqlite::Error) -> Self {
        ScoringError::DatabaseError(err.to_string())
    }
}

/// Query a card by its ID from the database
fn get_card_by_id(conn: &Connection, card_id: &str) -> Result<Option<CardData>, ScoringError> {
    let mut stmt = conn.prepare(
        r#"
        SELECT 
            id, name, clan, card_type, rarity, cost,
            base_value, tempo_score, value_score, keywords,
            description, expansion
        FROM cards
        WHERE id = ?1
        "#
    )?;

    let card_result = stmt.query_row([card_id], |row| {
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
    });

    match card_result {
        Ok(card) => Ok(Some(card)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(e.into()),
    }
}

/// Get multiple cards by their IDs
fn get_cards_by_ids(conn: &Connection, card_ids: &[String]) -> Result<Vec<CardData>, ScoringError> {
    if card_ids.is_empty() {
        return Ok(vec![]);
    }

    // Build parameterized query with placeholders
    let placeholders: Vec<String> = card_ids.iter().map(|_| "?".to_string()).collect();
    let sql = format!(
        r#"
        SELECT 
            id, name, clan, card_type, rarity, cost,
            base_value, tempo_score, value_score, keywords,
            description, expansion
        FROM cards
        WHERE id IN ({})
        "#,
        placeholders.join(", ")
    );

    let mut stmt = conn.prepare(&sql)?;

    let cards: Result<Vec<CardData>, rusqlite::Error> = stmt
        .query_map(rusqlite::params_from_iter(card_ids.iter()), |row| {
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
        })?
        .collect();

    cards.map_err(|e| e.into())
}

/// Get all synergies for a specific card
fn get_synergies_for_card(
    conn: &Connection,
    card_id: &str,
) -> Result<Vec<Synergy>, ScoringError> {
    let mut stmt = conn.prepare(
        r#"
        SELECT 
            card_a_id, card_b_id, synergy_type, weight, description, bidirectional
        FROM synergies
        WHERE card_a_id = ?1 
           OR (card_b_id = ?1 AND bidirectional = 1)
           OR card_b_id = '*'
        "#
    )?;

    let synergies: Result<Vec<Synergy>, rusqlite::Error> = stmt
        .query_map([card_id], |row| {
            let bidirectional: bool = row.get(5)?;

            Ok(Synergy {
                card_a_id: row.get(0)?,
                card_b_id: row.get(1)?,
                synergy_type: row.get(2)?,
                weight: row.get(3)?,
                description: row.get(4)?,
                bidirectional,
            })
        })?
        .collect();

    synergies.map_err(|e| e.into())
}

/// Get all active context modifiers
fn get_active_context_modifiers(conn: &Connection) -> Result<Vec<ContextModifier>, ScoringError> {
    let mut stmt = conn.prepare(
        r#"
        SELECT condition, card_tag, modifier, priority, description
        FROM context_modifiers
        WHERE active = 1
        "#
    )?;

    let modifiers: Result<Vec<ContextModifier>, rusqlite::Error> = stmt
        .query_map([], |row| {
            Ok(ContextModifier {
                condition: row.get(0)?,
                card_tag: row.get(1)?,
                modifier: row.get(2)?,
                priority: row.get(3)?,
                description: row.get(4)?,
            })
        })?
        .collect();

    modifiers.map_err(|e| e.into())
}

/// Get champion override value for a specific card and champion
fn get_champion_override(
    conn: &Connection,
    card_id: &str,
    champion: &str,
    _path: Option<&str>, // Path can be used for more specific matching
) -> Result<Option<i32>, ScoringError> {
    // First try exact champion match
    let mut stmt = conn.prepare(
        r#"
        SELECT value_override
        FROM champion_overrides
        WHERE card_id = ?1 AND champion = ?2
        ORDER BY 
            CASE path
                WHEN 'Any' THEN 0
                ELSE 1
            END
        LIMIT 1
        "#
    )?;

    let result = stmt.query_row([card_id, champion], |row| {
        let value: i32 = row.get(0)?;
        Ok(value)
    });

    match result {
        Ok(value) => Ok(Some(value)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(e.into()),
    }
}

/// Calculate draft score with real database data
#[tauri::command]
pub fn calculate_draft_score(
    request: DraftScoreRequest,
    state: State<DatabaseState>,
) -> Result<DraftScoreResponse, String> {
    // Validate input
    if request.card_id.trim().is_empty() {
        return Err("Card ID cannot be empty".to_string());
    }
    if request.champion.trim().is_empty() {
        return Err("Champion cannot be empty".to_string());
    }
    if request.ring_number < 1 || request.ring_number > 10 {
        return Err("Ring number must be between 1 and 10".to_string());
    }
    if request.covenant < 1 || request.covenant > 25 {
        return Err("Covenant must be between 1 and 25".to_string());
    }

    let conn = Connection::open(&state.db_path).map_err(|e| e.to_string())?;

    // 1. Query the card being evaluated
    let card = get_card_by_id(&conn, &request.card_id)
        .map_err(|e| e.to_string())?
        .ok_or_else(|| format!("Card '{}' not found", request.card_id))?;

    // 2. Query the current deck cards
    let current_deck = get_cards_by_ids(&conn, &request.current_deck)
        .map_err(|e| format!("Failed to fetch deck cards: {}", e))?;

    // 3. Query synergies for the card
    let synergies = get_synergies_for_card(&conn, &request.card_id)
        .map_err(|e| format!("Failed to fetch synergies: {}", e))?;

    // 4. Query context modifiers
    let context_modifiers = get_active_context_modifiers(&conn)
        .map_err(|e| format!("Failed to fetch context modifiers: {}", e))?;

    // 5. Query champion override
    let champion_override = get_champion_override(&conn, &request.card_id, &request.champion, None)
        .map_err(|e| format!("Failed to fetch champion override: {}", e))?;

    // 6. Calculate the score
    let calculator = ScoreCalculator::new();
    let result = calculator.calculate_full(
        &card,
        &current_deck,
        &request.champion,
        request.ring_number,
        request.covenant,
        &synergies,
        &context_modifiers,
        champion_override,
    );

    Ok(result.into())
}

/// Get synergies for a specific card
#[tauri::command]
pub fn get_synergies(
    card_id: String,
    state: State<DatabaseState>,
) -> Result<Vec<String>, String> {
    if card_id.trim().is_empty() {
        return Err("Card ID cannot be empty".to_string());
    }

    let conn = Connection::open(&state.db_path).map_err(|e| e.to_string())?;

    let synergies = get_synergies_for_card(&conn, &card_id)
        .map_err(|e| format!("Failed to fetch synergies: {}", e))?;

    // Return formatted synergy descriptions
    let descriptions: Vec<String> = synergies
        .into_iter()
        .map(|s| format!("{} (x{:.2})", s.description, s.weight))
        .collect();

    Ok(descriptions)
}

/// Get all active context modifiers
#[tauri::command]
pub fn get_context_modifiers(state: State<DatabaseState>) -> Result<Vec<String>, String> {
    let conn = Connection::open(&state.db_path).map_err(|e| e.to_string())?;

    let modifiers = get_active_context_modifiers(&conn)
        .map_err(|e| format!("Failed to fetch context modifiers: {}", e))?;

    // Return formatted modifier descriptions
    let descriptions: Vec<String> = modifiers
        .into_iter()
        .map(|m| format!("{}: {} ({} priority)", m.condition, m.modifier, m.priority))
        .collect();

    Ok(descriptions)
}

/// Internal function to calculate draft score directly from a connection (for testing)
fn calculate_draft_score_internal(
    conn: &Connection,
    request: DraftScoreRequest,
) -> Result<DraftScoreResponse, ScoringError> {
    // Validate input
    if request.card_id.trim().is_empty() {
        return Err(ScoringError::InvalidInput("Card ID cannot be empty".to_string()));
    }
    if request.champion.trim().is_empty() {
        return Err(ScoringError::InvalidInput("Champion cannot be empty".to_string()));
    }
    if request.ring_number < 1 || request.ring_number > 10 {
        return Err(ScoringError::InvalidInput("Ring number must be between 1 and 10".to_string()));
    }
    if request.covenant < 1 || request.covenant > 25 {
        return Err(ScoringError::InvalidInput("Covenant must be between 1 and 25".to_string()));
    }

    // 1. Query the card being evaluated
    let card = get_card_by_id(conn, &request.card_id)?
        .ok_or_else(|| ScoringError::CardNotFound(request.card_id.clone()))?;

    // 2. Query the current deck cards
    let current_deck = get_cards_by_ids(conn, &request.current_deck)?;

    // 3. Query synergies for the card
    let synergies = get_synergies_for_card(conn, &request.card_id)?;

    // 4. Query context modifiers
    let context_modifiers = get_active_context_modifiers(conn)?;

    // 5. Query champion override
    let champion_override = get_champion_override(conn, &request.card_id, &request.champion, None)?;

    // 6. Calculate the score
    let calculator = ScoreCalculator::new();
    let result = calculator.calculate_full(
        &card,
        &current_deck,
        &request.champion,
        request.ring_number,
        request.covenant,
        &synergies,
        &context_modifiers,
        champion_override,
    );

    Ok(result.into())
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
    fn test_get_card_by_id() {
        let (state, _temp) = setup_test_db();
        let conn = Connection::open(&state.db_path).unwrap();

        // Test finding a known card
        let card = get_card_by_id(&conn, "banished_fel").unwrap();
        assert!(card.is_some());
        assert_eq!(card.unwrap().name, "Fel");

        // Test not found
        let not_found = get_card_by_id(&conn, "nonexistent").unwrap();
        assert!(not_found.is_none());
    }

    #[test]
    fn test_get_cards_by_ids() {
        let (state, _temp) = setup_test_db();
        let conn = Connection::open(&state.db_path).unwrap();

        let ids = vec!["banished_fel".to_string(), "pyreborne_lord_fenix".to_string()];
        let cards = get_cards_by_ids(&conn, &ids).unwrap();

        assert_eq!(cards.len(), 2);
        assert!(cards.iter().any(|c| c.id == "banished_fel"));
        assert!(cards.iter().any(|c| c.id == "pyreborne_lord_fenix"));
    }

    #[test]
    fn test_get_cards_by_ids_empty() {
        let (state, _temp) = setup_test_db();
        let conn = Connection::open(&state.db_path).unwrap();

        let cards = get_cards_by_ids(&conn, &[]).unwrap();
        assert!(cards.is_empty());
    }

    #[test]
    fn test_get_synergies_for_card() {
        let (state, _temp) = setup_test_db();
        let conn = Connection::open(&state.db_path).unwrap();

        // banished_fel has synergies defined in seed data
        let synergies = get_synergies_for_card(&conn, "banished_fel").unwrap();
        assert!(!synergies.is_empty());
    }

    #[test]
    fn test_get_active_context_modifiers() {
        let (state, _temp) = setup_test_db();
        let conn = Connection::open(&state.db_path).unwrap();

        let modifiers = get_active_context_modifiers(&conn).unwrap();
        assert!(!modifiers.is_empty());
    }

    #[test]
    fn test_get_champion_override() {
        let (state, _temp) = setup_test_db();
        let conn = Connection::open(&state.db_path).unwrap();

        // Fel has an override for "Just Cause" with champion "Fel"
        let override_val = get_champion_override(&conn, "banished_just_cause", "Fel", None).unwrap();
        assert!(override_val.is_some());
        assert_eq!(override_val.unwrap(), 95);

        // No override for non-matching champion
        let no_override = get_champion_override(&conn, "banished_just_cause", "Random", None).unwrap();
        assert!(no_override.is_none());
    }

    #[test]
    fn test_calculate_draft_score() {
        let (state, _temp) = setup_test_db();
        let conn = Connection::open(&state.db_path).unwrap();

        let request = DraftScoreRequest {
            card_id: "banished_fel".to_string(),
            current_deck: vec!["banished_just_cause".to_string()],
            champion: "Fel".to_string(),
            ring_number: 1,
            covenant: 10,
        };

        let result = calculate_draft_score_internal(&conn, request);
        assert!(result.is_ok());

        let response = result.unwrap();
        assert!(response.score > 0);
        assert!(!response.tier.is_empty());
        // Fel + Just Cause should have synergy bonus
        assert!(!response.reasons.is_empty());
    }

    #[test]
    fn test_calculate_draft_score_invalid_card() {
        let (state, _temp) = setup_test_db();
        let conn = Connection::open(&state.db_path).unwrap();

        let request = DraftScoreRequest {
            card_id: "nonexistent_card".to_string(),
            current_deck: vec![],
            champion: "Fel".to_string(),
            ring_number: 1,
            covenant: 10,
        };

        let result = calculate_draft_score_internal(&conn, request);
        assert!(result.is_err());
    }

    #[test]
    fn test_calculate_draft_score_invalid_ring() {
        let (state, _temp) = setup_test_db();
        let conn = Connection::open(&state.db_path).unwrap();

        let request = DraftScoreRequest {
            card_id: "banished_fel".to_string(),
            current_deck: vec![],
            champion: "Fel".to_string(),
            ring_number: 99, // Invalid
            covenant: 10,
        };

        let result = calculate_draft_score_internal(&conn, request);
        assert!(result.is_err());
        match result.unwrap_err() {
            ScoringError::InvalidInput(msg) => assert!(msg.contains("Ring number")),
            _ => panic!("Expected InvalidInput error"),
        }
    }

    #[test]
    fn test_calculate_draft_score_empty_champion() {
        let (state, _temp) = setup_test_db();
        let conn = Connection::open(&state.db_path).unwrap();

        let request = DraftScoreRequest {
            card_id: "banished_fel".to_string(),
            current_deck: vec![],
            champion: "".to_string(),
            ring_number: 1,
            covenant: 10,
        };

        let result = calculate_draft_score_internal(&conn, request);
        assert!(result.is_err());
        match result.unwrap_err() {
            ScoringError::InvalidInput(msg) => assert!(msg.contains("Champion")),
            _ => panic!("Expected InvalidInput error"),
        }
    }

    #[test]
    fn test_get_synergies_command() {
        let (state, _temp) = setup_test_db();
        let conn = Connection::open(&state.db_path).unwrap();

        // Test the internal function directly
        let synergies = get_synergies_for_card(&conn, "banished_fel").unwrap();
        assert!(!synergies.is_empty());

        // Format descriptions like the command does
        let descriptions: Vec<String> = synergies
            .into_iter()
            .map(|s| format!("{} (x{:.2})", s.description, s.weight))
            .collect();
        assert!(!descriptions.is_empty());
    }

    #[test]
    fn test_get_context_modifiers_command() {
        let (state, _temp) = setup_test_db();
        let conn = Connection::open(&state.db_path).unwrap();

        // Test the internal function directly
        let modifiers = get_active_context_modifiers(&conn).unwrap();
        assert!(!modifiers.is_empty());

        // Format descriptions like the command does
        let descriptions: Vec<String> = modifiers
            .into_iter()
            .map(|m| format!("{}: {} ({} priority)", m.condition, m.modifier, m.priority))
            .collect();
        assert!(!descriptions.is_empty());
    }
}
