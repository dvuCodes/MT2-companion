pub mod commands;
pub mod database;
pub mod logging;
pub mod ocr;
pub mod scoring;

use commands::ocr::OcrState;
use tauri::Manager;

pub fn run() {
    // Initialize logging
    logging::init();
    
    log::info!("Starting MT2 Draft Assistant");
    
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .setup(|app| {
            log::info!("Running application setup");
            // Initialize database
            let db_path = app
                .path()
                .app_data_dir()
                .expect("Failed to get app data dir")
                .join("mt2_draft.db");
            
            database::init(&db_path)?;
            
            // Store database path in app state
            app.manage(database::DatabaseState::new(db_path));
            
            // Initialize OCR state
            app.manage(OcrState::new());
            
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // Card commands
            commands::cards::get_card_by_name,
            commands::cards::get_cards_by_clan,
            commands::cards::search_cards,
            commands::cards::get_all_cards,
            
            // Scoring commands
            commands::scoring::calculate_draft_score,
            commands::scoring::get_synergies,
            commands::scoring::get_context_modifiers,
            
            // OCR commands
            commands::ocr::detect_cards_on_screen,
            commands::ocr::calibrate_ocr_regions,
            commands::ocr::set_capture_regions,
            commands::ocr::get_capture_regions,
            commands::ocr::reset_capture_regions,
            commands::ocr::update_ocr_config,
            commands::ocr::test_ocr_region,
            
            // Window commands
            commands::window::toggle_overlay,
            commands::window::show_overlay,
            commands::window::hide_overlay,
            commands::window::set_overlay_position,
            
            // Export/Import commands
            commands::export::export_deck,
            commands::export::import_deck,
            commands::export::export_history_csv,
            commands::export::get_export_formats,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
