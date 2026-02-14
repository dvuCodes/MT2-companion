//! Export/Import commands for deck data

use crate::database::DatabaseState;
use serde::{Deserialize, Serialize};
use tauri::State;

/// Deck export format
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DeckExport {
    pub version: String,
    pub exported_at: String,
    pub champion: String,
    pub champion_path: String,
    pub covenant: i32,
    pub ring: i32,
    pub cards: Vec<ExportedCard>,
    pub metadata: ExportMetadata,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ExportedCard {
    pub id: String,
    pub name: String,
    pub draft_order: i32,
    pub ring_number: i32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ExportMetadata {
    pub total_value: i32,
    pub unit_count: i32,
    pub spell_count: i32,
    pub synergy_count: i32,
}

/// Export the current deck to a JSON file
#[tauri::command]
pub async fn export_deck(
    deck_data: DeckExport,
    file_path: String,
) -> Result<(), String> {
    log::info!("[Export] Exporting deck to: {}", file_path);
    
    let json = serde_json::to_string_pretty(&deck_data)
        .map_err(|e| format!("Failed to serialize deck: {}", e))?;
    
    tokio::fs::write(&file_path, json)
        .await
        .map_err(|e| format!("Failed to write file: {}", e))?;
    
    log::info!("[Export] Successfully exported deck to: {}", file_path);
    Ok(())
}

/// Import a deck from a JSON file
#[tauri::command]
pub async fn import_deck(file_path: String) -> Result<DeckExport, String> {
    log::info!("[Import] Importing deck from: {}", file_path);
    
    let json = tokio::fs::read_to_string(&file_path)
        .await
        .map_err(|e| format!("Failed to read file: {}", e))?;
    
    let deck: DeckExport = serde_json::from_str(&json)
        .map_err(|e| format!("Failed to parse deck data: {}", e))?;
    
    // Validate version
    if deck.version != "1.0" {
        return Err(format!("Unsupported deck version: {}", deck.version));
    }
    
    log::info!("[Import] Successfully imported deck with {} cards", deck.cards.len());
    Ok(deck)
}

/// Export deck history to CSV
#[tauri::command]
pub fn export_history_csv(
    state: State<'_, DatabaseState>,
    file_path: String,
) -> Result<(), String> {
    use rusqlite::Connection;
    
    log::info!("[Export] Exporting history to CSV: {}", file_path);
    
    let conn = Connection::open(&state.db_path)
        .map_err(|e| format!("Failed to open database: {}", e))?;
    
    let mut stmt = conn.prepare(
        "SELECT run_id, card_id, ring_number, draft_order, champion, covenant, score_at_draft, did_win, created_at 
         FROM deck_history 
         ORDER BY created_at DESC"
    ).map_err(|e| format!("Failed to prepare query: {}", e))?;
    
    let mut csv_content = String::from(
        "run_id,card_id,ring_number,draft_order,champion,covenant,score_at_draft,did_win,created_at\n"
    );
    
    let rows = stmt.query_map([], |row| {
        Ok((
            row.get::<_, String>(0)?,
            row.get::<_, String>(1)?,
            row.get::<_, i32>(2)?,
            row.get::<_, i32>(3)?,
            row.get::<_, String>(4)?,
            row.get::<_, i32>(5)?,
            row.get::<_, Option<i32>>(6)?,
            row.get::<_, Option<bool>>(7)?,
            row.get::<_, String>(8)?,
        ))
    }).map_err(|e| format!("Failed to query history: {}", e))?;
    
    for row in rows {
        let (run_id, card_id, ring, order, champion, covenant, score, did_win, created_at) = 
            row.map_err(|e| format!("Failed to read row: {}", e))?;
        
        csv_content.push_str(&format!(
            "{},{},{},{},{},{},{},{},{}\n",
            run_id,
            card_id,
            ring,
            order,
            champion,
            covenant,
            score.map(|s| s.to_string()).unwrap_or_default(),
            did_win.map(|w| w.to_string()).unwrap_or_default(),
            created_at
        ));
    }
    
    std::fs::write(&file_path, csv_content)
        .map_err(|e| format!("Failed to write CSV: {}", e))?;
    
    log::info!("[Export] Successfully exported history to: {}", file_path);
    Ok(())
}

/// Get available export formats
#[tauri::command]
pub fn get_export_formats() -> Vec<ExportFormat> {
    vec![
        ExportFormat {
            id: "json".to_string(),
            name: "JSON".to_string(),
            extension: "json".to_string(),
            description: "Full deck data with metadata".to_string(),
        },
        ExportFormat {
            id: "csv".to_string(),
            name: "CSV".to_string(),
            extension: "csv".to_string(),
            description: "Simple card list".to_string(),
        },
    ]
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ExportFormat {
    pub id: String,
    pub name: String,
    pub extension: String,
    pub description: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    
    #[tokio::test]
    async fn test_export_import_roundtrip() {
        let deck = DeckExport {
            version: "1.0".to_string(),
            exported_at: chrono::Utc::now().to_rfc3339(),
            champion: "Fel".to_string(),
            champion_path: "Unchained".to_string(),
            covenant: 10,
            ring: 5,
            cards: vec![
                ExportedCard {
                    id: "card-1".to_string(),
                    name: "Test Card".to_string(),
                    draft_order: 1,
                    ring_number: 1,
                },
            ],
            metadata: ExportMetadata {
                total_value: 75,
                unit_count: 0,
                spell_count: 1,
                synergy_count: 0,
            },
        };
        
        let temp_file = NamedTempFile::new().unwrap();
        let path = temp_file.path().to_str().unwrap().to_string();
        
        // Export
        export_deck(deck.clone(), path.clone()).await.unwrap();
        
        // Import
        let imported = import_deck(path).await.unwrap();
        
        assert_eq!(imported.champion, deck.champion);
        assert_eq!(imported.cards.len(), deck.cards.len());
    }
}
