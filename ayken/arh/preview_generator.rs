// Constitutional Module: PreviewGenerator
// This module MUST NOT mutate code, generate patches, or apply edits.
// All outputs are advisory-only previews.
// Forbidden behaviors: file writes, patch emission, workspace edits, auto-apply.

//! Preview generation for assisted fixes (advisory-only).

use crate::arh::signature_analysis::{CallSiteImpact, SignatureAnalysisResult};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PreviewInput {
    pub before_code: String,
    pub after_code: String,
    pub signature_analysis: SignatureAnalysisResult,
    pub security_delta: SecurityDelta,
    pub performance_cost: PerformanceCost,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PreviewOutput {
    pub before_code: String,
    pub after_code: String,
    pub security_delta: SecurityDelta,
    pub signature_change: SignatureChangePreview,
    pub performance_cost: PerformanceCost,
    pub validation: PreviewValidationResult,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SignatureChangePreview {
    pub added_params: Vec<String>,
    pub removed_params: Vec<String>,
    pub return_changed: bool,
    pub impacted_call_sites: Vec<CallSiteImpact>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SecurityDelta {
    pub new_parameters: Vec<String>,
    pub new_access_surfaces: Vec<String>,
    pub new_state_requirements: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PerformanceCost {
    pub stack_depth_increase: u32,
    pub allocation_count_increase: u32,
    pub indirection_cost: u32,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PreviewValidationIssue {
    EmptyBeforeCode,
    EmptyAfterCode,
    SignatureDeltaMismatch,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PreviewValidationResult {
    pub issues: Vec<PreviewValidationIssue>,
    pub is_consistent: bool,
}

pub struct PreviewGenerator;

impl PreviewGenerator {
    /// Generate a preview without mutating any source.
    pub fn generate_preview(&self, input: PreviewInput) -> PreviewOutput {
        let signature_change = SignatureChangePreview {
            added_params: input.signature_analysis.delta.added_params.clone(),
            removed_params: input.signature_analysis.delta.removed_params.clone(),
            return_changed: input.signature_analysis.delta.return_changed,
            impacted_call_sites: input.signature_analysis.impacted_call_sites.clone(),
        };

        let validation = self.validate_preview(
            &input.before_code,
            &input.after_code,
            &input.signature_analysis,
        );

        PreviewOutput {
            before_code: input.before_code,
            after_code: input.after_code,
            security_delta: input.security_delta,
            signature_change,
            performance_cost: input.performance_cost,
            validation,
        }
    }

    /// Validate preview consistency without mutating any source.
    pub fn validate_preview(
        &self,
        before_code: &str,
        after_code: &str,
        signature_analysis: &SignatureAnalysisResult,
    ) -> PreviewValidationResult {
        let mut issues = Vec::new();

        if before_code.trim().is_empty() {
            issues.push(PreviewValidationIssue::EmptyBeforeCode);
        }
        if after_code.trim().is_empty() {
            issues.push(PreviewValidationIssue::EmptyAfterCode);
        }

        if signature_analysis.delta.added_params.is_empty()
            && signature_analysis.delta.removed_params.is_empty()
            && !signature_analysis.delta.return_changed
            && !signature_analysis.impacted_call_sites.is_empty()
        {
            issues.push(PreviewValidationIssue::SignatureDeltaMismatch);
        }

        let is_consistent = issues.is_empty();
        PreviewValidationResult { issues, is_consistent }
    }
}
