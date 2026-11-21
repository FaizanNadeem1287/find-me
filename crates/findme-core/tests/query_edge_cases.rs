//! Comprehensive edge-case test suite for query lexer and AST parser
//! Verifies 50+ query variations, operator precedence, fault tolerance, and graceful fallbacks.

use findme_core::query::ast::{QueryNode, SearchPredicate};
use findme_core::query::units::{
    parse_attrib_filter, parse_date_filter, parse_iso_date, parse_size_bytes, parse_size_filter,
};
use findme_core::query::{Lexer, QueryParser, Token};
use findme_core::record::FileFlags;
use std::path::PathBuf;

// Reference timestamp for predictable date testing: 2026-11-21 12:00:00 UTC
const TEST_NOW_SECS: u64 = 1795262400;

#[test]
fn test_query_empty_and_whitespace() {
    assert!(QueryParser::parse("").is_empty());
    assert!(QueryParser::parse("   ").is_empty());
    assert!(QueryParser::parse("\t\n\r  ").is_empty());
}

#[test]
fn test_query_single_plain_terms() {
    let ast = QueryParser::parse("findme");
    assert_eq!(
        ast.root,
        QueryNode::Predicate(SearchPredicate::Text {
            term: "findme".to_string(),
            quoted: false,
        })
    );

    let ast_case = QueryParser::parse("CARGO_PKG");
    assert_eq!(
        ast_case.root,
        QueryNode::Predicate(SearchPredicate::Text {
            term: "CARGO_PKG".to_string(),
            quoted: false,
        })
    );
}

#[test]
fn test_query_implicit_and_multiple_words() {
    let ast = QueryParser::parse("src main rust");
    match ast.root {
        QueryNode::And(terms) => {
            assert_eq!(terms.len(), 3);
            assert_eq!(
                terms[0],
                QueryNode::Predicate(SearchPredicate::Text {
                    term: "src".to_string(),
                    quoted: false,
                })
            );
            assert_eq!(
                terms[1],
                QueryNode::Predicate(SearchPredicate::Text {
                    term: "main".to_string(),
                    quoted: false,
                })
            );
            assert_eq!(
                terms[2],
                QueryNode::Predicate(SearchPredicate::Text {
                    term: "rust".to_string(),
                    quoted: false,
                })
            );
        }
        _ => panic!("Expected QueryNode::And"),
    }
}

#[test]
fn test_query_quoted_exact_terms() {
    let ast = QueryParser::parse("\"exact phrase match\"");
    assert_eq!(
        ast.root,
        QueryNode::Predicate(SearchPredicate::Text {
            term: "exact phrase match".to_string(),
            quoted: true,
        })
    );
}

#[test]
fn test_query_unclosed_quotes() {
    let ast = QueryParser::parse("\"unclosed quote phrase");
    assert_eq!(
        ast.root,
        QueryNode::Predicate(SearchPredicate::Text {
            term: "unclosed quote phrase".to_string(),
            quoted: true,
        })
    );
}

#[test]
fn test_query_empty_quotes() {
    let ast = QueryParser::parse("\"\"");
    assert_eq!(
        ast.root,
        QueryNode::Predicate(SearchPredicate::Text {
            term: String::new(),
            quoted: true,
        })
    );
}

#[test]
fn test_query_logical_or_pipe() {
    let ast = QueryParser::parse("foo | bar | baz");
    match ast.root {
        QueryNode::Or(branches) => {
            assert_eq!(branches.len(), 3);
            assert_eq!(
                branches[0],
                QueryNode::Predicate(SearchPredicate::Text {
                    term: "foo".to_string(),
                    quoted: false,
                })
            );
            assert_eq!(
                branches[1],
                QueryNode::Predicate(SearchPredicate::Text {
                    term: "bar".to_string(),
                    quoted: false,
                })
            );
            assert_eq!(
                branches[2],
                QueryNode::Predicate(SearchPredicate::Text {
                    term: "baz".to_string(),
                    quoted: false,
                })
            );
        }
        _ => panic!("Expected QueryNode::Or"),
    }
}

#[test]
fn test_query_logical_not_exclamation() {
    let ast = QueryParser::parse("!unwanted");
    assert_eq!(
        ast.root,
        QueryNode::Not(Box::new(QueryNode::Predicate(SearchPredicate::Text {
            term: "unwanted".to_string(),
            quoted: false,
        })))
    );

    let ast_quoted = QueryParser::parse("!\"bad phrase\"");
    assert_eq!(
        ast_quoted.root,
        QueryNode::Not(Box::new(QueryNode::Predicate(SearchPredicate::Text {
            term: "bad phrase".to_string(),
            quoted: true,
        })))
    );
}

