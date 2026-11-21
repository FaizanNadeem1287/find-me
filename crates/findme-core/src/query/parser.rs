//! Fault-tolerant recursive-descent parser producing query AST

use crate::query::ast::{QueryAst, QueryNode, SearchPredicate};
use crate::query::lexer::{Lexer, Token};
use crate::query::units::{parse_attrib_filter, parse_date_filter, parse_size_filter};
use std::path::PathBuf;

/// Parser converting query strings into an Abstract Syntax Tree (AST)
pub struct QueryParser<'a> {
    tokens: Vec<Token<'a>>,
    pos: usize,
    now_secs: u64,
    depth: usize,
}

impl<'a> QueryParser<'a> {
    /// Parses a search query string into a `QueryAst` using current epoch time
    pub fn parse(query: &'a str) -> QueryAst {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        Self::parse_with_now(query, now)
    }

    /// Parses a search query string into a `QueryAst` with explicit reference epoch seconds
    pub fn parse_with_now(query: &'a str, now_secs: u64) -> QueryAst {
        let tokens: Vec<Token<'a>> = Lexer::new(query).collect();
        if tokens.is_empty() {
            return QueryAst::empty();
        }

        let mut parser = Self {
            tokens,
            pos: 0,
            now_secs,
            depth: 0,
        };

        let root = parser.parse_or_expr();
        QueryAst::new(root)
    }

