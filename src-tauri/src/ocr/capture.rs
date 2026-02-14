//! Screen capture functionality for OCR
//!
//! This module provides functionality to capture specific screen regions
//! where card names appear in Monster Train 2.

use image::{ImageBuffer, Rgba};
use std::fmt;

#[cfg(feature = "ocr")]
use screenshots::Screen;

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

impl fmt::Display for CaptureRegion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
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

impl fmt::Display for CaptureError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
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

/// Captures a specific region of the primary screen
#[cfg(feature = "ocr")]
pub fn capture_region(region: &CaptureRegion) -> CaptureResult<ImageBuffer<Rgba<u8>, Vec<u8>>> {
    if !region.is_valid() {
        return Err(CaptureError::InvalidRegion);
    }

    let screens = Screen::all().map_err(|e| CaptureError::CaptureFailed(e.to_string()))?;
    
    if screens.is_empty() {
        return Err(CaptureError::NoScreensAvailable);
    }

    // Use the primary screen (usually the first one)
    let screen = &screens[0];
    
    // Check if region is within screen bounds
    let screen_width = screen.display_info.width as i32;
    let screen_height = screen.display_info.height as i32;
    
    if region.x < 0 
        || region.y < 0 
        || region.x + region.width as i32 > screen_width 
        || region.y + region.height as i32 > screen_height {
        return Err(CaptureError::RegionOutOfBounds);
    }

    // Capture the region
    let image = screen
        .capture_area(region.x, region.y, region.width, region.height)
        .map_err(|e| CaptureError::CaptureFailed(e.to_string()))?;

    // Convert to image::ImageBuffer
    let img_buffer = ImageBuffer::from_raw(region.width, region.height, image.to_vec())
        .ok_or_else(|| CaptureError::CaptureFailed("Failed to create image buffer".to_string()))?;

    Ok(img_buffer)
}

/// Mock implementation when OCR feature is not enabled
#[cfg(not(feature = "ocr"))]
pub fn capture_region(_region: &CaptureRegion) -> CaptureResult<ImageBuffer<Rgba<u8>, Vec<u8>>> {
    // Return a blank image for testing purposes
    let width = 100u32;
    let height = 50u32;
    let img = ImageBuffer::from_fn(width, height, |_, _| Rgba([255, 255, 255, 255]));
    Ok(img)
}

/// Captures multiple regions and returns them as a vector
pub fn capture_multiple_regions(regions: &[CaptureRegion]) -> Vec<CaptureResult<ImageBuffer<Rgba<u8>, Vec<u8>>>> {
    regions.iter().map(capture_region).collect()
}