#[test]
fn test_query_parentheses_precedence() {
    let ast = QueryParser::parse("(foo | bar) baz");
    match ast.root {
        QueryNode::And(terms) => {
            assert_eq!(terms.len(), 2);
            assert!(matches!(terms[0], QueryNode::Or(_)));
            assert_eq!(
                terms[1],
                QueryNode::Predicate(SearchPredicate::Text {
                    term: "baz".to_string(),
                    quoted: false,
                })
            );
        }
        _ => panic!("Expected QueryNode::And at root"),
    }
}

#[test]
fn test_query_unbalanced_and_nested_parentheses() {
    // Should never panic on unmatched or unclosed parentheses
    let ast1 = QueryParser::parse("((foo | bar)");
    assert!(!ast1.is_empty());

    let ast2 = QueryParser::parse("foo) | (bar");
    assert!(!ast2.is_empty());

    let ast3 = QueryParser::parse("(((((())))))");
    assert!(ast3.root == QueryNode::True);
}

#[test]
fn test_query_consecutive_and_trailing_operators() {
    // Consecutive NOT
    let ast1 = QueryParser::parse("! ! foo");
    assert!(!ast1.is_empty());

    // Consecutive OR
    let ast2 = QueryParser::parse("foo || bar");
    assert!(!ast2.is_empty());

    // Trailing operators
    let ast3 = QueryParser::parse("!");
    assert!(matches!(ast3.root, QueryNode::Not(_) | QueryNode::True));

    let ast4 = QueryParser::parse("|");
    assert!(matches!(ast4.root, QueryNode::Or(_) | QueryNode::True));

    let ast5 = QueryParser::parse("foo |");
    assert!(!ast5.is_empty());

    let ast6 = QueryParser::parse("foo !");
    assert!(!ast6.is_empty());
}

#[test]
fn test_query_extension_filter() {
    let ast = QueryParser::parse("ext:rs");
    assert_eq!(
        ast.root,
        QueryNode::Predicate(SearchPredicate::Extension(vec!["rs".to_string()]))
    );

    let ast_multi = QueryParser::parse("ext:rs;toml;md");
    assert_eq!(
        ast_multi.root,
        QueryNode::Predicate(SearchPredicate::Extension(vec![
            "rs".to_string(),
            "toml".to_string(),
            "md".to_string(),
        ]))
    );

    let ast_dots = QueryParser::parse("ext:.tar.gz;.rs");
    assert_eq!(
        ast_dots.root,
        QueryNode::Predicate(SearchPredicate::Extension(vec![
            "tar.gz".to_string(),
            "rs".to_string(),
        ]))
    );

    let ast_case = QueryParser::parse("ext:RS,TOML");
    assert_eq!(
        ast_case.root,
        QueryNode::Predicate(SearchPredicate::Extension(vec![
            "rs".to_string(),
            "toml".to_string(),
        ]))
    );

    // Empty extension falls back to text predicate
    let ast_empty = QueryParser::parse("ext:");
    assert_eq!(
        ast_empty.root,
        QueryNode::Predicate(SearchPredicate::Text {
            term: "ext:".to_string(),
            quoted: false,
        })
    );
}

#[test]
fn test_query_size_filters() {
    let ast_gt = QueryParser::parse("size:>10mb");
    assert_eq!(
        ast_gt.root,
        QueryNode::Predicate(SearchPredicate::SizeRange {
            min: Some(10 * 1024 * 1024 + 1),
            max: None,
        })
    );

    let ast_lte = QueryParser::parse("size:<=500kb");
    assert_eq!(
        ast_lte.root,
        QueryNode::Predicate(SearchPredicate::SizeRange {
            min: None,
            max: Some(500 * 1024),
        })
    );

    let ast_range = QueryParser::parse("size:100kb..5mb");
    assert_eq!(
        ast_range.root,
        QueryNode::Predicate(SearchPredicate::SizeRange {
            min: Some(100 * 1024),
            max: Some(5 * 1024 * 1024),
        })
    );

    let ast_empty = QueryParser::parse("size:empty");
    assert_eq!(
        ast_empty.root,
        QueryNode::Predicate(SearchPredicate::SizeRange {
            min: Some(0),
            max: Some(0),
        })
    );

    // Fallback on invalid size
    let ast_bad = QueryParser::parse("size:gibberish");
    assert_eq!(
        ast_bad.root,
        QueryNode::Predicate(SearchPredicate::Text {
            term: "size:gibberish".to_string(),
            quoted: false,
        })
    );
}

