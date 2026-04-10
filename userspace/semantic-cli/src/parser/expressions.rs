//! Expression parsing implementation
//!
//! Implements operator precedence climbing for expressions.
//!
//! # Phase 3.5.1 Scope
//!
//! **CORE DSL OPERATORS:**
//! - Comparison: `==`, `!=`, `<`, `<=`, `>`, `>=`
//! - Logical: `and`, `or`, `not`
//!
//! **EXTENDED DSL OPERATORS (REJECTED):**
//! - Arithmetic: `+`, `-`, `*`, `/`
//!
//! # Precedence (highest to lowest)
//!
//! 1. Primary expressions (identifiers, literals, parentheses)
//! 2. Unary operators (`not`)
//! 3. Comparison operators (`<`, `<=`, `>`, `>=`)
//! 4. Equality operators (`==`, `!=`)
//! 5. Logical AND (`and`)
//! 6. Logical OR (`or`)

use crate::ast::{BinaryOp, Expr, UnaryOp};
use crate::error::{ErrorCode, Result, SemanticCLIError};
use crate::lexer::TokenKind;
use crate::parser::Parser;

/// Expression parsing implementation
pub trait ExpressionParser {
    fn parse_expression(&mut self) -> Result<Expr>;
    fn parse_expression_with_precedence(&mut self, min_precedence: u8) -> Result<Expr>;
    fn parse_unary(&mut self) -> Result<Expr>;
    fn parse_primary(&mut self) -> Result<Expr>;
}

impl ExpressionParser for Parser {
    /// Parse expression (entry point)
    fn parse_expression(&mut self) -> Result<Expr> {
        self.parse_expression_with_precedence(0)
    }

    /// Parse expression with operator precedence climbing
    fn parse_expression_with_precedence(&mut self, min_precedence: u8) -> Result<Expr> {
        let mut left = self.parse_unary()?;

        loop {
            // Check for Extended DSL operators and reject them
            match &self.peek().kind {
                TokenKind::Plus | TokenKind::Minus | TokenKind::Star | TokenKind::Slash => {
                    return Err(SemanticCLIError::semantic_error(
                        self.peek().location,
                        format!(
                            "Extended DSL operator '{}' is not allowed in Phase 3.5.1",
                            self.peek().kind
                        ),
                        "Arithmetic operators will be available in Phase 3.5.2+. Use comparison operators only.",
                        ErrorCode::E103,
                    ));
                }
                _ => {}
            }

            if let Some(op) = self.peek_binary_operator() {
                let precedence = op.precedence();
                if precedence < min_precedence {
                    break;
                }

                // Consume the operator
                self.advance();

                // Right-associative operators would use precedence, left-associative use precedence + 1
                let next_min_precedence = if op.is_left_associative() {
                    precedence + 1
                } else {
                    precedence
                };

                let right = self.parse_expression_with_precedence(next_min_precedence)?;

                left = Expr::Binary {
                    left: Box::new(left),
                    op,
                    right: Box::new(right),
                    location: self.current_location(),
                };
            } else {
                break;
            }
        }

        Ok(left)
    }

    /// Parse unary expression
    fn parse_unary(&mut self) -> Result<Expr> {
        if let Some(op) = self.peek_unary_operator() {
            let location = self.peek().location;
            self.advance(); // Consume operator

            let operand = self.parse_unary()?; // Right-associative

            Ok(Expr::Unary {
                op,
                operand: Box::new(operand),
                location,
            })
        } else {
            self.parse_primary()
        }
    }

    /// Parse primary expression (identifiers, literals, parentheses)
    fn parse_primary(&mut self) -> Result<Expr> {
        let token = self.peek();
        let location = token.location;

        match &token.kind {
            TokenKind::Identifier(name) => {
                let name = name.clone();
                self.advance();
                Ok(Expr::Identifier { name, location })
            }

            TokenKind::Number(value) => {
                let value = value.clone();
                self.advance();
                Ok(Expr::Number { value, location })
            }

            TokenKind::String(value) => {
                let value = value.clone();
                self.advance();
                Ok(Expr::String { value, location })
            }

            TokenKind::Boolean(value) => {
                let value = *value;
                self.advance();
                Ok(Expr::Boolean { value, location })
            }

            TokenKind::LParen => {
                self.advance(); // Consume '('
                let expr = self.parse_expression()?;
                self.consume(TokenKind::RParen, ")")?;
                Ok(expr)
            }

            // RULE 8 ENFORCEMENT: Reject Extended DSL operators in expressions
            TokenKind::Plus | TokenKind::Minus | TokenKind::Star | TokenKind::Slash => {
                Err(SemanticCLIError::semantic_error(
                    location,
                    format!(
                        "Extended DSL operator '{}' is not allowed in Phase 3.5.1",
                        token.kind
                    ),
                    "Arithmetic operators will be available in Phase 3.5.2+. Use comparison operators only.",
                    ErrorCode::E103,
                ))
            }

            _ => Err(SemanticCLIError::syntax_error(
                location,
                format!("unexpected token '{}' in expression", token.kind),
                "expected identifier, number, string, boolean, or '('",
                ErrorCode::E002,
            )),
        }
    }
}

