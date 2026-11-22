use crate::error::SemanticError;
use crate::models::{DocumentChunk, DocumentMetadata, DocumentType, ExtractedDocument};
use quick_xml::events::Event;
use quick_xml::reader::Reader;
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::Path;
use zip::ZipArchive;

pub fn extract_word(path: &Path) -> Result<ExtractedDocument, SemanticError> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let mut archive = ZipArchive::new(reader)
        .map_err(|e| SemanticError::Archive(format!("Failed to open DOCX archive: {e}")))?;

    let mut doc_file = archive
        .by_name("word/document.xml")
        .map_err(|e| SemanticError::Archive(format!("Failed to read word/document.xml: {e}")))?;

    let mut xml_data = String::new();
    doc_file.read_to_string(&mut xml_data)?;

    let full_text = extract_docx_paragraphs(&xml_data);
    let mut chunks = Vec::new();
    let mut chunk_idx = 0;

    let paragraphs: Vec<&str> = full_text.split("\n\n").collect();
    let mut current_para_batch = Vec::new();
    let mut current_chars = 0;

    for para in paragraphs {
        let trimmed = para.trim();
        if trimmed.is_empty() {
            continue;
        }

        current_para_batch.push(trimmed);
        current_chars += trimmed.len();

        if current_chars >= 800 {
            chunks.push(DocumentChunk::new(
                chunk_idx,
                format!("Section {}", chunk_idx + 1),
                current_para_batch.join("\n\n"),
            ));
            chunk_idx += 1;
            current_para_batch.clear();
            current_chars = 0;
        }
    }

    if !current_para_batch.is_empty() {
        chunks.push(DocumentChunk::new(
            chunk_idx,
            format!("Section {}", chunk_idx + 1),
            current_para_batch.join("\n\n"),
        ));
    }

    Ok(ExtractedDocument::new(
        path.to_path_buf(),
        DocumentType::WordDocument,
        chunks,
        DocumentMetadata {
            title: path
                .file_name()
                .and_then(|n| n.to_str())
                .map(|s| s.to_string()),
            language_or_ext: Some("docx".to_string()),
            ..Default::default()
        },
    ))
}

fn extract_docx_paragraphs(xml: &str) -> String {
    let mut reader = Reader::from_str(xml);
    reader.config_mut().trim_text(true);

    let mut paragraphs = Vec::new();
    let mut current_paragraph = Vec::new();
    let mut in_text_node = false;
    let mut buf = Vec::new();

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) => {
                let name = e.name();
                if name.as_ref() == b"w:t" || name.as_ref().ends_with(b":t") {
                    in_text_node = true;
                }
            }
            Ok(Event::Text(ref e)) => {
                if in_text_node {
                    if let Ok(text) = e.unescape() {
                        let t = text.trim();
                        if !t.is_empty() {
                            current_paragraph.push(t.to_string());
                        }
                    }
                }
            }
            Ok(Event::End(ref e)) => {
                let name = e.name();
                if name.as_ref() == b"w:t" || name.as_ref().ends_with(b":t") {
                    in_text_node = false;
                } else if (name.as_ref() == b"w:p" || name.as_ref().ends_with(b":p"))
                    && !current_paragraph.is_empty()
                {
                    paragraphs.push(current_paragraph.join(" "));
                    current_paragraph.clear();
                }
            }
            Ok(Event::Eof) => break,
            Err(_) => break,
            _ => {}
        }
        buf.clear();
    }

    paragraphs.join("\n\n")
}