#[test]
fn test_query_date_filters() {
    let ast_today = QueryParser::parse_with_now("dm:today", TEST_NOW_SECS);
    match ast_today.root {
        QueryNode::Predicate(SearchPredicate::DateRange { min_secs, max_secs }) => {
            assert!(min_secs <= TEST_NOW_SECS);
            assert!(max_secs >= TEST_NOW_SECS);
        }
        _ => panic!("Expected DateRange predicate"),
    }

    let ast_iso = QueryParser::parse_with_now("dm:2026-11-20", TEST_NOW_SECS);
    assert!(matches!(
        ast_iso.root,
        QueryNode::Predicate(SearchPredicate::DateRange { .. })
    ));

    let ast_range = QueryParser::parse_with_now("dm:2026-01-01..2026-06-30", TEST_NOW_SECS);
    assert!(matches!(
        ast_range.root,
        QueryNode::Predicate(SearchPredicate::DateRange { .. })
    ));

    // Fallback on invalid date
    let ast_bad = QueryParser::parse("dm:invalid-date");
    assert_eq!(
        ast_bad.root,
        QueryNode::Predicate(SearchPredicate::Text {
            term: "dm:invalid-date".to_string(),
            quoted: false,
        })
    );
}

#[test]
fn test_query_path_prefix_filter() {
    let ast = QueryParser::parse("path:/home/user/code");
    assert_eq!(
        ast.root,
        QueryNode::Predicate(SearchPredicate::PathPrefix(PathBuf::from(
            "/home/user/code"
        )))
    );

    let ast_quoted = QueryParser::parse("path:\"/media/usb drive/docs\"");
    assert_eq!(
        ast_quoted.root,
        QueryNode::Predicate(SearchPredicate::PathPrefix(PathBuf::from(
            "/media/usb drive/docs"
        )))
    );
}

#[test]
fn test_query_attribute_filter() {
    let (req, forb) = parse_attrib_filter("d");
    assert!(req.contains(FileFlags::IS_DIR));
    assert_eq!(forb, FileFlags::NONE);

    let (req2, forb2) = parse_attrib_filter("!d+h");
    assert!(forb2.contains(FileFlags::IS_DIR));
    assert!(req2.contains(FileFlags::IS_HIDDEN));

    let ast = QueryParser::parse("attrib:d");
    assert_eq!(
        ast.root,
        QueryNode::Predicate(SearchPredicate::Attributes {
            required: FileFlags::IS_DIR,
            forbidden: FileFlags::NONE,
        })
    );
}

#[test]
fn test_query_regex_filter() {
    let ast = QueryParser::parse("regex:^lib.*\\.rs$");
    match ast.root {
        QueryNode::Predicate(SearchPredicate::Regex(ref re)) => {
            assert!(re.is_match("libfindme.rs"));
            assert!(!re.is_match("main.rs"));
        }
        _ => panic!("Expected Regex predicate"),
    }

    // Invalid regex gracefully falls back to text predicate
    let ast_bad = QueryParser::parse("regex:[[[unclosed");
    assert_eq!(
        ast_bad.root,
        QueryNode::Predicate(SearchPredicate::Text {
            term: "regex:[[[unclosed".to_string(),
            quoted: false,
        })
    );
}

#[test]
fn test_query_complex_combined_queries() {
    let query = "ext:rs;toml size:>1mb !path:/target (dm:today | regex:^lib)";
    let ast = QueryParser::parse_with_now(query, TEST_NOW_SECS);

    match ast.root {
        QueryNode::And(nodes) => {
            assert_eq!(nodes.len(), 4);
            assert!(matches!(
                nodes[0],
                QueryNode::Predicate(SearchPredicate::Extension(_))
            ));
            assert!(matches!(
                nodes[1],
                QueryNode::Predicate(SearchPredicate::SizeRange { .. })
            ));
            assert!(matches!(nodes[2], QueryNode::Not(_)));
            match &nodes[3] {
                QueryNode::Or(branches) => {
                    assert_eq!(branches.len(), 2);
                    assert!(matches!(
                        branches[0],
                        QueryNode::Predicate(SearchPredicate::DateRange { .. })
                    ));
                    assert!(matches!(
                        branches[1],
                        QueryNode::Predicate(SearchPredicate::Regex(_))
                    ));
                }
                _ => panic!("Expected OR group inside complex query"),
            }
        }
        _ => panic!("Expected AND root node for complex query"),
    }
}

