use findme_semantic::extractors::code::extract_code;
use findme_semantic::extractors::extract_content;
use findme_semantic::extractors::presentation::extract_presentation;
use findme_semantic::extractors::spreadsheet::extract_csv;
use findme_semantic::extractors::text::extract_plain_text;
use findme_semantic::extractors::word::extract_word;
use findme_semantic::models::DocumentType;
use findme_semantic::SemanticError;
use std::io::Write;
use tempfile::NamedTempFile;

#[test]
fn test_extract_plain_text_simple() {
    let mut file = NamedTempFile::new().expect("tempfile");
    writeln!(
        file,
        "Hello, world!\nThis is a simple plain text document.\nIt contains multiple lines."
    )
    .expect("write");

    let doc = extract_plain_text(file.path()).expect("extract");
    assert_eq!(doc.doc_type, DocumentType::PlainText);
    assert!(!doc.chunks.is_empty());
    assert!(doc.chunks[0].text.contains("Hello, world!"));
    assert!(doc.chunks[0].location.starts_with("Lines 1-"));
}

#[test]
fn test_extract_plain_text_chunking() {
    let mut file = NamedTempFile::new().expect("tempfile");
    for i in 1..=150 {
        writeln!(file, "Line {i}: Content for testing chunking behavior.").expect("write");
    }

    let doc = extract_plain_text(file.path()).expect("extract");
    assert!(doc.chunks.len() >= 2);
    assert_eq!(doc.chunks[0].chunk_index, 0);
    assert_eq!(doc.chunks[1].chunk_index, 1);
    assert!(doc.chunks[0].location.contains("Lines 1-"));
}

#[test]
fn test_extract_code_python() {
    let mut file = NamedTempFile::new().expect("tempfile");
    writeln!(file, "import os\n\ndef calculate_sum(a: int, b: int) -> int:\n    \"\"\"Calculates sum of two numbers\"\"\"\n    return a + b\n\nif __name__ == '__main__':\n    print(calculate_sum(10, 20))\n").expect("write");

    let doc = extract_code(file.path(), "py").expect("extract");
    assert_eq!(doc.doc_type, DocumentType::SourceCode);
    assert_eq!(doc.metadata.language_or_ext.as_deref(), Some("py"));
    assert!(!doc.chunks.is_empty());
    assert!(doc.chunks[0].text.contains("def calculate_sum"));
}

#[test]
fn test_extract_code_dockerfile() {
    let mut file = NamedTempFile::new().expect("tempfile");
    writeln!(file, "FROM rust:1.80-slim as builder\nWORKDIR /app\nCOPY . .\nRUN cargo build --release\nCMD [\"./target/release/find-me\"]\n").expect("write");

    let doc = extract_code(file.path(), "Dockerfile").expect("extract");
    assert_eq!(doc.doc_type, DocumentType::SourceCode);
    assert!(!doc.chunks.is_empty());
    assert!(doc.chunks[0].text.contains("FROM rust:1.80-slim"));
}

#[test]
fn test_extract_csv_with_headers() {
    let mut file = NamedTempFile::new().expect("tempfile");
    writeln!(file, "id,name,role,salary").expect("write");
    writeln!(file, "1,Faizan,AI Engineer,150000").expect("write");
    writeln!(file, "2,Alice,Product Manager,140000").expect("write");
    writeln!(file, "3,Bob,Rust Developer,135000").expect("write");

    let doc = extract_csv(file.path()).expect("extract");
    assert_eq!(doc.doc_type, DocumentType::Csv);
    assert!(!doc.chunks.is_empty());
    let chunk_text = &doc.chunks[0].text;
    assert!(chunk_text.contains("Headers: [id, name, role, salary]"));
    assert!(chunk_text.contains("1 | Faizan | AI Engineer | 150000"));
}

#[test]
fn test_extract_word_docx_mock() {
    let mut file = NamedTempFile::new().expect("tempfile");
    {
        let mut zip = zip::ZipWriter::new(&mut file);
        let options = zip::write::SimpleFileOptions::default()
            .compression_method(zip::CompressionMethod::Stored);
        zip.start_file("word/document.xml", options)
            .expect("start_file");
        let xml_content = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
    <w:body>
        <w:p><w:r><w:t>Curriculum Vitae</w:t></w:r></w:p>
        <w:p><w:r><w:t>Muhammad Faizan Nadeem - Senior AI Engineer</w:t></w:r></w:p>
        <w:p><w:r><w:t>Specialized in Rust systems, offline local AI, and search engines.</w:t></w:r></w:p>
    </w:body>
</w:document>"#;
        zip.write_all(xml_content.as_bytes()).expect("write_all");
        zip.finish().expect("finish");
    }

    let doc = extract_word(file.path()).expect("extract");
    assert_eq!(doc.doc_type, DocumentType::WordDocument);
    assert!(!doc.chunks.is_empty());
    assert!(doc.chunks[0].text.contains("Curriculum Vitae"));
    assert!(doc.chunks[0].text.contains("Muhammad Faizan Nadeem"));
}

