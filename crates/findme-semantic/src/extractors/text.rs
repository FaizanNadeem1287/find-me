use crate::error::SemanticError;
use crate::models::{DocumentChunk, DocumentMetadata, DocumentType, ExtractedDocument};
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::Path;

pub fn extract_plain_text(path: &Path) -> Result<ExtractedDocument, SemanticError> {
    let file = File::open(path)?;
    let mut reader = BufReader::new(file);
    let mut buffer = Vec::new();
    reader.read_to_end(&mut buffer)?;

    let text = String::from_utf8_lossy(&buffer);
    let trimmed = text.trim();

    let mut chunks = Vec::new();
    if !trimmed.is_empty() {
        let lines: Vec<&str> = trimmed.lines().collect();
        let chunk_line_size = 60;
        let overlap = 10;

        let mut i = 0;
        let mut chunk_idx = 0;
        while i < lines.len() {
            let end = (i + chunk_line_size).min(lines.len());
            let chunk_text = lines[i..end].join("\n");
            chunks.push(DocumentChunk::new(
                chunk_idx,
                format!("Lines {}-{}", i + 1, end),
                chunk_text,
            ));
            chunk_idx += 1;
            if end == lines.len() {
                break;
            }
            i += chunk_line_size - overlap;
        }
    }

    let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("txt");

    Ok(ExtractedDocument::new(
        path.to_path_buf(),
        DocumentType::PlainText,
        chunks,
        DocumentMetadata {
            title: path
                .file_name()
                .and_then(|n| n.to_str())
                .map(|s| s.to_string()),
            language_or_ext: Some(ext.to_string()),
            ..Default::default()
        },
    ))
}
