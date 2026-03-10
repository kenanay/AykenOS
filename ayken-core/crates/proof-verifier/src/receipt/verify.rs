use crate::authority::resolution::resolve_verifier_authority;
use crate::crypto::ed25519::{is_allowed_signature_algorithm, verify_ed25519_bytes};
use crate::errors::VerifierRuntimeError;
use crate::receipt::schema::canonicalize_receipt_payload;
use crate::types::{
    DistributedReceiptVerification, ReceiptVerifierKey, VerdictSubject, VerificationFinding,
    VerificationReceipt, VerifierAuthorityResolution, VerifierAuthorityResolutionClass,
    VerifierTrustRegistrySnapshot,
};

const DISTRIBUTED_RECEIPT_ISSUER_SCOPE: &str = "distributed-receipt-issuer";

pub fn verify_signed_receipt(
    receipt: &VerificationReceipt,
    expected_subject: &VerdictSubject,
    verifier_key: &ReceiptVerifierKey,
) -> Result<Vec<VerificationFinding>, VerifierRuntimeError> {
    let mut findings = Vec::new();

    if !receipt_subject_matches(receipt, expected_subject) {
        findings.push(VerificationFinding::error(
            "PV0701",
            "signed receipt payload does not match recomputed verdict subject",
        ));
    }

    if receipt.payload.verifier_node_id != verifier_key.verifier_node_id {
        findings.push(VerificationFinding::error(
            "PV0702",
            "signed receipt verifier_node_id does not match verifier key identity",
        ));
    }

    if receipt.payload.verifier_key_id.as_deref() != Some(verifier_key.verifier_key_id.as_str()) {
        findings.push(VerificationFinding::error(
            "PV0703",
            "signed receipt verifier_key_id does not match verifier key identity",
        ));
    }

    let Some(signature_algorithm) = &receipt.verifier_signature_algorithm else {
        findings.push(VerificationFinding::error(
            "PV0704",
            "signed receipt is missing verifier_signature_algorithm",
        ));
        return Ok(findings);
    };
    let Some(signature) = &receipt.verifier_signature else {
        findings.push(VerificationFinding::error(
            "PV0705",
            "signed receipt is missing verifier_signature",
        ));
        return Ok(findings);
    };

    if !is_allowed_signature_algorithm(signature_algorithm) {
        findings.push(VerificationFinding::error(
            "PV0706",
            "signed receipt verifier_signature_algorithm is not allowlisted",
        ));
        return Ok(findings);
    }

    if !signature_algorithm.eq_ignore_ascii_case(&verifier_key.signature_algorithm) {
        findings.push(VerificationFinding::error(
            "PV0707",
            "signed receipt verifier_signature_algorithm does not match verifier key algorithm",
        ));
        return Ok(findings);
    }

    let payload_bytes = canonicalize_receipt_payload(&receipt.payload)?;
    if let Err(finding) = verify_ed25519_bytes(
        &verifier_key.public_key,
        signature,
        &payload_bytes,
        "PV0708",
        "signed receipt detached signature verification failed",
    ) {
        findings.push(finding);
    }

    Ok(findings)
}

pub fn verify_signed_receipt_with_authority(
    receipt: &VerificationReceipt,
    expected_subject: &VerdictSubject,
    verifier_key: &ReceiptVerifierKey,
    verifier_registry: &VerifierTrustRegistrySnapshot,
) -> Result<DistributedReceiptVerification, VerifierRuntimeError> {
    let authority_scope = vec![DISTRIBUTED_RECEIPT_ISSUER_SCOPE.to_string()];
    let authority_resolution = resolve_verifier_authority(
        verifier_registry,
        &receipt.payload.verifier_node_id,
        &authority_scope,
    )?;
    verify_signed_receipt_with_resolved_authority(
        receipt,
        expected_subject,
        verifier_key,
        verifier_registry,
        authority_resolution,
    )
}

