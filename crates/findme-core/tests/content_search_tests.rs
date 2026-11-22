//! Integration tests for Full-Text Content Search (Task 11.2)

use findme_core::{
    FileArena, FileFlags, QueryNode, QueryParser, SearchConfig, SearchEngine, SearchPredicate,
    SortColumn, SortDirection,
};
use std::fs::{self, File};
use std::io::Write;
use std::path::Path;
use tempfile::tempdir;

#[test]
fn test_lexer_and_parser_content_syntax() {
    // 1. Quoted content expression
    let ast1 = QueryParser::parse("content:\"fn main()\"");
    match ast1.root {
        QueryNode::Predicate(SearchPredicate::Content {
            pattern,
            case_sensitive,
        }) => {
            assert_eq!(pattern, "fn main()");
            assert!(!case_sensitive);
        }
        _ => panic!("Expected Content predicate for content:\"fn main()\""),
    }

    // 2. Unquoted content expression with utf8: alias
    let ast2 = QueryParser::parse("utf8:TODO");
    match ast2.root {
        QueryNode::Predicate(SearchPredicate::Content {
            pattern,
            case_sensitive,
        }) => {
            assert_eq!(pattern, "TODO");
            assert!(!case_sensitive);
        }
        _ => panic!("Expected Content predicate for utf8:TODO"),
    }

    // 3. Combined query: ext:rs content:"struct SearchEngine"
    let ast3 = QueryParser::parse("ext:rs content:\"struct SearchEngine\"");
    match ast3.root {
        QueryNode::And(children) => {
            assert_eq!(children.len(), 2);
            assert!(matches!(
                &children[0],
                QueryNode::Predicate(SearchPredicate::Extension(exts)) if exts.contains(&"rs".to_string())
            ));
            assert!(matches!(
                &children[1],
                QueryNode::Predicate(SearchPredicate::Content { pattern, .. }) if pattern == "struct SearchEngine"
            ));
        }
        _ => panic!("Expected And node for combined query"),
    }
}

#[test]
fn test_content_search_plain_text_files() {
    let dir = tempdir().expect("Failed to create tempdir");
    let base_path = dir.path();

    let file_rs = base_path.join("code.rs");
    let file_txt = base_path.join("notes.txt");
    let file_toml = base_path.join("config.toml");

    let mut f1 = File::create(&file_rs).unwrap();
    f1.write_all(b"pub fn calculate_primes(limit: u64) -> Vec<u64> {\n    vec![]\n}\n")
        .unwrap();
    f1.sync_all().unwrap();
    drop(f1);

    let mut f2 = File::create(&file_txt).unwrap();
    f2.write_all(b"Shopping list:\n- apples\n- bananas\n- dark roast coffee\n")
        .unwrap();
    f2.sync_all().unwrap();
    drop(f2);

    let mut f3 = File::create(&file_toml).unwrap();
    f3.write_all(b"[package]\nname = \"find-me-test\"\nversion = \"0.1.0\"\n")
        .unwrap();
    f3.sync_all().unwrap();
    drop(f3);

    let mut arena = FileArena::new();
    let meta1 = fs::metadata(&file_rs).unwrap();
    let meta2 = fs::metadata(&file_txt).unwrap();
    let meta3 = fs::metadata(&file_toml).unwrap();

    arena
        .add_file(&file_rs, FileFlags::NONE, meta1.len(), 1000)
        .unwrap();
    arena
        .add_file(&file_txt, FileFlags::NONE, meta2.len(), 2000)
        .unwrap();
    arena
        .add_file(&file_toml, FileFlags::NONE, meta3.len(), 3000)
        .unwrap();

    let config = SearchConfig {
        max_results: 10,
        sort_column: SortColumn::Score,
        sort_direction: SortDirection::Descending,
    };

    // 1. Search for calculate_primes
    let ast_primes = QueryParser::parse("content:calculate_primes");
    let res_primes = SearchEngine::search(&arena, &ast_primes, &config);
    assert_eq!(res_primes.len(), 1);
    let name_primes = arena.get_name(res_primes[0].record_idx).unwrap();
    assert_eq!(name_primes, "code.rs");

    // 2. Search for coffee (case-insensitive)
    let ast_coffee = QueryParser::parse("content:\"DARK ROAST COFFEE\"");
    let res_coffee = SearchEngine::search(&arena, &ast_coffee, &config);
    assert_eq!(res_coffee.len(), 1);
    let name_coffee = arena.get_name(res_coffee[0].record_idx).unwrap();
    assert_eq!(name_coffee, "notes.txt");

    // 3. Combined: ext:toml content:find-me-test
    let ast_toml = QueryParser::parse("ext:toml content:find-me-test");
    let res_toml = SearchEngine::search(&arena, &ast_toml, &config);
    assert_eq!(res_toml.len(), 1);
    let name_toml = arena.get_name(res_toml[0].record_idx).unwrap();
    assert_eq!(name_toml, "config.toml");

    // 4. Content not matching anything
    let ast_none = QueryParser::parse("content:nonexistent_keyword_xyz");
    let res_none = SearchEngine::search(&arena, &ast_none, &config);
    assert_eq!(res_none.len(), 0);
}

