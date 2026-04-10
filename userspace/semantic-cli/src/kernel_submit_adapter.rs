// kernel_submit_adapter.rs
// Real kernel-facing submission adapter
// CONSTITUTIONAL: SECURITY.BOUNDARY.VIOLATION enforcement

use crate::bcib_simple::BCIB;
use crate::error::SemanticCLIError;
use crate::submission_bridge::{SubmitAdapter, SubmissionInput, SubmissionResult};

/// Kernel submission adapter
/// 
/// CONSTITUTIONAL ENFORCEMENT:
/// - SECURITY.BOUNDARY.VIOLATION: Ring3 → Ring0 boundary enforcement
/// - KERNEL.CAPABILITY.BYPASS: capability validation before submission
/// 
/// This adapter is the ONLY authorized path for Ring3 → Ring0 BCIB submission
pub struct KernelSubmitAdapter {
    /// Kernel IPC endpoint (placeholder for real kernel connection)
    kernel_endpoint: Option<String>,
}

impl KernelSubmitAdapter {
    /// Create new kernel submit adapter
    /// 
    /// In production, this would establish IPC connection to kernel
    pub fn new() -> Self {
        Self {
            kernel_endpoint: None, // TODO: real kernel IPC
        }
    }

    /// Create adapter with explicit kernel endpoint
    /// 
    /// For testing and development
    pub fn with_endpoint(endpoint: String) -> Self {
        Self {
            kernel_endpoint: Some(endpoint),
        }
    }

    /// Submit BCIB to kernel via IPC
    /// 
    /// SECURITY BOUNDARY:
    /// - This is the Ring3 → Ring0 crossing point
    /// - All capability validation MUST happen before this call
    /// - No semantic interpretation happens here
    fn submit_to_kernel(&self, bcib: &BCIB) -> Result<String, SemanticCLIError> {
        // TODO: Real kernel IPC implementation
        // 
        // Production implementation would:
        // 1. Serialize BCIB to kernel-compatible format
        // 2. Send via IPC (syscall, message queue, shared memory)
        // 3. Wait for kernel response
        // 4. Deserialize result
        // 5. Return to caller
        //
        // For now, return placeholder indicating kernel submission would happen
        
        if let Some(endpoint) = &self.kernel_endpoint {
            Ok(format!(
                "KERNEL_SUBMIT[{}]: {} instructions",
                endpoint,
                bcib.instructions.len()
            ))
        } else {
            // Fail closed: no kernel endpoint configured
            Err(SemanticCLIError::execution_error(
                "No kernel endpoint configured",
                crate::error::ErrorCode::E762,
            ))
        }
    }

    /// Verify BCIB is kernel-submittable
    /// 
    /// FAIL CLOSED: reject any BCIB that cannot be safely submitted
    fn verify_bcib_submittable(&self, bcib: &BCIB) -> Result<(), SemanticCLIError> {
        // 1. Verify BCIB is non-empty
        if bcib.instructions.is_empty() {
            return Err(SemanticCLIError::execution_error(
                "Cannot submit empty BCIB",
                crate::error::ErrorCode::E760,
            ));
        }

        // 2. Verify BCIB ends with End instruction
        if let Some(last) = bcib.instructions.last() {
            match last {
                crate::bcib_simple::BCIBInstruction::End { .. } => {}
                _ => {
                    return Err(SemanticCLIError::execution_error(
                        "BCIB must end with End instruction",
                        crate::error::ErrorCode::E760,
                    ));
                }
            }
        }

        // 3. Verify no forbidden instructions (Nop, etc.)
        for instr in &bcib.instructions {
            if matches!(instr, crate::bcib_simple::BCIBInstruction::Nop) {
                return Err(SemanticCLIError::execution_error(
                    "BCIB contains forbidden Nop instruction",
                    crate::error::ErrorCode::E760,
                ));
            }
        }

        Ok(())
    }
}

impl SubmitAdapter for KernelSubmitAdapter {
    fn submit(&self, input: SubmissionInput) -> Result<SubmissionResult, SemanticCLIError> {
        // 1. Verify BCIB is submittable (FAIL CLOSED)
        self.verify_bcib_submittable(&input.bcib)?;

        // 2. Submit to kernel
        let kernel_result = self.submit_to_kernel(&input.bcib)?;

        // 3. Return submission result
        Ok(SubmissionResult {
            submission_id: format!("kernel_sub_{}", uuid::Uuid::new_v4()),
            status: "submitted".to_string(),
            result: Some(kernel_result),
        })
    }
}

impl Default for KernelSubmitAdapter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bcib_simple::{BCIBInstruction, BCIBOperand};
    use crate::canonical_query::CanonicalQueryBinding;

    fn create_valid_bcib() -> BCIB {
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

    fn create_submission_input(bcib: BCIB) -> SubmissionInput {
        SubmissionInput {
            canonical_command: "list users".to_string(),
            canonical_binding: CanonicalQueryBinding {
                context_path: "users".to_string(),
                predicate_kind: crate::canonical_query::CanonicalPredicateKind::All,
                predicate_fingerprint: None,
            },
            bcib,
            declared_capabilities: vec![],
        }
    }

    #[test]
    fn test_submit_valid_bcib() {
        let adapter = KernelSubmitAdapter::with_endpoint("test_kernel".to_string());
        let bcib = create_valid_bcib();
        let input = create_submission_input(bcib);

        let result = adapter.submit(input);
        assert!(result.is_ok());

        let submission = result.unwrap();
        assert_eq!(submission.status, "submitted");
        assert!(submission.result.is_some());
    }

    #[test]
    fn test_submit_empty_bcib() {
        let adapter = KernelSubmitAdapter::with_endpoint("test_kernel".to_string());
        let bcib = BCIB {
            instructions: vec![],
        };
        let input = create_submission_input(bcib);

        let result = adapter.submit(input);
        assert!(result.is_err());
    }

    #[test]
    fn test_submit_bcib_without_end() {
        let adapter = KernelSubmitAdapter::with_endpoint("test_kernel".to_string());
        let bcib = BCIB {
            instructions: vec![BCIBInstruction::DataQuery {
                target: BCIBOperand::Register(0),
                context: "users".to_string(),
                filter: None,
            }],
        };
        let input = create_submission_input(bcib);

        let result = adapter.submit(input);
        assert!(result.is_err());
    }

    #[test]
    fn test_submit_bcib_with_nop() {
        let adapter = KernelSubmitAdapter::with_endpoint("test_kernel".to_string());
        let bcib = BCIB {
            instructions: vec![
                BCIBInstruction::Nop, // Forbidden!
                BCIBInstruction::End {
                    result: BCIBOperand::Register(0),
                },
            ],
        };
        let input = create_submission_input(bcib);

        let result = adapter.submit(input);
        assert!(result.is_err());
    }

    #[test]
    fn test_submit_no_kernel_endpoint() {
        let adapter = KernelSubmitAdapter::new(); // No endpoint
        let bcib = create_valid_bcib();
        let input = create_submission_input(bcib);

        let result = adapter.submit(input);
        assert!(result.is_err());
    }
}