#[test]
fn test_50_query_variations_matrix() {
    // 50 diverse query variations ensuring fault tolerance and zero panics
    let variations = [
        "file.txt",
        "*.rs",
        "foo bar baz",
        "\"exact phrase\"",
        "\"unclosed quote",
        "foo | bar",
        "foo | bar | baz | qux",
        "!test",
        "!\"quoted unwanted\"",
        "(foo | bar)",
        "((nested))",
        "(a | b) (c | d)",
        "! (a | b)",
        "ext:rs",
        "ext:rs;toml",
        "ext:.png,.jpg,.jpeg",
        "ext:\"tar.gz\"",
        "ext:",
        "size:>10mb",
        "size:>=1gb",
        "size:<500kb",
        "size:<=1tb",
        "size:100kb..5mb",
        "size:..10mb",
        "size:10mb..",
        "size:empty",
        "size:=1024",
        "size:invalid",
        "dm:today",
        "dm:yesterday",
        "dm:lastweek",
        "dm:last7days",
        "dm:thismonth",
        "dm:2026-11-20",
        "dm:>2026-01-01",
        "dm:<2026-12-31",
        "dm:2026-01-01..2026-12-31",
        "dm:bad-date",
        "path:/home/user",
        "path:\"/path with spaces/\"",
        "attrib:d",
        "attrib:!d",
        "attrib:h",
        "attrib:!h",
        "attrib:d+h",
        "attrib:-d-h",
        "regex:^[a-z0-9_]+$",
        "regex:[[[bad",
        "ext:rs size:>100kb !path:/target",
        "(ext:pdf | ext:docx) \"financial report\" dm:thismonth",
    ];

    assert_eq!(variations.len(), 50);

    for (idx, q) in variations.iter().enumerate() {
        let ast = QueryParser::parse(q);
        assert!(
            !ast.is_empty(),
            "Query #{} '{}' should produce a non-empty AST",
            idx + 1,
            q
        );
    }
}

#[test]
fn test_unit_parsing_utilities_directly() {
    // Size parsing
    assert_eq!(parse_size_bytes("0"), Some(0));
    assert_eq!(parse_size_bytes("1024"), Some(1024));
    assert_eq!(parse_size_bytes("10b"), Some(10));
    assert_eq!(parse_size_bytes("1k"), Some(1024));
    assert_eq!(parse_size_bytes("1kb"), Some(1024));
    assert_eq!(parse_size_bytes("2m"), Some(2 * 1024 * 1024));
    assert_eq!(parse_size_bytes("3mb"), Some(3 * 1024 * 1024));
    assert_eq!(parse_size_bytes("1g"), Some(1024 * 1024 * 1024));
    assert_eq!(parse_size_bytes("1gb"), Some(1024 * 1024 * 1024));
    assert_eq!(parse_size_bytes("1t"), Some(1024 * 1024 * 1024 * 1024));
    assert_eq!(parse_size_bytes("1tb"), Some(1024 * 1024 * 1024 * 1024));
    assert_eq!(parse_size_bytes("empty"), Some(0));
    assert_eq!(parse_size_bytes("unknown_unit"), None);

    // Size filter ranges
    assert_eq!(parse_size_filter("10..20"), Some((Some(10), Some(20))));
    assert_eq!(parse_size_filter("..100"), Some((None, Some(100))));
    assert_eq!(parse_size_filter("100.."), Some((Some(100), None)));
    assert_eq!(parse_size_filter("=50"), Some((Some(50), Some(50))));

    // Date parsing
    assert!(parse_iso_date("2026-11-21").is_some());
    assert!(parse_iso_date("invalid-date").is_none());
    assert!(parse_iso_date("2026-13-01").is_none());
    assert!(parse_iso_date("2026-11-32").is_none());

    assert!(parse_date_filter("today", TEST_NOW_SECS).is_some());
    assert!(parse_date_filter("yesterday", TEST_NOW_SECS).is_some());
    assert!(parse_date_filter("lastweek", TEST_NOW_SECS).is_some());
    assert!(parse_date_filter("thismonth", TEST_NOW_SECS).is_some());
    assert!(parse_date_filter("2026-01-01..2026-12-31", TEST_NOW_SECS).is_some());
    assert!(parse_date_filter(">2026-01-01", TEST_NOW_SECS).is_some());
    assert!(parse_date_filter("<2026-01-01", TEST_NOW_SECS).is_some());
}

#[test]
fn test_lexer_token_coverage() {
    let query = "hello | !world (ext:rs \"quoted term\")";
    let mut lexer = Lexer::new(query);

    assert_eq!(lexer.next(), Some(Token::Word("hello")));
    assert_eq!(lexer.next(), Some(Token::Pipe));
    assert_eq!(lexer.next(), Some(Token::Exclamation));
    assert_eq!(lexer.next(), Some(Token::Word("world")));
    assert_eq!(lexer.next(), Some(Token::OpenParen));
    assert_eq!(
        lexer.next(),
        Some(Token::Operator {
            prefix: "ext",
            value: "rs"
        })
    );
    assert_eq!(lexer.next(), Some(Token::Quoted("quoted term")));
    assert_eq!(lexer.next(), Some(Token::CloseParen));
    assert_eq!(lexer.next(), None);
}
