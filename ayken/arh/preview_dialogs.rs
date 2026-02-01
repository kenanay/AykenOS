// Constitutional Module: PreviewDialogs
// This module MUST NOT mutate code, generate patches, or apply edits.
// All outputs are advisory-only and read-only.
// Forbidden behaviors: file writes, patch emission, workspace edits, auto-apply.

//! Preview dialog models (read-only, advisory-only).

use crate::arh::preview_generator::PreviewOutput;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PreviewDialog {
    pub before: String,
    pub after: String,
    pub security_delta: Vec<String>,
    pub performance_cost: Vec<String>,
    pub signature_ripple: Vec<String>,
    pub approval_required: bool,
    pub kernel_warning: bool,
}

pub fn build_preview_dialog(preview: &PreviewOutput, kernel_warning: bool) -> PreviewDialog {
    let security_delta = vec![
        format!("New parameters: {:?}", preview.security_delta.new_parameters),
        format!("New access surfaces: {:?}", preview.security_delta.new_access_surfaces),
        format!("New state requirements: {:?}", preview.security_delta.new_state_requirements),
    ];

    let performance_cost = vec![
        format!("Stack depth increase: {}", preview.performance_cost.stack_depth_increase),
        format!("Allocation increase: {}", preview.performance_cost.allocation_count_increase),
        format!("Indirection cost: {}", preview.performance_cost.indirection_cost),
    ];

    let signature_ripple = vec![
        format!("Added params: {:?}", preview.signature_change.added_params),
        format!("Removed params: {:?}", preview.signature_change.removed_params),
        format!("Return changed: {}", preview.signature_change.return_changed),
        format!("Impacted call sites: {}", preview.signature_change.impacted_call_sites.len()),
    ];

    PreviewDialog {
        before: preview.before_code.clone(),
        after: preview.after_code.clone(),
        security_delta,
        performance_cost,
        signature_ripple,
        approval_required: preview.signature_change.return_changed
            || !preview.signature_change.added_params.is_empty()
            || !preview.signature_change.removed_params.is_empty(),
        kernel_warning,
    }
}
