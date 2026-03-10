use crate::canonical::digest::sha256_hex;
use crate::canonical::jcs::canonicalize_json_value;
use crate::errors::VerifierRuntimeError;
use crate::policy::quorum::quorum_satisfied;
use crate::policy::schema::validate_policy;
use crate::types::{
    KeyStatus, PolicyDecision, ProducerDeclaration, ResolvedSigner, TrustPolicy,
    VerificationFinding, VerificationVerdict,
};
use serde_json::Value;

pub fn compute_policy_hash(policy: &TrustPolicy) -> Result<String, VerifierRuntimeError> {
    let mut policy_value = serde_json::to_value(policy)
        .map_err(|error| VerifierRuntimeError::json("serialize policy", error))?;
    if let Value::Object(map) = &mut policy_value {
        map.remove("policy_hash");
    }
    let bytes = canonicalize_json_value(&policy_value)?;
    Ok(sha256_hex(&bytes))
}

pub fn evaluate_policy(
    policy: &TrustPolicy,
    producer: &ProducerDeclaration,
    resolved_signers: &[ResolvedSigner],
) -> Result<PolicyDecision, VerifierRuntimeError> {
    let policy_hash = compute_policy_hash(policy)?;
    let mut findings = validate_policy(policy);

    let accepted_count = resolved_signers
        .iter()
        .filter(|signer| signer.status == KeyStatus::Active)
        .filter(|_| is_trusted_producer(policy, producer))
        .filter(|signer| is_trusted_key(policy, &signer.producer_pubkey_id))
        .count();

    let verdict = if !policy.revoked_pubkey_ids.is_empty()
        && resolved_signers.iter().any(|signer| {
            policy
                .revoked_pubkey_ids
                .contains(&signer.producer_pubkey_id)
        }) {
        findings.push(VerificationFinding::error(
            "PV0502",
            "policy marks a resolved key as revoked",
        ));
        VerificationVerdict::Invalid
    } else if !is_trusted_producer(policy, producer) {
        VerificationVerdict::Untrusted
    } else if !quorum_satisfied(policy.required_signature_count(), accepted_count) {
        findings.push(VerificationFinding::warning(
            "PV0503",
            "resolved trusted signer count does not satisfy required signature quorum",
        ));
        VerificationVerdict::RejectedByPolicy
    } else {
        VerificationVerdict::Trusted
    };

    Ok(PolicyDecision {
        policy_hash,
        verdict,
        findings,
    })
}

fn is_trusted_producer(policy: &TrustPolicy, producer: &ProducerDeclaration) -> bool {
    !policy.trusted_producers.is_empty() && policy.trusted_producers.contains(&producer.producer_id)
}

fn is_trusted_key(policy: &TrustPolicy, producer_pubkey_id: &str) -> bool {
    if policy.trusted_pubkey_ids.is_empty() {
        return true;
    }
    policy
        .trusted_pubkey_ids
        .iter()
        .any(|value| value == producer_pubkey_id)
}
