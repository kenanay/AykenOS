// capability_derivation_audit.rs
// Audit trail for capability derivation from lowered BCIB
// CONSTITUTIONAL: KERNEL.CAPABILITY.BYPASS enforcement

use crate::bcib_simple::{BCIB, BCIBInstruction};
use crate::canonical_query::CanonicalQueryBinding;
use crate::error::SemanticCLIError;
use crate::submission_bridge::Capability;

/// Capability derivation audit result
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CapabilityDerivationAudit {
    /// Canonical query binding that was lowered
    pub canonical_binding: CanonicalQueryBinding,
    
    /// Lowered BCIB
    pub bcib_fingerprint: String,
    
    /// Derived capabilities
    pub derived_capabilities: Vec<Capability>,
    
    /// Audit trail: which BCIB instructions required which capabilities
    pub derivation_trail: Vec<DerivationStep>,
}

/// Single step in capability derivation
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DerivationStep {
    /// BCIB instruction index
    pub instruction_index: usize,
    
    /// BCIB instruction type
    pub instruction_type: String,
    
    /// Capability derived from this instruction
    pub capability: Capability,
    
    /// Reason for derivation
    pub reason: String,
}

/// Capability derivation auditor
/// 
/// CONSTITUTIONAL ENFORCEMENT:
/// - KERNEL.CAPABILITY.BYPASS: verify lowerer derives correct capabilities
/// - Audit trail: prove capability derivation is deterministic
pub struct CapabilityDerivationAuditor;

impl CapabilityDerivationAuditor {
    /// Audit capability derivation from BCIB
    /// 
    /// Verifies that the lowerer correctly derives capabilities from BCIB instructions
    pub fn audit_derivation(
        canonical_binding: &CanonicalQueryBinding,
        bcib: &BCIB,
        derived_capabilities: &[Capability],
    ) -> Result<CapabilityDerivationAudit, SemanticCLIError> {
        // 1. Build expected capabilities from BCIB
        let (expected_capabilities, derivation_trail) = Self::derive_from_bcib(bcib)?;

        // 2. Verify derived capabilities match expected
        Self::verify_capability_match(&expected_capabilities, derived_capabilities)?;

        // 3. Return audit record
        Ok(CapabilityDerivationAudit {
            canonical_binding: canonical_binding.clone(),
            bcib_fingerprint: Self::compute_bcib_fingerprint(bcib),
            derived_capabilities: derived_capabilities.to_vec(),
            derivation_trail,
        })
    }

    /// Derive capabilities from BCIB instructions
    /// 
    /// This is the REFERENCE implementation that the lowerer must match
    fn derive_from_bcib(
        bcib: &BCIB,
    ) -> Result<(Vec<Capability>, Vec<DerivationStep>), SemanticCLIError> {
        let mut capabilities = Vec::new();
        let mut trail = Vec::new();

        for (idx, instr) in bcib.instructions.iter().enumerate() {
            match instr {
                BCIBInstruction::DataQuery { context, .. } => {
                    let cap = Capability {
                        name: "context.read".to_string(),
                        scope: "query".to_string(),
                        resource: context.clone(),
                        reason: format!("DataQuery on context '{}'", context),
                    };

                    trail.push(DerivationStep {
                        instruction_index: idx,
                        instruction_type: "DataQuery".to_string(),
                        capability: cap.clone(),
                        reason: format!("Read access to context '{}'", context),
                    });

                    capabilities.push(cap);
                }
                BCIBInstruction::DataCreate { context, .. } => {
                    let cap = Capability {
                        name: "context.write".to_string(),
                        scope: "mutation".to_string(),
                        resource: context.clone(),
                        reason: format!("DataCreate on context '{}'", context),
                    };

                    trail.push(DerivationStep {
                        instruction_index: idx,
                        instruction_type: "DataCreate".to_string(),
                        capability: cap.clone(),
                        reason: format!("Write access to context '{}'", context),
                    });

                    capabilities.push(cap);
                }
                BCIBInstruction::End { .. } => {
                    // End instruction requires no capabilities
                }
                BCIBInstruction::TraceEmit { .. } => {
                    // TraceEmit requires no capabilities (observability only)
                }
                BCIBInstruction::Nop => {
                    // Nop should never appear in production BCIB
                    return Err(SemanticCLIError::execution_error(
                        "Nop instruction in BCIB",
                        crate::error::ErrorCode::E770,
                    ));
                }
            }
        }

        Ok((capabilities, trail))
    }

