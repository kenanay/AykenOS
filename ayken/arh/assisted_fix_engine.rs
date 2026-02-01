// Constitutional Module: AssistedFixEngine
// This module MUST NOT mutate code, generate patches, or apply edits.
// All outputs are advisory-only previews or recommendations.
// Forbidden behaviors: file writes, patch emission, workspace edits, auto-apply.

//! Assisted fix orchestration (advisory-only).

use crate::arh::approval_workflow::{ApprovalRequest, ApprovalResult, ApprovalWorkflow};
use crate::arh::preview_generator::{
    PerformanceCost, PreviewGenerator, PreviewInput, PreviewOutput, SecurityDelta,
};
use crate::arh::signature_analysis::{CallSiteImpact, SignatureAnalysis, SignatureAnalysisResult};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ImpactScope {
    ProcessOnly,
    ApiPotential,
    Architecture,
    CrossModule,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AssistedFixRequest {
    pub is_kernel: bool,
    pub impact_scope: ImpactScope,
    pub violation: ViolationType,
    pub before_code: String,
    pub after_code: String,
    pub before_signature: String,
    pub after_signature: String,
    pub impacted_call_sites: Vec<CallSiteImpact>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AssistedFixDisposition {
    AdvisoryOnly,
    KernelDefaultDeny,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AssistedFixDecision {
    pub disposition: AssistedFixDisposition,
    pub requires_opt_in: bool,
    pub preview: Option<PreviewOutput>,
    pub approval_request: Option<ApprovalRequest>,
    pub approval_result: Option<ApprovalResult>,
    pub automation_boundary: AutomationBoundary,
    pub recommendation: Recommendation,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ViolationType {
    Time,
    Error,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AutomationLevel {
    AssistedOnly,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AutomationBoundary {
    pub min_percent: u8,
    pub max_percent: u8,
    pub level: AutomationLevel,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Recommendation {
    pub title: String,
    pub details: Vec<String>,
}

pub struct AssistedFixEngine {
    // Placeholder for future orchestration dependencies.
}

impl AssistedFixEngine {
    /// Create a new engine instance.
    pub fn new() -> Self {
        Self {}
    }

    /// Run assisted fix analysis and produce advisory outputs.
    /// NOTE: This function MUST remain non-mutating.
    pub fn analyze(&self, request: AssistedFixRequest) -> AssistedFixDecision {
        if request.is_kernel {
            return AssistedFixDecision {
                disposition: AssistedFixDisposition::KernelDefaultDeny,
                requires_opt_in: true,
                preview: None,
                approval_request: None,
                approval_result: None,
                automation_boundary: AutomationBoundary {
                    min_percent: 60,
                    max_percent: 90,
                    level: AutomationLevel::AssistedOnly,
                },
                recommendation: Recommendation {
                    title: "Kernel default deny".to_string(),
                    details: vec![
                        "Kernel paths require explicit opt-in approval.".to_string(),
                        "No advisory preview emitted without kernel approval.".to_string(),
                    ],
                },
            };
        }

        let signature_analysis = self.signature_analysis(
            &request.before_signature,
            &request.after_signature,
            request.impacted_call_sites,
        );

        let security_delta = security_delta_for(&request.violation);
        let performance_cost = performance_cost_for(&request.violation);

        let preview = self.preview_generator().generate_preview(PreviewInput {
            before_code: request.before_code,
            after_code: request.after_code,
            signature_analysis,
            security_delta,
            performance_cost,
        });

        let approval_request = self.approval_workflow().build_request(
            preview.signature_change.return_changed
                || !preview.signature_change.added_params.is_empty()
                || !preview.signature_change.removed_params.is_empty(),
            !preview.signature_change.impacted_call_sites.is_empty(),
            !preview.security_delta.new_parameters.is_empty(),
            request.is_kernel,
        );

        AssistedFixDecision {
            disposition: AssistedFixDisposition::AdvisoryOnly,
            requires_opt_in: false,
            preview: Some(preview),
            approval_request: Some(approval_request),
            approval_result: None,
            automation_boundary: AutomationBoundary {
                min_percent: 60,
                max_percent: 90,
                level: AutomationLevel::AssistedOnly,
            },
            recommendation: recommendation_for(&request.violation),
        }
    }

    fn preview_generator(&self) -> PreviewGenerator {
        PreviewGenerator
    }

    fn approval_workflow(&self) -> ApprovalWorkflow {
        ApprovalWorkflow
    }

    fn signature_analysis(
        &self,
        before_signature: &str,
        after_signature: &str,
        impacted_call_sites: Vec<CallSiteImpact>,
    ) -> SignatureAnalysisResult {
        let analyzer = SignatureAnalysis;
        analyzer.analyze(before_signature, after_signature, impacted_call_sites)
    }

    /// Apply approval results to an advisory decision without mutating code.
    pub fn apply_approval_result(
        &self,
        decision: AssistedFixDecision,
        approval_result: ApprovalResult,
    ) -> AssistedFixDecision {
        AssistedFixDecision {
            approval_result: Some(approval_result),
            ..decision
        }
    }
}

fn recommendation_for(violation: &ViolationType) -> Recommendation {
    match violation {
        ViolationType::Time => Recommendation {
            title: "Clock injection (advisory)".to_string(),
            details: vec![
                "Inject a clock dependency instead of calling wall-clock directly.".to_string(),
                "Preserve determinism by routing time through an explicit interface.".to_string(),
            ],
        },
        ViolationType::Error => Recommendation {
            title: "Result propagation (advisory)".to_string(),
            details: vec![
                "Replace unwrap/expect with Result propagation.".to_string(),
                "Use explicit error types or map into existing error enums.".to_string(),
            ],
        },
    }
}

fn security_delta_for(violation: &ViolationType) -> SecurityDelta {
    match violation {
        ViolationType::Time => SecurityDelta {
            new_parameters: vec!["clock: &dyn Clock".to_string()],
            new_access_surfaces: vec!["time source injection".to_string()],
            new_state_requirements: Vec::new(),
        },
        ViolationType::Error => SecurityDelta {
            new_parameters: Vec::new(),
            new_access_surfaces: Vec::new(),
            new_state_requirements: vec!["error propagation in call chain".to_string()],
        },
    }
}

fn performance_cost_for(violation: &ViolationType) -> PerformanceCost {
    match violation {
        ViolationType::Time => PerformanceCost {
            stack_depth_increase: 1,
            allocation_count_increase: 0,
            indirection_cost: 1,
        },
        ViolationType::Error => PerformanceCost {
            stack_depth_increase: 0,
            allocation_count_increase: 0,
            indirection_cost: 0,
        },
    }
}
