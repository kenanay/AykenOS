use std::fmt;

use crate::capability_gate::AiCapabilitySet;
use crate::suggestion::suggest;

/// Logger for AI stub operations
pub struct Logger;

impl Logger {
    pub fn log(&self, message: &str) {
        println!("[AI-STUB] {}", message);
    }
}

/// Error type for AI operations
#[derive(Debug)]
pub enum AiError {
    InvalidPrompt,
    CapabilityDenied,
    AuthorityBoundaryViolation,
    RuntimeError(String),
}

impl fmt::Display for AiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AiError::InvalidPrompt => write!(f, "Invalid prompt provided"),
            AiError::CapabilityDenied => write!(f, "AI capability denied"),
            AiError::AuthorityBoundaryViolation => {
                write!(f, "AI authority boundary violation")
            }
            AiError::RuntimeError(msg) => write!(f, "Runtime error: {}", msg),
        }
    }
}

impl std::error::Error for AiError {}

/// AI stub implementation for Phase 2 documentation
/// This is a placeholder implementation that logs queries and returns placeholder responses
pub struct AiStub {
    logger: Logger,
}

impl AiStub {
    /// Create a new AI stub instance
    pub fn new() -> Self {
        Self {
            logger: Logger,
        }
    }

    /// Process an AI query - Phase 2 implementation logs only
    pub fn ask(&self, prompt: &str) -> Result<String, AiError> {
        self.logger.log(&format!("AI query: {}", prompt));
        Ok(suggest(prompt, &AiCapabilitySet::suggestion_only())?
            .content()
            .to_string())
    }

    pub fn ask_with_capabilities(
        &self,
        prompt: &str,
        capabilities: &AiCapabilitySet,
    ) -> Result<String, AiError> {
        self.logger.log(&format!("AI query: {}", prompt));
        Ok(suggest(prompt, capabilities)?.content().to_string())
    }
}

impl Default for AiStub {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ai_stub_ask() {
        let ai_stub = AiStub::new();
        let result = ai_stub.ask("What is the weather today?");
        
        assert!(result.is_ok());
        assert_eq!(
            result.unwrap(),
            "Consider documenting the request and asking a human operator to choose the next step."
        );
    }

    #[test]
    fn test_ai_stub_empty_prompt() {
        let ai_stub = AiStub::new();
        let result = ai_stub.ask("");
        
        assert!(matches!(result, Err(AiError::InvalidPrompt)));
    }

    #[test]
    fn test_ai_stub_requires_capability_in_explicit_path() {
        let ai_stub = AiStub::new();
        let result = ai_stub.ask_with_capabilities("review security policy", &AiCapabilitySet::none());

        assert!(matches!(result, Err(AiError::CapabilityDenied)));
    }
}
