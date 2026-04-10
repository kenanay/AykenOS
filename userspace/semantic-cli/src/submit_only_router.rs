//! Submit-only router for canonical query lowering output.
//!
//! The router validates the request, delegates exactly once to a submit
//! adapter, and returns a proof-bound submission record. It does not execute
//! or wait for results.

use crate::canonical_query::CanonicalPlan;
use crate::canonical_query_lowering::LoweredCanonicalQuery;
use crate::gate_c::{
    deterministic::{deterministic_id_from_plan, fixed_logical_timestamp},
    error::{GateCResult, SubmissionError},
    types::SubmissionId,
};
use crate::proof_chain::{build_proof_chain_record, ProofChainRecord};
use crate::submission_validation::{
    SubmissionCapability, SubmissionValidationInput, SubmissionValidationReport,
    SubmissionValidator,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CanonicalQuerySubmissionRequest {
    pub canonical_command: String,
    pub plan: CanonicalPlan,
    pub lowered: LoweredCanonicalQuery,
    pub target_context_id: u64,
    pub declared_capabilities: Vec<SubmissionCapability>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CanonicalQuerySubmission {
    pub submission_id: SubmissionId,
    pub validation: SubmissionValidationReport,
    pub proof_chain: ProofChainRecord,
}

pub trait SubmitAdapter {
    fn is_available(&self) -> bool;

    fn submit(
        &self,
        request: &CanonicalQuerySubmissionRequest,
        validation: &SubmissionValidationReport,
    ) -> GateCResult<SubmissionId>;
}

#[derive(Debug, Clone, Copy, Default)]
pub struct DeterministicSubmitAdapter {
    available: bool,
}

impl DeterministicSubmitAdapter {
    pub fn available() -> Self {
        Self { available: true }
    }

    pub fn unavailable() -> Self {
        Self { available: false }
    }
}

impl SubmitAdapter for DeterministicSubmitAdapter {
    fn is_available(&self) -> bool {
        self.available
    }

    fn submit(
        &self,
        request: &CanonicalQuerySubmissionRequest,
        validation: &SubmissionValidationReport,
    ) -> GateCResult<SubmissionId> {
        if !self.available {
            return Err(SubmissionError::OrchestratorUnavailable.into());
        }

        let plan_id = format!(
            "{}:{}:{}",
            validation.bcib_sha256, validation.target_context_id, request.canonical_command
        );

        Ok(SubmissionId {
            id: deterministic_id_from_plan("submit", &plan_id),
            timestamp: fixed_logical_timestamp(),
            fingerprint: Some(validation.bcib_sha256.clone()),
        })
    }
}

pub struct SubmitOnlyRouter<A> {
    adapter: A,
    validator: SubmissionValidator,
}

impl<A> SubmitOnlyRouter<A> {
    pub fn new(adapter: A, validator: SubmissionValidator) -> Self {
        Self { adapter, validator }
    }
}

impl<A> SubmitOnlyRouter<A>
where
    A: SubmitAdapter,
{
    pub fn submit(
        &self,
        request: &CanonicalQuerySubmissionRequest,
    ) -> GateCResult<CanonicalQuerySubmission> {
        let validation = self.validator.validate(&SubmissionValidationInput {
            canonical_command: request.canonical_command.clone(),
            plan: request.plan.clone(),
            lowered: request.lowered.clone(),
            target_context_id: request.target_context_id,
            declared_capabilities: request.declared_capabilities.clone(),
            submission_surface_available: self.adapter.is_available(),
        })?;

        let submission_id = self.adapter.submit(request, &validation)?;
        let proof_chain = build_proof_chain_record(
            &request.canonical_command,
            &request.plan,
            &request.lowered,
            &validation,
            submission_id.clone(),
        );

        Ok(CanonicalQuerySubmission {
            submission_id,
            validation,
            proof_chain,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::canonical_query::parse_canonical_plan;
    use crate::canonical_query_lowering::lower_canonical_query_to_bcib;
    use crate::submission_validation::SubmissionCapability;

    fn request() -> CanonicalQuerySubmissionRequest {
        let plan = parse_canonical_plan("query data.users {active == true}").unwrap();
        let lowered = lower_canonical_query_to_bcib(&plan).unwrap();

        CanonicalQuerySubmissionRequest {
            canonical_command: "query data.users where active == true".to_string(),
            plan,
            lowered,
            target_context_id: 11,
            declared_capabilities: vec![SubmissionCapability::context_read(
                "data.users",
                "approved active user query",
            )],
        }
    }

    #[test]
    fn submit_only_router_rejects_unavailable_adapter() {
        let router = SubmitOnlyRouter::new(
            DeterministicSubmitAdapter::unavailable(),
            SubmissionValidator::new(),
        );

        let error = router.submit(&request()).unwrap_err();
        assert!(matches!(
            error,
            crate::gate_c::error::GateCError::Submission(
                SubmissionError::OrchestratorUnavailable
            )
        ));
    }

    #[test]
    fn submit_only_router_builds_submission_and_proof_chain() {
        let router =
            SubmitOnlyRouter::new(DeterministicSubmitAdapter::available(), SubmissionValidator::new());

        let submission = router.submit(&request()).unwrap();

        assert!(submission.submission_id.id.starts_with("submit_"));
        assert_eq!(
            submission.proof_chain.replay_binding.bcib_sha256,
            submission.validation.bcib_sha256
        );
        assert_eq!(
            submission.proof_chain.target_context_id,
            submission.validation.target_context_id
        );
    }

    #[test]
    fn submit_only_router_is_deterministic_for_same_request() {
        let router =
            SubmitOnlyRouter::new(DeterministicSubmitAdapter::available(), SubmissionValidator::new());
        let request = request();

        let left = router.submit(&request).unwrap();
        let right = router.submit(&request).unwrap();

        assert_eq!(left, right);
    }
}
