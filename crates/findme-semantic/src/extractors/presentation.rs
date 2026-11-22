use crate::error::SemanticError;
use crate::models::{DocumentChunk, DocumentMetadata, DocumentType, ExtractedDocument};
use quick_xml::events::Event;
use quick_xml::reader::Reader;
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::Path;
use zip::ZipArchive;

pub fn extract_presentation(path: &Path) -> Result<ExtractedDocument, SemanticError> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let mut archive = ZipArchive::new(reader)
        .map_err(|e| SemanticError::Archive(format!("Failed to open PPTX archive: {e}")))?;

    let mut chunks = Vec::new();
    let mut chunk_idx = 0;

    // Collect slide file names
    let mut slide_files: Vec<String> = Vec::new();
    for i in 0..archive.len() {
        if let Ok(file) = archive.by_index(i) {
            let name = file.name();
            if name.starts_with("ppt/slides/slide") && name.ends_with(".xml") {
                slide_files.push(name.to_string());
            }
        }
    }

    // Sort slide files by natural order (slide1.xml, slide2.xml, ...)
    slide_files.sort_by(|a, b| {
        let num_a = extract_slide_num(a);
        let num_b = extract_slide_num(b);
        num_a.cmp(&num_b)
    });

    let slide_count = slide_files.len();

    for slide_name in slide_files {
        let mut slide_file = archive
            .by_name(&slide_name)
            .map_err(|e| SemanticError::Archive(format!("Failed to read {slide_name}: {e}")))?;

        let mut xml_data = String::new();
        slide_file.read_to_string(&mut xml_data)?;

        let slide_num = extract_slide_num(&slide_name);
        let slide_text = extract_xml_text(&xml_data, b"a:t");

        let trimmed = slide_text.trim();
        if !trimmed.is_empty() {
            chunks.push(DocumentChunk::new(
                chunk_idx,
                format!("Slide {slide_num}"),
                trimmed.to_string(),
            ));
            chunk_idx += 1;
        }
    }

    Ok(ExtractedDocument::new(
        path.to_path_buf(),
        DocumentType::Presentation,
        chunks,
        DocumentMetadata {
            title: path
                .file_name()
                .and_then(|n| n.to_str())
                .map(|s| s.to_string()),
            slide_count: Some(slide_count),
            language_or_ext: Some("pptx".to_string()),
            ..Default::default()
        },
    ))
}

fn extract_slide_num(name: &str) -> usize {
    name.trim_start_matches("ppt/slides/slide")
        .trim_end_matches(".xml")
        .parse::<usize>()
        .unwrap_or(0)
}

fn extract_xml_text(xml: &str, target_tag: &[u8]) -> String {
    let mut reader = Reader::from_str(xml);
    reader.config_mut().trim_text(true);

    let mut text_parts = Vec::new();
    let mut in_target_tag = false;
    let mut buf = Vec::new();

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) => {
                let name = e.name();
                if name.as_ref() == target_tag || name.as_ref().ends_with(b":t") {
                    in_target_tag = true;
                }
            }
            Ok(Event::Text(ref e)) => {
                if in_target_tag {
                    if let Ok(text) = e.unescape() {
                        let t = text.trim();
                        if !t.is_empty() {
                            text_parts.push(t.to_string());
                        }
                    }
                }
            }
            Ok(Event::End(ref e)) => {
                let name = e.name();
                if name.as_ref() == target_tag || name.as_ref().ends_with(b":t") {
                    in_target_tag = false;
                }
            }
            Ok(Event::Eof) => break,
            Err(_) => break,
            _ => {}
        }
        buf.clear();
    }

    text_parts.join(" ")
}
