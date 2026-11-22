use thiserror::Error;

/// Strongly-typed error enum for content extraction and semantic processing
#[derive(Debug, Error)]
pub enum SemanticError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("PDF extraction error: {0}")]
    Pdf(String),

    #[error("Spreadsheet extraction error: {0}")]
    Spreadsheet(String),

    #[error("XML parsing error: {0}")]
    Xml(String),

    #[error("Archive/Zip error: {0}")]
    Archive(String),

    #[error("CSV parsing error: {0}")]
    Csv(#[from] csv::Error),

    #[error("Unsupported format: {0}")]
    UnsupportedFormat(String),

    #[error("File exceeds maximum allowed size ({0} bytes)")]
    FileTooLarge(u64),

    #[error("Extraction failure: {0}")]
    Extraction(String),
}
