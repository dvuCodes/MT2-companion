//! Mock OCR implementations when OCR feature is disabled
//!
//! This module provides stub implementations of OCR types and functions
//! that return empty results or errors gracefully when the OCR feature
//! is not enabled. This allows the code to compile without the OCR dependencies.

use image::{GrayImage, ImageBuffer, Rgba};
use std::path::Path;

// ============================================================================
// Mock Capture Module
// ============================================================================

/// Represents a screen region to capture
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CaptureRegion {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
}

impl CaptureRegion {
    /// Create a new capture region
    pub fn new(x: i32, y: i32, width: u32, height: u32) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }

    /// Validate that the region has positive dimensions
    pub fn is_valid(&self) -> bool {
        self.width > 0 && self.height > 0
    }

    /// Check if this region contains a point
    pub fn contains(&self, px: i32, py: i32) -> bool {
        px >= self.x && px < self.x + self.width as i32 && py >= self.y && py < self.y + self.height as i32
    }
}

impl std::fmt::Display for CaptureRegion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Region(x={}, y={}, w={}, h={})", self.x, self.y, self.width, self.height)
    }
}

/// Error types for screen capture operations
#[derive(Debug, PartialEq)]
pub enum CaptureError {
    NoScreensAvailable,
    RegionOutOfBounds,
    InvalidRegion,
    CaptureFailed(String),
}

impl std::fmt::Display for CaptureError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CaptureError::NoScreensAvailable => write!(f, "No screens available for capture"),
            CaptureError::RegionOutOfBounds => write!(f, "Capture region is outside screen bounds"),
            CaptureError::InvalidRegion => write!(f, "Invalid capture region (zero or negative dimensions)"),
            CaptureError::CaptureFailed(msg) => write!(f, "Screen capture failed: {}", msg),
        }
    }
}

impl std::error::Error for CaptureError {}

/// Result type for capture operations
pub type CaptureResult<T> = Result<T, CaptureError>;

/// Mock: Captures a specific region - returns error since OCR is disabled
pub fn capture_region(_region: &CaptureRegion) -> CaptureResult<ImageBuffer<Rgba<u8>, Vec<u8>>> {
    log::warn!("OCR feature is disabled - screen capture not available");
    Err(CaptureError::CaptureFailed("OCR feature not enabled".to_string()))
}

/// Mock: Captures multiple regions
pub fn capture_multiple_regions(regions: &[CaptureRegion]) -> Vec<CaptureResult<ImageBuffer<Rgba<u8>, Vec<u8>>>> {
    log::warn!("OCR feature is disabled - screen capture not available");
    regions.iter().map(|_| Err(CaptureError::CaptureFailed("OCR feature not enabled".to_string()))).collect()
}

/// Get default card name regions for Monster Train 2 draft screen
pub fn get_default_card_regions(screen_width: u32, screen_height: u32) -> Vec<CaptureRegion> {
    let scale_x = screen_width as f32 / 1920.0;
    let scale_y = screen_height as f32 / 1080.0;

    let base_regions = vec![
        CaptureRegion::new(350, 200, 300, 60),
        CaptureRegion::new(810, 200, 300, 60),
        CaptureRegion::new(1270, 200, 300, 60),
        CaptureRegion::new(810, 500, 300, 60),
    ];

    base_regions
        .into_iter()
        .map(|r| CaptureRegion {
            x: (r.x as f32 * scale_x) as i32,
            y: (r.y as f32 * scale_y) as i32,
            width: (r.width as f32 * scale_x) as u32,
            height: (r.height as f32 * scale_y) as u32,
        })
        .collect()
}

/// Mock: Gets the primary screen dimensions
pub fn get_primary_screen_dimensions() -> CaptureResult<(u32, u32)> {
    log::debug!("OCR feature disabled - returning default screen dimensions");
    Ok((1920, 1080))
}

/// Configuration for OCR capture regions
#[derive(Debug, Clone)]
pub struct CaptureConfig {
    pub regions: Vec<CaptureRegion>,
    pub screen_width: u32,
    pub screen_height: u32,
}

impl CaptureConfig {
    /// Create a new capture configuration with default regions
    pub fn new() -> CaptureResult<Self> {
        let (screen_width, screen_height) = get_primary_screen_dimensions()?;
        let regions = get_default_card_regions(screen_width, screen_height);
        
        Ok(Self {
            regions,
            screen_width,
            screen_height,
        })
    }

    /// Create with custom regions
    pub fn with_regions(regions: Vec<CaptureRegion>) -> CaptureResult<Self> {
        let (screen_width, screen_height) = get_primary_screen_dimensions()?;
        
        Ok(Self {
            regions,
            screen_width,
            screen_height,
        })
    }

