//! OCR pipeline implementation
//!
//! This module provides a complete OCR pipeline for detecting card names
//! in Monster Train 2 screenshots. The pipeline consists of:
//!
//! 1. **Capture** (`capture`): Screen capture of specific regions
//! 2. **Preprocess** (`preprocess`): Image preprocessing for better OCR accuracy
//! 3. **Recognize** (`recognize`): Tesseract OCR and card name matching
//!
//! # Example Usage
//!
//! ```rust,ignore
//! use mt2_draft_assistant::ocr::{OcrPipeline, CardDetectionOptions};
//!
//! let options = CardDetectionOptions::default();
//! let pipeline = OcrPipeline::new(card_names, options)?;
//! let detected_cards = pipeline.detect_cards()?;
//! ```

// Conditional compilation for OCR feature - real modules
#[cfg(feature = "ocr")]
pub mod capture;
#[cfg(feature = "ocr")]
pub mod preprocess;
#[cfg(feature = "ocr")]
pub mod recognize;

// Mock implementations when OCR feature is disabled
#[cfg(not(feature = "ocr"))]
mod mock;

// Re-export mock module items as if they were submodules
#[cfg(not(feature = "ocr"))]
pub mod capture {
    pub use super::mock::{
        CaptureConfig, CaptureError, CaptureRegion, CaptureResult,
        capture_multiple_regions, capture_region, get_default_card_regions,
        get_primary_screen_dimensions,
    };
}

#[cfg(not(feature = "ocr"))]
pub mod preprocess {
    pub use super::mock::{
        PreprocessConfig, PreprocessError, PreprocessResult,
        preprocess_default, preprocess_for_ocr, save_debug_image,
    };
}

#[cfg(not(feature = "ocr"))]
pub mod recognize {
    pub use super::mock::{
        CardMatch, OcrEngine, OcrResult, RecognizeConfig, RecognizeError,
        RecognizeResult, RecognitionPipeline, normalize_card_name, build_card_map,
    };
}

// Re-export commonly used types at the module level for convenience
pub use capture::{
    CaptureConfig, CaptureError, CaptureRegion, CaptureResult,
    capture_multiple_regions, capture_region, get_default_card_regions,
    get_primary_screen_dimensions,
};

pub use preprocess::{
    PreprocessConfig, PreprocessError, PreprocessResult,
    preprocess_default, preprocess_for_ocr, save_debug_image,
};

pub use recognize::{
    CardMatch, OcrEngine, OcrResult, RecognizeConfig, RecognizeError,
    RecognizeResult, RecognitionPipeline, normalize_card_name, build_card_map,
};

use std::path::PathBuf;

/// Error type for OCR pipeline operations
#[derive(Debug)]
pub enum OcrPipelineError {
    Capture(capture::CaptureError),
    Preprocess(preprocess::PreprocessError),
    Recognize(recognize::RecognizeError),
    Configuration(String),
}

impl std::fmt::Display for OcrPipelineError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OcrPipelineError::Capture(e) => write!(f, "Capture error: {}", e),
            OcrPipelineError::Preprocess(e) => write!(f, "Preprocess error: {}", e),
            OcrPipelineError::Recognize(e) => write!(f, "Recognize error: {}", e),
            OcrPipelineError::Configuration(msg) => write!(f, "Configuration error: {}", msg),
        }
    }
}

impl std::error::Error for OcrPipelineError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            OcrPipelineError::Capture(e) => Some(e),
            OcrPipelineError::Preprocess(e) => Some(e),
            OcrPipelineError::Recognize(e) => Some(e),
            OcrPipelineError::Configuration(_) => None,
        }
    }
}

impl From<capture::CaptureError> for OcrPipelineError {
    fn from(err: capture::CaptureError) -> Self {
        OcrPipelineError::Capture(err)
    }
}

impl From<preprocess::PreprocessError> for OcrPipelineError {
    fn from(err: preprocess::PreprocessError) -> Self {
        OcrPipelineError::Preprocess(err)
    }
}

impl From<recognize::RecognizeError> for OcrPipelineError {
    fn from(err: recognize::RecognizeError) -> Self {
        OcrPipelineError::Recognize(err)
    }
}

/// Result type for OCR pipeline operations
pub type OcrPipelineResult<T> = Result<T, OcrPipelineError>;

/// Options for card detection
#[derive(Debug, Clone)]
pub struct CardDetectionOptions {
    /// Capture configuration
    pub capture: CaptureConfig,
    /// Preprocessing configuration
    pub preprocess: PreprocessConfig,
    /// Recognition configuration
    pub recognize: RecognizeConfig,
    /// Whether to save debug images
    pub save_debug_images: bool,
    /// Path for debug images (if enabled)
    pub debug_image_path: Option<PathBuf>,
    /// Minimum overall confidence for a valid detection (0.0-1.0)
    pub min_overall_confidence: f64,
}

