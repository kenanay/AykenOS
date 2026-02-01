// Constitutional Module: SignatureAnalysis
// This module MUST NOT mutate code, generate patches, or apply edits.
// All outputs are advisory-only analyses.
// Forbidden behaviors: file writes, patch emission, workspace edits, auto-apply.

//! Signature analysis for assisted fixes (advisory-only).

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LayerImpact {
    SameModule,
    CrossModule,
    CrossCrate,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CallSiteImpact {
    pub location: String,
    pub layer: LayerImpact,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SignatureDelta {
    pub added_params: Vec<String>,
    pub removed_params: Vec<String>,
    pub return_changed: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SignatureAnalysisResult {
    pub delta: SignatureDelta,
    pub impacted_call_sites: Vec<CallSiteImpact>,
}

pub struct SignatureAnalysis;

impl SignatureAnalysis {
    /// Analyze signatures without mutating any source.
    pub fn analyze(
        &self,
        before_signature: &str,
        after_signature: &str,
        impacted_call_sites: Vec<CallSiteImpact>,
    ) -> SignatureAnalysisResult {
        let delta = compute_signature_delta(before_signature, after_signature);
        SignatureAnalysisResult {
            delta,
            impacted_call_sites,
        }
    }
}

fn compute_signature_delta(before_signature: &str, after_signature: &str) -> SignatureDelta {
    let before_params = extract_params(before_signature);
    let after_params = extract_params(after_signature);

    let added_params = after_params
        .iter()
        .filter(|param| !before_params.contains(*param))
        .cloned()
        .collect();

    let removed_params = before_params
        .iter()
        .filter(|param| !after_params.contains(*param))
        .cloned()
        .collect();

    let return_changed = extract_return(before_signature) != extract_return(after_signature);

    SignatureDelta {
        added_params,
        removed_params,
        return_changed,
    }
}

fn extract_params(signature: &str) -> Vec<String> {
    let open = signature.find('(');
    let close = signature.find(')');
    match (open, close) {
        (Some(start), Some(end)) if end > start + 1 => signature[start + 1..end]
            .split(',')
            .map(|param| param.trim().to_string())
            .filter(|param| !param.is_empty())
            .collect(),
        _ => Vec::new(),
    }
}

fn extract_return(signature: &str) -> Option<String> {
    signature
        .split("->")
        .nth(1)
        .map(|ret| ret.trim().to_string())
}
