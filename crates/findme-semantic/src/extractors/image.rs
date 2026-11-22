//! Multimodal image metadata extraction, perceptual hashing, and visual preprocessing
//!
//! Extracts image properties (dimensions, aspect ratio, color type) and computes
//! a 64-bit difference perceptual hash (dHash) for visual similarity matching and deduplication.

use crate::error::SemanticError;
use crate::models::{DocumentChunk, DocumentMetadata, DocumentType, ExtractedDocument};
use image::GenericImageView;
use std::fs::File;
use std::io::Read;
use std::path::Path;

/// Maximum bytes read for SVG text parsing
const MAX_SVG_READ_BYTES: usize = 1024 * 1024;

/// Computes greatest common divisor for aspect ratio simplification
fn gcd(mut a: u32, mut b: u32) -> u32 {
    while b != 0 {
        let t = b;
        b = a % b;
        a = t;
    }
    a
}

/// Computes human-readable aspect ratio string (e.g., "16:9", "4:3", "1:1")
pub fn calculate_aspect_ratio(width: u32, height: u32) -> String {
    if width == 0 || height == 0 {
        return "Unknown".to_string();
    }
    let d = gcd(width, height);
    let rw = width / d;
    let rh = height / d;

    // Standardize common approximations
    if (rw == 16 && rh == 9)
        || (rw == 4 && rh == 3)
        || (rw == 1 && rh == 1)
        || (rw == 16 && rh == 10)
    {
        format!("{rw}:{rh}")
    } else {
        let ratio = width as f64 / height as f64;
        if (ratio - 16.0 / 9.0).abs() < 0.02 {
            "16:9".to_string()
        } else if (ratio - 4.0 / 3.0).abs() < 0.02 {
            "4:3".to_string()
        } else if (ratio - 1.0).abs() < 0.02 {
            "1:1".to_string()
        } else if (ratio - 16.0 / 10.0).abs() < 0.02 {
            "16:10".to_string()
        } else {
            format!("{rw}:{rh}")
        }
    }
}

/// Classifies orientation into Landscape, Portrait, or Square
pub fn calculate_orientation(width: u32, height: u32) -> &'static str {
    if width > height {
        "Landscape"
    } else if height > width {
        "Portrait"
    } else {
        "Square"
    }
}

/// Computes a 64-bit difference perceptual hash (dHash) from a dynamic image.
///
/// 1. Resizes image to 9x8 grayscale thumbnail.
/// 2. Compares adjacent horizontal pixels: if left > right, sets bit in 64-bit integer.
/// 3. Returns 16-character hexadecimal fingerprint.
pub fn compute_dhash(img: &image::DynamicImage) -> String {
    let thumbnail = img.resize_exact(9, 8, image::imageops::FilterType::Nearest);
    let gray = thumbnail.to_luma8();

    let mut hash = 0u64;
    let mut bit = 0;

    for y in 0..8 {
        for x in 0..8 {
            let left = gray.get_pixel(x, y)[0];
            let right = gray.get_pixel(x + 1, y)[0];
            if left > right {
                hash |= 1 << bit;
            }
            bit += 1;
        }
    }

    format!("{hash:016x}")
}

/// Extracts metadata, dimensions, and visual hash from an image file
pub fn extract_image(path: &Path) -> Result<ExtractedDocument, SemanticError> {
    let ext = path
        .extension()
        .and_then(|s| s.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();

    if ext == "svg" {
        return extract_svg_image(path);
    }

    // Try reading dimensions using fast reader
    let reader = image::ImageReader::open(path)
        .map_err(SemanticError::Io)?
        .with_guessed_format()
        .map_err(|e| SemanticError::Extraction(e.to_string()))?;

    let format = reader.format();
    let format_name = format.map_or_else(|| ext.to_uppercase(), |f| format!("{f:?}"));

    let dynamic_img = reader
        .decode()
        .map_err(|e| SemanticError::Extraction(format!("Failed to decode image: {e}")))?;

    let (width, height) = dynamic_img.dimensions();
    let color_type = format!("{:?}", dynamic_img.color());
    let aspect_ratio = calculate_aspect_ratio(width, height);
    let orientation = calculate_orientation(width, height);
    let perceptual_hash = compute_dhash(&dynamic_img);

    let summary = format!(
        "Image ({format_name}) • {width}x{height} ({aspect_ratio} {orientation}) • Color: {color_type} • Perceptual Hash: {perceptual_hash}"
    );

    let chunks = vec![DocumentChunk::new(0, "Image Visual Metadata", summary)];

    let metadata = DocumentMetadata {
        title: path.file_stem().and_then(|s| s.to_str()).map(String::from),
        image_width: Some(width),
        image_height: Some(height),
        color_type: Some(color_type),
        aspect_ratio: Some(aspect_ratio),
        perceptual_hash: Some(perceptual_hash),
        language_or_ext: Some(ext),
        ..Default::default()
    };

    Ok(ExtractedDocument::new(
        path.to_path_buf(),
        DocumentType::Image,
        chunks,
        metadata,
    ))
}

/// Specialized extractor for Vector SVG graphics
fn extract_svg_image(path: &Path) -> Result<ExtractedDocument, SemanticError> {
    let file = File::open(path)?;
    let mut content = String::new();
    let mut take = file.take(MAX_SVG_READ_BYTES as u64);
    take.read_to_string(&mut content)?;

    let mut width = None;
    let mut height = None;

    // Scan for width and height or viewBox attributes
    for part in content.split_whitespace() {
        if let Some(w_str) = part.strip_prefix("width=\"") {
            if let Some(val) = w_str.split('"').next() {
                width = val
                    .trim_end_matches("px")
                    .parse::<f64>()
                    .ok()
                    .map(|v| v as u32);
            }
        } else if let Some(h_str) = part.strip_prefix("height=\"") {
            if let Some(val) = h_str.split('"').next() {
                height = val
                    .trim_end_matches("px")
                    .parse::<f64>()
                    .ok()
                    .map(|v| v as u32);
            }
        }
    }

    let w = width.unwrap_or(0);
    let h = height.unwrap_or(0);
    let aspect = if w > 0 && h > 0 {
        calculate_aspect_ratio(w, h)
    } else {
        "Vector Scale".to_string()
    };

    let summary = format!(
        "Vector Graphic (SVG) • Dimensions: {dim} • Aspect: {aspect}",
        dim = if w > 0 && h > 0 {
            format!("{w}x{h}")
        } else {
            "Scalable ViewBox".to_string()
        }
    );

    let chunks = vec![DocumentChunk::new(0, "Vector Image Metadata", summary)];

    let metadata = DocumentMetadata {
        title: path.file_stem().and_then(|s| s.to_str()).map(String::from),
        image_width: width,
        image_height: height,
        color_type: Some("Vector XML".to_string()),
        aspect_ratio: Some(aspect),
        language_or_ext: Some("svg".to_string()),
        ..Default::default()
    };

    Ok(ExtractedDocument::new(
        path.to_path_buf(),
        DocumentType::Image,
        chunks,
        metadata,
    ))
}
