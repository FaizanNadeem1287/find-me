use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// High-level classification of extracted document types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DocumentType {
    Pdf,
    Csv,
    Spreadsheet,
    Presentation,
    WordDocument,
    PlainText,
    SourceCode,
    Audio,
    Image,
    Unknown,
}

impl DocumentType {
    /// Detects document type based on file extension
    pub fn from_extension(ext: &str) -> Self {
        let ext_lower = ext.to_ascii_lowercase();
        match ext_lower.as_str() {
            "pdf" => DocumentType::Pdf,
            "csv" | "tsv" => DocumentType::Csv,
            "xlsx" | "xls" | "ods" => DocumentType::Spreadsheet,
            "pptx" | "ppt" | "odp" => DocumentType::Presentation,
            "docx" | "doc" | "odt" => DocumentType::WordDocument,
            "txt" | "md" | "markdown" | "rst" | "log" | "json" | "yaml" | "yml" | "toml"
            | "xml" | "html" | "htm" | "ini" | "cfg" | "conf" => DocumentType::PlainText,
            "py" | "rs" | "go" | "js" | "ts" | "jsx" | "tsx" | "c" | "cpp" | "h" | "hpp"
            | "java" | "kt" | "swift" | "rb" | "php" | "sh" | "bash" | "zsh" | "sql"
            | "dockerfile" | "makefile" => DocumentType::SourceCode,
            "mp3" | "wav" | "m4a" | "ogg" | "flac" | "aac" => DocumentType::Audio,
            "png" | "jpg" | "jpeg" | "webp" | "bmp" | "gif" | "svg" => DocumentType::Image,
            _ => DocumentType::Unknown,
        }
    }
}

/// A discrete semantic chunk of an extracted document with location context
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DocumentChunk {
    /// 0-indexed chunk sequence within this document
    pub chunk_index: usize,
    /// Location identifier (e.g. "Page 1", "Slide 3", "Lines 1-50", "Sheet: Summary")
    pub location: String,
    /// Extracted text content for this chunk
    pub text: String,
    /// Character count of the text
    pub char_count: usize,
}

impl DocumentChunk {
    pub fn new(chunk_index: usize, location: impl Into<String>, text: impl Into<String>) -> Self {
        let text = text.into();
        let char_count = text.chars().count();
        Self {
            chunk_index,
            location: location.into(),
            text,
            char_count,
        }
    }
}

/// Document-level metadata extracted during parsing
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct DocumentMetadata {
    pub title: Option<String>,
    pub author: Option<String>,
    pub page_count: Option<usize>,
    pub sheet_count: Option<usize>,
    pub slide_count: Option<usize>,
    pub language_or_ext: Option<String>,
    // Multimodal audio metadata
    pub duration_secs: Option<u64>,
    pub sample_rate: Option<u32>,
    pub channels: Option<u16>,
    pub audio_codec: Option<String>,
    // Multimodal image metadata
    pub image_width: Option<u32>,
    pub image_height: Option<u32>,
    pub color_type: Option<String>,
    pub aspect_ratio: Option<String>,
    pub perceptual_hash: Option<String>,
}

/// Fully extracted document containing all segmented chunks and metadata
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExtractedDocument {
    pub path: PathBuf,
    pub doc_type: DocumentType,
    pub total_chunks: usize,
    pub chunks: Vec<DocumentChunk>,
    pub metadata: DocumentMetadata,
}

impl ExtractedDocument {
    pub fn new(
        path: PathBuf,
        doc_type: DocumentType,
        chunks: Vec<DocumentChunk>,
        metadata: DocumentMetadata,
    ) -> Self {
        let total_chunks = chunks.len();
        Self {
            path,
            doc_type,
            total_chunks,
            chunks,
            metadata,
        }
    }
}