#[test]
fn test_content_search_binary_file_skipped() {
    let dir = tempdir().expect("Failed to create tempdir");
    let base_path = dir.path();

    let bin_file = base_path.join("executable.bin");
    let mut f = File::create(&bin_file).unwrap();
    // Insert null bytes before text to simulate ELF binary or compressed payload
    let mut bytes = vec![0x7f, b'E', b'L', b'F', 0x00, 0x02, 0x01, 0x00];
    bytes.extend_from_slice(b"target_search_keyword_inside_binary");
    f.write_all(&bytes).unwrap();
    f.sync_all().unwrap();
    drop(f);

    let mut arena = FileArena::new();
    let meta = fs::metadata(&bin_file).unwrap();
    arena
        .add_file(&bin_file, FileFlags::NONE, meta.len(), 5000)
        .unwrap();

    let ast = QueryParser::parse("content:target_search_keyword");
    let config = SearchConfig::default();
    let res = SearchEngine::search(&arena, &ast, &config);
    assert_eq!(
        res.len(),
        0,
        "Binary file containing null bytes must be skipped"
    );
}

#[test]
fn test_content_search_boundary_overlap() {
    let dir = tempdir().expect("Failed to create tempdir");
    let base_path = dir.path();
    let large_file = base_path.join("large_log.txt");

    let chunk_size = 64 * 1024;
    let token = b"SPECIAL_BOUNDARY_CROSSING_TOKEN_12345";

    let mut file = File::create(&large_file).unwrap();
    // Fill first chunk up to chunk_size - 10 bytes
    let pad1 = vec![b'A'; chunk_size - 10];
    file.write_all(&pad1).unwrap();

    // Write token spanning across 64KB boundary (starts at 65526, ends at 65563)
    file.write_all(token).unwrap();

    // Fill remaining second chunk
    let pad2 = vec![b'B'; chunk_size];
    file.write_all(&pad2).unwrap();
    file.sync_all().unwrap();
    drop(file);

    let mut arena = FileArena::new();
    let meta = fs::metadata(&large_file).unwrap();
    arena
        .add_file(&large_file, FileFlags::NONE, meta.len(), 6000)
        .unwrap();

    let ast = QueryParser::parse("content:SPECIAL_BOUNDARY_CROSSING_TOKEN_12345");
    let config = SearchConfig::default();
    let res = SearchEngine::search(&arena, &ast, &config);
    assert_eq!(
        res.len(),
        1,
        "Token straddling 64KB buffer boundary must be detected"
    );
    let name = arena.get_name(res[0].record_idx).unwrap();
    assert_eq!(name, "large_log.txt");
}

#[test]
fn test_content_search_oversized_file_skipped() {
    let mut arena = FileArena::new();
    // File with reported size of 30MB (> 20MB limit)
    arena
        .add_file(
            Path::new("/home/user/huge_dataset.csv"),
            FileFlags::NONE,
            30 * 1024 * 1024,
            7000,
        )
        .unwrap();

    let ast = QueryParser::parse("content:test");
    let config = SearchConfig::default();
    let res = SearchEngine::search(&arena, &ast, &config);
    assert_eq!(res.len(), 0, "Files >20MB must be safely skipped");
}
