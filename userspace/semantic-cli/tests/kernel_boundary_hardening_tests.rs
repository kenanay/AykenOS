//! Kernel Boundary Hardening Tests
//!
//! Tests for Phase-16 Task 2: Kernel boundary hardening implementation.
//! Verifies that syscall submission path hardening, boundary violation detection,
//! and fail-closed enforcement work correctly.

use semantic_cli::{
    error::{ErrorCode, SemanticCLIError},
    isolation::{
        IsolationLevel, KernelBoundaryDetector, SecurityContext, SyscallSubmissionEnforcer,
    },
    kernel_submit_adapter::KernelSubmitAdapter,
    submission_bridge::{SubmissionInput, SubmitAdapter},
    bcib_simple::{BCIB, BCIBInstruction, BCIBOperand},
    canonical_query::CanonicalQueryBinding,
};

/// Test that syscall submission path hardening works correctly
#[test]
fn test_syscall_submission_path_hardening() {
    let enforcer = SyscallSubmissionEnforcer::new();
    
    // Test approved syscall
    let result = enforcer.validate_submission_path("SYS_V2_SUBMIT_EXECUTION");
    assert!(result.is_ok(), "SYS_V2_SUBMIT_EXECUTION should be approved");
    
    // Test forbidden syscalls
    let forbidden_syscalls = [
        "SYS_DIRECT_CALL",
        "SYS_KERNEL_ACCESS",
        "SYS_DEVICE_MMIO",
        "SYS_INTERRUPT_HANDLER",
        "SYS_RING0_TRANSITION",
    ];
    
    for syscall in &forbidden_syscalls {
        let result = enforcer.validate_submission_path(syscall);
        assert!(result.is_err(), "Syscall {} should be forbidden", syscall);
        
        if let Err(SemanticCLIError::KernelBoundaryViolation { code, .. }) = result {
            assert_eq!(code, ErrorCode::E963, "Should return syscall surface violation error");
        } else {
            panic!("Expected KernelBoundaryViolation error for {}", syscall);
        }
    }
}

/// Test that kernel boundary violation detection works correctly
#[test]
fn test_kernel_boundary_violation_detection() {
    let detector = KernelBoundaryDetector::new();
    
    // Test safe operations
    let safe_operations = [
        "bcib_submission",
        "safe_operation",
        "user_space_call",
        "runtime_bridge_call",
    ];
    
    for operation in &safe_operations {
        let result = detector.detect_violation(operation);
        assert!(result.is_ok(), "Operation {} should be safe", operation);
    }
    
    // Test forbidden operations
    let forbidden_operations = [
        "direct_syscall",
        "kernel_memory_access",
        "device_mmio",
        "interrupt_handler",
        "ring0_transition",
    ];
    
    for operation in &forbidden_operations {
        let result = detector.detect_violation(operation);
        assert!(result.is_err(), "Operation {} should be forbidden", operation);
        
        if let Err(SemanticCLIError::KernelBoundaryViolation { code, .. }) = result {
            assert_eq!(code, ErrorCode::E962, "Should return kernel boundary violation error");
        } else {
            panic!("Expected KernelBoundaryViolation error for {}", operation);
        }
    }
}

/// Test that kernel submit adapter enforces boundary hardening
#[test]
fn test_kernel_submit_adapter_boundary_enforcement() {
    let adapter = KernelSubmitAdapter::with_endpoint("test_kernel".to_string());
    
    // Create a valid BCIB
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
    
    let input = SubmissionInput {
        canonical_command: "list users".to_string(),
        canonical_binding: CanonicalQueryBinding {
            context_path: "users".to_string(),
            predicate_kind: semantic_cli::canonical_query::CanonicalPredicateKind::All,
            predicate_fingerprint: None,
        },
        bcib,
        declared_capabilities: vec![],
    };
    
    // Test successful submission with boundary enforcement
    let result = adapter.submit(input);
    assert!(result.is_ok(), "Valid BCIB should be submitted successfully");
    
    let submission = result.unwrap();
    assert_eq!(submission.status, "submitted_with_boundary_enforcement");
    assert!(submission.result.is_some());
    assert!(submission.result.unwrap().contains("SYS_V2_SUBMIT_EXECUTION"));
}

