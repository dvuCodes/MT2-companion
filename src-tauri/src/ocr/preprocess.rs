//! Image preprocessing for OCR
//!
//! This module provides image preprocessing operations to improve OCR accuracy:
//! - Grayscale conversion
//! - Adaptive thresholding
//! - Noise reduction
//! - Contrast enhancement

use image::{GrayImage, ImageBuffer, Luma, Rgba};

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
    /// Threshold value for binary conversion (0-255)
    pub threshold: u8,
    /// Whether to use adaptive thresholding
    pub use_adaptive_threshold: bool,
    /// Block size for adaptive thresholding (must be odd)
    pub adaptive_block_size: u32,
    /// Constant C for adaptive thresholding
    pub adaptive_c: i32,
    /// Whether to apply denoising
    pub denoise: bool,
    /// Whether to invert colors (white text on black background)
    pub invert: bool,
    /// Scale factor for upscaling (1.0 = no scaling)
    pub scale_factor: f32,
    /// Contrast enhancement factor (1.0 = no enhancement)
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
            scale_factor: 2.0, // Upscale by 2x for better OCR
            contrast_factor: 1.5,
        }
    }
}

/// Convert an RGBA image to grayscale
pub fn to_grayscale(img: &ImageBuffer<Rgba<u8>, Vec<u8>>) -> GrayImage {
    image::imageops::grayscale(img)
}

/// Apply binary thresholding to a grayscale image
pub fn apply_threshold(img: &GrayImage, threshold: u8) -> GrayImage {
    let mut result = img.clone();
    
    for pixel in result.pixels_mut() {
        let Luma([value]) = *pixel;
        let new_value = if value > threshold { 255 } else { 0 };
        *pixel = Luma([new_value]);
    }
    
    result
}

/// Apply adaptive thresholding using mean method
/// This is more robust to varying lighting conditions
pub fn apply_adaptive_threshold(img: &GrayImage, block_size: u32, c: i32) -> GrayImage {
    if block_size == 0 || block_size % 2 == 0 {
        // Block size must be odd and positive
        return img.clone();
    }

    let (width, height) = img.dimensions();
    let mut result = GrayImage::new(width, height);
    let half_block = (block_size / 2) as i32;

    for y in 0..height {
        for x in 0..width {
            // Calculate mean of neighborhood
            let mut sum: u32 = 0;
            let mut count: u32 = 0;

            for dy in -half_block..=half_block {
                for dx in -half_block..=half_block {
                    let nx = x as i32 + dx;
                    let ny = y as i32 + dy;

                    if nx >= 0 && nx < width as i32 && ny >= 0 && ny < height as i32 {
                        sum += img.get_pixel(nx as u32, ny as u32)[0] as u32;
                        count += 1;
                    }
                }
            }

            let mean = if count > 0 { (sum / count) as i32 } else { 128 };
            let pixel_value = img.get_pixel(x, y)[0] as i32;
            
            // Apply threshold: pixel > (mean - C) ? white : black
            let threshold_value = mean - c;
            let new_value = if pixel_value > threshold_value { 255 } else { 0 };
            
            result.put_pixel(x, y, Luma([new_value as u8]));
        }
    }

    result
}

/// Apply Gaussian blur for noise reduction
pub fn apply_gaussian_blur(img: &GrayImage, sigma: f32) -> GrayImage {
    image::imageops::blur(img, sigma)
}

/// Simple median filter for noise reduction
pub fn apply_median_filter(img: &GrayImage, kernel_size: u32) -> GrayImage {
    if kernel_size < 3 || kernel_size % 2 == 0 {
        return img.clone();
    }

    let (width, height) = img.dimensions();
    let mut result = GrayImage::new(width, height);
    let half_kernel = (kernel_size / 2) as i32;

    for y in 0..height {
        for x in 0..width {
            let mut values: Vec<u8> = Vec::new();

            for dy in -half_kernel..=half_kernel {
                for dx in -half_kernel..=half_kernel {
                    let nx = x as i32 + dx;
                    let ny = y as i32 + dy;

                    if nx >= 0 && nx < width as i32 && ny >= 0 && ny < height as i32 {
                        values.push(img.get_pixel(nx as u32, ny as u32)[0]);
                    }
                }
            }

            values.sort_unstable();
            let median = values[values.len() / 2];
            result.put_pixel(x, y, Luma([median]));
        }
    }

    result
}

