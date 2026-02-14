//! OCR recognition and card name matching
//!
//! This module provides Tesseract OCR integration and fuzzy matching
//! to identify card names from preprocessed images.

use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;
use image::GrayImage;
use std::collections::HashMap;

#[cfg(feature = "ocr")]
use leptess::tesseract::TessInitError;
#[cfg(feature = "ocr")]
use leptess::LepTess;

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

#[cfg(feature = "ocr")]
impl From<TessInitError> for RecognizeError {
    fn from(err: TessInitError) -> Self {
        RecognizeError::TesseractInitFailed(err.to_string())
    }
}

/// Result type for recognition operations
pub type RecognizeResult<T> = Result<T, RecognizeError>;

/// Configuration for OCR recognition
#[derive(Debug, Clone)]
pub struct RecognizeConfig {
    /// Tesseract data path (None for default)
    pub tesseract_data_path: Option<String>,
    /// Language for OCR (e.g., "eng")
    pub language: String,
    /// Page segmentation mode (PSM)
    /// 6 = Assume a single uniform block of text
    /// 7 = Treat the image as a single text line
    /// 8 = Treat the image as a single word
    pub psm: i32,
    /// OCR engine mode (OEM)
    /// 1 = LSTM only
    /// 3 = Default, based on what is available
    pub oem: i32,
    /// Minimum confidence threshold (0-100)
    pub min_confidence: i32,
    /// Minimum fuzzy match score (0-100)
    pub min_match_score: i32,
    /// Whitelist of characters (None for all)
    pub whitelist: Option<String>,
}

