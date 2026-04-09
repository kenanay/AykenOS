use crate::ai_stub::AiError;
use crate::capability_gate::AiCapabilitySet;

const FORBIDDEN_AUTHORITY_TERMS: &[&str] = &[
    "execute",
    "schedule",
    "route",
    "routing",
    "authority",
    "authoritative",
    "decision",
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AiSuggestion {
    content: String,
    advisory_only: bool,
}

impl AiSuggestion {
    pub fn content(&self) -> &str {
        &self.content
    }

    pub fn is_advisory_only(&self) -> bool {
        self.advisory_only
    }
}

#[derive(Debug, Default)]
pub struct SuggestionEngine;

impl SuggestionEngine {
    pub fn new() -> Self {
        Self
    }

    pub fn suggest(
        &self,
        input: &str,
        capabilities: &AiCapabilitySet,
    ) -> Result<AiSuggestion, AiError> {
        if input.trim().is_empty() {
            return Err(AiError::InvalidPrompt);
        }
        if !capabilities.allow_ai() {
            return Err(AiError::CapabilityDenied);
        }

        let lower = input.to_ascii_lowercase();
        let content = if lower.contains("error") || lower.contains("fail") {
            "Consider collecting diagnostics and asking a human operator to review the issue."
        } else if lower.contains("policy") || lower.contains("security") {
            "Consider reviewing the policy impact and requesting human approval before any change."
        } else {
            "Consider documenting the request and asking a human operator to choose the next step."
        };

        validate_boundary(content)?;

        Ok(AiSuggestion {
            content: content.to_string(),
            advisory_only: true,
        })
    }
}

pub fn suggest(input: &str, capabilities: &AiCapabilitySet) -> Result<AiSuggestion, AiError> {
    SuggestionEngine::new().suggest(input, capabilities)
}

pub fn validate_boundary(content: &str) -> Result<(), AiError> {
    let lower = content.to_ascii_lowercase();
    if FORBIDDEN_AUTHORITY_TERMS
        .iter()
        .any(|term| lower.contains(term))
    {
        return Err(AiError::AuthorityBoundaryViolation);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{suggest, validate_boundary, SuggestionEngine};
    use crate::ai_stub::AiError;
    use crate::capability_gate::AiCapabilitySet;

    #[test]
    fn suggestion_is_capability_gated() {
        let err = suggest("review this plan", &AiCapabilitySet::none())
            .expect_err("missing capability must fail");
        assert!(matches!(err, AiError::CapabilityDenied));
    }

    #[test]
    fn suggestion_is_advisory_only() {
        let suggestion = suggest("review this plan", &AiCapabilitySet::suggestion_only())
            .expect("suggestion capability must succeed");
        assert!(suggestion.is_advisory_only());
        let content = suggestion.content().to_ascii_lowercase();
        assert!(!content.contains("execute"));
        assert!(!content.contains("schedule"));
        assert!(!content.contains("route"));
    }

    #[test]
    fn dangerous_terms_are_rejected_by_boundary_validator() {
        let err = validate_boundary("schedule the rollout now")
            .expect_err("authority language must be rejected");
        assert!(matches!(err, AiError::AuthorityBoundaryViolation));
    }

    #[test]
    fn empty_prompt_is_rejected() {
        let err = SuggestionEngine::new()
            .suggest("", &AiCapabilitySet::suggestion_only())
            .expect_err("empty prompt must fail");
        assert!(matches!(err, AiError::InvalidPrompt));
    }

    #[test]
    fn error_prompts_stay_advisory() {
        let suggestion = suggest(
            "kernel error observed during query",
            &AiCapabilitySet::suggestion_only(),
        )
        .expect("error prompt should still produce a bounded suggestion");
        assert_eq!(
            suggestion.content(),
            "Consider collecting diagnostics and asking a human operator to review the issue."
        );
    }
}