/// Invert image colors
pub fn invert(img: &GrayImage) -> GrayImage {
    let mut result = img.clone();
    
    for pixel in result.pixels_mut() {
        let Luma([value]) = *pixel;
        *pixel = Luma([255 - value]);
    }
    
    result
}

/// Enhance contrast using histogram stretching
pub fn enhance_contrast(img: &GrayImage, factor: f32) -> GrayImage {
    let mut result = img.clone();
    
    for pixel in result.pixels_mut() {
        let Luma([value]) = *pixel;
        let new_value = ((value as f32 - 128.0) * factor + 128.0)
            .clamp(0.0, 255.0) as u8;
        *pixel = Luma([new_value]);
    }
    
    result
}

/// Scale up image for better OCR accuracy
pub fn upscale(img: &GrayImage, factor: f32) -> GrayImage {
    if factor <= 1.0 {
        return img.clone();
    }

    let (width, height) = img.dimensions();
    let new_width = (width as f32 * factor) as u32;
    let new_height = (height as f32 * factor) as u32;

    image::imageops::resize(
        img,
        new_width,
        new_height,
        image::imageops::FilterType::Lanczos3,
    )
}

/// Main preprocessing pipeline for OCR
/// 
/// # Arguments
/// * `img` - Input RGBA image
/// * `config` - Preprocessing configuration
/// 
/// # Returns
/// Preprocessed grayscale image ready for OCR
pub fn preprocess_for_ocr(
    img: &ImageBuffer<Rgba<u8>, Vec<u8>>,
    config: &PreprocessConfig,
) -> PreprocessResult<GrayImage> {
    if img.width() == 0 || img.height() == 0 {
        return Err(PreprocessError::EmptyImage);
    }

    // Step 1: Convert to grayscale
    let mut processed = to_grayscale(img);

    // Step 2: Enhance contrast
    if config.contrast_factor != 1.0 {
        processed = enhance_contrast(&processed, config.contrast_factor);
    }

    // Step 3: Upscale for better OCR accuracy
    if config.scale_factor > 1.0 {
        processed = upscale(&processed, config.scale_factor);
    }

    // Step 4: Denoise
    if config.denoise {
        // Apply mild Gaussian blur followed by median filter
        processed = apply_gaussian_blur(&processed, 0.5);
        processed = apply_median_filter(&processed, 3);
    }

    // Step 5: Apply thresholding
    if config.use_adaptive_threshold {
        processed = apply_adaptive_threshold(&processed, config.adaptive_block_size, config.adaptive_c);
    } else {
        processed = apply_threshold(&processed, config.threshold);
    }

    // Step 6: Invert if needed (for white text on dark background)
    if config.invert {
        processed = invert(&processed);
    }

    Ok(processed)
}

/// Convenience function with default preprocessing settings
pub fn preprocess_default(img: &ImageBuffer<Rgba<u8>, Vec<u8>>) -> PreprocessResult<GrayImage> {
    preprocess_for_ocr(img, &PreprocessConfig::default())
}

