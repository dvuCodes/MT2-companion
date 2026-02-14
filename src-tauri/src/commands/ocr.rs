//! OCR command handlers
//!
//! This module provides Tauri command handlers for OCR operations,
//! including card detection on screen and OCR region calibration.

use crate::database::DatabaseState;
use crate::ocr::{
    self, capture::CaptureRegion, CalibrationReport, CardDetectionOptions,
    DetectedCard, OcrPipeline,
};
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use tauri::State;

/// Response structure for card detection
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CardDetectionResponse {
    pub detected_cards: Vec<String>,
    pub confidence: f64,
    pub success: bool,
    pub error: Option<String>,
    pub details: Vec<DetectedCardInfo>,
}

/// Detailed information about a detected card
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DetectedCardInfo {
    pub card_id: String,
    pub card_name: String,
    pub confidence: f64,
    pub ocr_confidence: i32,
    pub match_score: i32,
    pub raw_text: String,
    pub region: CaptureRegionInfo,
}

/// Information about a capture region
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CaptureRegionInfo {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
}

impl From<ocr::capture::CaptureRegion> for CaptureRegionInfo {
    fn from(region: ocr::capture::CaptureRegion) -> Self {
        Self {
            x: region.x,
            y: region.y,
            width: region.width,
            height: region.height,
        }
    }
}

impl From<DetectedCard> for DetectedCardInfo {
    fn from(card: DetectedCard) -> Self {
        Self {
            card_id: card.card_id,
            card_name: card.card_name,
            confidence: card.overall_confidence,
            ocr_confidence: card.ocr_confidence,
            match_score: card.match_score,
            raw_text: card.raw_ocr_text,
            region: card.region.into(),
        }
    }
}

/// Calibration result response
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CalibrationResult {
    pub success: bool,
    pub message: String,
    pub screen_width: u32,
    pub screen_height: u32,
    pub recommended_regions: Vec<CaptureRegionInfo>,
    pub success_rate: f64,
}

impl From<CalibrationReport> for CalibrationResult {
    fn from(report: CalibrationReport) -> Self {
        let recommended_regions: Vec<CaptureRegionInfo> = report
            .recommended_regions
            .clone()
            .into_iter()
            .map(|r| r.into())
            .collect();

        let message = if report.is_successful() {
            "Calibration successful! All regions can be captured.".to_string()
        } else {
            format!(
                "Calibration partial. {}/{} regions captured successfully.",
                report.successful_captures, report.regions_tested
            )
        };

        Self {
            success: report.is_successful(),
            message,
            screen_width: report.screen_dimensions.0,
            screen_height: report.screen_dimensions.1,
            recommended_regions,
            success_rate: report.success_rate(),
        }
    }
}

/// Request to set custom capture regions
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SetRegionsRequest {
    pub regions: Vec<CaptureRegionInfo>,
}

/// Response for region setting
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SetRegionsResult {
    pub success: bool,
    pub message: String,
    pub regions_set: usize,
}

/// Application state for OCR configuration
/// This is managed by Tauri and persists across calls
pub struct OcrState {
    pub config: Mutex<CardDetectionOptions>,
}

impl OcrState {
    pub fn new() -> Self {
        Self {
            config: Mutex::new(CardDetectionOptions::default()),
        }
    }

    pub fn with_config(config: CardDetectionOptions) -> Self {
        Self {
            config: Mutex::new(config),
        }
    }
}

impl Default for OcrState {
    fn default() -> Self {
        Self::new()
    }
}

/// Get all card names from the database
fn get_card_names_from_db(db_path: &std::path::Path) -> Result<Vec<(String, String)>, String> {
    let conn = Connection::open(db_path).map_err(|e| e.to_string())?;

    let mut stmt = conn
        .prepare("SELECT id, name FROM cards ORDER BY name")
        .map_err(|e| e.to_string())?;

    let card_names: Vec<(String, String)> = stmt
        .query_map([], |row| {
            let id: String = row.get(0)?;
            let name: String = row.get(1)?;
            Ok((id, name))
        })
        .map_err(|e| e.to_string())?
        .filter_map(|r| r.ok())
        .collect();

    Ok(card_names)
}

