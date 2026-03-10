use crate::errors::VerifierRuntimeError;
use crate::registry::snapshot::validate_registry_snapshot;
use crate::types::{
    KeyStatus, ProducerDeclaration, RegistryResolution, RegistrySnapshot, ResolvedSigner,
    SignatureEnvelope, VerificationFinding,
};

pub fn resolve_signers(
    snapshot: &RegistrySnapshot,
    producer: &ProducerDeclaration,
    signature_envelope: &SignatureEnvelope,
) -> Result<RegistryResolution, VerifierRuntimeError> {
    let validation = validate_registry_snapshot(snapshot)?;
    let mut findings = validation.findings;
    let mut resolved_signers = Vec::new();
    let key_owners = build_key_owner_index(snapshot);

    if !snapshot.producers.contains_key(&producer.producer_id) {
        findings.push(VerificationFinding::error(
            "PV0402",
            "producer_id is not present in registry snapshot",
        ));
    }

    for signature in &signature_envelope.signatures {
        let Some(entry) = snapshot.producers.get(&signature.signer_id) else {
            findings.push(VerificationFinding::error(
                "PV0407",
                "signature signer_id is not present in registry snapshot",
            ));
            resolved_signers.push(ResolvedSigner {
                signer_id: signature.signer_id.clone(),
                producer_pubkey_id: signature.producer_pubkey_id.clone(),
                status: KeyStatus::Unknown,
                public_key: None,
            });
            continue;
        };

        if key_owners
            .get(signature.producer_pubkey_id.as_str())
            .map(|owners| owners.len() > 1)
            .unwrap_or(false)
        {
            findings.push(VerificationFinding::error(
                "PV0405",
                "producer_pubkey_id ownership is ambiguous across registry snapshot",
            ));
        }

        let status = if entry
            .active_pubkey_ids
            .contains(&signature.producer_pubkey_id)
        {
            KeyStatus::Active
        } else if entry
            .revoked_pubkey_ids
            .contains(&signature.producer_pubkey_id)
        {
            findings.push(VerificationFinding::error(
                "PV0403",
                "signature references a revoked producer key",
            ));
            KeyStatus::Revoked
        } else if entry
            .superseded_pubkey_ids
            .contains(&signature.producer_pubkey_id)
        {
            KeyStatus::Superseded
        } else {
            findings.push(VerificationFinding::error(
                "PV0404",
                "signature references a producer key not present in registry snapshot",
            ));
            KeyStatus::Unknown
        };

        let public_key = entry
            .public_keys
            .get(&signature.producer_pubkey_id)
            .cloned();
        if public_key.is_none() {
            findings.push(VerificationFinding::error(
                "PV0406",
                "registry snapshot does not provide concrete public key material for producer_pubkey_id",
            ));
        }

        resolved_signers.push(ResolvedSigner {
            signer_id: signature.signer_id.clone(),
            producer_pubkey_id: signature.producer_pubkey_id.clone(),
            status,
            public_key,
        });
    }

    Ok(RegistryResolution {
        registry_snapshot_hash: validation.recomputed_hash,
        resolved_signers,
        findings,
    })
}

fn build_key_owner_index<'a>(
    snapshot: &'a RegistrySnapshot,
) -> std::collections::BTreeMap<&'a str, Vec<&'a str>> {
    let mut owners = std::collections::BTreeMap::new();
    for (producer_id, entry) in &snapshot.producers {
        for key_id in entry.public_keys.keys() {
            owners
                .entry(key_id.as_str())
                .or_insert_with(Vec::new)
                .push(producer_id.as_str());
        }
    }
    owners
}