    /// Verify derived capabilities match expected
    fn verify_capability_match(
        expected: &[Capability],
        derived: &[Capability],
    ) -> Result<(), SemanticCLIError> {
        // 1. Verify count matches
        if expected.len() != derived.len() {
            return Err(SemanticCLIError::execution_error(
                format!(
                    "Capability count mismatch: expected {}, got {}",
                    expected.len(),
                    derived.len()
                ),
                crate::error::ErrorCode::E771,
            ));
        }

        // 2. Verify each capability matches (semantic identity, not string equality)
        for (exp, der) in expected.iter().zip(derived.iter()) {
            if !Self::capabilities_match(exp, der) {
                return Err(SemanticCLIError::execution_error(
                    format!("Capability mismatch: expected {:?}, got {:?}", exp, der),
                    crate::error::ErrorCode::E771,
                ));
            }
        }

        Ok(())
    }

    /// Check if two capabilities match semantically
    /// 
    /// CRITICAL: reason field is NOT part of semantic identity
    fn capabilities_match(a: &Capability, b: &Capability) -> bool {
        a.name == b.name && a.scope == b.scope && a.resource == b.resource
    }

    /// Compute BCIB fingerprint for audit trail
    fn compute_bcib_fingerprint(bcib: &BCIB) -> String {
        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        
        for instr in &bcib.instructions {
            hasher.update(format!("{:?}", instr).as_bytes());
        }

        format!("{:x}", hasher.finalize())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bcib_simple::BCIBOperand;

    fn create_test_binding() -> CanonicalQueryBinding {
        CanonicalQueryBinding {
            context_path: "users".to_string(),
            predicate_kind: crate::canonical_query::CanonicalPredicateKind::All,
            predicate_fingerprint: None,
        }
    }

    #[test]
    fn test_audit_data_query() {
        let binding = create_test_binding();
        let bcib = BCIB {
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
        };

        let derived = vec![Capability {
            name: "context.read".to_string(),
            scope: "query".to_string(),
            resource: "users".to_string(),
            reason: "DataQuery on context 'users'".to_string(),
        }];

        let audit = CapabilityDerivationAuditor::audit_derivation(&binding, &bcib, &derived);
        assert!(audit.is_ok());

        let audit = audit.unwrap();
        assert_eq!(audit.derived_capabilities.len(), 1);
        assert_eq!(audit.derivation_trail.len(), 1);
    }

    #[test]
    fn test_audit_capability_mismatch() {
        let binding = create_test_binding();
        let bcib = BCIB {
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
        };

        // Wrong capability!
        let derived = vec![Capability {
            name: "context.write".to_string(), // Should be read!
            scope: "query".to_string(),
            resource: "users".to_string(),
            reason: "Wrong capability".to_string(),
        }];

        let audit = CapabilityDerivationAuditor::audit_derivation(&binding, &bcib, &derived);
        assert!(audit.is_err());
    }

    #[test]
    fn test_audit_nop_instruction() {
        let binding = create_test_binding();
        let bcib = BCIB {
            instructions: vec![
                BCIBInstruction::Nop, // Forbidden!
                BCIBInstruction::End {
                    result: BCIBOperand::Register(0),
                },
            ],
        };

        let derived = vec![];

        let audit = CapabilityDerivationAuditor::audit_derivation(&binding, &bcib, &derived);
        assert!(audit.is_err());
    }

    #[test]
    fn test_capability_match_ignores_reason() {
        let cap1 = Capability {
            name: "context.read".to_string(),
            scope: "query".to_string(),
            resource: "users".to_string(),
            reason: "Reason A".to_string(),
        };

        let cap2 = Capability {
            name: "context.read".to_string(),
            scope: "query".to_string(),
            resource: "users".to_string(),
            reason: "Reason B".to_string(), // Different reason!
        };

        assert!(CapabilityDerivationAuditor::capabilities_match(&cap1, &cap2));
    }

    #[test]
    fn test_derive_from_bcib_multiple_queries() {
        let bcib = BCIB {
            instructions: vec![
                BCIBInstruction::DataQuery {
                    target: BCIBOperand::Register(0),
                    context: "users".to_string(),
                    filter: None,
                },
                BCIBInstruction::DataQuery {
                    target: BCIBOperand::Register(1),
                    context: "posts".to_string(),
                    filter: None,
                },
                BCIBInstruction::End {
                    result: BCIBOperand::Register(0),
                },
            ],
        };

        let (capabilities, trail) = CapabilityDerivationAuditor::derive_from_bcib(&bcib).unwrap();
        
        assert_eq!(capabilities.len(), 2);
        assert_eq!(trail.len(), 2);
        
        assert_eq!(capabilities[0].resource, "users");
        assert_eq!(capabilities[1].resource, "posts");
    }
}