/// Tauri command: Detect cards on screen
///
/// This command captures screen regions, runs OCR, and matches
/// against known card names from the database.
#[tauri::command]
pub fn detect_cards_on_screen(
    db_state: State<DatabaseState>,
    ocr_state: State<OcrState>,
) -> Result<CardDetectionResponse, String> {
    // Get card names from database
    let card_names = get_card_names_from_db(&db_state.db_path)?;

    if card_names.is_empty() {
        return Ok(CardDetectionResponse {
            detected_cards: vec![],
            confidence: 0.0,
            success: false,
            error: Some("No cards found in database".to_string()),
            details: vec![],
        });
    }

    // Get OCR configuration from state
    let config = ocr_state
        .config
        .lock()
        .map_err(|e| format!("Failed to lock OCR config: {}", e))?
        .clone();

    // Check if OCR feature is enabled
    #[cfg(not(feature = "ocr"))]
    {
        log::warn!("OCR feature is disabled - detect_cards_on_screen will return empty results");
    }

    // Create OCR pipeline
    let pipeline = match OcrPipeline::new(card_names.clone(), config) {
        Ok(p) => p,
        Err(e) => {
            return Ok(CardDetectionResponse {
                detected_cards: vec![],
                confidence: 0.0,
                success: false,
                error: Some(format!("Failed to initialize OCR: {}", e)),
                details: vec![],
            });
        }
    };

    // Run detection
    match pipeline.detect_cards() {
        Ok(result) => {
            let detected_cards: Vec<String> = result
                .detected_cards
                .iter()
                .map(|c| c.card_name.clone())
                .collect();

            let details: Vec<DetectedCardInfo> = result
                .detected_cards
                .into_iter()
                .map(|c| c.into())
                .collect();

            Ok(CardDetectionResponse {
                detected_cards,
                confidence: result.average_confidence,
                success: result.success,
                error: result.error_message,
                details,
            })
        }
        Err(e) => Ok(CardDetectionResponse {
            detected_cards: vec![],
            confidence: 0.0,
            success: false,
            error: Some(format!("Detection failed: {}", e)),
            details: vec![],
        }),
    }
}

/// Tauri command: Calibrate OCR regions
///
/// Tests the current capture configuration and returns
/// recommended regions based on screen dimensions.
#[tauri::command]
pub fn calibrate_ocr_regions(
    ocr_state: State<OcrState>,
) -> Result<CalibrationResult, String> {
    let config = ocr_state
        .config
        .lock()
        .map_err(|e| format!("Failed to lock OCR config: {}", e))?
        .clone();

    #[cfg(not(feature = "ocr"))]
    {
        log::warn!("OCR feature is disabled - calibrate_ocr_regions returning default values");
    }

    match ocr::calibrate_regions(&config) {
        Ok(report) => Ok(report.into()),
        Err(e) => Ok(CalibrationResult {
            success: false,
            message: format!("Calibration failed: {}", e),
            screen_width: 0,
            screen_height: 0,
            recommended_regions: vec![],
            success_rate: 0.0,
        }),
    }
}

/// Tauri command: Set custom capture regions
#[tauri::command]
pub fn set_capture_regions(
    request: SetRegionsRequest,
    ocr_state: State<OcrState>,
) -> Result<SetRegionsResult, String> {
    let regions: Vec<CaptureRegion> = request
        .regions
        .into_iter()
        .map(|r| CaptureRegion::new(r.x, r.y, r.width, r.height))
        .collect();

    let mut config = ocr_state
        .config
        .lock()
        .map_err(|e| format!("Failed to lock OCR config: {}", e))?;

    config.capture.update_regions(regions.clone());

    Ok(SetRegionsResult {
        success: true,
        message: format!("Set {} capture regions", regions.len()),
        regions_set: regions.len(),
    })
}

/// Tauri command: Get current capture regions
#[tauri::command]
pub fn get_capture_regions(ocr_state: State<OcrState>) -> Result<Vec<CaptureRegionInfo>, String> {
    let config = ocr_state
        .config
        .lock()
        .map_err(|e| format!("Failed to lock OCR config: {}", e))?;

    let regions: Vec<CaptureRegionInfo> = config
        .capture
        .get_regions()
        .iter()
        .copied()
        .map(|r| r.into())
        .collect();

    Ok(regions)
}

/// Tauri command: Reset capture regions to defaults
#[tauri::command]
pub fn reset_capture_regions(
    ocr_state: State<OcrState>,
) -> Result<SetRegionsResult, String> {
    let mut config = ocr_state
        .config
        .lock()
        .map_err(|e| format!("Failed to lock OCR config: {}", e))?;

    // Get screen dimensions and reset to default regions
    match ocr::capture::get_primary_screen_dimensions() {
        Ok((width, height)) => {
            let default_regions = ocr::capture::get_default_card_regions(width, height);
            let count = default_regions.len();
            config.capture.update_regions(default_regions);

            Ok(SetRegionsResult {
                success: true,
                message: format!("Reset to {} default regions for {}x{}", count, width, height),
                regions_set: count,
            })
        }
        Err(e) => Ok(SetRegionsResult {
            success: false,
            message: format!("Failed to get screen dimensions: {}", e),
            regions_set: 0,
        }),
    }
}

/// Tauri command: Update OCR configuration
#[tauri::command]
pub fn update_ocr_config(
    min_confidence: Option<f64>,
    save_debug: Option<bool>,
    ocr_state: State<OcrState>,
) -> Result<bool, String> {
    let mut config = ocr_state
        .config
        .lock()
        .map_err(|e| format!("Failed to lock OCR config: {}", e))?;

    if let Some(confidence) = min_confidence {
        config.min_overall_confidence = confidence.clamp(0.0, 1.0);
    }

    if let Some(debug) = save_debug {
        config.save_debug_images = debug;
    }

    Ok(true)
}

