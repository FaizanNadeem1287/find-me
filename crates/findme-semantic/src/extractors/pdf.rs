use crate::error::SemanticError;
use crate::models::{DocumentChunk, DocumentMetadata, DocumentType, ExtractedDocument};
use std::path::Path;

pub fn extract_pdf(path: &Path) -> Result<ExtractedDocument, SemanticError> {
    let text = pdf_extract::extract_text(path)
        .map_err(|e| SemanticError::Pdf(format!("Failed to extract PDF text: {e}")))?;

    let mut chunks = Vec::new();
    let trimmed = text.trim();
    if !trimmed.is_empty() {
        // Split text into ~1000 character chunks with overlap
        let words: Vec<&str> = trimmed.split_whitespace().collect();
        let chunk_word_size = 200;
        let overlap = 30;

        let mut i = 0;
        let mut chunk_idx = 0;
        while i < words.len() {
            let end = (i + chunk_word_size).min(words.len());
            let chunk_text = words[i..end].join(" ");
            chunks.push(DocumentChunk::new(
                chunk_idx,
                format!("Excerpt {}", chunk_idx + 1),
                chunk_text,
            ));
            chunk_idx += 1;
            if end == words.len() {
                break;
            }
            i += chunk_word_size - overlap;
        }
    }

    Ok(ExtractedDocument::new(
        path.to_path_buf(),
        DocumentType::Pdf,
        chunks,
        DocumentMetadata {
            title: path
                .file_name()
                .and_then(|n| n.to_str())
                .map(|s| s.to_string()),
            language_or_ext: Some("pdf".to_string()),
            ..Default::default()
        },
    ))
}
