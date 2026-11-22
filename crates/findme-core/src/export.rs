//! Multi-format search results export facility
//!
//! Provides high-speed streaming exporters for:
//! - Plain Text (`.txt`): one absolute path per line (UNIX piping compatible)
//! - CSV (`.csv`): RFC 4180 compliant CSV format with properly escaped quotes
//! - EFU (`.efu`): Everything File List format with Windows FILETIME timestamps
//! - JSON (`.json`): Structured JSON array of `SearchResultItem`

use crate::error::{EngineError, Result};
use crate::record::SearchResultItem;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;

/// Supported search result export formats
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ExportFormat {
    /// Plain text list with one absolute path per line
    Txt,
    /// Standard comma-separated values (RFC 4180)
    Csv,
    /// Windows Everything File List (.efu) format
    Efu,
    /// Structured JSON array
    Json,
}

impl ExportFormat {
    /// Returns the recommended file extension (without leading dot)
    pub fn extension(&self) -> &'static str {
        match self {
            Self::Txt => "txt",
            Self::Csv => "csv",
            Self::Efu => "efu",
            Self::Json => "json",
        }
    }

    /// Attempts to detect format from a file path's extension
    pub fn from_path<P: AsRef<Path>>(path: P) -> Option<Self> {
        let ext = path.as_ref().extension()?.to_str()?.to_ascii_lowercase();
        match ext.as_str() {
            "txt" => Some(Self::Txt),
            "csv" => Some(Self::Csv),
            "efu" => Some(Self::Efu),
            "json" => Some(Self::Json),
            _ => None,
        }
    }
}

/// Converts Unix timestamp in seconds to Windows FILETIME (100-ns intervals since Jan 1, 1601 UTC)
#[inline]
pub fn unix_secs_to_windows_filetime(secs: u64) -> u64 {
    const EPOCH_DIFFERENCE_SECS: u64 = 11_644_473_600;
    const INTERVALS_PER_SEC: u64 = 10_000_000;
    secs.saturating_add(EPOCH_DIFFERENCE_SECS)
        .saturating_mul(INTERVALS_PER_SEC)
}

/// Converts Windows FILETIME back to Unix timestamp in seconds
#[inline]
pub fn windows_filetime_to_unix_secs(filetime: u64) -> u64 {
    const EPOCH_DIFFERENCE_SECS: u64 = 11_644_473_600;
    const INTERVALS_PER_SEC: u64 = 10_000_000;
    (filetime / INTERVALS_PER_SEC).saturating_sub(EPOCH_DIFFERENCE_SECS)
}

/// Escapes a CSV field according to RFC 4180 rules
pub fn escape_csv_field(field: &str) -> String {
    if field.contains(',') || field.contains('"') || field.contains('\n') || field.contains('\r') {
        let escaped = field.replace('"', "\"\"");
        format!("\"{}\"", escaped)
    } else {
        field.to_string()
    }
}

/// Escapes an EFU path string according to Everything File List conventions
pub fn escape_efu_field(field: &str) -> String {
    let escaped = field.replace('"', "\"\"");
    format!("\"{}\"", escaped)
}

/// Exports search results to an arbitrary `Write` stream in the requested format
pub fn export_to_writer<W: Write>(
    format: ExportFormat,
    results: &[SearchResultItem],
    writer: &mut W,
) -> Result<usize> {
    match format {
        ExportFormat::Txt => {
            for item in results {
                writeln!(writer, "{}", item.path).map_err(|e| EngineError::Io {
                    path: std::path::PathBuf::from("<writer>"),
                    source: e,
                })?;
            }
        }
        ExportFormat::Csv => {
            writeln!(
                writer,
                "\"Name\",\"Path\",\"Size\",\"Date Modified\",\"Type\""
            )
            .map_err(|e| EngineError::Io {
                path: std::path::PathBuf::from("<writer>"),
                source: e,
            })?;
            for item in results {
                let name = escape_csv_field(&item.name);
                let path = escape_csv_field(&item.path);
                let size = item.size_bytes;
                let mtime = escape_csv_field(&item.formatted_mtime());
                let kind = if item.is_dir { "Folder" } else { "File" };
                writeln!(writer, "{},{},{},{},{}", name, path, size, mtime, kind).map_err(|e| {
                    EngineError::Io {
                        path: std::path::PathBuf::from("<writer>"),
                        source: e,
                    }
                })?;
            }
        }
        ExportFormat::Efu => {
            writeln!(
                writer,
                "Filename,Size,Date Modified,Date Created,Attributes"
            )
            .map_err(|e| EngineError::Io {
                path: std::path::PathBuf::from("<writer>"),
                source: e,
            })?;
            for item in results {
                let filename = escape_efu_field(&item.path);
                let size = if item.is_dir { 0 } else { item.size_bytes };
                let filetime = unix_secs_to_windows_filetime(item.mtime_secs);
                // Windows attribute constants: 0x10 (16) = Directory, 0x20 (32) = Archive
                let attrib = if item.is_dir { 16 } else { 32 };
                writeln!(
                    writer,
                    "{},{},{},{},{}",
                    filename, size, filetime, filetime, attrib
                )
                .map_err(|e| EngineError::Io {
                    path: std::path::PathBuf::from("<writer>"),
                    source: e,
                })?;
            }
        }
        ExportFormat::Json => {
            serde_json::to_writer_pretty(&mut *writer, results).map_err(|e| {
                EngineError::Persistence(format!(
                    "Failed to serialize search results to JSON: {}",
                    e
                ))
            })?;
            writeln!(writer).map_err(|e| EngineError::Io {
                path: std::path::PathBuf::from("<writer>"),
                source: e,
            })?;
        }
    }
    Ok(results.len())
}