/// Tauri command: Test OCR on a specific region
///
/// This is useful for debugging OCR issues on specific screen regions.
/// Only available when OCR feature is enabled.
#[cfg(feature = "ocr")]
#[tauri::command]
pub fn test_ocr_region(
    x: i32,
    y: i32,
    width: u32,
    height: u32,
    db_state: State<DatabaseState>,
) -> Result<DetectedCardInfo, String> {
    use crate::ocr::capture::capture_region;
    use crate::ocr::preprocess::preprocess_default;
    use crate::ocr::recognize::OcrEngine;
    use fuzzy_matcher::skim::SkimMatcherV2;
    use fuzzy_matcher::FuzzyMatcher;

    // Get card names from database
    let card_names = get_card_names_from_db(&db_state.db_path)?;

    // Capture the region
    let region = CaptureRegion::new(x, y, width, height);
    let rgba_image = capture_region(&region).map_err(|e| e.to_string())?;

    // Preprocess
    let gray_image = preprocess_default(&rgba_image).map_err(|e| e.to_string())?;

    // Run OCR
    let ocr_engine = OcrEngine::new().map_err(|e| e.to_string())?;
    let ocr_result = ocr_engine.recognize(&gray_image).map_err(|e| e.to_string())?;

    // Find best matching card
    let matcher = SkimMatcherV2::default();
    let mut best_match: Option<(String, String, i64)> = None;
    let ocr_text = ocr_result.text.to_lowercase();

    for (card_id, card_name) in &card_names {
        if let Some(score) = matcher.fuzzy_match(&card_name.to_lowercase(), &ocr_text) {
            if best_match.as_ref().map_or(true, |(_, _, s)| score > *s) {
                best_match = Some((card_id.clone(), card_name.clone(), score));
            }
        }
    }

    match best_match {
        Some((card_id, card_name, match_score)) => Ok(DetectedCardInfo {
            card_id,
            card_name,
            confidence: match_score as f64 / 100.0,
            ocr_confidence: ocr_result.confidence,
            match_score: match_score.min(100) as i32,
            raw_text: ocr_result.text,
            region: region.into(),
        }),
        None => Err("No matching card found".to_string()),
    }
}

/// Mock implementation when OCR feature is disabled
#[cfg(not(feature = "ocr"))]
#[tauri::command]
pub fn test_ocr_region(
    _x: i32,
    _y: i32,
    _width: u32,
    _height: u32,
    _db_state: State<DatabaseState>,
) -> Result<DetectedCardInfo, String> {
    log::error!("test_ocr_region called but OCR feature is disabled");
    Err("OCR feature is not enabled. Rebuild with --features ocr to use this functionality.".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_capture_region_info_from_capture_region() {
        let region = ocr::capture::CaptureRegion::new(100, 200, 300, 400);
        let info: CaptureRegionInfo = region.into();
        assert_eq!(info.x, 100);
        assert_eq!(info.y, 200);
        assert_eq!(info.width, 300);
        assert_eq!(info.height, 400);
    }

    #[test]
    fn test_detected_card_info_from_detected_card() {
        let card = DetectedCard {
            card_id: "test_card".to_string(),
            card_name: "Test Card".to_string(),
            region: ocr::capture::CaptureRegion::new(10, 20, 100, 50),
            ocr_confidence: 85,
            match_score: 90,
            overall_confidence: 0.87,
            raw_ocr_text: "Test".to_string(),
        };

        let info: DetectedCardInfo = card.into();
        assert_eq!(info.card_id, "test_card");
        assert_eq!(info.card_name, "Test Card");
        assert_eq!(info.confidence, 0.87);
        assert_eq!(info.ocr_confidence, 85);
        assert_eq!(info.match_score, 90);
        assert_eq!(info.raw_text, "Test");
        assert_eq!(info.region.x, 10);
    }

    #[test]
    fn test_card_detection_response_default() {
        let response = CardDetectionResponse {
            detected_cards: vec!["Card A".to_string(), "Card B".to_string()],
            confidence: 0.85,
            success: true,
            error: None,
            details: vec![],
        };

        assert_eq!(response.detected_cards.len(), 2);
        assert!(response.success);
        assert!(response.error.is_none());
    }

    #[test]
    fn test_set_regions_request() {
        let request = SetRegionsRequest {
            regions: vec![
                CaptureRegionInfo {
                    x: 100,
                    y: 200,
                    width: 300,
                    height: 400,
                },
            ],
        };

        assert_eq!(request.regions.len(), 1);
        assert_eq!(request.regions[0].x, 100);
    }

    #[test]
    fn test_ocr_state_new() {
        let state = OcrState::new();
        let config = state.config.lock().unwrap();
        assert!(!config.save_debug_images);
    }
}
