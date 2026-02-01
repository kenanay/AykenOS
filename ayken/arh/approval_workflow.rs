// Constitutional Module: ApprovalWorkflow
// This module MUST NOT mutate code, generate patches, or apply edits.
// All outputs are advisory-only and require explicit human approval.
// Forbidden behaviors: silent approval, auto-apply, implicit mutation.

//! Approval workflow for assisted fixes (human-in-the-loop only).

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ApprovalDecision {
    Approve,
    Deny,
    Hold,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ApprovalStep {
    SignatureChangeApproval,
    CallerUpdateApproval,
    DependencyInjectionApproval,
    KernelSafetyApproval,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ApprovalRequest {
    pub steps: Vec<ApprovalStep>,
    pub is_kernel: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ApprovalResult {
    pub decisions: Vec<(ApprovalStep, ApprovalDecision)>,
    pub final_decision: ApprovalDecision,
}

pub struct ApprovalWorkflow;

impl ApprovalWorkflow {
    /// Create an approval request with explicit decision points.
    /// Default behavior is deny/hold until all steps are explicitly approved.
    pub fn build_request(
        &self,
        requires_signature_change: bool,
        requires_caller_updates: bool,
        requires_dependency_injection: bool,
        is_kernel: bool,
    ) -> ApprovalRequest {
        let mut steps = Vec::new();
        if requires_signature_change {
            steps.push(ApprovalStep::SignatureChangeApproval);
        }
        if requires_caller_updates {
            steps.push(ApprovalStep::CallerUpdateApproval);
        }
        if requires_dependency_injection {
            steps.push(ApprovalStep::DependencyInjectionApproval);
        }
        if is_kernel {
            steps.push(ApprovalStep::KernelSafetyApproval);
        }
        ApprovalRequest { steps, is_kernel }
    }

    /// Evaluate approval decisions. Any missing approval results in Hold.
    pub fn evaluate(&self, request: &ApprovalRequest, decisions: &[(ApprovalStep, ApprovalDecision)]) -> ApprovalResult {
        let mut final_decision = ApprovalDecision::Approve;
        let mut recorded = Vec::new();

        for step in &request.steps {
            let decision = decisions
                .iter()
                .find(|(s, _)| s == step)
                .map(|(_, d)| *d)
                .unwrap_or(ApprovalDecision::Hold);

            if decision != ApprovalDecision::Approve {
                final_decision = decision;
            }
            recorded.push((step.clone(), decision));
        }

        ApprovalResult {
            decisions: recorded,
            final_decision,
        }
    }
}
