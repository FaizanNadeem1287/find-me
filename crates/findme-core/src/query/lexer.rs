//! Zero-allocation lexical analyzer for search query strings

/// Tokens yielded by the query lexer
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Token<'a> {
    /// Plain word or search term
    Word(&'a str),
    /// Explicitly quoted string (quotes stripped)
    Quoted(&'a str),
    /// Search operator with prefix and value (e.g. prefix="ext", value="rs;toml")
    Operator { prefix: &'a str, value: &'a str },
    /// Logical OR operator ('|')
    Pipe,
    /// Logical NOT operator ('!')
    Exclamation,
    /// Grouping opening parenthesis ('(')
    OpenParen,
    /// Grouping closing parenthesis (')')
    CloseParen,
}

/// Known operators supported by the query parser
pub const KNOWN_OPERATORS: &[&str] = &["ext:", "size:", "dm:", "path:", "attrib:", "regex:"];

/// Lightweight zero-copy tokenizer for query strings
pub struct Lexer<'a> {
    input: &'a str,
    pos: usize,
}

impl<'a> Lexer<'a> {
    /// Creates a new lexer for the given query input
    pub fn new(input: &'a str) -> Self {
        Self { input, pos: 0 }
    }

    /// Skips whitespace and returns next character without advancing
    fn peek(&self) -> Option<char> {
        self.input[self.pos..].chars().next()
    }

    /// Advances position by number of bytes
    fn advance(&mut self, bytes: usize) {
        self.pos = (self.pos + bytes).min(self.input.len());
    }

    /// Skips all ASCII whitespace
    fn skip_whitespace(&mut self) {
        while let Some(c) = self.peek() {
            if c.is_whitespace() {
                self.advance(c.len_utf8());
            } else {
                break;
            }
        }
    }
}

impl<'a> Iterator for Lexer<'a> {
    type Item = Token<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        self.skip_whitespace();

        if self.pos >= self.input.len() {
            return None;
        }

        let remaining = &self.input[self.pos..];

        // 1. Single character punctuation tokens
        let first_char = remaining.chars().next()?;
        match first_char {
            '|' => {
                self.advance(1);
                return Some(Token::Pipe);
            }
            '!' => {
                self.advance(1);
                return Some(Token::Exclamation);
            }
            '(' => {
                self.advance(1);
                return Some(Token::OpenParen);
            }
            ')' => {
                self.advance(1);
                return Some(Token::CloseParen);
            }
            '"' => {
                // Quoted string
                self.advance(1);
                let start = self.pos;
                let end = match self.input[start..].find('"') {
                    Some(idx) => {
                        self.pos = start + idx + 1;
                        start + idx
                    }
                    None => {
                        // Unclosed quote: consume rest of string without panicking
                        self.pos = self.input.len();
                        self.input.len()
                    }
                };
                return Some(Token::Quoted(&self.input[start..end]));
            }
            _ => {}
        }

        // 2. Check for known operators: ext:, size:, dm:, path:, attrib:, regex:
        for &op in KNOWN_OPERATORS {
            if remaining.starts_with(op) {
                let prefix = &op[..op.len() - 1]; // strip ':'
                self.advance(op.len());

                // Value can be quoted or word
                let val_remaining = &self.input[self.pos..];
                if val_remaining.starts_with('"') {
                    self.advance(1);
                    let start = self.pos;
                    let end = match self.input[start..].find('"') {
                        Some(idx) => {
                            self.pos = start + idx + 1;
                            start + idx
                        }
                        None => {
                            self.pos = self.input.len();
                            self.input.len()
                        }
                    };
                    return Some(Token::Operator {
                        prefix,
                        value: &self.input[start..end],
                    });
                }

                // Unquoted operator value: consume until whitespace or punctuation
                let start = self.pos;
                let mut len = 0;
                for c in val_remaining.chars() {
                    if c.is_whitespace() || c == '|' || c == '(' || c == ')' {
                        break;
                    }
                    len += c.len_utf8();
                }
                self.advance(len);
                return Some(Token::Operator {
                    prefix,
                    value: &self.input[start..start + len],
                });
            }
        }

        // 3. Regular word / search term
        let start = self.pos;
        let mut len = 0;
        for c in remaining.chars() {
            if c.is_whitespace() || c == '|' || c == '!' || c == '(' || c == ')' || c == '"' {
                break;
            }
            len += c.len_utf8();
        }

        if len == 0 {
            // Safety advance to prevent infinite loop on edge-case characters
            let char_len = first_char.len_utf8();
            self.advance(char_len);
            return Some(Token::Word(&self.input[start..start + char_len]));
        }

        self.advance(len);
        Some(Token::Word(&self.input[start..start + len]))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lexer_basic_tokens() {
        let input = "cargo test | !release (ext:rs size:>10mb)";
        let tokens: Vec<_> = Lexer::new(input).collect();

        assert_eq!(
            tokens,
            vec![
                Token::Word("cargo"),
                Token::Word("test"),
                Token::Pipe,
                Token::Exclamation,
                Token::Word("release"),
                Token::OpenParen,
                Token::Operator {
                    prefix: "ext",
                    value: "rs"
                },
                Token::Operator {
                    prefix: "size",
                    value: ">10mb"
                },
                Token::CloseParen,
            ]
        );
    }

    #[test]
    fn test_lexer_quoted_operator_value() {
        let input = "path:\"/home/user/my documents\" ext:\"rs;toml\"";
        let tokens: Vec<_> = Lexer::new(input).collect();

        assert_eq!(
            tokens,
            vec![
                Token::Operator {
                    prefix: "path",
                    value: "/home/user/my documents"
                },
                Token::Operator {
                    prefix: "ext",
                    value: "rs;toml"
                },
            ]
        );
    }

    #[test]
    fn test_lexer_unclosed_quotes() {
        let input = "\"unclosed quote at end";
        let tokens: Vec<_> = Lexer::new(input).collect();
        assert_eq!(tokens, vec![Token::Quoted("unclosed quote at end")]);
    }
}