impl Default for CardDetectionOptions {
    fn default() -> Self {
        Self {
            capture: CaptureConfig::default(),
            preprocess: PreprocessConfig::default(),
            recognize: RecognizeConfig::default(),
            save_debug_images: false,
            debug_image_path: None,
            min_overall_confidence: 0.6,
        }
    }
}

impl CardDetectionOptions {
    /// Create with custom capture regions
    pub fn with_regions(regions: Vec<CaptureRegion>) -> OcrPipelineResult<Self> {
        let capture = CaptureConfig::with_regions(regions)?;
        Ok(Self {
            capture,
            ..Default::default()
        })
    }

    /// Enable debug image saving
    pub fn with_debug_images(mut self, path: PathBuf) -> Self {
        self.save_debug_images = true;
        self.debug_image_path = Some(path);
        self
    }
}

/// Individual card detection result
#[derive(Debug, Clone, PartialEq)]
pub struct DetectedCard {
    /// Card ID from database
    pub card_id: String,
    /// Card name
    pub card_name: String,
    /// Region where the card was detected
    pub region: CaptureRegion,
    /// OCR confidence (0-100)
    pub ocr_confidence: i32,
    /// Match score (0-100)
    pub match_score: i32,
    /// Overall confidence (0.0-1.0)
    pub overall_confidence: f64,
    /// Raw OCR text
    pub raw_ocr_text: String,
}

impl DetectedCard {
    /// Check if the detection is considered confident enough
    pub fn is_confident(&self, threshold: f64) -> bool {
        self.overall_confidence >= threshold
    }
}

/// Result of card detection operation
#[derive(Debug, Clone, PartialEq)]
pub struct CardDetectionResult {
    /// List of detected cards
    pub detected_cards: Vec<DetectedCard>,
    /// Average confidence across all detections
    pub average_confidence: f64,
    /// Whether the detection was successful
    pub success: bool,
    /// Error message if detection failed
    pub error_message: Option<String>,
}

impl CardDetectionResult {
    /// Create a new successful detection result
    pub fn new(detected_cards: Vec<DetectedCard>) -> Self {
        let avg_confidence = if detected_cards.is_empty() {
            0.0
        } else {
            detected_cards.iter().map(|c| c.overall_confidence).sum::<f64>() / detected_cards.len() as f64
        };

        Self {
            detected_cards,
            average_confidence: avg_confidence,
            success: true,
            error_message: None,
        }
    }

    /// Create a failed detection result
    pub fn failed(error: impl ToString) -> Self {
        Self {
            detected_cards: vec![],
            average_confidence: 0.0,
            success: false,
            error_message: Some(error.to_string()),
        }
    }

    /// Get only confident detections
    pub fn confident_detections(&self, threshold: f64) -> Vec<&DetectedCard> {
        self.detected_cards
            .iter()
            .filter(|c| c.is_confident(threshold))
            .collect()
    }

    /// Get the number of detected cards
    pub fn len(&self) -> usize {
        self.detected_cards.len()
    }

    /// Check if any cards were detected
    pub fn is_empty(&self) -> bool {
        self.detected_cards.is_empty()
    }
}

/// High-level OCR pipeline for card detection
pub struct OcrPipeline {
    recognition_pipeline: RecognitionPipeline,
    options: CardDetectionOptions,
    card_names: Vec<(String, String)>,
}

impl OcrPipeline {
    /// Create a new OCR pipeline with the given card names and options
    ///
    /// # Arguments
    /// * `card_names` - Vector of (card_id, card_name) tuples from the database
    /// * `options` - Detection options including capture regions, preprocessing, etc.
    pub fn new(
        card_names: Vec<(String, String)>,
        options: CardDetectionOptions,
    ) -> OcrPipelineResult<Self> {
        let recognition_pipeline = RecognitionPipeline::with_config(
            card_names.clone(),
            options.recognize.clone(),
        )?;

        Ok(Self {
            recognition_pipeline,
            options,
            card_names,
        })
    }

    /// Create with default options
    pub fn with_default_options(card_names: Vec<(String, String)>) -> OcrPipelineResult<Self> {
        Self::new(card_names, CardDetectionOptions::default())
    }