impl Default for RecognizeConfig {
    fn default() -> Self {
        Self {
            tesseract_data_path: None,
            language: "eng".to_string(),
            psm: 7, // Single text line - good for card names
            oem: 3, // Default engine mode
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
    /// Recognized text
    pub text: String,
    /// Confidence score (0-100)
    pub confidence: i32,
    /// Whether the recognition met minimum confidence threshold
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

    /// Get the text as a normalized string (lowercase, trimmed)
    pub fn normalized_text(&self) -> String {
        self.text.to_lowercase().trim().to_string()
    }
}

/// Result of card name matching
#[derive(Debug, Clone, PartialEq)]
pub struct CardMatch {
    /// The matched card name
    pub card_name: String,
    /// The card ID from database
    pub card_id: String,
    /// OCR text that was matched
    pub ocr_text: String,
    /// Fuzzy match score (0-100)
    pub match_score: i32,
    /// OCR confidence score (0-100)
    pub ocr_confidence: i32,
    /// Overall confidence (weighted combination)
    pub overall_confidence: f64,
}

impl CardMatch {
    /// Calculate overall confidence from OCR confidence and match score
    pub fn calculate_overall_confidence(ocr_confidence: i32, match_score: i32) -> f64 {
        // Weight: 40% OCR confidence, 60% match score
        (ocr_confidence as f64 * 0.4 + match_score as f64 * 0.6) / 100.0
    }
}

/// OCR engine wrapper for Tesseract
pub struct OcrEngine {
    config: RecognizeConfig,
}

impl OcrEngine {
    /// Create a new OCR engine with default configuration
    pub fn new() -> RecognizeResult<Self> {
        Ok(Self {
            config: RecognizeConfig::default(),
        })
    }

    /// Create a new OCR engine with custom configuration
    pub fn with_config(config: RecognizeConfig) -> RecognizeResult<Self> {
        Ok(Self { config })
    }

    /// Initialize Tesseract with the configured settings
    #[cfg(feature = "ocr")]
    fn init_tesseract(&self) -> RecognizeResult<LepTess> {
        let mut tess = if let Some(ref data_path) = self.config.tesseract_data_path {
            LepTess::new(Some(data_path), &self.config.language)?
        } else {
            LepTess::new(None, &self.config.language)?
        };

        // Set page segmentation mode
        tess.set_variable("tessedit_pageseg_mode", &self.config.psm.to_string())
            .map_err(|e| RecognizeError::TesseractError(format!("Failed to set PSM: {:?}", e)))?;

        // Set OCR engine mode
        tess.set_variable("oem_mode", &self.config.oem.to_string())
            .map_err(|e| RecognizeError::TesseractError(format!("Failed to set OEM: {:?}", e)))?;

        // Set character whitelist if specified
        if let Some(ref whitelist) = self.config.whitelist {
            tess.set_variable("tessedit_char_whitelist", whitelist)
                .map_err(|e| RecognizeError::TesseractError(format!("Failed to set whitelist: {:?}", e)))?;
        }

        Ok(tess)
    }

    /// Mock Tesseract initialization when OCR feature is not enabled
    #[cfg(not(feature = "ocr"))]
    fn init_tesseract(&self) -> RecognizeResult<()> {
        // No-op when OCR is disabled
        Ok(())
    }

    /// Recognize text from a grayscale image
    #[cfg(feature = "ocr")]
    pub fn recognize(&self, img: &GrayImage) -> RecognizeResult<OcrResult> {
        if img.width() == 0 || img.height() == 0 {
            return Err(RecognizeError::InvalidImage);
        }

        let mut tess = self.init_tesseract()?;

        // Convert image to bytes for Tesseract
        let width = img.width() as i32;
        let height = img.height() as i32;
        let bytes_per_pixel = 1; // Grayscale
        let bytes_per_line = width;

        let img_bytes: Vec<u8> = img.pixels().map(|p| p[0]).collect();

        tess.set_image(
            &img_bytes,
            width,
            height,
            bytes_per_pixel,
            bytes_per_line,
        )
        .map_err(|e| RecognizeError::TesseractError(format!("Failed to set image: {:?}", e)))?;

        // Get recognized text
        let text = tess.get_utf8_text()
            .map_err(|e| RecognizeError::TesseractError(format!("OCR failed: {:?}", e)))?;

        // Get confidence
        let confidence = tess.mean_text_conf();

        Ok(OcrResult::new(
            text,
            confidence,
            self.config.min_confidence,
        ))
    }

    /// Mock recognition when OCR feature is not enabled
    #[cfg(not(feature = "ocr"))]
    pub fn recognize(&self, _img: &GrayImage) -> RecognizeResult<OcrResult> {
        // Return a mock result for testing
        Ok(OcrResult::new(
            "Mock Card".to_string(),
            95,
            self.config.min_confidence,
        ))
    }

    /// Recognize text from multiple images
    pub fn recognize_multiple(&self, images: &[GrayImage]) -> Vec<RecognizeResult<OcrResult>> {
        images.iter().map(|img| self.recognize(img)).collect()
    }
}

impl Default for OcrEngine {
    fn default() -> Self {
        Self::new().expect("Failed to create default OCR engine")
    }
}

/// Card name matcher using fuzzy string matching
pub struct CardMatcher {
    card_names: Vec<(String, String)>, // (card_id, card_name)
    matcher: SkimMatcherV2,
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
            matcher: SkimMatcherV2::default(),
            min_score,
        })
    }

    /// Find the best matching card for the given OCR text
    pub fn find_best_match(&self, ocr_text: &str) -> Option<CardMatch> {
        let ocr_normalized = ocr_text.to_lowercase().trim().to_string();
        
        if ocr_normalized.is_empty() {
            return None;
        }

        let mut best_match: Option<CardMatch> = None;
        let mut best_score = self.min_score as i64;

        for (card_id, card_name) in &self.card_names {
            // Try fuzzy matching
            if let Some(score) = self.matcher.fuzzy_match(&card_name.to_lowercase(), &ocr_normalized) {
                if score > best_score {
                    best_score = score;
                    best_match = Some(CardMatch {
                        card_name: card_name.clone(),
                        card_id: card_id.clone(),
                        ocr_text: ocr_text.to_string(),
                        match_score: score.min(100) as i32,
                        ocr_confidence: 0, // Will be set by caller
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

    /// Find all cards that match above the threshold (for ambiguous matches)
    pub fn find_all_matches(&self, ocr_text: &str, threshold: i32) -> Vec<CardMatch> {
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

    /// Process a single image through the full pipeline
    pub fn process(&self, img: &GrayImage) -> RecognizeResult<Option<CardMatch>> {
        let ocr_result = self.ocr_engine.recognize(img)?;

        if !ocr_result.is_confident {
            return Ok(None);
        }

        Ok(self.card_matcher.find_best_match(&ocr_result.text))
    }

    /// Process multiple images through the full pipeline
    pub fn process_multiple(&self, images: &[GrayImage]) -> Vec<CardMatch> {
        let ocr_results: Vec<OcrResult> = self
            .ocr_engine
            .recognize_multiple(images)
            .into_iter()
            .filter_map(|r| r.ok())
            .filter(|r| r.is_confident)
            .collect();

        self.card_matcher.match_results(ocr_results)
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
pub fn build_card_map(cards: Vec<(String, String)>) -> HashMap<String, String> {
    cards.into_iter().collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_card_names() -> Vec<(String, String)> {
        vec![
            ("banished_fel".to_string(), "Fel".to_string()),
            ("banished_talos".to_string(), "Talos".to_string()),
            ("banished_just_cause".to_string(), "Just Cause".to_string()),
            ("banished_cleave".to_string(), "Cleave".to_string()),
            ("pyreborne_lord_fenix".to_string(), "Lord Fenix".to_string()),
            ("underlegion_bolete".to_string(), "Bolete the Guillotine".to_string()),
        ]
    }

    #[test]
    fn test_ocr_result_new() {
        let result = OcrResult::new("Fel".to_string(), 85, 60);
        assert_eq!(result.text, "Fel");
        assert_eq!(result.confidence, 85);
        assert!(result.is_confident);

        let result_low = OcrResult::new("Talos".to_string(), 50, 60);
        assert!(!result_low.is_confident);
    }

    #[test]
    fn test_ocr_result_normalized() {
        let result = OcrResult::new("  FEL  ".to_string(), 85, 60);
        assert_eq!(result.normalized_text(), "fel");
    }

    #[test]
    fn test_card_match_overall_confidence() {
        let conf = CardMatch::calculate_overall_confidence(80, 80);
        assert!((conf - 0.8).abs() < 0.001);

        // Weighted: 0.4 * 80 + 0.6 * 60 = 68 / 100 = 0.68
        let conf2 = CardMatch::calculate_overall_confidence(80, 60);
        assert!((conf2 - 0.68).abs() < 0.001);
    }

    #[test]
    fn test_card_matcher_new_empty() {
        let result = CardMatcher::new(vec![], 60);
        assert!(matches!(result, Err(RecognizeError::NoCardNamesAvailable)));
    }

    #[test]
    fn test_card_matcher_find_best_match() {
        let cards = create_test_card_names();
        let matcher = CardMatcher::new(cards, 60).unwrap();

        // Exact match
        let result = matcher.find_best_match("Fel");
        assert!(result.is_some());
        let m = result.unwrap();
        assert_eq!(m.card_name, "Fel");
        assert!(m.match_score >= 60);

        // Fuzzy match
        let result = matcher.find_best_match("Fell");
        assert!(result.is_some());
        assert_eq!(result.unwrap().card_name, "Fel");

        // Case insensitive
        let result = matcher.find_best_match("FEL");
        assert!(result.is_some());
        assert_eq!(result.unwrap().card_name, "Fel");

        // No match
        let result = matcher.find_best_match("NonExistentCard123");
        assert!(result.is_none());

        // Empty string
        let result = matcher.find_best_match("");
        assert!(result.is_none());
    }

    #[test]
    fn test_card_matcher_partial_match() {
        let cards = create_test_card_names();
        let matcher = CardMatcher::new(cards, 60).unwrap();

        // Partial match on multi-word name
        let result = matcher.find_best_match("Guillotine");
        assert!(result.is_some());
        assert_eq!(result.unwrap().card_name, "Bolete the Guillotine");

        // Match on "the"
        let result = matcher.find_best_match("Just");
        assert!(result.is_some());
        assert_eq!(result.unwrap().card_name, "Just Cause");
    }

    #[test]
    fn test_card_matcher_find_all_matches() {
        let cards = create_test_card_names();
        let matcher = CardMatcher::new(cards, 60).unwrap();

        let results = matcher.find_all_matches("Fe", 50);
        assert!(!results.is_empty());
        // Should find "Fel" and possibly "Lord Fenix"
    }

    #[test]
    fn test_recognize_config_default() {
        let config = RecognizeConfig::default();
        assert_eq!(config.language, "eng");
        assert_eq!(config.psm, 7);
        assert_eq!(config.oem, 3);
        assert_eq!(config.min_confidence, 60);
        assert!(config.whitelist.is_some());
    }

    #[test]
    fn test_recognize_config_with_language() {
        let config = RecognizeConfig::with_language("fra");
        assert_eq!(config.language, "fra");
    }

    #[test]
    fn test_normalize_card_name() {
        assert_eq!(normalize_card_name("Fel"), "fel");
        assert_eq!(normalize_card_name("Lord Fenix"), "lord fenix");
        assert_eq!(normalize_card_name("  Multiple   Spaces  "), "multiple spaces");
        assert_eq!(normalize_card_name("Card-Name!"), "cardname");
    }

    #[test]
    fn test_build_card_map() {
        let cards = create_test_card_names();
        let map = build_card_map(cards);
        
        assert_eq!(map.get("banished_fel"), Some(&"Fel".to_string()));
        assert_eq!(map.get("pyreborne_lord_fenix"), Some(&"Lord Fenix".to_string()));
    }

    #[test]
    fn test_error_display() {
        assert!(RecognizeError::InvalidImage.to_string().contains("Invalid"));
        assert!(RecognizeError::NoCardNamesAvailable.to_string().contains("No card names"));
        assert!(RecognizeError::TesseractError("test".to_string()).to_string().contains("test"));
    }

    // Note: Tests that actually call Tesseract are integration tests
    // and would require Tesseract to be installed. We skip those here.
}