    /// Peeks at the current token without consuming
    fn peek(&self) -> Option<Token<'a>> {
        self.tokens.get(self.pos).copied()
    }

    /// Advances and consumes the current token
    fn advance(&mut self) -> Option<Token<'a>> {
        let tok = self.tokens.get(self.pos).copied();
        if self.pos < self.tokens.len() {
            self.pos += 1;
        }
        tok
    }

    /// Parses logical OR expression: `AndExpr ('|' AndExpr)*`
    fn parse_or_expr(&mut self) -> QueryNode {
        let mut branches = Vec::new();
        let first = self.parse_and_expr();
        branches.push(first);

        while let Some(Token::Pipe) = self.peek() {
            self.advance(); // consume '|'
            let next_branch = self.parse_and_expr();
            branches.push(next_branch);
        }

        if branches.len() == 1 {
            branches.pop().unwrap_or(QueryNode::True)
        } else {
            QueryNode::Or(branches)
        }
    }

    /// Parses logical AND expression (space-separated sequence of unary expressions)
    fn parse_and_expr(&mut self) -> QueryNode {
        let mut terms = Vec::new();

        while let Some(tok) = self.peek() {
            // Stop on OR or CloseParen
            if tok == Token::Pipe || tok == Token::CloseParen {
                break;
            }
            let node = self.parse_unary_expr();
            terms.push(node);
        }

        if terms.is_empty() {
            QueryNode::True
        } else if terms.len() == 1 {
            terms.pop().unwrap_or(QueryNode::True)
        } else {
            QueryNode::And(terms)
        }
    }

    /// Parses logical NOT or primary expression: `! UnaryExpr | PrimaryExpr`
    fn parse_unary_expr(&mut self) -> QueryNode {
        if let Some(Token::Exclamation) = self.peek() {
            self.advance(); // consume '!'
            let inner = self.parse_unary_expr();
            QueryNode::Not(Box::new(inner))
        } else {
            self.parse_primary_expr()
        }
    }

    /// Parses primary expressions (parenthesized groups, operators, terms)
    fn parse_primary_expr(&mut self) -> QueryNode {
        const MAX_RECURSION_DEPTH: usize = 128;

        match self.peek() {
            Some(Token::OpenParen) => {
                self.advance(); // consume '('
                if self.depth >= MAX_RECURSION_DEPTH {
                    // Prevent stack overflow on pathological nesting
                    return QueryNode::True;
                }
                self.depth += 1;
                let inner = self.parse_or_expr();
                self.depth -= 1;

                if let Some(Token::CloseParen) = self.peek() {
                    self.advance(); // consume ')'
                }
                inner
            }
            Some(Token::Operator { prefix, value }) => {
                self.advance();
                let pred = self.parse_operator_predicate(prefix, value);
                QueryNode::Predicate(pred)
            }
            Some(Token::Quoted(text)) => {
                self.advance();
                QueryNode::Predicate(SearchPredicate::Text {
                    term: text.to_string(),
                    quoted: true,
                })
            }
            Some(Token::Word(text)) => {
                self.advance();
                QueryNode::Predicate(SearchPredicate::Text {
                    term: text.to_string(),
                    quoted: false,
                })
            }
            Some(Token::Pipe) | Some(Token::CloseParen) | None => QueryNode::True,
            Some(Token::Exclamation) => {
                self.advance();
                QueryNode::True
            }
        }
    }

    /// Maps operator tokens into typed search predicates with fallback on parse failure
    fn parse_operator_predicate(&self, prefix: &str, value: &str) -> SearchPredicate {
        match prefix {
            "ext" => {
                let exts: Vec<String> = value
                    .split([';', ','])
                    .map(|s| s.trim().trim_start_matches('.').to_ascii_lowercase())
                    .filter(|s| !s.is_empty())
                    .collect();
                if exts.is_empty() {
                    SearchPredicate::Text {
                        term: format!("ext:{}", value),
                        quoted: false,
                    }
                } else {
                    SearchPredicate::Extension(exts)
                }
            }
            "size" => {
                if let Some((min, max)) = parse_size_filter(value) {
                    SearchPredicate::SizeRange { min, max }
                } else {
                    // Graceful fallback to literal text
                    SearchPredicate::Text {
                        term: format!("size:{}", value),
                        quoted: false,
                    }
                }
            }
            "dm" => {
                if let Some((min_secs, max_secs)) = parse_date_filter(value, self.now_secs) {
                    SearchPredicate::DateRange { min_secs, max_secs }
                } else {
                    SearchPredicate::Text {
                        term: format!("dm:{}", value),
                        quoted: false,
                    }
                }
            }
            "path" => SearchPredicate::PathPrefix(PathBuf::from(value)),
            "attrib" => {
                let (required, forbidden) = parse_attrib_filter(value);
                SearchPredicate::Attributes {
                    required,
                    forbidden,
                }
            }
            "regex" => {
                match regex::RegexBuilder::new(value)
                    .case_insensitive(true)
                    .build()
                {
                    Ok(r) => SearchPredicate::Regex(r),
                    Err(_) => SearchPredicate::Text {
                        term: format!("regex:{}", value),
                        quoted: false,
                    },
                }
            }
            _ => SearchPredicate::Text {
                term: format!("{}:{}", prefix, value),
                quoted: false,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_plain_terms() {
        let ast = QueryParser::parse("foo bar");
        match ast.root {
            QueryNode::And(nodes) => {
                assert_eq!(nodes.len(), 2);
            }
            _ => panic!("Expected AND node"),
        }
    }

    #[test]
    fn test_parse_or_and_not() {
        let ast = QueryParser::parse("report | !doc");
        match ast.root {
            QueryNode::Or(nodes) => {
                assert_eq!(nodes.len(), 2);
                assert!(matches!(nodes[1], QueryNode::Not(_)));
            }
            _ => panic!("Expected OR node"),
        }
    }

    #[test]
    fn test_parse_extension_and_size() {
        let ast = QueryParser::parse("ext:rs;toml size:>10mb");
        match ast.root {
            QueryNode::And(nodes) => {
                assert_eq!(nodes.len(), 2);
                match &nodes[0] {
                    QueryNode::Predicate(SearchPredicate::Extension(exts)) => {
                        assert_eq!(exts, &["rs", "toml"]);
                    }
                    _ => panic!("Expected Extension predicate"),
                }
                match &nodes[1] {
                    QueryNode::Predicate(SearchPredicate::SizeRange { min, max: _ }) => {
                        assert!(min.is_some());
                    }
                    _ => panic!("Expected SizeRange predicate"),
                }
            }
            _ => panic!("Expected AND node"),
        }
    }

    #[test]
    fn test_parse_malformed_operator_graceful_fallback() {
        let ast = QueryParser::parse("size:invalid_size regex:[[[unclosed");
        // Must not panic, and must produce valid AST
        assert!(!ast.is_empty());
    }
}