impl Parser {
    /// Peek at binary operator (if current token is one)
    pub fn peek_binary_operator(&self) -> Option<BinaryOp> {
        match &self.peek().kind {
            TokenKind::Eq => Some(BinaryOp::Eq),
            TokenKind::Ne => Some(BinaryOp::Ne),
            TokenKind::Lt => Some(BinaryOp::Lt),
            TokenKind::Le => Some(BinaryOp::Le),
            TokenKind::Gt => Some(BinaryOp::Gt),
            TokenKind::Ge => Some(BinaryOp::Ge),
            TokenKind::And => Some(BinaryOp::And),
            TokenKind::Or => Some(BinaryOp::Or),

            // RULE 8 ENFORCEMENT: Extended DSL operators are not supported
            TokenKind::Plus | TokenKind::Minus | TokenKind::Star | TokenKind::Slash => {
                // These will be handled as errors in parse_expression_with_precedence
                None
            }

            _ => None,
        }
    }

    /// Peek at unary operator (if current token is one)
    pub fn peek_unary_operator(&self) -> Option<UnaryOp> {
        match &self.peek().kind {
            TokenKind::Not => Some(UnaryOp::Not),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::Lexer;

    fn parse_expression(input: &str) -> Result<Expr> {
        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize()?;
        let mut parser = Parser::new(tokens);
        parser.parse_expression()
    }

    #[test]
    fn test_parse_identifier() {
        let expr = parse_expression("age").unwrap();
        match expr {
            Expr::Identifier { name, .. } => assert_eq!(name, "age"),
            _ => panic!("Expected Identifier"),
        }
    }

    #[test]
    fn test_parse_number() {
        let expr = parse_expression("42").unwrap();
        match expr {
            Expr::Number { value, .. } => assert_eq!(value, "42"),
            _ => panic!("Expected Number"),
        }
    }

    #[test]
    fn test_parse_string() {
        let expr = parse_expression("\"hello\"").unwrap();
        match expr {
            Expr::String { value, .. } => assert_eq!(value, "hello"),
            _ => panic!("Expected String"),
        }
    }

    #[test]
    fn test_parse_boolean() {
        let expr = parse_expression("true").unwrap();
        match expr {
            Expr::Boolean { value, .. } => assert!(value),
            _ => panic!("Expected Boolean"),
        }
    }

    #[test]
    fn test_parse_comparison() {
        let expr = parse_expression("age > 18").unwrap();
        match expr {
            Expr::Binary { op, .. } => assert_eq!(op, BinaryOp::Gt),
            _ => panic!("Expected Binary expression"),
        }
    }

    #[test]
    fn test_parse_equality() {
        let expr = parse_expression("name == \"Alice\"").unwrap();
        match expr {
            Expr::Binary { op, .. } => assert_eq!(op, BinaryOp::Eq),
            _ => panic!("Expected Binary expression"),
        }
    }

    #[test]
    fn test_parse_logical_and() {
        let expr = parse_expression("age > 18 and active").unwrap();
        match expr {
            Expr::Binary { op, .. } => assert_eq!(op, BinaryOp::And),
            _ => panic!("Expected Binary expression"),
        }
    }

    #[test]
    fn test_parse_logical_or() {
        let expr = parse_expression("age < 18 or age > 65").unwrap();
        match expr {
            Expr::Binary { op, .. } => assert_eq!(op, BinaryOp::Or),
            _ => panic!("Expected Binary expression"),
        }
    }

    #[test]
    fn test_parse_unary_not() {
        let expr = parse_expression("not active").unwrap();
        match expr {
            Expr::Unary { op, .. } => assert_eq!(op, UnaryOp::Not),
            _ => panic!("Expected Unary expression"),
        }
    }

    #[test]
    fn test_parse_parentheses() {
        let expr = parse_expression("(age > 18)").unwrap();
        match expr {
            Expr::Binary { op, .. } => assert_eq!(op, BinaryOp::Gt),
            _ => panic!("Expected Binary expression"),
        }
    }

    #[test]
    fn test_operator_precedence() {
        // age > 18 and active should parse as (age > 18) and active
        let expr = parse_expression("age > 18 and active").unwrap();
        match expr {
            Expr::Binary {
                op: BinaryOp::And,
                left,
                ..
            } => match left.as_ref() {
                Expr::Binary {
                    op: BinaryOp::Gt, ..
                } => {}
                _ => panic!("Expected Gt as left operand of And"),
            },
            _ => panic!("Expected And expression"),
        }
    }

    #[test]
    fn test_complex_expression() {
        let expr = parse_expression("age > 18 and name == \"Alice\" or not active").unwrap();
        match expr {
            Expr::Binary {
                op: BinaryOp::Or, ..
            } => {}
            _ => panic!("Expected Or as top-level operator"),
        }
    }

    #[test]
    fn test_arithmetic_operator_rejected() {
        let result = parse_expression("age + 5");
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("Extended DSL operator"));
        assert!(err.to_string().contains("Phase 3.5.1"));
    }

    #[test]
    fn test_invalid_expression() {
        let result = parse_expression("@invalid");
        assert!(result.is_err());
    }
}
