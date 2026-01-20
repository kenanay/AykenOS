//! Token types for Semantic CLI
//!
//! Defines all token types used by the lexer.
//!
//! # Architectural Rule 8: Core DSL vs Extended DSL
//!
//! **CRITICAL:** This file defines tokens for BOTH Core DSL (Phase 3.5.1) and Extended DSL (Phase 3.5.2+).
//!
//! ## Why are Extended tokens defined here?
//!
//! 1. **Lexer is syntax-agnostic:** The lexer only recognizes tokens, it doesn't enforce semantics.
//! 2. **Parser enforces phase boundaries:** The parser MUST reject Extended tokens in Phase 3.5.1.
//! 3. **Forward compatibility:** Defining all tokens now prevents breaking changes later.
//!
//! ## Phase Enforcement
//!
//! - **Lexer (this file):** Recognizes all tokens (Core + Extended)
//! - **Parser:** MUST reject Extended tokens in Phase 3.5.1
//! - **Validator:** MUST enforce Core DSL semantics in Phase 3.5.1
//!
//! ## Core DSL (LOCKED - Phase 3.5.1)
//!
//! - Context operations: `query`, `list`, `show`
//! - System operations: `status`, `agents`
//! - Debug operations: `explain`, `dry-run`, `history`
//! - Comparison operators: `==`, `!=`, `<`, `<=`, `>`, `>=`
//! - Delimiters: `.`, `,`, `:`, `|`, `{`, `}`, `[`, `]`, `(`, `)`
//!
//! ## Extended DSL (Phase 3.5.2+)
//!
//! - Mutation operations: `add`, `update`, `delete`
//! - Pipeline operations: `pipeline`
//! - Orchestration: `orchestrate`
//! - Security: `permissions`, `sandbox`
//! - Logical operators: `and`, `or`, `not`
//! - Arithmetic operators: `+`, `-`, `*`, `/`
//!
//! **These Extended tokens are recognized by the lexer but MUST be rejected by the parser in Phase 3.5.1.**

use crate::types::SourceLocation;
use serde::{Deserialize, Serialize};
use std::fmt;

/// Token type
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Token {
    /// Token type
    pub kind: TokenKind,
    /// Source location
    pub location: SourceLocation,
    /// Lexeme (original text)
    pub lexeme: String,
}

impl Token {
    /// Create a new token
    pub fn new(kind: TokenKind, location: SourceLocation, lexeme: impl Into<String>) -> Self {
        Self {
            kind,
            location,
            lexeme: lexeme.into(),
        }
    }
}

/// Token kind
///
/// **ARCHITECTURAL RULE 8:** DSL Core vs Extended
///
/// This enum is divided into:
/// - **CORE DSL (LOCKED):** Minimal, stable, Phase 3.5.1 only
/// - **EXTENDED DSL:** Rich features, Phase 3.5.2+
///
/// **CRITICAL:** Core DSL MUST NOT grow without architectural review.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TokenKind {
    // ========================================================================
    // CORE DSL (LOCKED - Phase 3.5.1)
    // ========================================================================
    // These tokens are STABLE and UNCHANGING.
    // Changes require architectural review.
    
    // Context Operations (CORE)
    Query,         // query <context> <filter>
    List,          // list <context>
    Show,          // show <context> <id>
    
    // System Operations (CORE)
    Status,        // status
    Agents,        // agents
    
    // Debug Operations (CORE)
    Explain,       // explain <command>
    DryRun,        // dry-run <command>
    History,       // history
    
    // ========================================================================
    // EXTENDED DSL (Phase 3.5.2+)
    // ========================================================================
    // These tokens are for future phases.
    // They are defined here for completeness but MUST NOT be used in Phase 3.5.1.
    
    // Mutation Operations (EXTENDED - Phase 3.5.2)
    Add,           // add <context> <data>
    Update,        // update <context> <id> <data>
    Delete,        // delete <context> <id>
    
    // Pipeline Operations (EXTENDED - Phase 3.5.2)
    Pipeline,      // pipeline[...]
    
    // Orchestration Operations (EXTENDED - Phase 3.5.2)
    Orchestrate,   // orchestrate <task>
    
    // Security Operations (EXTENDED - Phase 3.5.2)
    Permissions,   // permissions <context>
    Sandbox,       // sandbox <command>
    
    // Logical Operators (EXTENDED - Phase 3.5.2)
    And,           // and
    Or,            // or
    Not,           // not
    
    // Arithmetic Operators (EXTENDED - Phase 3.5.2)
    Plus,          // +
    Minus,         // -
    Star,          // *
    Slash,         // /
    
    // ========================================================================
    // LITERALS AND IDENTIFIERS (CORE)
    // ========================================================================
    
    Identifier(String),
    String(String),
    Number(String), // Store as string to preserve precision
    Boolean(bool),
    
    // ========================================================================
    // OPERATORS (CORE)
    // ========================================================================
    
    Dot,           // .
    Comma,         // ,
    Colon,         // :
    Pipe,          // |
    LBrace,        // {
    RBrace,        // }
    LBracket,      // [
    RBracket,      // ]
    LParen,        // (
    RParen,        // )
    
    // Comparison (CORE)
    Eq,            // ==
    Ne,            // !=
    Lt,            // <
    Le,            // <=
    Gt,            // >
    Ge,            // >=
    
    // ========================================================================
    // SPECIAL
    // ========================================================================
    
    Eof,
    Newline,
}