/// Save a grayscale image for debugging purposes
pub fn save_debug_image(img: &GrayImage, path: &std::path::Path) -> PreprocessResult<()> {
    img.save(path)
        .map_err(|e| PreprocessError::ProcessingFailed(e.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_image(width: u32, height: u32, value: u8) -> ImageBuffer<Rgba<u8>, Vec<u8>> {
        ImageBuffer::from_fn(width, height, |_, _| Rgba([value, value, value, 255]))
    }

    #[test]
    fn test_to_grayscale() {
        let img = create_test_image(10, 10, 128);
        let gray = to_grayscale(&img);
        assert_eq!(gray.dimensions(), (10, 10));
    }

    #[test]
    fn test_apply_threshold() {
        // Create gradient image
        let mut img = GrayImage::new(256, 1);
        for i in 0..256u32 {
            img.put_pixel(i, 0, Luma([i as u8]));
        }

        let thresholded = apply_threshold(&img, 127);
        
        // Values <= 127 should be 0, > 127 should be 255
        assert_eq!(thresholded.get_pixel(0, 0)[0], 0);
        assert_eq!(thresholded.get_pixel(127, 0)[0], 0);
        assert_eq!(thresholded.get_pixel(128, 0)[0], 255);
        assert_eq!(thresholded.get_pixel(255, 0)[0], 255);
    }

    #[test]
    fn test_adaptive_threshold_basic() {
        let mut img = GrayImage::new(10, 10);
        for y in 0..10 {
            for x in 0..10 {
                img.put_pixel(x, y, Luma([128]));
            }
        }

        // Set center to brighter
        for y in 3..7 {
            for x in 3..7 {
                img.put_pixel(x, y, Luma([200]));
            }
        }

        let result = apply_adaptive_threshold(&img, 5, 10);
        assert_eq!(result.dimensions(), (10, 10));
    }

    #[test]
    fn test_invert() {
        let mut img = GrayImage::new(2, 1);
        img.put_pixel(0, 0, Luma([0]));
        img.put_pixel(1, 0, Luma([255]));

        let inverted = invert(&img);
        assert_eq!(inverted.get_pixel(0, 0)[0], 255);
        assert_eq!(inverted.get_pixel(1, 0)[0], 0);
    }

    #[test]
    fn test_enhance_contrast() {
        let mut img = GrayImage::new(3, 1);
        img.put_pixel(0, 0, Luma([64]));
        img.put_pixel(1, 0, Luma([128]));
        img.put_pixel(2, 0, Luma([192]));

        let enhanced = enhance_contrast(&img, 2.0);
        
        // Middle should stay around 128, edges should be more extreme
        assert!(enhanced.get_pixel(0, 0)[0] < 64);
        assert!(enhanced.get_pixel(2, 0)[0] > 192);
    }

    #[test]
    fn test_upscale() {
        let img = GrayImage::new(10, 10);
        let upscaled = upscale(&img, 2.0);
        assert_eq!(upscaled.dimensions(), (20, 20));

        // No upscaling when factor <= 1
        let same = upscale(&img, 0.5);
        assert_eq!(same.dimensions(), (10, 10));
    }

    #[test]
    fn test_preprocess_default_empty_image() {
        let img = create_test_image(0, 0, 128);
        assert!(preprocess_default(&img).is_err());

        let img = create_test_image(10, 0, 128);
        assert!(preprocess_default(&img).is_err());
    }

    #[test]
    fn test_preprocess_default_valid_image() {
        let img = create_test_image(100, 50, 128);
        let result = preprocess_default(&img);
        assert!(result.is_ok());
        
        let processed = result.unwrap();
        // Should be upscaled by 2x
        assert_eq!(processed.dimensions(), (200, 100));
    }

    #[test]
    fn test_preprocess_config_default() {
        let config = PreprocessConfig::default();
        assert_eq!(config.threshold, 127);
        assert!(config.use_adaptive_threshold);
        assert_eq!(config.adaptive_block_size, 11);
        assert!(config.denoise);
        assert!(config.scale_factor > 1.0);
    }

    #[test]
    fn test_median_filter() {
        let mut img = GrayImage::new(5, 5);
        for y in 0..5 {
            for x in 0..5 {
                img.put_pixel(x, y, Luma([100]));
            }
        }
        // Add noise
        img.put_pixel(2, 2, Luma([255]));

        let filtered = apply_median_filter(&img, 3);
        assert_eq!(filtered.dimensions(), (5, 5));
        
        // Center should be smoothed
        assert!(filtered.get_pixel(2, 2)[0] < 255);
    }

    #[test]
    fn test_error_display() {
        assert!(PreprocessError::EmptyImage.to_string().contains("empty"));
        assert!(PreprocessError::InvalidImage("test".to_string()).to_string().contains("test"));
    }
}
