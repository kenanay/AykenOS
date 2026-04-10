//! Submission validation for canonical query submission.
//!
//! This module owns the fail-closed checks that must pass before any
//! submit-only adapter sees a canonical query request.

use crate::bcib::Capability;
use crate::canonical_query::CanonicalPlan;
use crate::canonical_query_lowering::{validate_canonical_query_bcib, LoweredCanonicalQuery};
use crate::gate_c::{
    error::{GateCResult, SubmissionError},
    types::SubmissionId,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SubmissionCapabilityScope {
    Read,
    Write,
    Execute,
    Admin,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SubmissionCapability {
    pub name: String,
    pub scope: SubmissionCapabilityScope,
    pub resource: Option<String>,
    pub reason: String,
}

impl SubmissionCapability {
    pub fn context_read(context_path: impl Into<String>, reason: impl Into<String>) -> Self {
        Self {
            name: "context.read".to_string(),
            scope: SubmissionCapabilityScope::Read,
            resource: Some(context_path.into()),
            reason: reason.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SubmissionValidationInput {
    pub canonical_command: String,
    pub plan: CanonicalPlan,
    pub lowered: LoweredCanonicalQuery,
    pub target_context_id: u64,
    pub declared_capabilities: Vec<SubmissionCapability>,
    pub submission_surface_available: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SubmissionValidationReport {
    pub target_context_id: u64,
    pub required_capabilities: Vec<SubmissionCapability>,
    pub declared_capabilities: Vec<SubmissionCapability>,
    pub bcib_sha256: String,
    pub canonical_plan_fingerprint: String,
    pub canonical_binding_fingerprint: String,
}

#[derive(Debug, Default, Clone, Copy)]
pub struct SubmissionValidator;

impl SubmissionValidator {
    pub fn new() -> Self {
        Self
    }

    pub fn validate(
        &self,
        input: &SubmissionValidationInput,
    ) -> GateCResult<SubmissionValidationReport> {
        if !input.submission_surface_available {
            return Err(SubmissionError::OrchestratorUnavailable.into());
        }

        if input.canonical_command.trim().is_empty() {
            return Err(SubmissionError::InvalidPlan(
                "canonical command string cannot be empty".to_string(),
            )
            .into());
        }

        if input.target_context_id == 0 {
            return Err(SubmissionError::InvalidPlan(
                "target context identifier must be explicit and non-zero".to_string(),
            )
            .into());
        }

        input
            .plan
            .validate()
            .map_err(|e| SubmissionError::InvalidPlan(e.to_string()))?;

        if input.lowered.command_kind != input.plan.command_kind {
            return Err(SubmissionError::InvalidPlan(
                "lowered command kind drifted from canonical plan".to_string(),
            )
            .into());
        }

        if input.lowered.binding != input.plan.binding {
            return Err(SubmissionError::InvalidPlan(
                "lowered canonical query binding drifted from canonical plan".to_string(),
            )
            .into());
        }

        validate_canonical_query_bcib(&input.lowered.bytes)
            .map_err(|e| SubmissionError::InvalidPlan(e.to_string()))?;

        if input.lowered.contains_forbidden_opcode() {
            return Err(SubmissionError::InvalidPlan(
                "lowered BCIB contains forbidden opcode".to_string(),
            )
            .into());
        }

        if input.declared_capabilities.is_empty() {
            return Err(SubmissionError::CapabilityDenied(
                "declared capability set must be explicit and non-empty".to_string(),
            )
            .into());
        }

        let required_capabilities = derive_required_capabilities(&input.lowered)?;
        if required_capabilities.is_empty() {
            return Err(SubmissionError::CapabilityDenied(
                "derived capability set must be explicit and non-empty".to_string(),
            )
            .into());
        }

        for required in &required_capabilities {
            if !input
                .declared_capabilities
                .iter()
                .any(|declared| capability_satisfies(declared, required))
            {
                return Err(SubmissionError::CapabilityDenied(format!(
                    "missing required capability {} for resource {}",
                    required.name,
                    required.resource.as_deref().unwrap_or("<none>")
                ))
                .into());
            }
        }

        let expected_context_capability = SubmissionCapability::context_read(
            input.plan.context_path.clone(),
            "canonical query requires explicit context read capability",
        );
        if !required_capabilities
            .iter()
            .any(|capability| capability_satisfies(capability, &expected_context_capability))
        {
            return Err(SubmissionError::CapabilityDenied(
                "derived capability set does not contain explicit context-read capability"
                    .to_string(),
            )
            .into());
        }

        Ok(SubmissionValidationReport {
            target_context_id: input.target_context_id,
            required_capabilities,
            declared_capabilities: input.declared_capabilities.clone(),
            bcib_sha256: input.lowered.bcib_sha256.clone(),
            canonical_plan_fingerprint: input.plan.fingerprint_hex(),
            canonical_binding_fingerprint: input.plan.binding.fingerprint_hex(),
        })
    }
}

fn capability_satisfies(
    declared: &SubmissionCapability,
    required: &SubmissionCapability,
) -> bool {
    declared.name == required.name
        && declared.scope == required.scope
        && declared.resource == required.resource
}

pub fn derive_required_capabilities(
    lowered: &LoweredCanonicalQuery,
) -> GateCResult<Vec<SubmissionCapability>> {
    let mut capabilities = Vec::with_capacity(lowered.required_capabilities.len());

    for capability in &lowered.required_capabilities {
        match capability {
            Capability::Read { context } => {
                capabilities.push(SubmissionCapability::context_read(
                    context.clone(),
                    "required by canonical query context load",
                ));
            }
            other => {
                return Err(SubmissionError::InvalidPlan(format!(
                    "unsupported canonical query capability in submission validation: {:?}",
                    other
                ))
                .into());
            }
        }
    }

    capabilities.sort_by(|left, right| {
        left.name
            .cmp(&right.name)
            .then(left.resource.cmp(&right.resource))
            .then(left.reason.cmp(&right.reason))
    });
    capabilities.dedup();

    Ok(capabilities)
}

pub fn submission_result_fingerprint(submission_id: &SubmissionId) -> String {
    submission_id
        .fingerprint
        .clone()
        .unwrap_or_else(|| submission_id.id.clone())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::canonical_query::parse_canonical_plan;
    use crate::canonical_query_lowering::lower_canonical_query_to_bcib;

    fn validation_input() -> SubmissionValidationInput {
        let plan = parse_canonical_plan("query data.users {age > 18}").unwrap();
        let lowered = lower_canonical_query_to_bcib(&plan).unwrap();

        SubmissionValidationInput {
            canonical_command: "query data.users where age > 18".to_string(),
            plan,
            lowered,
            target_context_id: 7,
            declared_capabilities: vec![SubmissionCapability::context_read(
                "data.users",
                "operator approved context read",
            )],
            submission_surface_available: true,
        }
    }

    #[test]
    fn submission_validation_accepts_explicit_context_read_capability() {
        let validator = SubmissionValidator::new();
        let input = validation_input();

        let report = validator.validate(&input).unwrap();
        assert_eq!(report.target_context_id, 7);
        assert_eq!(report.required_capabilities.len(), 1);
        assert_eq!(report.required_capabilities[0].name, "context.read");
        assert_eq!(
            report.required_capabilities[0].resource.as_deref(),
            Some("data.users")
        );
    }

    #[test]
    fn submission_validation_rejects_empty_declared_capability_set() {
        let validator = SubmissionValidator::new();
        let mut input = validation_input();
        input.declared_capabilities.clear();

        let error = validator.validate(&input).unwrap_err();
        assert!(matches!(
            error,
            crate::gate_c::error::GateCError::Submission(SubmissionError::CapabilityDenied(_))
        ));
    }

    #[test]
    fn submission_validation_rejects_missing_context_read_capability() {
        let validator = SubmissionValidator::new();
        let mut input = validation_input();
        input.declared_capabilities = vec![SubmissionCapability {
            name: "context.read".to_string(),
            scope: SubmissionCapabilityScope::Read,
            resource: Some("data.logs".to_string()),
            reason: "wrong context".to_string(),
        }];

        let error = validator.validate(&input).unwrap_err();
        assert!(matches!(
            error,
            crate::gate_c::error::GateCError::Submission(SubmissionError::CapabilityDenied(_))
        ));
    }

    #[test]
    fn submission_validation_rejects_unavailable_surface() {
        let validator = SubmissionValidator::new();
        let mut input = validation_input();
        input.submission_surface_available = false;

        let error = validator.validate(&input).unwrap_err();
        assert!(matches!(
            error,
            crate::gate_c::error::GateCError::Submission(
                SubmissionError::OrchestratorUnavailable
            )
        ));
    }
}