    /// Update regions after calibration
    pub fn update_regions(&mut self, regions: Vec<CaptureRegion>) {
        self.regions = regions;
    }

    /// Get the current capture regions
    pub fn get_regions(&self) -> &[CaptureRegion] {
        &self.regions
    }

    /// Mock: Capture all configured regions
    pub fn capture_all(&self) -> Vec<CaptureResult<ImageBuffer<Rgba<u8>, Vec<u8>>>> {
        log::warn!("OCR feature is disabled - capture_all returning empty results");
        vec![]
    }
}

impl Default for CaptureConfig {
    fn default() -> Self {
        let regions = get_default_card_regions(1920, 1080);
        Self {
            regions,
            screen_width: 1920,
            screen_height: 1080,
        }
    }
}

// ============================================================================
// Mock Preprocess Module
// ============================================================================

/// Error types for image preprocessing
#[derive(Debug, PartialEq)]
pub enum PreprocessError {
    InvalidImage(String),
    ProcessingFailed(String),
    EmptyImage,
}

impl std::fmt::Display for PreprocessError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PreprocessError::InvalidImage(msg) => write!(f, "Invalid image: {}", msg),
            PreprocessError::ProcessingFailed(msg) => write!(f, "Processing failed: {}", msg),
            PreprocessError::EmptyImage => write!(f, "Image is empty"),
        }
    }
}

impl std::error::Error for PreprocessError {}

/// Result type for preprocessing operations
pub type PreprocessResult<T> = Result<T, PreprocessError>;

/// Configuration for image preprocessing
#[derive(Debug, Clone, Copy)]
pub struct PreprocessConfig {
    pub threshold: u8,
    pub use_adaptive_threshold: bool,
    pub adaptive_block_size: u32,
    pub adaptive_c: i32,
    pub denoise: bool,
    pub invert: bool,
    pub scale_factor: f32,
    pub contrast_factor: f32,
}

impl Default for PreprocessConfig {
    fn default() -> Self {
        Self {
            threshold: 127,
            use_adaptive_threshold: true,
            adaptive_block_size: 11,
            adaptive_c: 2,
            denoise: true,
            invert: false,
            scale_factor: 2.0,
            contrast_factor: 1.5,
        }
    }
}

/// Mock: Main preprocessing pipeline - returns empty image
pub fn preprocess_for_ocr(
    img: &ImageBuffer<Rgba<u8>, Vec<u8>>,
    _config: &PreprocessConfig,
) -> PreprocessResult<GrayImage> {
    if img.width() == 0 || img.height() == 0 {
        return Err(PreprocessError::EmptyImage);
    }
    // Return a simple grayscale conversion
    Ok(image::imageops::grayscale(img))
}

/// Mock: Convenience function with default preprocessing settings
pub fn preprocess_default(img: &ImageBuffer<Rgba<u8>, Vec<u8>>) -> PreprocessResult<GrayImage> {
    preprocess_for_ocr(img, &PreprocessConfig::default())
}

/// Mock: Save a grayscale image for debugging
pub fn save_debug_image(img: &GrayImage, path: &Path) -> PreprocessResult<()> {
    log::debug!("OCR feature disabled - saving debug image to {:?}", path);
    img.save(path)
        .map_err(|e| PreprocessError::ProcessingFailed(e.to_string()))
}

// ============================================================================
// Mock Recognize Module
// ============================================================================

/// Error types for OCR recognition
#[derive(Debug)]
pub enum RecognizeError {
    TesseractInitFailed(String),
    TesseractError(String),
    NoCardNamesAvailable,
    InvalidImage,
    MatchingFailed(String),
}

impl std::fmt::Display for RecognizeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RecognizeError::TesseractInitFailed(msg) => {
                write!(f, "Tesseract initialization failed: {}", msg)
            }
            RecognizeError::TesseractError(msg) => write!(f, "Tesseract error: {}", msg),
            RecognizeError::NoCardNamesAvailable => write!(f, "No card names available for matching"),
            RecognizeError::InvalidImage => write!(f, "Invalid image for OCR"),
            RecognizeError::MatchingFailed(msg) => write!(f, "Card matching failed: {}", msg),
        }
    }
}

impl std::error::Error for RecognizeError {}

/// Result type for recognition operations
pub type RecognizeResult<T> = Result<T, RecognizeError>;

/// Configuration for OCR recognition
#[derive(Debug, Clone)]
pub struct RecognizeConfig {
    pub tesseract_data_path: Option<String>,
    pub language: String,
    pub psm: i32,
    pub oem: i32,
    pub min_confidence: i32,
    pub min_match_score: i32,
    pub whitelist: Option<String>,
}

