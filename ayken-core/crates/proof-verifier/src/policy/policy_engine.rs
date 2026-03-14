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
use std::collections::BTreeSet;

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
        .map(|signer| signer.producer_pubkey_id.as_str())
        .collect::<BTreeSet<_>>()
        .len();

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

#[cfg(test)]
mod tests {
    use super::evaluate_policy;
    use crate::types::{
        KeyStatus, ProducerDeclaration, ResolvedSigner, SignatureRequirement, TrustPolicy,
        VerificationVerdict,
    };

    fn baseline_policy(required_count: u32) -> TrustPolicy {
        TrustPolicy {
            policy_version: 1,
            policy_hash: None,
            quorum_policy_ref: Some("policy://quorum/at-least-2-of-n".to_string()),
            trusted_producers: vec!["ayken-ci".to_string()],
            trusted_pubkey_ids: vec![
                "ed25519-key-2026-03-a".to_string(),
                "ed25519-key-2026-03-b".to_string(),
            ],
            required_signatures: Some(SignatureRequirement {
                kind: "at_least".to_string(),
                count: required_count,
            }),
            revoked_pubkey_ids: Vec::new(),
        }
    }

    fn producer() -> ProducerDeclaration {
        ProducerDeclaration {
            metadata_version: 1,
            producer_id: "ayken-ci".to_string(),
            producer_pubkey_id: "ed25519-key-2026-03-a".to_string(),
            producer_registry_ref: "trust://registry/ayken-ci".to_string(),
            producer_key_epoch: "2026-03".to_string(),
            build_id: None,
        }
    }

    #[test]
    fn duplicate_key_entries_do_not_satisfy_quorum() {
        let policy = baseline_policy(2);
        let resolved_signers = vec![
            ResolvedSigner {
                signer_id: "ayken-ci".to_string(),
                producer_pubkey_id: "ed25519-key-2026-03-a".to_string(),
                status: KeyStatus::Active,
                public_key: None,
            },
            ResolvedSigner {
                signer_id: "ayken-ci".to_string(),
                producer_pubkey_id: "ed25519-key-2026-03-a".to_string(),
                status: KeyStatus::Active,
                public_key: None,
            },
        ];

        let decision =
            evaluate_policy(&policy, &producer(), &resolved_signers).expect("policy evaluation");
        assert_eq!(decision.verdict, VerificationVerdict::RejectedByPolicy);
    }

    #[test]
    fn distinct_active_keys_can_satisfy_quorum() {
        let policy = baseline_policy(2);
        let resolved_signers = vec![
            ResolvedSigner {
                signer_id: "ayken-ci".to_string(),
                producer_pubkey_id: "ed25519-key-2026-03-a".to_string(),
                status: KeyStatus::Active,
                public_key: None,
            },
            ResolvedSigner {
                signer_id: "ayken-ci".to_string(),
                producer_pubkey_id: "ed25519-key-2026-03-b".to_string(),
                status: KeyStatus::Active,
                public_key: None,
            },
        ];

        let decision =
            evaluate_policy(&policy, &producer(), &resolved_signers).expect("policy evaluation");
        assert_eq!(decision.verdict, VerificationVerdict::Trusted);
    }
}
