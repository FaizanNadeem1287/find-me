use crate::error::SemanticError;
use crate::models::{DocumentChunk, DocumentMetadata, DocumentType, ExtractedDocument};
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

pub fn extract_code(path: &Path, ext: &str) -> Result<ExtractedDocument, SemanticError> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);

    let mut lines = Vec::new();
    for line in reader.lines() {
        match line {
            Ok(l) => lines.push(l),
            Err(_) => break, // Graceful stop on binary or encoding error
        }
    }

    let mut chunks = Vec::new();
    if !lines.is_empty() {
        let window_size = 50;
        let overlap = 10;

        let mut i = 0;
        let mut chunk_idx = 0;
        while i < lines.len() {
            let end = (i + window_size).min(lines.len());
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
            i += window_size - overlap;
        }
    }

    Ok(ExtractedDocument::new(
        path.to_path_buf(),
        DocumentType::SourceCode,
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