/// Test that empty BCIB triggers isolation violation
#[test]
fn test_empty_bcib_isolation_violation() {
    let adapter = KernelSubmitAdapter::with_endpoint("test_kernel".to_string());
    
    let bcib = BCIB {
        instructions: vec![],
    };
    
    let input = SubmissionInput {
        canonical_command: "empty".to_string(),
        canonical_binding: CanonicalQueryBinding {
            context_path: "test".to_string(),
            predicate_kind: semantic_cli::canonical_query::CanonicalPredicateKind::All,
            predicate_fingerprint: None,
        },
        bcib,
        declared_capabilities: vec![],
    };
    
    let result = adapter.submit(input);
    assert!(result.is_err(), "Empty BCIB should be rejected");
    
    if let Err(SemanticCLIError::BcibIsolationViolation { code, .. }) = result {
        assert_eq!(code, ErrorCode::E950, "Should return BCIB isolation violation error");
    } else {
        panic!("Expected BcibIsolationViolation error");
    }
}

/// Test that BCIB without End instruction triggers isolation violation
#[test]
fn test_bcib_without_end_isolation_violation() {
    let adapter = KernelSubmitAdapter::with_endpoint("test_kernel".to_string());
    
    let bcib = BCIB {
        instructions: vec![
            BCIBInstruction::DataQuery {
                target: BCIBOperand::Register(0),
                context: "users".to_string(),
                filter: None,
            },
            // Missing End instruction
        ],
    };
    
    let input = SubmissionInput {
        canonical_command: "incomplete".to_string(),
        canonical_binding: CanonicalQueryBinding {
            context_path: "test".to_string(),
            predicate_kind: semantic_cli::canonical_query::CanonicalPredicateKind::All,
            predicate_fingerprint: None,
        },
        bcib,
        declared_capabilities: vec![],
    };
    
    let result = adapter.submit(input);
    assert!(result.is_err(), "BCIB without End should be rejected");
    
    if let Err(SemanticCLIError::BcibIsolationViolation { code, .. }) = result {
        assert_eq!(code, ErrorCode::E950, "Should return BCIB isolation violation error");
    } else {
        panic!("Expected BcibIsolationViolation error");
    }
}

/// Test that forbidden instructions trigger isolation violation
#[test]
fn test_forbidden_instruction_isolation_violation() {
    let adapter = KernelSubmitAdapter::with_endpoint("test_kernel".to_string());
    
    let bcib = BCIB {
        instructions: vec![
            BCIBInstruction::Nop, // Forbidden instruction
            BCIBInstruction::End {
                result: BCIBOperand::Register(0),
            },
        ],
    };
    
    let input = SubmissionInput {
        canonical_command: "forbidden".to_string(),
        canonical_binding: CanonicalQueryBinding {
            context_path: "test".to_string(),
            predicate_kind: semantic_cli::canonical_query::CanonicalPredicateKind::All,
            predicate_fingerprint: None,
        },
        bcib,
        declared_capabilities: vec![],
    };
    
    let result = adapter.submit(input);
    assert!(result.is_err(), "BCIB with forbidden instruction should be rejected");
    
    if let Err(SemanticCLIError::BcibIsolationViolation { code, .. }) = result {
        assert_eq!(code, ErrorCode::E950, "Should return BCIB isolation violation error");
    } else {
        panic!("Expected BcibIsolationViolation error");
    }
}

/// Test that missing kernel endpoint triggers boundary violation
#[test]
fn test_missing_kernel_endpoint_boundary_violation() {
    let adapter = KernelSubmitAdapter::new(); // No endpoint configured
    
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
    
    let input = SubmissionInput {
        canonical_command: "no_endpoint".to_string(),
        canonical_binding: CanonicalQueryBinding {
            context_path: "test".to_string(),
            predicate_kind: semantic_cli::canonical_query::CanonicalPredicateKind::All,
            predicate_fingerprint: None,
        },
        bcib,
        declared_capabilities: vec![],
    };
    
    let result = adapter.submit(input);
    assert!(result.is_err(), "Missing kernel endpoint should trigger boundary violation");
    
    if let Err(SemanticCLIError::KernelBoundaryViolation { code, .. }) = result {
        assert_eq!(code, ErrorCode::E962, "Should return kernel boundary violation error");
    } else {
        panic!("Expected KernelBoundaryViolation error");
    }
}

/// Test that hardening can be disabled for testing
#[test]
fn test_hardening_disabled_mode() {
    let adapter = KernelSubmitAdapter::with_hardening_disabled();
    
    // Even with hardening disabled, missing endpoint should still fail
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
    
    let input = SubmissionInput {
        canonical_command: "hardening_disabled".to_string(),
        canonical_binding: CanonicalQueryBinding {
            context_path: "test".to_string(),
            predicate_kind: semantic_cli::canonical_query::CanonicalPredicateKind::All,
            predicate_fingerprint: None,
        },
        bcib,
        declared_capabilities: vec![],
    };
    
    let result = adapter.submit(input);
    assert!(result.is_err(), "Should still fail without endpoint even with hardening disabled");
}

