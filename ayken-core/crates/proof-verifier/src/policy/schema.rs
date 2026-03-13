use crate::types::{TrustPolicy, VerificationFinding};

pub fn validate_policy(policy: &TrustPolicy) -> Vec<VerificationFinding> {
    let mut findings = Vec::new();
    if policy.policy_version == 0 {
        findings.push(VerificationFinding::error(
            "PV0500",
            "policy_version must be non-zero",
        ));
    }
    if policy.required_signature_count() == 0 {
        findings.push(VerificationFinding::error(
            "PV0501",
            "required signature count must be at least 1",
        ));
    }
    if policy
        .quorum_policy_ref
        .as_deref()
        .map(|value| value.trim().is_empty())
        .unwrap_or(true)
    {
        findings.push(VerificationFinding::error(
            "PV0505",
            "quorum_policy_ref must be present and non-empty",
        ));
    }
    if let Some(requirement) = &policy.required_signatures {
        if requirement.kind.trim() != "at_least" {
            findings.push(VerificationFinding::error(
                "PV0504",
                "required_signatures.kind must be at_least",
            ));
        }
    }
    findings
}
