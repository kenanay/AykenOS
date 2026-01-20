//! Parser for Semantic CLI
//!
//! Converts token streams into Abstract Syntax Trees (AST).
//!
//! # Architecture
//!
//! - **Recursive descent parser:** Simple and predictable
//! - **Phase enforcement:** Rejects Extended DSL tokens in Phase 3.5.1
//! - **Error recovery:** Provides helpful error messages
//! - **Performance:** < 5ms for typical commands
//!
//! # RULE 8 Enforcement (CRITICAL)
//!
//! The parser is responsible for enforcing Phase boundaries:
//! - **Lexer:** Recognizes all tokens (Core + Extended)
//! - **Parser:** REJECTS Extended tokens in Phase 3.5.1
//! - **Validator:** Enforces semantic correctness
//!
//! This ensures that Extended DSL features cannot be used until Phase 3.5.2+.

pub mod commands;
pub mod expressions;

// Re-export for convenience
pub use commands::CommandParser;
pub use expressions::ExpressionParser;

use crate::ast::{AstNode, CommandNode};
use crate::error::{ErrorCode, Result, SemanticCLIError};
use crate::lexer::{Token, TokenKind};
use crate::types::SourceLocation;

/// Parser state
pub struct Parser {
    tokens: Vec<Token>,
    position: usize,
}

