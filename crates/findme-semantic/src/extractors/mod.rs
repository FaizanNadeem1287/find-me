//! Content extraction modules for various document and code file formats

pub mod code;
pub mod pdf;
pub mod presentation;
pub mod spreadsheet;
pub mod text;
pub mod word;

use crate::error::SemanticError;
use crate::models::{DocumentType, ExtractedDocument};
use std::path::Path;

/// Maximum file size allowed for content parsing (50MB) to prevent OOM
pub const MAX_PARSE_FILE_SIZE: u64 = 50 * 1024 * 1024;

/// Main extraction router: inspects file extension/type and dispatches to appropriate parser
pub fn extract_content(path: &Path) -> Result<ExtractedDocument, SemanticError> {
    let metadata = std::fs::metadata(path)?;
    if metadata.len() > MAX_PARSE_FILE_SIZE {
        return Err(SemanticError::FileTooLarge(metadata.len()));
    }

    let ext = path
        .extension()
        .and_then(|s| s.to_str())
        .unwrap_or_else(|| {
            // Check file name for extensionless code files like Dockerfile or Makefile
            path.file_name().and_then(|n| n.to_str()).unwrap_or("")
        });

    let doc_type = DocumentType::from_extension(ext);

    match doc_type {
        DocumentType::Pdf => pdf::extract_pdf(path),
        DocumentType::Csv => spreadsheet::extract_csv(path),
        DocumentType::Spreadsheet => spreadsheet::extract_spreadsheet(path),
        DocumentType::Presentation => presentation::extract_presentation(path),
        DocumentType::WordDocument => word::extract_word(path),
        DocumentType::PlainText => text::extract_plain_text(path),
        DocumentType::SourceCode => code::extract_code(path, ext),
        _ => Err(SemanticError::UnsupportedFormat(format!(
            "Unsupported or binary file type: {}",
            path.display()
        ))),
    }
}