impl fmt::Display for TokenKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Query => write!(f, "query"),
            Self::List => write!(f, "list"),
            Self::Show => write!(f, "show"),
            Self::Add => write!(f, "add"),
            Self::Update => write!(f, "update"),
            Self::Delete => write!(f, "delete"),
            Self::Pipeline => write!(f, "pipeline"),
            Self::Status => write!(f, "status"),
            Self::Agents => write!(f, "agents"),
            Self::Orchestrate => write!(f, "orchestrate"),
            Self::Explain => write!(f, "explain"),
            Self::DryRun => write!(f, "dry-run"),
            Self::History => write!(f, "history"),
            Self::Permissions => write!(f, "permissions"),
            Self::Sandbox => write!(f, "sandbox"),
            Self::Identifier(s) => write!(f, "identifier({})", s),
            Self::String(s) => write!(f, "string(\"{}\")", s),
            Self::Number(n) => write!(f, "number({})", n),
            Self::Boolean(b) => write!(f, "boolean({})", b),
            Self::Dot => write!(f, "."),
            Self::Comma => write!(f, ","),
            Self::Colon => write!(f, ":"),
            Self::Pipe => write!(f, "|"),
            Self::LBrace => write!(f, "{{"),
            Self::RBrace => write!(f, "}}"),
            Self::LBracket => write!(f, "["),
            Self::RBracket => write!(f, "]"),
            Self::LParen => write!(f, "("),
            Self::RParen => write!(f, ")"),
            Self::Eq => write!(f, "=="),
            Self::Ne => write!(f, "!="),
            Self::Lt => write!(f, "<"),
            Self::Le => write!(f, "<="),
            Self::Gt => write!(f, ">"),
            Self::Ge => write!(f, ">="),
            Self::And => write!(f, "and"),
            Self::Or => write!(f, "or"),
            Self::Not => write!(f, "not"),
            Self::Plus => write!(f, "+"),
            Self::Minus => write!(f, "-"),
            Self::Star => write!(f, "*"),
            Self::Slash => write!(f, "/"),
            Self::Eof => write!(f, "EOF"),
            Self::Newline => write!(f, "\\n"),
        }
    }
}

impl TokenKind {
    /// Check if token is a CORE DSL keyword (Phase 3.5.1)
    pub fn is_core_keyword(&self) -> bool {
        matches!(
            self,
            Self::Query
                | Self::List
                | Self::Show
                | Self::Status
                | Self::Agents
                | Self::Explain
                | Self::DryRun
                | Self::History
        )
    }

    /// Check if token is an EXTENDED DSL keyword (Phase 3.5.2+)
    pub fn is_extended_keyword(&self) -> bool {
        matches!(
            self,
            Self::Add
                | Self::Update
                | Self::Delete
                | Self::Pipeline
                | Self::Orchestrate
                | Self::Permissions
                | Self::Sandbox
                | Self::And
                | Self::Or
                | Self::Not
        )
    }

    /// Check if token is any keyword (CORE or EXTENDED)
    pub fn is_keyword(&self) -> bool {
        self.is_core_keyword() || self.is_extended_keyword()
    }

    /// Check if token is a CORE operator (Phase 3.5.1)
    pub fn is_core_operator(&self) -> bool {
        matches!(
            self,
            Self::Dot
                | Self::Comma
                | Self::Colon
                | Self::Pipe
                | Self::Eq
                | Self::Ne
                | Self::Lt
                | Self::Le
                | Self::Gt
                | Self::Ge
        )
    }

    /// Check if token is an EXTENDED operator (Phase 3.5.2+)
    pub fn is_extended_operator(&self) -> bool {
        matches!(
            self,
            Self::Plus | Self::Minus | Self::Star | Self::Slash
        )
    }