impl Default for RecognizeConfig {
    fn default() -> Self {
        Self {
            tesseract_data_path: None,
            language: "eng".to_string(),
            psm: 7,
            oem: 3,
            min_confidence: 60,
            min_match_score: 60,
            whitelist: Some("ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789 '-".to_string()),
        }
    }
}

impl RecognizeConfig {
    /// Create a new config with custom language
    pub fn with_language(lang: &str) -> Self {
        Self {
            language: lang.to_string(),
            ..Default::default()
        }
    }

    /// Set custom whitelist of characters
    pub fn with_whitelist(self, whitelist: &str) -> Self {
        Self {
            whitelist: Some(whitelist.to_string()),
            ..self
        }
    }
}

/// Result of OCR text recognition
#[derive(Debug, Clone, PartialEq)]
pub struct OcrResult {
    pub text: String,
    pub confidence: i32,
    pub is_confident: bool,
}

impl OcrResult {
    /// Create a new OCR result
    pub fn new(text: String, confidence: i32, min_confidence: i32) -> Self {
        let text = text.trim().to_string();
        Self {
            text,
            confidence,
            is_confident: confidence >= min_confidence,
        }
    }

    /// Get the text as a normalized string
    pub fn normalized_text(&self) -> String {
        self.text.to_lowercase().trim().to_string()
    }
}

/// Result of card name matching
#[derive(Debug, Clone, PartialEq)]
pub struct CardMatch {
    pub card_name: String,
    pub card_id: String,
    pub ocr_text: String,
    pub match_score: i32,
    pub ocr_confidence: i32,
    pub overall_confidence: f64,
}

impl CardMatch {
    /// Calculate overall confidence from OCR confidence and match score
    pub fn calculate_overall_confidence(ocr_confidence: i32, match_score: i32) -> f64 {
        (ocr_confidence as f64 * 0.4 + match_score as f64 * 0.6) / 100.0
    }
}

/// Mock OCR engine
pub struct OcrEngine {
    config: RecognizeConfig,
}

impl OcrEngine {
    /// Create a new OCR engine with default configuration
    pub fn new() -> RecognizeResult<Self> {
        log::warn!("OCR feature is disabled - OCR engine will return mock results");
        Ok(Self {
            config: RecognizeConfig::default(),
        })
    }

    /// Create a new OCR engine with custom configuration
    pub fn with_config(config: RecognizeConfig) -> RecognizeResult<Self> {
        log::warn!("OCR feature is disabled - OCR engine will return mock results");
        Ok(Self { config })
    }

    /// Mock: Recognize text from a grayscale image
    pub fn recognize(&self, _img: &GrayImage) -> RecognizeResult<OcrResult> {
        log::warn!("OCR feature is disabled - returning empty recognition result");
        Ok(OcrResult::new(
            "".to_string(),
            0,
            self.config.min_confidence,
        ))
    }

    /// Mock: Recognize text from multiple images
    pub fn recognize_multiple(&self, images: &[GrayImage]) -> Vec<RecognizeResult<OcrResult>> {
        log::warn!("OCR feature is disabled - returning empty recognition results");
        images.iter().map(|_| self.recognize(&GrayImage::new(1, 1))).collect()
    }
}

impl Default for OcrEngine {
    fn default() -> Self {
        Self::new().expect("Failed to create default OCR engine")
    }
}

/// Card name matcher using fuzzy string matching
pub struct CardMatcher {
    card_names: Vec<(String, String)>,
    matcher: fuzzy_matcher::skim::SkimMatcherV2,
    min_score: i32,
}

impl CardMatcher {
    /// Create a new card matcher with the given card names
    pub fn new(card_names: Vec<(String, String)>, min_score: i32) -> RecognizeResult<Self> {
        if card_names.is_empty() {
            return Err(RecognizeError::NoCardNamesAvailable);
        }

        Ok(Self {
            card_names,
            matcher: fuzzy_matcher::skim::SkimMatcherV2::default(),
            min_score,
        })
    }