    /// Detect cards on screen using the configured regions
    pub fn detect_cards(&self) -> OcrPipelineResult<CardDetectionResult> {
        // Step 1: Capture screen regions
        let capture_results = self.options.capture.capture_all();

        let mut detected_cards = Vec::new();
        let mut debug_image_index = 0;

        for (i, capture_result) in capture_results.into_iter().enumerate() {
            match capture_result {
                Ok(rgba_image) => {
                    // Step 2: Preprocess
                    let gray_image = match preprocess_for_ocr(&rgba_image, &self.options.preprocess) {
                        Ok(img) => img,
                        Err(e) => {
                            log::warn!("Preprocessing failed for region {}: {}", i, e);
                            continue;
                        }
                    };

                    // Save debug image if enabled
                    if self.options.save_debug_images {
                        if let Some(ref path) = self.options.debug_image_path {
                            let debug_path = path.join(format!("debug_region_{}.png", debug_image_index));
                            let _ = save_debug_image(&gray_image, &debug_path);
                            debug_image_index += 1;
                        }
                    }

                    // Step 3: Recognize
                    match self.recognition_pipeline.process(&gray_image) {
                        Ok(Some(card_match)) => {
                            if card_match.overall_confidence >= self.options.min_overall_confidence {
                                let region = self.options.capture.get_regions().get(i).copied()
                                    .unwrap_or_else(|| CaptureRegion::new(0, 0, 0, 0));

                                detected_cards.push(DetectedCard {
                                    card_id: card_match.card_id,
                                    card_name: card_match.card_name,
                                    region,
                                    ocr_confidence: card_match.ocr_confidence,
                                    match_score: card_match.match_score,
                                    overall_confidence: card_match.overall_confidence,
                                    raw_ocr_text: card_match.ocr_text,
                                });
                            }
                        }
                        Ok(None) => {
                            log::debug!("No card detected in region {}", i);
                        }
                        Err(e) => {
                            log::warn!("Recognition failed for region {}: {}", i, e);
                        }
                    }
                }
                Err(e) => {
                    log::warn!("Capture failed for region {}: {}", i, e);
                }
            }
        }

        Ok(CardDetectionResult::new(detected_cards))
    }

    /// Update capture regions
    pub fn update_regions(&mut self, regions: Vec<CaptureRegion>) {
        self.options.capture.update_regions(regions);
    }

    /// Get current capture regions
    pub fn get_regions(&self) -> &[CaptureRegion] {
        self.options.capture.get_regions()
    }

    /// Get the list of card names this pipeline can recognize
    pub fn available_card_names(&self) -> &[(String, String)] {
        &self.card_names
    }
}

/// Convenience function to quickly detect cards with default settings
///
/// # Arguments
/// * `card_names` - Vector of (card_id, card_name) tuples
///
/// # Returns
/// Result containing detected cards or an error
pub fn quick_detect(card_names: Vec<(String, String)>) -> OcrPipelineResult<CardDetectionResult> {
    let pipeline = OcrPipeline::with_default_options(card_names)?;
    pipeline.detect_cards()
}

/// Calibrate capture regions by testing different configurations
///
/// This function helps find the best capture regions for a given screen
/// by testing the current configuration and returning capture statistics.
pub fn calibrate_regions(options: &CardDetectionOptions) -> OcrPipelineResult<CalibrationReport> {
    let dimensions = get_primary_screen_dimensions()?;
    let regions = options.capture.get_regions();

    // Test capture of each region
    let mut successful_captures = 0;
    let mut failed_captures = 0;

    for region in regions {
        match capture_region(region) {
            Ok(_) => successful_captures += 1,
            Err(_) => failed_captures += 1,
        }
    }

    Ok(CalibrationReport {
        screen_dimensions: dimensions,
        regions_tested: regions.len(),
        successful_captures,
        failed_captures,
        recommended_regions: get_default_card_regions(dimensions.0, dimensions.1),
    })
}

/// Report from calibration operation
#[derive(Debug, Clone)]
pub struct CalibrationReport {
    /// Screen dimensions (width, height)
    pub screen_dimensions: (u32, u32),
    /// Number of regions tested
    pub regions_tested: usize,
    /// Number of successful captures
    pub successful_captures: usize,
    /// Number of failed captures
    pub failed_captures: usize,
    /// Recommended regions based on screen size
    pub recommended_regions: Vec<CaptureRegion>,
}

impl CalibrationReport {
    /// Check if calibration was successful
    pub fn is_successful(&self) -> bool {
        self.failed_captures == 0 && self.successful_captures > 0
    }