#[test]
fn test_extract_presentation_pptx_mock() {
    let mut file = NamedTempFile::new().expect("tempfile");
    {
        let mut zip = zip::ZipWriter::new(&mut file);
        let options = zip::write::SimpleFileOptions::default()
            .compression_method(zip::CompressionMethod::Stored);

        zip.start_file("ppt/slides/slide1.xml", options)
            .expect("start_file 1");
        let slide1_xml = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<p:sld xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main" xmlns:p="http://schemas.openxmlformats.org/presentationml/2006/main">
    <p:cSld>
        <p:spTree>
            <p:sp>
                <p:txBody>
                    <a:p><a:r><a:t>Find-Me Architecture Overview</a:t></a:r></a:p>
                </p:txBody>
            </p:sp>
        </p:spTree>
    </p:cSld>
</p:sld>"#;
        zip.write_all(slide1_xml.as_bytes()).expect("write slide 1");

        zip.start_file("ppt/slides/slide2.xml", options)
            .expect("start_file 2");
        let slide2_xml = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<p:sld xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main" xmlns:p="http://schemas.openxmlformats.org/presentationml/2006/main">
    <p:cSld>
        <p:spTree>
            <p:sp>
                <p:txBody>
                    <a:p><a:r><a:t>Deep Semantic Search and Local HNSW Indexing</a:t></a:r></a:p>
                </p:txBody>
            </p:sp>
        </p:spTree>
    </p:cSld>
</p:sld>"#;
        zip.write_all(slide2_xml.as_bytes()).expect("write slide 2");

        zip.finish().expect("finish");
    }

    let doc = extract_presentation(file.path()).expect("extract");
    assert_eq!(doc.doc_type, DocumentType::Presentation);
    assert_eq!(doc.metadata.slide_count, Some(2));
    assert_eq!(doc.chunks.len(), 2);
    assert_eq!(doc.chunks[0].location, "Slide 1");
    assert!(doc.chunks[0].text.contains("Find-Me Architecture Overview"));
    assert_eq!(doc.chunks[1].location, "Slide 2");
    assert!(doc.chunks[1].text.contains("Deep Semantic Search"));
}

#[test]
fn test_extract_content_router() {
    let dir = tempfile::tempdir().expect("tempdir");

    // Test python file routing
    let py_path = dir.path().join("script.py");
    std::fs::write(&py_path, "print('hello')").expect("write");
    let doc = extract_content(&py_path).expect("extract");
    assert_eq!(doc.doc_type, DocumentType::SourceCode);

    // Test markdown file routing
    let md_path = dir.path().join("README.md");
    std::fs::write(&md_path, "# Find-Me\nHigh performance search").expect("write");
    let doc = extract_content(&md_path).expect("extract");
    assert_eq!(doc.doc_type, DocumentType::PlainText);

    // Test CSV routing
    let csv_path = dir.path().join("data.csv");
    std::fs::write(&csv_path, "col1,col2\nval1,val2").expect("write");
    let doc = extract_content(&csv_path).expect("extract");
    assert_eq!(doc.doc_type, DocumentType::Csv);

    // Test unsupported binary file routing
    let bin_path = dir.path().join("blob.iso");
    std::fs::write(&bin_path, [0u8; 100]).expect("write");
    let err = extract_content(&bin_path).expect_err("should fail");
    match err {
        SemanticError::UnsupportedFormat(msg) => {
            assert!(msg.contains("blob.iso"));
        }
        _ => panic!("Expected UnsupportedFormat error"),
    }
}

#[test]
fn test_extract_record_content_with_arena() {
    use findme_core::arena::FileArena;
    use findme_core::record::FileFlags;
    use findme_semantic::extract_record_content;

    let dir = tempfile::tempdir().expect("tempdir");
    let file_path = dir.path().join("cv_faizan.txt");
    std::fs::write(
        &file_path,
        "Muhammad Faizan Nadeem\nAI Systems Engineer\nRust, Machine Learning, Deep Retrieval",
    )
    .expect("write");

    let mut arena = FileArena::new();
    let rec_idx = arena
        .add_file(&file_path, FileFlags::NONE, 150, 1700000000)
        .expect("add_file");

    let record = arena.get(rec_idx as usize).expect("get record");
    let extracted = extract_record_content(&arena, record).expect("extract_record_content");

    assert_eq!(extracted.doc_type, DocumentType::PlainText);
    assert!(!extracted.chunks.is_empty());
    assert!(extracted.chunks[0].text.contains("Muhammad Faizan Nadeem"));
}

#[test]
fn test_extract_empty_file() {
    let dir = tempfile::tempdir().expect("tempdir");
    let empty_path = dir.path().join("empty.txt");
    std::fs::write(&empty_path, "").expect("write");

    let doc = extract_content(&empty_path).expect("extract");
    assert_eq!(doc.doc_type, DocumentType::PlainText);
    assert_eq!(doc.total_chunks, 0);
    assert!(doc.chunks.is_empty());
}

#[test]
fn test_extract_file_too_large() {
    let dir = tempfile::tempdir().expect("tempdir");
    let large_file = dir.path().join("huge.txt");
    let file = std::fs::File::create(&large_file).expect("create");
    // Set file length to 51 MB (exceeding MAX_PARSE_FILE_SIZE = 50MB) without allocating disk space
    file.set_len(51 * 1024 * 1024).expect("set_len");

    let err = extract_content(&large_file).expect_err("should reject huge file");
    match err {
        SemanticError::FileTooLarge(len) => {
            assert!(len > 50 * 1024 * 1024);
        }
        _ => panic!("Expected FileTooLarge error, got {:?}", err),
    }
}