    /// Find the best matching card for the given OCR text
    pub fn find_best_match(&self, ocr_text: &str) -> Option<CardMatch> {
        use fuzzy_matcher::FuzzyMatcher;
        
        let ocr_normalized = ocr_text.to_lowercase().trim().to_string();
        
        if ocr_normalized.is_empty() {
            return None;
        }

        let mut best_match: Option<CardMatch> = None;
        let mut best_score = self.min_score as i64;

        for (card_id, card_name) in &self.card_names {
            if let Some(score) = self.matcher.fuzzy_match(&card_name.to_lowercase(), &ocr_normalized) {
                if score > best_score {
                    best_score = score;
                    best_match = Some(CardMatch {
                        card_name: card_name.clone(),
                        card_id: card_id.clone(),
                        ocr_text: ocr_text.to_string(),
                        match_score: score.min(100) as i32,
                        ocr_confidence: 0,
                        overall_confidence: 0.0,
                    });
                }
            }

            // Also try matching individual words for short OCR text
            if ocr_normalized.len() < 10 {
                let card_name_lower = card_name.to_lowercase();
                let card_words: Vec<&str> = card_name_lower.split_whitespace().collect();
                for word in &card_words {
                    if let Some(word_score) = self.matcher.fuzzy_match(word, &ocr_normalized) {
                        if word_score > best_score {
                            best_score = word_score;
                            best_match = Some(CardMatch {
                                card_name: card_name.clone(),
                                card_id: card_id.clone(),
                                ocr_text: ocr_text.to_string(),
                                match_score: word_score.min(100) as i32,
                                ocr_confidence: 0,
                                overall_confidence: 0.0,
                            });
                        }
                    }
                }
            }
        }

        best_match
    }

    /// Match multiple OCR results and update their confidence scores
    pub fn match_results(&self, ocr_results: Vec<OcrResult>) -> Vec<CardMatch> {
        let mut matches = Vec::new();

        for result in ocr_results {
            if let Some(mut card_match) = self.find_best_match(&result.text) {
                card_match.ocr_confidence = result.confidence;
                card_match.overall_confidence = CardMatch::calculate_overall_confidence(
                    result.confidence,
                    card_match.match_score,
                );
                matches.push(card_match);
            }
        }

        // Sort by overall confidence (highest first)
        matches.sort_by(|a, b| {
            b.overall_confidence
                .partial_cmp(&a.overall_confidence)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        matches
    }

    /// Find all cards that match above the threshold
    pub fn find_all_matches(&self, ocr_text: &str, threshold: i32) -> Vec<CardMatch> {
        use fuzzy_matcher::FuzzyMatcher;
        
        let ocr_normalized = ocr_text.to_lowercase().trim().to_string();
        let mut matches = Vec::new();

        if ocr_normalized.is_empty() {
            return matches;
        }

        for (card_id, card_name) in &self.card_names {
            if let Some(score) = self.matcher.fuzzy_match(&card_name.to_lowercase(), &ocr_normalized) {
                if score >= threshold as i64 {
                    matches.push(CardMatch {
                        card_name: card_name.clone(),
                        card_id: card_id.clone(),
                        ocr_text: ocr_text.to_string(),
                        match_score: score.min(100) as i32,
                        ocr_confidence: 0,
                        overall_confidence: score as f64 / 100.0,
                    });
                }
            }
        }

        // Sort by match score (highest first)
        matches.sort_by(|a, b| {
            b.match_score
                .cmp(&a.match_score)
        });

        matches
    }
}

/// Complete recognition pipeline combining OCR and card matching
pub struct RecognitionPipeline {
    ocr_engine: OcrEngine,
    card_matcher: CardMatcher,
}

impl RecognitionPipeline {
    /// Create a new recognition pipeline
    pub fn new(card_names: Vec<(String, String)>) -> RecognizeResult<Self> {
        let ocr_engine = OcrEngine::new()?;
        let config = RecognizeConfig::default();
        let card_matcher = CardMatcher::new(card_names, config.min_match_score)?;

        Ok(Self {
            ocr_engine,
            card_matcher,
        })
    }

    /// Create with custom configuration
    pub fn with_config(
        card_names: Vec<(String, String)>,
        config: RecognizeConfig,
    ) -> RecognizeResult<Self> {
        let ocr_engine = OcrEngine::with_config(config.clone())?;
        let card_matcher = CardMatcher::new(card_names, config.min_match_score)?;

        Ok(Self {
            ocr_engine,
            card_matcher,
        })
    }

    /// Mock: Process a single image through the full pipeline
    pub fn process(&self, _img: &GrayImage) -> RecognizeResult<Option<CardMatch>> {
        log::warn!("OCR feature is disabled - process() returning None");
        Ok(None)
    }

    /// Mock: Process multiple images through the full pipeline
    pub fn process_multiple(&self, _images: &[GrayImage]) -> Vec<CardMatch> {
        log::warn!("OCR feature is disabled - process_multiple() returning empty");
        vec![]
    }
}

/// Helper function to normalize card name for better matching
pub fn normalize_card_name(name: &str) -> String {
    name.chars()
        .filter(|c| c.is_alphanumeric() || c.is_whitespace())
        .collect::<String>()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .to_lowercase()
}

/// Build a card name map from database results
pub fn build_card_map(cards: Vec<(String, String)>) -> std::collections::HashMap<String, String> {
    cards.into_iter().collect()
}