/// Default card name regions for Monster Train 2 draft screen
/// These are approximate positions and should be calibrated per resolution
pub fn get_default_card_regions(screen_width: u32, screen_height: u32) -> Vec<CaptureRegion> {
    // These positions are estimates for 1920x1080 and need scaling
    let scale_x = screen_width as f32 / 1920.0;
    let scale_y = screen_height as f32 / 1080.0;

    // Card name regions in the draft selection area (typically 3-4 cards)
    // Positions are relative to a typical draft layout
    let base_regions = vec![
        // Left card name area
        CaptureRegion::new(350, 200, 300, 60),
        // Center card name area
        CaptureRegion::new(810, 200, 300, 60),
        // Right card name area
        CaptureRegion::new(1270, 200, 300, 60),
        // Fourth card (if present in some draft modes)
        CaptureRegion::new(810, 500, 300, 60),
    ];

    // Scale regions based on screen resolution
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

/// Gets the primary screen dimensions
#[cfg(feature = "ocr")]
pub fn get_primary_screen_dimensions() -> CaptureResult<(u32, u32)> {
    let screens = Screen::all().map_err(|e| CaptureError::CaptureFailed(e.to_string()))?;
    
    if screens.is_empty() {
        return Err(CaptureError::NoScreensAvailable);
    }

    let info = &screens[0].display_info;
    Ok((info.width, info.height))
}

/// Mock implementation when OCR feature is not enabled
#[cfg(not(feature = "ocr"))]
pub fn get_primary_screen_dimensions() -> CaptureResult<(u32, u32)> {
    // Return default dimensions
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

    /// Capture all configured regions
    pub fn capture_all(&self) -> Vec<CaptureResult<ImageBuffer<Rgba<u8>, Vec<u8>>>> {
        capture_multiple_regions(&self.regions)
    }
}

impl Default for CaptureConfig {
    fn default() -> Self {
        // Use 1920x1080 as default, will be updated on first capture
        let regions = get_default_card_regions(1920, 1080);
        Self {
            regions,
            screen_width: 1920,
            screen_height: 1080,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_capture_region_new() {
        let region = CaptureRegion::new(100, 200, 300, 400);
        assert_eq!(region.x, 100);
        assert_eq!(region.y, 200);
        assert_eq!(region.width, 300);
        assert_eq!(region.height, 400);
    }

    #[test]
    fn test_capture_region_valid() {
        assert!(CaptureRegion::new(0, 0, 100, 100).is_valid());
        assert!(CaptureRegion::new(10, 10, 1, 1).is_valid());
        assert!(!CaptureRegion::new(0, 0, 0, 100).is_valid());
        assert!(!CaptureRegion::new(0, 0, 100, 0).is_valid());
    }

    #[test]
    fn test_capture_region_contains() {
        let region = CaptureRegion::new(100, 100, 200, 150);
        assert!(region.contains(100, 100)); // Top-left corner
        assert!(region.contains(299, 249)); // Bottom-right corner (inside)
        assert!(!region.contains(99, 100)); // Left of region
        assert!(!region.contains(100, 99)); // Above region
        assert!(!region.contains(300, 100)); // Right of region
        assert!(!region.contains(100, 250)); // Below region
    }

    #[test]
    fn test_capture_region_display() {
        let region = CaptureRegion::new(10, 20, 100, 200);
        let display = format!("{}", region);
        assert!(display.contains("x=10"));
        assert!(display.contains("y=20"));
        assert!(display.contains("w=100"));
        assert!(display.contains("h=200"));
    }

    #[test]
    fn test_capture_error_display() {
        assert_eq!(
            format!("{}", CaptureError::NoScreensAvailable),
            "No screens available for capture"
        );
        assert_eq!(
            format!("{}", CaptureError::InvalidRegion),
            "Invalid capture region (zero or negative dimensions)"
        );
        assert_eq!(
            format!("{}", CaptureError::RegionOutOfBounds),
            "Capture region is outside screen bounds"
        );
        assert_eq!(
            format!("{}", CaptureError::CaptureFailed("test".to_string())),
            "Screen capture failed: test"
        );
    }

    #[test]
    fn test_get_default_card_regions() {
        let regions = get_default_card_regions(1920, 1080);
        assert!(!regions.is_empty());
        
        // All regions should be valid
        for region in &regions {
            assert!(region.is_valid(), "Region {} should be valid", region);
        }

        // Test scaling
        let scaled_regions = get_default_card_regions(3840, 2160);
        assert_eq!(regions.len(), scaled_regions.len());
        
        // Scaled regions should be approximately 2x for 4K
        assert!(scaled_regions[0].x > regions[0].x);
    }

    #[test]
    fn test_capture_config_default() {
        let config = CaptureConfig::default();
        assert!(!config.regions.is_empty());
        assert_eq!(config.screen_width, 1920);
        assert_eq!(config.screen_height, 1080);
    }

    #[test]
    fn test_capture_config_update_regions() {
        let mut config = CaptureConfig::default();
        let new_regions = vec![CaptureRegion::new(0, 0, 100, 100)];
        config.update_regions(new_regions.clone());
        assert_eq!(config.regions.len(), 1);
        assert_eq!(config.regions[0].x, 0);
    }
}