    /// Get success rate as a percentage
    pub fn success_rate(&self) -> f64 {
        let total = self.successful_captures + self.failed_captures;
        if total == 0 {
            0.0
        } else {
            (self.successful_captures as f64 / total as f64) * 100.0
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detected_card_is_confident() {
        let card = DetectedCard {
            card_id: "test".to_string(),
            card_name: "Test Card".to_string(),
            region: CaptureRegion::new(0, 0, 100, 100),
            ocr_confidence: 80,
            match_score: 90,
            overall_confidence: 0.85,
            raw_ocr_text: "Test".to_string(),
        };

        assert!(card.is_confident(0.8));
        assert!(!card.is_confident(0.9));
    }

    #[test]
    fn test_card_detection_result_new() {
        let cards = vec![
            DetectedCard {
                card_id: "1".to_string(),
                card_name: "Card 1".to_string(),
                region: CaptureRegion::new(0, 0, 100, 100),
                ocr_confidence: 80,
                match_score: 85,
                overall_confidence: 0.8,
                raw_ocr_text: "Card 1".to_string(),
            },
            DetectedCard {
                card_id: "2".to_string(),
                card_name: "Card 2".to_string(),
                region: CaptureRegion::new(100, 0, 100, 100),
                ocr_confidence: 85,
                match_score: 90,
                overall_confidence: 0.85,
                raw_ocr_text: "Card 2".to_string(),
            },
        ];

        let result = CardDetectionResult::new(cards);
        assert!(result.success);
        assert_eq!(result.len(), 2);
        assert!((result.average_confidence - 0.825).abs() < 0.001);
        assert!(result.error_message.is_none());
    }

    #[test]
    fn test_card_detection_result_empty() {
        let result = CardDetectionResult::new(vec![]);
        assert!(result.success);
        assert!(result.is_empty());
        assert_eq!(result.average_confidence, 0.0);
    }

    #[test]
    fn test_card_detection_result_failed() {
        let result = CardDetectionResult::failed("Test error");
        assert!(!result.success);
        assert!(result.is_empty());
        assert_eq!(result.error_message, Some("Test error".to_string()));
    }

    #[test]
    fn test_card_detection_result_confident_detections() {
        let cards = vec![
            DetectedCard {
                card_id: "1".to_string(),
                card_name: "Card 1".to_string(),
                region: CaptureRegion::new(0, 0, 100, 100),
                ocr_confidence: 80,
                match_score: 85,
                overall_confidence: 0.5,
                raw_ocr_text: "Card 1".to_string(),
            },
            DetectedCard {
                card_id: "2".to_string(),
                card_name: "Card 2".to_string(),
                region: CaptureRegion::new(100, 0, 100, 100),
                ocr_confidence: 85,
                match_score: 90,
                overall_confidence: 0.8,
                raw_ocr_text: "Card 2".to_string(),
            },
        ];

        let result = CardDetectionResult::new(cards);
        let confident = result.confident_detections(0.7);
        assert_eq!(confident.len(), 1);
        assert_eq!(confident[0].card_id, "2");
    }

    #[test]
    fn test_calibration_report() {
        let report = CalibrationReport {
            screen_dimensions: (1920, 1080),
            regions_tested: 4,
            successful_captures: 3,
            failed_captures: 1,
            recommended_regions: vec![CaptureRegion::new(0, 0, 100, 100)],
        };

        assert!(!report.is_successful());
        assert!((report.success_rate() - 75.0).abs() < 0.001);
    }

    #[test]
    fn test_calibration_report_successful() {
        let report = CalibrationReport {
            screen_dimensions: (1920, 1080),
            regions_tested: 4,
            successful_captures: 4,
            failed_captures: 0,
            recommended_regions: vec![],
        };

        assert!(report.is_successful());
        assert_eq!(report.success_rate(), 100.0);
    }

    #[test]
    fn test_card_detection_options_default() {
        let options = CardDetectionOptions::default();
        assert!(!options.save_debug_images);
        assert!(options.debug_image_path.is_none());
        assert!(options.min_overall_confidence > 0.0);
    }

    #[test]
    fn test_ocr_pipeline_error_display() {
        let err = OcrPipelineError::Configuration("test".to_string());
        assert!(err.to_string().contains("test"));
    }

    #[test]
    fn test_error_from_conversions() {
        let capture_err = CaptureError::InvalidRegion;
        let pipeline_err: OcrPipelineError = capture_err.into();
        assert!(matches!(pipeline_err, OcrPipelineError::Capture(_)));

        let preprocess_err = PreprocessError::EmptyImage;
        let pipeline_err: OcrPipelineError = preprocess_err.into();
        assert!(matches!(pipeline_err, OcrPipelineError::Preprocess(_)));
    }
}
