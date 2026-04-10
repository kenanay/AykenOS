//! Lexer (Tokenizer) for Semantic CLI
//!
//! Converts input strings into token streams using a hand-written lexer.
//! Features:
//! - Zero-copy tokenization where possible
//! - Source location tracking
//! - Error recovery
//! - Performance target: < 5ms for typical commands

pub mod tokens;

// Re-export for convenience
pub use tokens::{keyword_from_str, Token, TokenKind};

use crate::error::{ErrorCode, Result, SemanticCLIError};
use crate::types::SourceLocation;

/// Lexer state
pub struct Lexer<'a> {
    input: &'a str,
    position: usize,
    line: usize,
    column: usize,
}

impl<'a> Lexer<'a> {
    /// Create a new lexer
    pub fn new(input: &'a str) -> Self {
        Self {
            input,
            position: 0,
            line: 1,
            column: 1,
        }
    }

    /// Tokenize the entire input
    pub fn tokenize(&mut self) -> Result<Vec<Token>> {
        let mut tokens = Vec::new();

        loop {
            self.skip_whitespace();

            if self.is_at_end() {
                tokens.push(Token::new(TokenKind::Eof, self.current_location(), ""));
                break;
            }

            let token = self.next_token()?;
            tokens.push(token);
        }

        Ok(tokens)
    }

    /// Get the next token
    fn next_token(&mut self) -> Result<Token> {
        let start_location = self.current_location();
        let ch = self.peek();

        match ch {
            // Single-character tokens
            '.' => self.make_token(TokenKind::Dot, 1),
            ',' => self.make_token(TokenKind::Comma, 1),
            ':' => self.make_token(TokenKind::Colon, 1),
            '|' => self.make_token(TokenKind::Pipe, 1),
            '{' => self.make_token(TokenKind::LBrace, 1),
            '}' => self.make_token(TokenKind::RBrace, 1),
            '[' => self.make_token(TokenKind::LBracket, 1),
            ']' => self.make_token(TokenKind::RBracket, 1),
            '(' => self.make_token(TokenKind::LParen, 1),
            ')' => self.make_token(TokenKind::RParen, 1),
            '+' => self.make_token(TokenKind::Plus, 1),
            '-' => self.make_token(TokenKind::Minus, 1),
            '*' => self.make_token(TokenKind::Star, 1),
            '/' => self.make_token(TokenKind::Slash, 1),

            // Two-character tokens
            '=' => {
                if self.peek_next() == '=' {
                    self.make_token(TokenKind::Eq, 2)
                } else {
                    Err(SemanticCLIError::syntax_error(
                        start_location,
                        "unexpected character '='",
                        "did you mean '=='?",
                        ErrorCode::E001,
                    ))
                }
            }
            '!' => {
                if self.peek_next() == '=' {
                    self.make_token(TokenKind::Ne, 2)
                } else {
                    Err(SemanticCLIError::syntax_error(
                        start_location,
                        "unexpected character '!'",
                        "did you mean '!='?",
                        ErrorCode::E001,
                    ))
                }
            }
            '<' => {
                if self.peek_next() == '=' {
                    self.make_token(TokenKind::Le, 2)
                } else {
                    self.make_token(TokenKind::Lt, 1)
                }
            }
            '>' => {
                if self.peek_next() == '=' {
                    self.make_token(TokenKind::Ge, 2)
                } else {
                    self.make_token(TokenKind::Gt, 1)
                }
            }

            // String literals
            '"' => self.tokenize_string(),

            // Numbers
            '0'..='9' => self.tokenize_number(),

            // Identifiers and keywords
            'a'..='z' | 'A'..='Z' | '_' => self.tokenize_identifier(),

            // Newline
            '\n' => {
                // advance() will handle line/column tracking
                self.make_token(TokenKind::Newline, 1)
            }

            // Invalid character
            _ => Err(SemanticCLIError::syntax_error(
                start_location,
                format!("unexpected character '{}'", ch),
                "remove this character",
                ErrorCode::E001,
            )),
        }
    }

    /// Tokenize a string literal
    fn tokenize_string(&mut self) -> Result<Token> {
        let start_location = self.current_location();
        let start_pos = self.position;

        self.advance(); // Skip opening quote

        let mut value = String::new();
        let mut escaped = false;

        while !self.is_at_end() {
            let ch = self.peek();

            if escaped {
                // Handle escape sequences
                let escaped_char = match ch {
                    'n' => '\n',
                    't' => '\t',
                    'r' => '\r',
                    '\\' => '\\',
                    '"' => '"',
                    _ => {
                        return Err(SemanticCLIError::syntax_error(
                            self.current_location(),
                            format!("invalid escape sequence '\\{}'", ch),
                            "valid escape sequences: \\n, \\t, \\r, \\\\, \\\"",
                            ErrorCode::E005,
                        ))
                    }
                };
                value.push(escaped_char);
                escaped = false;
                self.advance();
            } else if ch == '\\' {
                escaped = true;
                self.advance();
            } else if ch == '"' {
                self.advance(); // Skip closing quote
                let lexeme = &self.input[start_pos..self.position];
                return Ok(Token::new(TokenKind::String(value), start_location, lexeme));
            } else if ch == '\n' {
                return Err(SemanticCLIError::syntax_error(
                    self.current_location(),
                    "unterminated string literal",
                    "add closing quote",
                    ErrorCode::E003,
                ));
            } else {
                value.push(ch);
                self.advance();
            }
        }

        Err(SemanticCLIError::syntax_error(
            start_location,
            "unterminated string literal",
            "add closing quote",
            ErrorCode::E003,
        ))
    }