pub(crate) fn verify_signed_receipt_with_resolved_authority(
    receipt: &VerificationReceipt,
    expected_subject: &VerdictSubject,
    verifier_key: &ReceiptVerifierKey,
    verifier_registry: &VerifierTrustRegistrySnapshot,
    authority_resolution: VerifierAuthorityResolution,
) -> Result<DistributedReceiptVerification, VerifierRuntimeError> {
    let mut findings = verify_signed_receipt(receipt, expected_subject, verifier_key)?;
    findings.extend(authority_resolution.findings.iter().cloned());

    let Some(verifier_key_id) = receipt.payload.verifier_key_id.as_deref() else {
        findings.push(VerificationFinding::error(
            "PV0710",
            "signed receipt verifier_key_id is required for verifier authority binding",
        ));
        return Ok(DistributedReceiptVerification {
            authority_resolution,
            findings,
        });
    };

    match authority_resolution.result_class {
        VerifierAuthorityResolutionClass::AuthorityResolvedRoot
        | VerifierAuthorityResolutionClass::AuthorityResolvedDelegated => {
            if authority_resolution.authority_chain_id.is_none() {
                findings.push(VerificationFinding::error(
                    "PV0713",
                    "signed receipt verifier authority resolution did not produce authority_chain_id",
                ));
            }
        }
        VerifierAuthorityResolutionClass::AuthorityHistoricalOnly => {
            findings.push(VerificationFinding::error(
                "PV0711",
                "signed receipt verifier authority is historical-only and cannot support current distributed trust acceptance",
            ));
            if authority_resolution.authority_chain_id.is_none() {
                findings.push(VerificationFinding::error(
                    "PV0713",
                    "signed receipt verifier authority resolution did not produce authority_chain_id",
                ));
            }
        }
        _ => findings.push(VerificationFinding::error(
            "PV0712",
            "signed receipt verifier authority could not be resolved as current distributed authority",
        )),
    }

    let Some(resolved_node) = verifier_registry
        .verifiers
        .get(&receipt.payload.verifier_node_id)
    else {
        findings.push(VerificationFinding::error(
            "PV0714",
            "signed receipt verifier identity is missing from verifier trust registry",
        ));
        return Ok(DistributedReceiptVerification {
            authority_resolution,
            findings,
        });
    };

    if resolved_node.verifier_pubkey_id != verifier_key_id
        || resolved_node.verifier_pubkey_id != verifier_key.verifier_key_id
    {
        findings.push(VerificationFinding::error(
            "PV0715",
            "signed receipt verifier key identity does not match resolved verifier authority node",
        ));
    }

    let Some(registry_public_key) = verifier_registry.public_keys.get(verifier_key_id) else {
        findings.push(VerificationFinding::error(
            "PV0716",
            "signed receipt verifier authority key is missing from verifier trust registry public_keys",
        ));
        return Ok(DistributedReceiptVerification {
            authority_resolution,
            findings,
        });
    };

    if !registry_public_key
        .algorithm
        .eq_ignore_ascii_case(&verifier_key.signature_algorithm)
    {
        findings.push(VerificationFinding::error(
            "PV0717",
            "signed receipt verifier key algorithm does not match verifier trust registry public key algorithm",
        ));
    }

    if registry_public_key.public_key != verifier_key.public_key {
        findings.push(VerificationFinding::error(
            "PV0718",
            "signed receipt verifier key material does not match verifier trust registry public key material",
        ));
    }

    Ok(DistributedReceiptVerification {
        authority_resolution,
        findings,
    })
}

fn receipt_subject_matches(
    receipt: &VerificationReceipt,
    expected_subject: &VerdictSubject,
) -> bool {
    receipt.payload.bundle_id == expected_subject.bundle_id
        && receipt.payload.trust_overlay_hash == expected_subject.trust_overlay_hash
        && receipt.payload.policy_hash == expected_subject.policy_hash
        && receipt.payload.registry_snapshot_hash == expected_subject.registry_snapshot_hash
}
