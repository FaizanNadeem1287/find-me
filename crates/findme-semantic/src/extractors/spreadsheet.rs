use crate::error::SemanticError;
use crate::models::{DocumentChunk, DocumentMetadata, DocumentType, ExtractedDocument};
use calamine::{open_workbook_auto, Data, Reader};
use std::fs::File;
use std::io::BufReader;
use std::path::Path;

pub fn extract_csv(path: &Path) -> Result<ExtractedDocument, SemanticError> {
    let file = File::open(path)?;
    let mut rdr = csv::ReaderBuilder::new()
        .has_headers(true)
        .flexible(true)
        .from_reader(BufReader::new(file));

    let headers = rdr
        .headers()
        .map(|h| h.iter().map(|s| s.to_string()).collect::<Vec<_>>())
        .unwrap_or_default();

    let header_line = if !headers.is_empty() {
        format!("Headers: [{}]\n", headers.join(", "))
    } else {
        String::new()
    };

    let mut chunks = Vec::new();
    let mut current_rows = Vec::new();
    let mut row_count = 0;
    let mut chunk_idx = 0;

    for record in rdr.records().flatten() {
        row_count += 1;
        let row_str = record.iter().collect::<Vec<_>>().join(" | ");
        current_rows.push(row_str);

        if current_rows.len() >= 50 {
            let text = format!("{}{}", header_line, current_rows.join("\n"));
            chunks.push(DocumentChunk::new(
                chunk_idx,
                format!("Rows {}-{}", row_count - current_rows.len() + 1, row_count),
                text,
            ));
            chunk_idx += 1;
            current_rows.clear();
        }
    }

    if !current_rows.is_empty() {
        let text = format!("{}{}", header_line, current_rows.join("\n"));
        chunks.push(DocumentChunk::new(
            chunk_idx,
            format!("Rows {}-{}", row_count - current_rows.len() + 1, row_count),
            text,
        ));
    }

    Ok(ExtractedDocument::new(
        path.to_path_buf(),
        DocumentType::Csv,
        chunks,
        DocumentMetadata {
            title: path
                .file_name()
                .and_then(|n| n.to_str())
                .map(|s| s.to_string()),
            language_or_ext: Some("csv".to_string()),
            ..Default::default()
        },
    ))
}

pub fn extract_spreadsheet(path: &Path) -> Result<ExtractedDocument, SemanticError> {
    let mut workbook = open_workbook_auto(path)
        .map_err(|e| SemanticError::Spreadsheet(format!("Failed to open spreadsheet: {e}")))?;

    let sheet_names = workbook.sheet_names().to_vec();
    let mut chunks = Vec::new();
    let mut chunk_idx = 0;

    for name in &sheet_names {
        if let Ok(range) = workbook.worksheet_range(name) {
            let mut sheet_rows = Vec::new();
            for row in range.rows() {
                let row_str = row
                    .iter()
                    .map(|cell| match cell {
                        Data::Empty => String::new(),
                        Data::String(s) => s.clone(),
                        Data::Float(f) => format!("{f}"),
                        Data::Int(i) => format!("{i}"),
                        Data::Bool(b) => format!("{b}"),
                        Data::DateTime(dt) => format!("{dt}"),
                        Data::DateTimeIso(s) => s.clone(),
                        Data::DurationIso(s) => s.clone(),
                        Data::Error(e) => format!("{e:?}"),
                    })
                    .collect::<Vec<_>>()
                    .join(" | ");

                if !row_str.trim().is_empty() {
                    sheet_rows.push(row_str);
                }

                if sheet_rows.len() >= 50 {
                    let text = format!("Sheet: {name}\n{}", sheet_rows.join("\n"));
                    chunks.push(DocumentChunk::new(
                        chunk_idx,
                        format!("Sheet: {name}"),
                        text,
                    ));
                    chunk_idx += 1;
                    sheet_rows.clear();
                }
            }

            if !sheet_rows.is_empty() {
                let text = format!("Sheet: {name}\n{}", sheet_rows.join("\n"));
                chunks.push(DocumentChunk::new(
                    chunk_idx,
                    format!("Sheet: {name}"),
                    text,
                ));
                chunk_idx += 1;
            }
        }
    }

    Ok(ExtractedDocument::new(
        path.to_path_buf(),
        DocumentType::Spreadsheet,
        chunks,
        DocumentMetadata {
            title: path
                .file_name()
                .and_then(|n| n.to_str())
                .map(|s| s.to_string()),
            sheet_count: Some(sheet_names.len()),
            language_or_ext: path
                .extension()
                .and_then(|e| e.to_str())
                .map(|s| s.to_string()),
            ..Default::default()
        },
    ))
}