    /// Tokenize a number literal
    fn tokenize_number(&mut self) -> Result<Token> {
        let start_location = self.current_location();
        let start_pos = self.position;

        // Integer part
        while !self.is_at_end() && self.peek().is_ascii_digit() {
            self.advance();
        }

        // Decimal part
        if !self.is_at_end() && self.peek() == '.' && self.peek_next().is_ascii_digit() {
            self.advance(); // Skip '.'
            while !self.is_at_end() && self.peek().is_ascii_digit() {
                self.advance();
            }
        }

        let lexeme = &self.input[start_pos..self.position];
        Ok(Token::new(
            TokenKind::Number(lexeme.to_string()),
            start_location,
            lexeme,
        ))
    }

    /// Tokenize an identifier or keyword
    fn tokenize_identifier(&mut self) -> Result<Token> {
        let start_location = self.current_location();
        let start_pos = self.position;

        while !self.is_at_end() {
            let ch = self.peek();
            if ch.is_alphanumeric() || ch == '_' || ch == '-' {
                self.advance();
            } else {
                break;
            }
        }

        let lexeme = &self.input[start_pos..self.position];

        // Check if it's a keyword
        let kind =
            keyword_from_str(lexeme).unwrap_or_else(|| TokenKind::Identifier(lexeme.to_string()));

        Ok(Token::new(kind, start_location, lexeme))
    }

    /// Make a token with a fixed length
    fn make_token(&mut self, kind: TokenKind, len: usize) -> Result<Token> {
        let location = self.current_location();
        let start_pos = self.position;
        for _ in 0..len {
            self.advance();
        }
        let lexeme = &self.input[start_pos..self.position];
        Ok(Token::new(kind, location, lexeme))
    }

    /// Skip whitespace (except newlines)
    fn skip_whitespace(&mut self) {
        while !self.is_at_end() {
            let ch = self.peek();
            if ch == ' ' || ch == '\t' || ch == '\r' {
                self.advance();
            } else {
                break;
            }
        }
    }

    /// Peek at the current character
    fn peek(&self) -> char {
        self.input[self.position..].chars().next().unwrap_or('\0')
    }

    /// Peek at the next character
    fn peek_next(&self) -> char {
        self.input[self.position..].chars().nth(1).unwrap_or('\0')
    }

    /// Advance to the next character
    ///
    /// Automatically handles newline tracking for correct line/column numbers.
    fn advance(&mut self) {
        if !self.is_at_end() {
            let ch = self.peek();
            self.position += ch.len_utf8();

            // Handle newline for correct line/column tracking
            if ch == '\n' {
                self.line += 1;
                self.column = 1;
            } else {
                self.column += 1;
            }
        }
    }

    /// Check if at end of input
    fn is_at_end(&self) -> bool {
        self.position >= self.input.len()
    }