    /// Check if token is any operator (CORE or EXTENDED)
    pub fn is_operator(&self) -> bool {
        self.is_core_operator() || self.is_extended_operator()
    }

    /// Check if token is a delimiter
    pub fn is_delimiter(&self) -> bool {
        matches!(
            self,
            Self::LBrace
                | Self::RBrace
                | Self::LBracket
                | Self::RBracket
                | Self::LParen
                | Self::RParen
        )
    }
}

/// Keyword lookup
///
/// **ARCHITECTURAL RULE 8:** This function recognizes both CORE and EXTENDED keywords.
/// Parser MUST enforce that only CORE keywords are used in Phase 3.5.1.
pub fn keyword_from_str(s: &str) -> Option<TokenKind> {
    match s {
        // CORE DSL (Phase 3.5.1)
        "query" => Some(TokenKind::Query),
        "list" => Some(TokenKind::List),
        "show" => Some(TokenKind::Show),
        "status" => Some(TokenKind::Status),
        "agents" => Some(TokenKind::Agents),
        "explain" => Some(TokenKind::Explain),
        "dry-run" => Some(TokenKind::DryRun),
        "history" => Some(TokenKind::History),
        
        // EXTENDED DSL (Phase 3.5.2+)
        // These are recognized by lexer but MUST be rejected by parser in Phase 3.5.1
        "add" => Some(TokenKind::Add),
        "update" => Some(TokenKind::Update),
        "delete" => Some(TokenKind::Delete),
        "pipeline" => Some(TokenKind::Pipeline),
        "orchestrate" => Some(TokenKind::Orchestrate),
        "permissions" => Some(TokenKind::Permissions),
        "sandbox" => Some(TokenKind::Sandbox),
        "and" => Some(TokenKind::And),
        "or" => Some(TokenKind::Or),
        "not" => Some(TokenKind::Not),
        
        // Literals
        "true" => Some(TokenKind::Boolean(true)),
        "false" => Some(TokenKind::Boolean(false)),
        
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_token_creation() {
        let token = Token::new(
            TokenKind::Query,
            SourceLocation::new(1, 1, 0),
            "query",
        );
        assert_eq!(token.kind, TokenKind::Query);
        assert_eq!(token.lexeme, "query");
    }

    #[test]
    fn test_keyword_lookup() {
        assert_eq!(keyword_from_str("query"), Some(TokenKind::Query));
        assert_eq!(keyword_from_str("list"), Some(TokenKind::List));
        assert_eq!(keyword_from_str("true"), Some(TokenKind::Boolean(true)));
        assert_eq!(keyword_from_str("false"), Some(TokenKind::Boolean(false)));
        assert_eq!(keyword_from_str("unknown"), None);
    }

    #[test]
    fn test_token_kind_checks() {
        // Core keywords
        assert!(TokenKind::Query.is_core_keyword());
        assert!(TokenKind::List.is_core_keyword());
        assert!(TokenKind::Status.is_core_keyword());
        assert!(!TokenKind::Add.is_core_keyword());
        
        // Extended keywords
        assert!(TokenKind::Add.is_extended_keyword());
        assert!(TokenKind::Pipeline.is_extended_keyword());
        assert!(TokenKind::And.is_extended_keyword());
        assert!(!TokenKind::Query.is_extended_keyword());
        
        // All keywords
        assert!(TokenKind::Query.is_keyword());
        assert!(TokenKind::Add.is_keyword());
        assert!(!TokenKind::Identifier("test".to_string()).is_keyword());

        // Core operators
        assert!(TokenKind::Dot.is_core_operator());
        assert!(TokenKind::Eq.is_core_operator());
        assert!(!TokenKind::Plus.is_core_operator());
        
        // Extended operators
        assert!(TokenKind::Plus.is_extended_operator());
        assert!(TokenKind::Star.is_extended_operator());
        assert!(!TokenKind::Dot.is_extended_operator());
        
        // All operators
        assert!(TokenKind::Dot.is_operator());
        assert!(TokenKind::Plus.is_operator());
        assert!(!TokenKind::Query.is_operator());

        // Delimiters
        assert!(TokenKind::LBrace.is_delimiter());
        assert!(TokenKind::RBracket.is_delimiter());
        assert!(!TokenKind::Dot.is_delimiter());
    }

    #[test]
    fn test_token_display() {
        assert_eq!(format!("{}", TokenKind::Query), "query");
        assert_eq!(format!("{}", TokenKind::Eq), "==");
        assert_eq!(format!("{}", TokenKind::LBrace), "{");
    }
}
