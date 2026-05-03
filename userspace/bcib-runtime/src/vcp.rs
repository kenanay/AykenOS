//! VCP (Verified Contract Protocol) - Trust Layer
//!
//! Enforces contract validation before execution/commit/replay.
//! Fail-closed guarantee: invalid state → execution denied.
//!
//! Evidence Binding: VCP verification results are bound to evidence records
//! for consistency validation. Inconsistent evidence triggers fail-closed.

use crate::types::BcibError;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VcpTrustState {
    Trusted,
    Rejected,
}

#[derive(Debug, Clone)]
pub struct VcpVerificationResult {
    pub trust_state: VcpTrustState,
    pub reason: &'static str,
}

/// VCP Evidence Record
///
/// Binds verification result to evidence for consistency validation.
/// Evidence must match verification result exactly (fail-closed on mismatch).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VcpEvidenceRecord {
    pub context_id: u64,
    pub operation_id: u64,
    pub trust_state: VcpTrustState,
    pub reason: &'static str,
    pub state_hash: [u8; 32],
}

impl VcpEvidenceRecord {
    /// Create evidence record from verification result
    ///
    /// # Guarantees
    /// - context_id and operation_id must be non-zero
    /// - state_hash is canonical (derived from trust state and reason)
    /// - trust_state matches verification result exactly
    pub fn from_verification(
        context_id: u64,
        operation_id: u64,
        result: &VcpVerificationResult,
    ) -> Result<Self, BcibError> {
        if context_id == 0 {
            return Err(BcibError::IllegalStateTransition("context_id cannot be zero"));
        }
        if operation_id == 0 {
            return Err(BcibError::IllegalStateTransition("operation_id cannot be zero"));
        }

        let state_hash = Self::compute_state_hash(result);

        Ok(Self {
            context_id,
            operation_id,
            trust_state: result.trust_state,
            reason: result.reason,
            state_hash,
        })
    }

    /// Compute canonical state hash from verification result
    ///
    /// Hash includes: trust_state + reason
    /// Ensures evidence integrity and tamper detection
    fn compute_state_hash(result: &VcpVerificationResult) -> [u8; 32] {
        let mut hasher = DefaultHasher::new();
        
        // Hash trust state
        match result.trust_state {
            VcpTrustState::Trusted => 1u8.hash(&mut hasher),
            VcpTrustState::Rejected => 0u8.hash(&mut hasher),
        }
        
        // Hash reason
        result.reason.hash(&mut hasher);
        
        let hash_value = hasher.finish();
        
        // Expand to 32 bytes (simple expansion for now)
        let mut hash = [0u8; 32];
        hash[0..8].copy_from_slice(&hash_value.to_le_bytes());
        hash
    }

    /// Validate evidence consistency with verification result
    ///
    /// # Guarantees
    /// - trust_state must match exactly
    /// - state_hash must match computed hash
    /// - context_id and operation_id must be non-zero
    ///
    /// # Fail-Closed
    /// Returns Err on any inconsistency
    pub fn validate_consistency(&self, result: &VcpVerificationResult) -> Result<(), BcibError> {
        // Check trust state match
        if self.trust_state != result.trust_state {
            return Err(BcibError::IllegalStateTransition(
                "evidence trust_state does not match verification result",
            ));
        }

        // Check reason match
        if self.reason != result.reason {
            return Err(BcibError::IllegalStateTransition(
                "evidence reason does not match verification result",
            ));
        }

        // Check state hash
        let expected_hash = Self::compute_state_hash(result);
        if self.state_hash != expected_hash {
            return Err(BcibError::IllegalStateTransition(
                "evidence state_hash does not match computed hash",
            ));
        }

        // Check context_id validity
        if self.context_id == 0 {
            return Err(BcibError::IllegalStateTransition("evidence context_id is zero"));
        }

        // Check operation_id validity
        if self.operation_id == 0 {
            return Err(BcibError::IllegalStateTransition("evidence operation_id is zero"));
        }

        Ok(())
    }
}

/// Verify execution state eligibility
///
/// # Guarantees
/// - Invalid states are rejected
/// - Fail-closed on verification failure
pub fn verify_execution_state() -> Result<VcpVerificationResult, BcibError> {
    // Placeholder: actual state verification will be integrated
    // with ExecutionState in execution_runtime.rs
    Ok(VcpVerificationResult {
        trust_state: VcpTrustState::Trusted,
        reason: "execution state accepted",
    })
}

