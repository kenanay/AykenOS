//! Command parsing implementation
//!
//! Implements parsing for all Core DSL commands in Phase 3.5.1:
//! - Query operations: `query`, `list`, `show`
//! - System operations: `status`, `agents`
//! - Debug operations: `explain`, `dry-run`, `history`

use crate::ast::{CommandNode, Expr};
use crate::error::{Result};
use crate::lexer::TokenKind;
use crate::parser::{ExpressionParser, Parser};

/// Command parsing implementation
pub trait CommandParser {
    fn parse_query(&mut self) -> Result<CommandNode>;
    fn parse_list(&mut self) -> Result<CommandNode>;
    fn parse_show(&mut self) -> Result<CommandNode>;
    fn parse_status(&mut self) -> Result<CommandNode>;
    fn parse_agents(&mut self) -> Result<CommandNode>;
    fn parse_explain(&mut self) -> Result<CommandNode>;
    fn parse_dry_run(&mut self) -> Result<CommandNode>;
    fn parse_history(&mut self) -> Result<CommandNode>;
}

impl CommandParser for Parser {
    /// Parse query command: `query <context> <filter>`
    fn parse_query(&mut self) -> Result<CommandNode> {
        let location = self.peek().location;
        self.consume(TokenKind::Query, "query")?;

        let context = self.parse_context()?;

        // Optional filter
        let filter = if self.check(&TokenKind::LBrace) {
            Some(self.parse_filter()?)
        } else {
            None
        };

        Ok(CommandNode::Query {
            location,
            context,
            filter,
        })
    }

    /// Parse list command: `list <context>`
    fn parse_list(&mut self) -> Result<CommandNode> {
        let location = self.peek().location;
        self.consume(TokenKind::List, "list")?;

        let context = self.parse_context()?;

        Ok(CommandNode::List { location, context })
    }

    /// Parse show command: `show <context> <id>`
    fn parse_show(&mut self) -> Result<CommandNode> {
        let location = self.peek().location;
        self.consume(TokenKind::Show, "show")?;

        let context = self.parse_context()?;
        let id = self.parse_expression()?;

        Ok(CommandNode::Show {
            location,
            context,
            id,
        })
    }

    /// Parse status command: `status`
    fn parse_status(&mut self) -> Result<CommandNode> {
        let location = self.peek().location;
        self.consume(TokenKind::Status, "status")?;

        Ok(CommandNode::Status { location })
    }

    /// Parse agents command: `agents`
    fn parse_agents(&mut self) -> Result<CommandNode> {
        let location = self.peek().location;
        self.consume(TokenKind::Agents, "agents")?;

        Ok(CommandNode::Agents { location })
    }

    /// Parse explain command: `explain <command>`
    fn parse_explain(&mut self) -> Result<CommandNode> {
        let location = self.peek().location;
        self.consume(TokenKind::Explain, "explain")?;

        let command = Box::new(self.parse_command()?);

        Ok(CommandNode::Explain { location, command })
    }

    /// Parse dry-run command: `dry-run <command>`
    fn parse_dry_run(&mut self) -> Result<CommandNode> {
        let location = self.peek().location;
        self.consume(TokenKind::DryRun, "dry-run")?;

        let command = Box::new(self.parse_command()?);

        Ok(CommandNode::DryRun { location, command })
    }

    /// Parse history command: `history`
    fn parse_history(&mut self) -> Result<CommandNode> {
        let location = self.peek().location;
        self.consume(TokenKind::History, "history")?;

        Ok(CommandNode::History { location })
    }
}

impl Parser {
    /// Parse filter expression: `{age > 18}`
    pub fn parse_filter(&mut self) -> Result<Expr> {
        self.consume(TokenKind::LBrace, "{")?;
        let expr = self.parse_expression()?;
        self.consume(TokenKind::RBrace, "}")?;
        Ok(expr)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::Lexer;
    use crate::parser::Parser;

    fn parse_command(input: &str) -> Result<CommandNode> {
        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize()?;
        let mut parser = Parser::new(tokens);
        parser.parse_command()
    }

    #[test]
    fn test_parse_query_without_filter() {
        let cmd = parse_command("query data.users").unwrap();
        match cmd {
            CommandNode::Query {
                context, filter, ..
            } => {
                assert_eq!(context, vec!["data", "users"]);
                assert!(filter.is_none());
            }
            _ => panic!("Expected Query command"),
        }
    }

    #[test]
    fn test_parse_query_with_filter() {
        let cmd = parse_command("query data.users {age > 18}").unwrap();
        match cmd {
            CommandNode::Query {
                context, filter, ..
            } => {
                assert_eq!(context, vec!["data", "users"]);
                assert!(filter.is_some());
            }
            _ => panic!("Expected Query command"),
        }
    }

    #[test]
    fn test_parse_list() {
        let cmd = parse_command("list data.users").unwrap();
        match cmd {
            CommandNode::List { context, .. } => {
                assert_eq!(context, vec!["data", "users"]);
            }
            _ => panic!("Expected List command"),
        }
    }

    #[test]
    fn test_parse_show() {
        let cmd = parse_command("show data.users 123").unwrap();
        match cmd {
            CommandNode::Show { context, .. } => {
                assert_eq!(context, vec!["data", "users"]);
            }
            _ => panic!("Expected Show command"),
        }
    }

    #[test]
    fn test_parse_status() {
        let cmd = parse_command("status").unwrap();
        match cmd {
            CommandNode::Status { .. } => {}
            _ => panic!("Expected Status command"),
        }
    }

    #[test]
    fn test_parse_agents() {
        let cmd = parse_command("agents").unwrap();
        match cmd {
            CommandNode::Agents { .. } => {}
            _ => panic!("Expected Agents command"),
        }
    }

    #[test]
    fn test_parse_explain() {
        let cmd = parse_command("explain status").unwrap();
        match cmd {
            CommandNode::Explain { command, .. } => match command.as_ref() {
                CommandNode::Status { .. } => {}
                _ => panic!("Expected Status command inside Explain"),
            },
            _ => panic!("Expected Explain command"),
        }
    }

    #[test]
    fn test_parse_dry_run() {
        let cmd = parse_command("dry-run agents").unwrap();
        match cmd {
            CommandNode::DryRun { command, .. } => match command.as_ref() {
                CommandNode::Agents { .. } => {}
                _ => panic!("Expected Agents command inside DryRun"),
            },
            _ => panic!("Expected DryRun command"),
        }
    }

    #[test]
    fn test_parse_history() {
        let cmd = parse_command("history").unwrap();
        match cmd {
            CommandNode::History { .. } => {}
            _ => panic!("Expected History command"),
        }
    }

    #[test]
    fn test_invalid_command() {
        let result = parse_command("invalid");
        assert!(result.is_err());
    }

    #[test]
    fn test_missing_context() {
        let result = parse_command("query");
        assert!(result.is_err());
    }
}