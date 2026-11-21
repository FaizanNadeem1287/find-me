//! Query parsing and Abstract Syntax Tree (AST) evaluation module.
//!
//! Provides zero-allocation lexical analysis, unit and date parsing,
//! and recursive-descent AST generation for Windows Everything-style syntax.

pub mod ast;
pub mod lexer;
pub mod parser;
pub mod units;

pub use ast::{QueryAst, QueryNode, SearchPredicate};
pub use lexer::{Lexer, Token, KNOWN_OPERATORS};
pub use parser::QueryParser;
pub use units::{
    parse_attrib_filter, parse_date_filter, parse_iso_date, parse_size_bytes, parse_size_filter,
};