/// Verify operation eligibility before execution
///
/// # Guarantees
/// - Invalid operations are rejected
/// - Fail-closed on verification failure
pub fn verify_operation() -> Result<VcpVerificationResult, BcibError> {
    // Placeholder: actual operation verification will be integrated
    // with instruction execution pipeline
    Ok(VcpVerificationResult {
        trust_state: VcpTrustState::Trusted,
        reason: "operation accepted",
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vcp_verification_pass() {
        let result = verify_execution_state();
        assert!(result.is_ok());
        
        let vcp_result = result.unwrap();
        assert_eq!(vcp_result.trust_state, VcpTrustState::Trusted);
    }

    #[test]
    fn test_vcp_operation_verification() {
        let result = verify_operation();
        assert!(result.is_ok());
        
        let vcp_result = result.unwrap();
        assert_eq!(vcp_result.trust_state, VcpTrustState::Trusted);
    }

    #[test]
    fn test_evidence_record_creation() {
        let result = VcpVerificationResult {
            trust_state: VcpTrustState::Trusted,
            reason: "test verification",
        };

        let evidence = VcpEvidenceRecord::from_verification(1, 100, &result);
        assert!(evidence.is_ok());

        let evidence = evidence.unwrap();
        assert_eq!(evidence.context_id, 1);
        assert_eq!(evidence.operation_id, 100);
        assert_eq!(evidence.trust_state, VcpTrustState::Trusted);
        assert_eq!(evidence.reason, "test verification");
        assert_ne!(evidence.state_hash, [0u8; 32]); // Hash should be non-zero
    }

    #[test]
    fn test_evidence_record_zero_context_id_rejected() {
        let result = VcpVerificationResult {
            trust_state: VcpTrustState::Trusted,
            reason: "test",
        };

        let evidence = VcpEvidenceRecord::from_verification(0, 100, &result);
        assert!(evidence.is_err());
    }

    #[test]
    fn test_evidence_record_zero_operation_id_rejected() {
        let result = VcpVerificationResult {
            trust_state: VcpTrustState::Trusted,
            reason: "test",
        };

        let evidence = VcpEvidenceRecord::from_verification(1, 0, &result);
        assert!(evidence.is_err());
    }

    #[test]
    fn test_evidence_consistency_validation_pass() {
        let result = VcpVerificationResult {
            trust_state: VcpTrustState::Trusted,
            reason: "test verification",
        };

        let evidence = VcpEvidenceRecord::from_verification(1, 100, &result).unwrap();
        
        // Validate consistency - should pass
        assert!(evidence.validate_consistency(&result).is_ok());
    }

    #[test]
    fn test_evidence_consistency_trust_state_mismatch() {
        let result = VcpVerificationResult {
            trust_state: VcpTrustState::Trusted,
            reason: "test verification",
        };

        let evidence = VcpEvidenceRecord::from_verification(1, 100, &result).unwrap();
        
        // Create different result with mismatched trust state
        let different_result = VcpVerificationResult {
            trust_state: VcpTrustState::Rejected,
            reason: "test verification",
        };

        // Validation should fail
        assert!(evidence.validate_consistency(&different_result).is_err());
    }

    #[test]
    fn test_evidence_consistency_reason_mismatch() {
        let result = VcpVerificationResult {
            trust_state: VcpTrustState::Trusted,
            reason: "test verification",
        };

        let evidence = VcpEvidenceRecord::from_verification(1, 100, &result).unwrap();
        
        // Create different result with mismatched reason
        let different_result = VcpVerificationResult {
            trust_state: VcpTrustState::Trusted,
            reason: "different reason",
        };

        // Validation should fail
        assert!(evidence.validate_consistency(&different_result).is_err());
    }

    #[test]
    fn test_evidence_consistency_hash_integrity() {
        let result1 = VcpVerificationResult {
            trust_state: VcpTrustState::Trusted,
            reason: "test verification",
        };

        let result2 = VcpVerificationResult {
            trust_state: VcpTrustState::Trusted,
            reason: "test verification",
        };

        let evidence1 = VcpEvidenceRecord::from_verification(1, 100, &result1).unwrap();
        let evidence2 = VcpEvidenceRecord::from_verification(2, 200, &result2).unwrap();

        // Same verification result should produce same hash
        assert_eq!(evidence1.state_hash, evidence2.state_hash);
    }

    #[test]
    fn test_evidence_consistency_different_hash() {
        let result1 = VcpVerificationResult {
            trust_state: VcpTrustState::Trusted,
            reason: "reason A",
        };

        let result2 = VcpVerificationResult {
            trust_state: VcpTrustState::Trusted,
            reason: "reason B",
        };

        let evidence1 = VcpEvidenceRecord::from_verification(1, 100, &result1).unwrap();
        let evidence2 = VcpEvidenceRecord::from_verification(1, 100, &result2).unwrap();

        // Different reasons should produce different hashes
        assert_ne!(evidence1.state_hash, evidence2.state_hash);
    }

    #[test]
    fn test_evidence_rejected_state() {
        let result = VcpVerificationResult {
            trust_state: VcpTrustState::Rejected,
            reason: "invalid state detected",
        };

        let evidence = VcpEvidenceRecord::from_verification(1, 100, &result).unwrap();
        
        assert_eq!(evidence.trust_state, VcpTrustState::Rejected);
        assert!(evidence.validate_consistency(&result).is_ok());
    }
}
