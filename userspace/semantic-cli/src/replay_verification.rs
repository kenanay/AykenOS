// replay_verification.rs
// Deterministic replay verification for BCIB execution
// CONSTITUTIONAL: DETERMINISM.GLOBAL enforcement

use crate::bcib_simple::{BCIB, BCIBInstruction};
use crate::error::SemanticCLIError;
use crate::proof_chain::{ProofChainRecord, ProofReplayBinding};
use sha2::{Digest, Sha256};

/// Replay verification result
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReplayVerificationResult {
    /// Replay matches original execution
    Match {
        bcib_sha256: String,
        submission_result_fingerprint: String,
    },
    /// Replay deviates from original execution (FAIL CLOSED)
    Deviation {
        bcib_sha256: String,
        expected_result: String,
        actual_result: String,
        deviation_reason: String,
    },
}

/// Replay verification engine
/// 
/// CONSTITUTIONAL ENFORCEMENT:
/// - DETERMINISM.GLOBAL: same BCIB → same result
/// - FAIL CLOSED: any deviation → reject
pub struct ReplayVerifier;

impl ReplayVerifier {
    /// Verify that replaying a BCIB produces the same result
    /// 
    /// FAIL CLOSED: any deviation is a constitutional violation
    pub fn verify_replay(
        bcib: &BCIB,
        original_proof: &ProofChainRecord,
        replay_result: &str,
    ) -> Result<ReplayVerificationResult, SemanticCLIError> {
        // 1. Verify BCIB identity matches
        let bcib_sha256 = Self::compute_bcib_sha256(bcib);
        
        if bcib_sha256 != original_proof.bcib_sha256 {
            return Err(SemanticCLIError::execution_error(
                format!(
                    "BCIB identity mismatch: expected {}, got {}",
                    original_proof.bcib_sha256, bcib_sha256
                ),
                crate::error::ErrorCode::E750,
            ));
        }

        // 2. Compute replay result fingerprint
        let replay_fingerprint = Self::compute_result_fingerprint(replay_result);

        // 3. Extract expected result from proof chain
        let expected_fingerprint = Self::extract_expected_result(&original_proof.replay_binding)?;

        // 4. Compare results (FAIL CLOSED)
        if replay_fingerprint == expected_fingerprint {
            Ok(ReplayVerificationResult::Match {
                bcib_sha256,
                submission_result_fingerprint: replay_fingerprint,
            })
        } else {
            Ok(ReplayVerificationResult::Deviation {
                bcib_sha256,
                expected_result: expected_fingerprint,
                actual_result: replay_fingerprint,
                deviation_reason: "Result fingerprint mismatch".to_string(),
            })
        }
    }

    /// Verify replay binding integrity
    /// 
    /// Ensures the replay binding is internally consistent
    pub fn verify_binding_integrity(
        binding: &ProofReplayBinding,
    ) -> Result<(), SemanticCLIError> {
        // 1. Verify canonical plan fingerprint is non-empty
        if binding.canonical_plan_fingerprint.is_empty() {
            return Err(SemanticCLIError::execution_error(
                "Empty canonical plan fingerprint",
                crate::error::ErrorCode::E752,
            ));
        }

        // 2. Verify canonical binding fingerprint is non-empty
        if binding.canonical_binding_fingerprint.is_empty() {
            return Err(SemanticCLIError::execution_error(
                "Empty canonical binding fingerprint",
                crate::error::ErrorCode::E752,
            ));
        }

        // 3. Verify BCIB SHA-256 is non-empty
        if binding.bcib_sha256.is_empty() {
            return Err(SemanticCLIError::execution_error(
                "Empty BCIB SHA-256",
                crate::error::ErrorCode::E752,
            ));
        }

        // 4. Verify submission result fingerprint is non-empty
        if binding.submission_result_fingerprint.is_empty() {
            return Err(SemanticCLIError::execution_error(
                "Empty submission result fingerprint",
                crate::error::ErrorCode::E752,
            ));
        }

        Ok(())
    }

    /// Compute BCIB SHA-256 fingerprint
    fn compute_bcib_sha256(bcib: &BCIB) -> String {
        let mut hasher = Sha256::new();
        
        // Hash each instruction deterministically
        for instr in &bcib.instructions {
            hasher.update(format!("{:?}", instr).as_bytes());
        }

        format!("{:x}", hasher.finalize())
    }

    /// Compute result fingerprint
    fn compute_result_fingerprint(result: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(result.as_bytes());
        format!("{:x}", hasher.finalize())
    }

