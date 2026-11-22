//! Abstract Syntax Tree (AST) definitions for search queries

use crate::record::FileFlags;
use std::path::PathBuf;

/// Leaf predicate evaluating a specific file attribute or name condition
#[derive(Clone, Debug)]
pub enum SearchPredicate {
    /// Case-insensitive text term for substring or fuzzy matching
    Text { term: String, quoted: bool },
    /// Extension filter matching any of the specified extensions (e.g. ["rs", "toml"])
    Extension(Vec<String>),
    /// File size range in bytes [min, max]
    SizeRange { min: Option<u64>, max: Option<u64> },
    /// Date modified range in Unix epoch seconds [min_secs, max_secs]
    DateRange { min_secs: u64, max_secs: u64 },
    /// Directory path prefix constraint (e.g. "/home/user/code")
    PathPrefix(PathBuf),
    /// Regular expression compiled for filename matching
    Regex(regex::Regex),
    /// File attribute flags filter (required flags must be set, forbidden must not)
    Attributes {
        required: FileFlags,
        forbidden: FileFlags,
    },
    /// Full-text file content search inside plain text files
    Content {
        pattern: String,
        case_sensitive: bool,
    },
    /// Explicit typo-tolerant fuzzy search requested via fuzzy: prefix
    Fuzzy(String),
}

impl PartialEq for SearchPredicate {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Fuzzy(f1), Self::Fuzzy(f2)) => f1 == f2,
            (
                Self::Text {
                    term: t1,
                    quoted: q1,
                },
                Self::Text {
                    term: t2,
                    quoted: q2,
                },
            ) => t1 == t2 && q1 == q2,
            (Self::Extension(e1), Self::Extension(e2)) => e1 == e2,
            (
                Self::SizeRange {
                    min: min1,
                    max: max1,
                },
                Self::SizeRange {
                    min: min2,
                    max: max2,
                },
            ) => min1 == min2 && max1 == max2,
            (
                Self::DateRange {
                    min_secs: min1,
                    max_secs: max1,
                },
                Self::DateRange {
                    min_secs: min2,
                    max_secs: max2,
                },
            ) => min1 == min2 && max1 == max2,
            (Self::PathPrefix(p1), Self::PathPrefix(p2)) => p1 == p2,
            (Self::Regex(r1), Self::Regex(r2)) => r1.as_str() == r2.as_str(),
            (
                Self::Attributes {
                    required: req1,
                    forbidden: forb1,
                },
                Self::Attributes {
                    required: req2,
                    forbidden: forb2,
                },
            ) => req1 == req2 && forb1 == forb2,
            (
                Self::Content {
                    pattern: p1,
                    case_sensitive: c1,
                },
                Self::Content {
                    pattern: p2,
                    case_sensitive: c2,
                },
            ) => p1 == p2 && c1 == c2,
            _ => false,
        }
    }
}

/// Node in the query expression tree
#[derive(Clone, Debug, PartialEq)]
pub enum QueryNode {
    /// Matches all child nodes (AND logic, implicit between terms)
    And(Vec<QueryNode>),
    /// Matches any child node (OR logic, separated by '|')
    Or(Vec<QueryNode>),
    /// Inverts match result (NOT logic, prefixed by '!')
    Not(Box<QueryNode>),
    /// Leaf search predicate
    Predicate(SearchPredicate),
    /// Always matches (e.g. empty query)
    True,
}

impl QueryNode {
    /// Checks whether the query node is empty (matches everything)
    pub fn is_empty(&self) -> bool {
        match self {
            Self::True => true,
            Self::And(v) if v.is_empty() => true,
            Self::Or(v) if v.is_empty() => true,
            _ => false,
        }
    }
}

/// Complete parsed query AST
#[derive(Clone, Debug, PartialEq)]
pub struct QueryAst {
    pub root: QueryNode,
}

impl QueryAst {
    /// Creates a new query AST from a root node
    pub fn new(root: QueryNode) -> Self {
        Self { root }
    }

    /// Creates an empty query AST that matches everything
    pub fn empty() -> Self {
        Self {
            root: QueryNode::True,
        }
    }

    /// Whether the query is completely empty
    pub fn is_empty(&self) -> bool {
        self.root.is_empty()
    }
}