    /// Get current source location
    fn current_location(&self) -> SourceLocation {
        SourceLocation::new(self.line, self.column, self.position)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_input() {
        let mut lexer = Lexer::new("");
        let tokens = lexer.tokenize().unwrap();
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0].kind, TokenKind::Eof);
    }

    #[test]
    fn test_single_token() {
        let mut lexer = Lexer::new("query");
        let tokens = lexer.tokenize().unwrap();
        assert_eq!(tokens.len(), 2); // query + EOF
        assert_eq!(tokens[0].kind, TokenKind::Query);
        assert_eq!(tokens[1].kind, TokenKind::Eof);
    }

    #[test]
    fn test_operators() {
        let mut lexer = Lexer::new(". , : | { } [ ] ( )");
        let tokens = lexer.tokenize().unwrap();
        assert_eq!(tokens[0].kind, TokenKind::Dot);
        assert_eq!(tokens[1].kind, TokenKind::Comma);
        assert_eq!(tokens[2].kind, TokenKind::Colon);
        assert_eq!(tokens[3].kind, TokenKind::Pipe);
        assert_eq!(tokens[4].kind, TokenKind::LBrace);
        assert_eq!(tokens[5].kind, TokenKind::RBrace);
        assert_eq!(tokens[6].kind, TokenKind::LBracket);
        assert_eq!(tokens[7].kind, TokenKind::RBracket);
        assert_eq!(tokens[8].kind, TokenKind::LParen);
        assert_eq!(tokens[9].kind, TokenKind::RParen);
    }

    #[test]
    fn test_comparison_operators() {
        let mut lexer = Lexer::new("== != < <= > >=");
        let tokens = lexer.tokenize().unwrap();
        assert_eq!(tokens[0].kind, TokenKind::Eq);
        assert_eq!(tokens[1].kind, TokenKind::Ne);
        assert_eq!(tokens[2].kind, TokenKind::Lt);
        assert_eq!(tokens[3].kind, TokenKind::Le);
        assert_eq!(tokens[4].kind, TokenKind::Gt);
        assert_eq!(tokens[5].kind, TokenKind::Ge);
    }

    #[test]
    fn test_string_literal() {
        let mut lexer = Lexer::new(r#""hello world""#);
        let tokens = lexer.tokenize().unwrap();
        assert_eq!(tokens[0].kind, TokenKind::String("hello world".to_string()));
    }

    #[test]
    fn test_string_escape_sequences() {
        let mut lexer = Lexer::new(r#""hello\nworld\t\"test\"""#);
        let tokens = lexer.tokenize().unwrap();
        assert_eq!(
            tokens[0].kind,
            TokenKind::String("hello\nworld\t\"test\"".to_string())
        );
    }

    #[test]
    fn test_unterminated_string() {
        let mut lexer = Lexer::new(r#""hello"#);
        let result = lexer.tokenize();
        assert!(result.is_err());
    }

    #[test]
    fn test_number_integer() {
        let mut lexer = Lexer::new("42");
        let tokens = lexer.tokenize().unwrap();
        assert_eq!(tokens[0].kind, TokenKind::Number("42".to_string()));
    }

    #[test]
    fn test_number_decimal() {
        let mut lexer = Lexer::new("3.14");
        let tokens = lexer.tokenize().unwrap();
        assert_eq!(tokens[0].kind, TokenKind::Number("3.14".to_string()));
    }

    #[test]
    fn test_identifier() {
        let mut lexer = Lexer::new("data users_table");
        let tokens = lexer.tokenize().unwrap();
        assert_eq!(tokens[0].kind, TokenKind::Identifier("data".to_string()));
        assert_eq!(
            tokens[1].kind,
            TokenKind::Identifier("users_table".to_string())
        );
    }

    #[test]
    fn test_keywords() {
        let mut lexer = Lexer::new("query list show add update delete");
        let tokens = lexer.tokenize().unwrap();
        assert_eq!(tokens[0].kind, TokenKind::Query);
        assert_eq!(tokens[1].kind, TokenKind::List);
        assert_eq!(tokens[2].kind, TokenKind::Show);
        assert_eq!(tokens[3].kind, TokenKind::Add);
        assert_eq!(tokens[4].kind, TokenKind::Update);
        assert_eq!(tokens[5].kind, TokenKind::Delete);
    }

    #[test]
    fn test_boolean_literals() {
        let mut lexer = Lexer::new("true false");
        let tokens = lexer.tokenize().unwrap();
        assert_eq!(tokens[0].kind, TokenKind::Boolean(true));
        assert_eq!(tokens[1].kind, TokenKind::Boolean(false));
    }

    #[test]
    fn test_complex_command() {
        let mut lexer = Lexer::new(r#"query data.users {age > 18}"#);
        let tokens = lexer.tokenize().unwrap();
        assert_eq!(tokens[0].kind, TokenKind::Query);
        assert_eq!(tokens[1].kind, TokenKind::Identifier("data".to_string()));
        assert_eq!(tokens[2].kind, TokenKind::Dot);
        assert_eq!(tokens[3].kind, TokenKind::Identifier("users".to_string()));
        assert_eq!(tokens[4].kind, TokenKind::LBrace);
        assert_eq!(tokens[5].kind, TokenKind::Identifier("age".to_string()));
        assert_eq!(tokens[6].kind, TokenKind::Gt);
        assert_eq!(tokens[7].kind, TokenKind::Number("18".to_string()));
        assert_eq!(tokens[8].kind, TokenKind::RBrace);
    }

    #[test]
    fn test_source_location_tracking() {
        let mut lexer = Lexer::new("query\nlist");
        let tokens = lexer.tokenize().unwrap();
        assert_eq!(tokens[0].location.line, 1);
        assert_eq!(tokens[0].location.column, 1);
        assert_eq!(tokens[1].location.line, 1); // newline
        assert_eq!(tokens[2].location.line, 2);
        assert_eq!(tokens[2].location.column, 1);
    }

    #[test]
    fn test_invalid_character() {
        let mut lexer = Lexer::new("@");
        let result = lexer.tokenize();
        assert!(result.is_err());
    }
}