/// Exports search results to a file path on disk
pub fn export_to_file<P: AsRef<Path>>(
    format: ExportFormat,
    results: &[SearchResultItem],
    path: P,
) -> Result<usize> {
    let path_ref = path.as_ref();
    if let Some(parent) = path_ref.parent() {
        if !parent.as_os_str().is_empty() {
            std::fs::create_dir_all(parent).map_err(|e| EngineError::Io {
                path: parent.to_path_buf(),
                source: e,
            })?;
        }
    }

    let file = File::create(path_ref).map_err(|e| EngineError::Io {
        path: path_ref.to_path_buf(),
        source: e,
    })?;

    let mut writer = BufWriter::with_capacity(64 * 1024, file);
    let count = export_to_writer(format, results, &mut writer)?;
    writer.flush().map_err(|e| EngineError::Io {
        path: path_ref.to_path_buf(),
        source: e,
    })?;

    Ok(count)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_extensions_and_detection() {
        assert_eq!(ExportFormat::Txt.extension(), "txt");
        assert_eq!(ExportFormat::Csv.extension(), "csv");
        assert_eq!(ExportFormat::Efu.extension(), "efu");
        assert_eq!(ExportFormat::Json.extension(), "json");

        assert_eq!(
            ExportFormat::from_path("/tmp/out.csv"),
            Some(ExportFormat::Csv)
        );
        assert_eq!(
            ExportFormat::from_path("/tmp/out.TXT"),
            Some(ExportFormat::Txt)
        );
        assert_eq!(
            ExportFormat::from_path("/tmp/out.efu"),
            Some(ExportFormat::Efu)
        );
        assert_eq!(
            ExportFormat::from_path("/tmp/out.json"),
            Some(ExportFormat::Json)
        );
        assert_eq!(ExportFormat::from_path("/tmp/out.bin"), None);
    }

    #[test]
    fn test_filetime_conversion() {
        // Unix epoch (1970-01-01 00:00:00 UTC) = 116444736000000000 in FILETIME
        let unix_zero = 0u64;
        let filetime_zero = unix_secs_to_windows_filetime(unix_zero);
        assert_eq!(filetime_zero, 116_444_736_000_000_000);
        assert_eq!(windows_filetime_to_unix_secs(filetime_zero), unix_zero);

        let test_secs = 1_732_000_000u64;
        let filetime = unix_secs_to_windows_filetime(test_secs);
        assert_eq!(windows_filetime_to_unix_secs(filetime), test_secs);
    }

    #[test]
    fn test_csv_field_escaping() {
        assert_eq!(escape_csv_field("normal_file.txt"), "normal_file.txt");
        assert_eq!(
            escape_csv_field("file,with,commas.txt"),
            "\"file,with,commas.txt\""
        );
        assert_eq!(
            escape_csv_field("file\"with\"quotes.txt"),
            "\"file\"\"with\"\"quotes.txt\""
        );
        assert_eq!(
            escape_csv_field("file\nwith\nnewlines.txt"),
            "\"file\nwith\nnewlines.txt\""
        );
    }

    #[test]
    fn test_export_txt_csv_efu_json() {
        let items = vec![
            SearchResultItem {
                id: 1,
                name: "file1.txt".to_string(),
                path: "/home/user/file1.txt".to_string(),
                size_bytes: 1024,
                mtime_secs: 1732000000,
                is_dir: false,
            },
            SearchResultItem {
                id: 2,
                name: "folder_a".to_string(),
                path: "/home/user/folder_a".to_string(),
                size_bytes: 0,
                mtime_secs: 1732000100,
                is_dir: true,
            },
        ];

        // 1. Txt export
        let mut txt_buf = Vec::new();
        let count = export_to_writer(ExportFormat::Txt, &items, &mut txt_buf).unwrap();
        assert_eq!(count, 2);
        let txt_str = String::from_utf8(txt_buf).unwrap();
        assert_eq!(txt_str, "/home/user/file1.txt\n/home/user/folder_a\n");

        // 2. CSV export
        let mut csv_buf = Vec::new();
        let count = export_to_writer(ExportFormat::Csv, &items, &mut csv_buf).unwrap();
        assert_eq!(count, 2);
        let csv_str = String::from_utf8(csv_buf).unwrap();
        assert!(csv_str.starts_with("\"Name\",\"Path\",\"Size\",\"Date Modified\",\"Type\"\n"));
        assert!(csv_str.contains("file1.txt,/home/user/file1.txt,1024,"));
        assert!(csv_str.contains("folder_a,/home/user/folder_a,0,"));

        // 3. EFU export
        let mut efu_buf = Vec::new();
        let count = export_to_writer(ExportFormat::Efu, &items, &mut efu_buf).unwrap();
        assert_eq!(count, 2);
        let efu_str = String::from_utf8(efu_buf).unwrap();
        assert!(efu_str.starts_with("Filename,Size,Date Modified,Date Created,Attributes\n"));
        assert!(efu_str.contains("\"/home/user/file1.txt\",1024,"));
        assert!(efu_str.ends_with(",16\n")); // folder_a has attrib 16

        // 4. JSON export
        let mut json_buf = Vec::new();
        let count = export_to_writer(ExportFormat::Json, &items, &mut json_buf).unwrap();
        assert_eq!(count, 2);
        let parsed: Vec<SearchResultItem> = serde_json::from_slice(&json_buf).unwrap();
        assert_eq!(parsed.len(), 2);
        assert_eq!(parsed[0].name, "file1.txt");
        assert_eq!(parsed[1].name, "folder_a");
    }
}