impl Parser {
    /// Create a new parser
    pub fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, position: 0 }
    }

    /// Parse tokens into AST
    pub fn parse(tokens: Vec<Token>) -> Result<AstNode> {
        let mut parser = Self::new(tokens);
        let command = parser.parse_command()?;
        parser.expect_eof()?;
        Ok(AstNode::new(command))
    }

    /// Parse a command
    pub fn parse_command(&mut self) -> Result<CommandNode> {
        self.skip_newlines();

        if self.is_at_end() {
            return Err(SemanticCLIError::syntax_error(
                self.current_location(),
                "unexpected end of input",
                "expected command",
                ErrorCode::E002,
            ));
        }

        let token = self.peek();

        // RULE 8 ENFORCEMENT: Reject Extended DSL tokens
        self.reject_extended_token(token)?;

        match &token.kind {
            TokenKind::Query => self.parse_query(),
            TokenKind::List => self.parse_list(),
            TokenKind::Show => self.parse_show(),
            TokenKind::Status => self.parse_status(),
            TokenKind::Agents => self.parse_agents(),
            TokenKind::Explain => self.parse_explain(),
            TokenKind::DryRun => self.parse_dry_run(),
            TokenKind::History => self.parse_history(),
            _ => Err(SemanticCLIError::syntax_error(
                token.location,
                format!("unexpected token '{}'", token.kind),
                "expected command (query, list, show, status, agents, explain, dry-run, history)",
                ErrorCode::E002,
            )),
        }
    }

    /// RULE 8 ENFORCEMENT: Reject Extended DSL tokens in Phase 3.5.1
    ///
    /// This is the critical security boundary that prevents Extended DSL
    /// features from being used before Phase 3.5.2+.
    fn reject_extended_token(&self, token: &Token) -> Result<()> {
        if token.kind.is_extended_keyword() {
            return Err(SemanticCLIError::semantic_error(
                token.location,
                format!(
                    "Extended DSL keyword '{}' is not allowed in Phase 3.5.1",
                    token.kind
                ),
                "This feature will be available in Phase 3.5.2+. Use Core DSL commands only.",
                ErrorCode::E103,
            ));
        }

        if token.kind.is_extended_operator() {
            return Err(SemanticCLIError::semantic_error(
                token.location,
                format!(
                    "Extended DSL operator '{}' is not allowed in Phase 3.5.1",
                    token.kind
                ),
                "Arithmetic operators will be available in Phase 3.5.2+. Use comparison operators only.",
                ErrorCode::E103,
            ));
        }

        Ok(())
    }

    /// Parse context path (e.g., "data.users")
    pub fn parse_context(&mut self) -> Result<Vec<String>> {
        let mut context = Vec::new();

        // First identifier
        let token = self.consume_identifier()?;
        context.push(token.lexeme.clone());

        // Additional identifiers separated by dots
        while self.match_token(&TokenKind::Dot) {
            let token = self.consume_identifier()?;
            context.push(token.lexeme.clone());
        }

        Ok(context)
    }

    /// Peek at current token
    pub fn peek(&self) -> &Token {
        if self.position < self.tokens.len() {
            &self.tokens[self.position]
        } else {
            // Return last token (should be EOF)
            self.tokens.last().unwrap()
        }
    }

    /// Peek at next token
    pub fn peek_next(&self) -> &Token {
        if self.position + 1 < self.tokens.len() {
            &self.tokens[self.position + 1]
        } else {
            // Return last token (should be EOF)
            self.tokens.last().unwrap()
        }
    }

    /// Advance to next token
    pub fn advance(&mut self) -> &Token {
        if !self.is_at_end() {
            self.position += 1;
        }
        self.previous()
    }

    /// Get previous token
    pub fn previous(&self) -> &Token {
        &self.tokens[self.position - 1]
    }

    /// Check if at end of tokens
    pub fn is_at_end(&self) -> bool {
        self.peek().kind == TokenKind::Eof
    }

    /// Match token kind and advance if matched
    pub fn match_token(&mut self, kind: &TokenKind) -> bool {
        if self.check(kind) {
            self.advance();
            true
        } else {
            false
        }
    }

    /// Check if current token matches kind
    pub fn check(&self, kind: &TokenKind) -> bool {
        if self.is_at_end() {
            false
        } else {
            std::mem::discriminant(&self.peek().kind) == std::mem::discriminant(kind)
        }
    }

    /// Consume token of specific kind
    pub fn consume(&mut self, kind: TokenKind, message: &str) -> Result<Token> {
        if self.check(&kind) {
            Ok(self.advance().clone())
        } else {
            Err(SemanticCLIError::syntax_error(
                self.current_location(),
                format!("expected {}, found '{}'", message, self.peek().kind),
                format!("add {}", message),
                ErrorCode::E002,
            ))
        }
    }

    /// Consume identifier token
    pub fn consume_identifier(&mut self) -> Result<Token> {
        let token = self.peek();
        match &token.kind {
            TokenKind::Identifier(_) => Ok(self.advance().clone()),
            _ => Err(SemanticCLIError::syntax_error(
                token.location,
                format!("expected identifier, found '{}'", token.kind),
                "add identifier",
                ErrorCode::E002,
            )),
        }
    }

    /// Skip newline tokens
    pub fn skip_newlines(&mut self) {
        while self.match_token(&TokenKind::Newline) {
            // Skip
        }
    }

    /// Expect end of file
    pub fn expect_eof(&mut self) -> Result<()> {
        self.skip_newlines();
        if !self.is_at_end() {
            return Err(SemanticCLIError::syntax_error(
                self.peek().location,
                format!("unexpected token '{}' after command", self.peek().kind),
                "remove extra tokens",
                ErrorCode::E002,
            ));
        }
        Ok(())
    }

    /// Get current source location
    pub fn current_location(&self) -> SourceLocation {
        self.peek().location
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::Lexer;

    fn parse_command(input: &str) -> Result<AstNode> {
        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize()?;
        Parser::parse(tokens)
    }

    #[test]
    fn test_empty_input() {
        let result = parse_command("");
        assert!(result.is_err());
    }

    #[test]
    fn test_status_command() {
        let ast = parse_command("status").unwrap();
        match ast.command {
            CommandNode::Status { .. } => {}
            _ => panic!("Expected Status command"),
        }
    }

    #[test]
    fn test_agents_command() {
        let ast = parse_command("agents").unwrap();
        match ast.command {
            CommandNode::Agents { .. } => {}
            _ => panic!("Expected Agents command"),
        }
    }

    #[test]
    fn test_extended_keyword_rejected() {
        let result = parse_command("add data.users {name: \"Alice\"}");
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("Extended DSL keyword"));
        assert!(err.to_string().contains("Phase 3.5.1"));
    }

    #[test]
    fn test_arithmetic_operator_rejected() {
        let result = parse_command("query data.users {age + 5 > 18}");
        assert!(result.is_err());
        // This will be caught during expression parsing
    }

    #[test]
    fn test_context_parsing() {
        let mut lexer = Lexer::new("data.users.active");
        let tokens = lexer.tokenize().unwrap();
        let mut parser = Parser::new(tokens);
        let context = parser.parse_context().unwrap();
        assert_eq!(context, vec!["data", "users", "active"]);
    }
}