/// Test security context isolation level enforcement
#[test]
fn test_security_context_isolation_levels() {
    // Test isolation level ordering
    assert!(IsolationLevel::None < IsolationLevel::Sandboxed);
    assert!(IsolationLevel::Sandboxed < IsolationLevel::FullyIsolated);
    
    // Test security context meets isolation requirements
    let ctx_none = SecurityContext::with_isolation(IsolationLevel::None);
    let ctx_sandboxed = SecurityContext::with_isolation(IsolationLevel::Sandboxed);
    let ctx_fully_isolated = SecurityContext::with_isolation(IsolationLevel::FullyIsolated);
    
    // None level context
    assert!(ctx_none.meets_isolation_requirement(IsolationLevel::None));
    assert!(!ctx_none.meets_isolation_requirement(IsolationLevel::Sandboxed));
    assert!(!ctx_none.meets_isolation_requirement(IsolationLevel::FullyIsolated));
    
    // Sandboxed level context
    assert!(ctx_sandboxed.meets_isolation_requirement(IsolationLevel::None));
    assert!(ctx_sandboxed.meets_isolation_requirement(IsolationLevel::Sandboxed));
    assert!(!ctx_sandboxed.meets_isolation_requirement(IsolationLevel::FullyIsolated));
    
    // Fully isolated level context
    assert!(ctx_fully_isolated.meets_isolation_requirement(IsolationLevel::None));
    assert!(ctx_fully_isolated.meets_isolation_requirement(IsolationLevel::Sandboxed));
    assert!(ctx_fully_isolated.meets_isolation_requirement(IsolationLevel::FullyIsolated));
}

/// Test fail-closed enforcement behavior
#[test]
fn test_fail_closed_enforcement() {
    let adapter = KernelSubmitAdapter::with_endpoint("test_kernel".to_string());
    
    // Test various violation scenarios that should all result in fail-closed behavior
    let violation_scenarios = vec![
        // Empty BCIB
        BCIB { instructions: vec![] },
        
        // BCIB without End instruction
        BCIB {
            instructions: vec![
                BCIBInstruction::DataQuery {
                    target: BCIBOperand::Register(0),
                    context: "users".to_string(),
                    filter: None,
                },
            ],
        },
        
        // BCIB with forbidden instruction
        BCIB {
            instructions: vec![
                BCIBInstruction::Nop,
                BCIBInstruction::End {
                    result: BCIBOperand::Register(0),
                },
            ],
        },
    ];
    
    for (i, bcib) in violation_scenarios.into_iter().enumerate() {
        let input = SubmissionInput {
            canonical_command: format!("violation_scenario_{}", i),
            canonical_binding: CanonicalQueryBinding {
                context_path: "test".to_string(),
                predicate_kind: semantic_cli::canonical_query::CanonicalPredicateKind::All,
                predicate_fingerprint: None,
            },
            bcib,
            declared_capabilities: vec![],
        };
        
        let result = adapter.submit(input);
        assert!(result.is_err(), "Violation scenario {} should fail closed", i);
        
        // All violations should result in specific error types, not generic failures
        match result.unwrap_err() {
            SemanticCLIError::BcibIsolationViolation { .. } => {
                // Expected for BCIB-related violations
            }
            SemanticCLIError::KernelBoundaryViolation { .. } => {
                // Expected for kernel boundary violations
            }
            other => {
                panic!("Unexpected error type for violation scenario {}: {:?}", i, other);
            }
        }
    }
}

/// Test constitutional compliance enforcement
#[test]
fn test_constitutional_compliance() {
    // Test that the new error codes are properly defined
    let boundary_error = SemanticCLIError::kernel_boundary_violation(
        "Test boundary violation",
        ErrorCode::E962,
    );
    
    assert_eq!(boundary_error.code(), Some(ErrorCode::E962));
    assert!(format!("{}", boundary_error).contains("Kernel boundary violation"));
    
    let isolation_error = SemanticCLIError::bcib_isolation_violation(
        "Test isolation violation",
        ErrorCode::E950,
    );
    
    assert_eq!(isolation_error.code(), Some(ErrorCode::E950));
    assert!(format!("{}", isolation_error).contains("BCIB isolation violation"));
    
    let bridge_error = SemanticCLIError::runtime_bridge_violation(
        "Test bridge violation",
        ErrorCode::E951,
    );
    
    assert_eq!(bridge_error.code(), Some(ErrorCode::E951));
    assert!(format!("{}", bridge_error).contains("Runtime bridge violation"));
}