    /// Extract expected result from replay binding
    fn extract_expected_result(binding: &ProofReplayBinding) -> Result<String, SemanticCLIError> {
        if binding.submission_result_fingerprint.is_empty() {
            return Err(SemanticCLIError::execution_error(
                "Empty submission result fingerprint in replay binding",
                crate::error::ErrorCode::E750,
            ));
        }

        Ok(binding.submission_result_fingerprint.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bcib_simple::{BCIBInstruction, BCIBOperand};

    fn create_test_bcib() -> BCIB {
        BCIB {
            instructions: vec![
                BCIBInstruction::DataQuery {
                    target: BCIBOperand::Register(0),
                    context: "users".to_string(),
                    filter: None,
                },
                BCIBInstruction::End {
                    result: BCIBOperand::Register(0),
                },
            ],
        }
    }

    fn create_test_proof(bcib: &BCIB, result: &str) -> ProofChainRecord {
        let bcib_sha256 = ReplayVerifier::compute_bcib_sha256(bcib);
        let result_fingerprint = ReplayVerifier::compute_result_fingerprint(result);

        ProofChainRecord {
            canonical_command: "list users".to_string(),
            canonical_command_sha256: "test_command_hash".to_string(),
            canonical_plan_fingerprint: "test_plan_fp".to_string(),
            canonical_binding_fingerprint: "test_binding_fp".to_string(),
            bcib_sha256: bcib_sha256.clone(),
            target_context_id: 1,
            submission_id: crate::gate_c::types::SubmissionId {
                id: "test_submission_001".to_string(),
                timestamp: 0,
                fingerprint: Some(bcib_sha256.clone()),
            },
            required_capabilities: vec![],
            declared_capabilities: vec![],
            replay_binding: ProofReplayBinding {
                canonical_plan_fingerprint: "test_plan_fp".to_string(),
                canonical_binding_fingerprint: "test_binding_fp".to_string(),
                bcib_sha256,
                submission_result_fingerprint: result_fingerprint,
            },
            proof_chain_sha256: "test_proof_hash".to_string(),
        }
    }

    #[test]
    fn test_replay_match() {
        let bcib = create_test_bcib();
        let result = "user1,user2,user3";
        let proof = create_test_proof(&bcib, result);

        let verification = ReplayVerifier::verify_replay(&bcib, &proof, result);
        assert!(verification.is_ok());

        match verification.unwrap() {
            ReplayVerificationResult::Match { .. } => {}
            _ => panic!("Expected Match result"),
        }
    }

    #[test]
    fn test_replay_deviation() {
        let bcib = create_test_bcib();
        let original_result = "user1,user2,user3";
        let proof = create_test_proof(&bcib, original_result);

        let different_result = "user1,user2,user4"; // Different!
        let verification = ReplayVerifier::verify_replay(&bcib, &proof, different_result);
        assert!(verification.is_ok());

        match verification.unwrap() {
            ReplayVerificationResult::Deviation { .. } => {}
            _ => panic!("Expected Deviation result"),
        }
    }

    #[test]
    fn test_bcib_identity_mismatch() {
        let bcib = create_test_bcib();
        let result = "user1,user2,user3";
        let mut proof = create_test_proof(&bcib, result);

        // Tamper with BCIB SHA-256
        proof.bcib_sha256 = "tampered_hash".to_string();

        let verification = ReplayVerifier::verify_replay(&bcib, &proof, result);
        assert!(verification.is_err());
    }

    #[test]
    fn test_binding_integrity_empty_plan_fingerprint() {
        let binding = ProofReplayBinding {
            canonical_plan_fingerprint: "".to_string(), // Empty!
            canonical_binding_fingerprint: "test_binding".to_string(),
            bcib_sha256: "test_bcib".to_string(),
            submission_result_fingerprint: "test_result".to_string(),
        };

        let result = ReplayVerifier::verify_binding_integrity(&binding);
        assert!(result.is_err());
    }

    #[test]
    fn test_binding_integrity_valid() {
        let binding = ProofReplayBinding {
            canonical_plan_fingerprint: "test_plan".to_string(),
            canonical_binding_fingerprint: "test_binding".to_string(),
            bcib_sha256: "test_bcib".to_string(),
            submission_result_fingerprint: "test_result".to_string(),
        };

        let result = ReplayVerifier::verify_binding_integrity(&binding);
        assert!(result.is_ok());
    }
}
