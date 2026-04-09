use crate::ai_stub::AiError;
use crate::capability_gate::AiCapabilitySet;

/// Authority terms that MUST NEVER appear in AI output.
/// Presence of any of these in a suggestion = AuthorityBoundaryViolation.
pub const FORBIDDEN_AUTHORITY_TERMS: &[&str] = &[
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

// Feature: phase15-bcib-execution-engine, WS 3.6 AI Runtime Boundary
// Property tests: AI NEVER produces authority, decision, or scheduling output.
#[cfg(test)]
mod property_tests {
    use super::{suggest, validate_boundary, FORBIDDEN_AUTHORITY_TERMS};
    use crate::capability_gate::AiCapabilitySet;
    use proptest::prelude::*;

    // Property: for any non-empty input with Suggest capability,
    // the output is always advisory_only=true and contains no forbidden authority terms.
    proptest! {
        #![proptest_config(ProptestConfig::with_cases(200))]

        #[test]
        fn ai_never_produces_authority_output(input in "[a-zA-Z0-9 .,!?]{1,128}") {
            let caps = AiCapabilitySet::suggestion_only();
            match suggest(&input, &caps) {
                Ok(suggestion) => {
                    // advisory_only MUST always be true — no exceptions
                    prop_assert!(
                        suggestion.is_advisory_only(),
                        "suggestion.advisory_only must be true, got false for input: {:?}",
                        input
                    );
                    // output MUST NOT contain any forbidden authority term
                    let lower = suggestion.content().to_ascii_lowercase();
                    for term in FORBIDDEN_AUTHORITY_TERMS {
                        prop_assert!(
                            !lower.contains(term),
                            "forbidden authority term {:?} found in output for input: {:?}",
                            term,
                            input
                        );
                    }
                }
                // InvalidPrompt is acceptable (empty/whitespace after trim)
                Err(crate::ai_stub::AiError::InvalidPrompt) => {}
                // Any other error is a contract violation
                Err(e) => {
                    prop_assert!(
                        false,
                        "unexpected error {:?} for non-empty input with valid capability",
                        e
                    );
                }
            }
        }

        // Property: without Suggest capability, AI MUST always deny — no capability bypass.
        #[test]
        fn ai_always_denied_without_capability(input in "[a-zA-Z0-9 .,!?]{1,128}") {
            let caps = AiCapabilitySet::none();
            match suggest(&input, &caps) {
                Err(crate::ai_stub::AiError::CapabilityDenied) => {}
                Err(crate::ai_stub::AiError::InvalidPrompt) => {}
                Ok(_) => {
                    prop_assert!(
                        false,
                        "AI produced output without capability for input: {:?}",
                        input
                    );
                }
                Err(e) => {
                    prop_assert!(
                        false,
                        "unexpected error {:?} — expected CapabilityDenied",
                        e
                    );
                }
            }
        }

        // Property: validate_boundary rejects any content containing a forbidden term.
        #[test]
        fn boundary_validator_rejects_all_authority_terms(
            prefix in "[a-z ]{0,20}",
            suffix in "[a-z ]{0,20}",
            term_idx in 0usize..7usize,
        ) {
            let term = FORBIDDEN_AUTHORITY_TERMS[term_idx];
            let content = format!("{}{}{}", prefix, term, suffix);
            let result = validate_boundary(&content);
            prop_assert!(
                result.is_err(),
                "boundary validator must reject content containing {:?}: {:?}",
                term,
                content
            );
        }
    }
}
