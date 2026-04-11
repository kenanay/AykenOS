/// Integration Tests for Phase-16 Isolation Infrastructure
///
/// This module contains integration tests that demonstrate the core isolation
/// infrastructure components working together to enforce fail-closed semantics
/// and constitutional compliance.

#[cfg(test)]
mod tests {
    use super::super::*;
    use crate::isolation::fail_closed::AuditLogEntry;
    use crate::types::ExecutionContextId;

    /// Test that constitutional violations trigger fail-closed termination
    #[test]
    fn constitutional_violation_triggers_fail_closed() {
        let enforcer = ConstitutionalEnforcer::new();

        // Attempt a constitutional violation
        let result = enforcer.validate_operation("global_state_mutation", Some(42));

        // Should return an isolation error that requires fail-closed termination
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(error.requires_fail_closed());
        assert_eq!(error.code, ErrorCode::DeterminismGlobal);
    }

    /// Test that multiple violations are properly handled
    #[test]
    fn multiple_violations_handled_correctly() {
        let errors = vec![
            IsolationError::new(ErrorCode::IsolationViolation, "Violation 1", Some(1)),
            IsolationError::new(ErrorCode::BoundaryViolation, "Violation 2", Some(2)),
            IsolationError::new(ErrorCode::ConstitutionalViolation, "Violation 3", None),
        ];

        let reason = TerminationReason::MultipleViolations(errors.clone());

        assert_eq!(reason.all_error_codes().len(), 3);
        assert!(reason.has_constitutional_violations());
        assert!(reason.has_security_violations());
    }

    /// Test that error taxonomy correctly classifies violations
    #[test]
    fn error_taxonomy_classification() {
        let isolation_error = IsolationError::new(
            ErrorCode::IsolationViolation,
            "Test isolation violation",
            Some(123),
        );

        assert_eq!(isolation_error.violation_type(), ViolationType::Isolation);
        assert!(isolation_error.requires_fail_closed());

        let capability_error = IsolationError::new(
            ErrorCode::CapabilityDenied,
            "Test capability denial",
            Some(456),
        );

        assert_eq!(capability_error.violation_type(), ViolationType::Capability);
        assert!(!capability_error.requires_fail_closed()); // Capability denials don't require fail-closed
    }

    /// Test that constitutional enforcer validates BCIB operations
    #[test]
    fn constitutional_enforcer_validates_bcib_operations() {
        let enforcer = ConstitutionalEnforcer::new();

        // Normal BCIB opcode should pass
        assert!(enforcer.validate_bcib_execution(0x01, 1).is_ok());

        // Syscall validation
        assert!(enforcer.validate_syscall(1003, 1).is_ok()); // SYS_V2_SUBMIT_EXECUTION
        assert!(enforcer.validate_syscall(1000, 1).is_err()); // Other syscalls should fail
    }

    /// Test that audit logging works correctly
    #[test]
    fn audit_logging_works() {
        let error = IsolationError::new(
            ErrorCode::SecurityBoundaryViolation,
            "Security boundary violated",
            Some(789),
        );
        let reason = TerminationReason::from_isolation_error(error);

        let audit_entry = AuditLogEntry::new(reason, Some(789)).with_context("Test context");

        assert_eq!(audit_entry.context_id, Some(789));
        assert!(audit_entry.additional_context.is_some());
        assert!(audit_entry.timestamp > 0);

        let display = format!("{}", audit_entry);
        assert!(display.contains("Security boundary"));
        assert!(display.contains("[context: 789]"));
        assert!(display.contains("Test context"));
    }

    /// Test that placeholder components are properly initialized
    #[test]
    fn placeholder_components_initialization() {
        use crate::capability_manager::NoopCapabilityManager;
        use crate::isolation::abdf_handle::{HandleManager, SegmentType};
        use std::sync::{Arc, Mutex};

        let context_id: ExecutionContextId = 42;

        // Runtime Bridge
        let handle_manager = Arc::new(Mutex::new(HandleManager::new_default()));
        let capability_checker = Arc::new(NoopCapabilityManager);
        let bridge = RuntimeBridge::new(context_id, handle_manager, capability_checker);
        let intent = SideEffectIntent::AbdfRead {
            handle_id: 123,
            expected_segment_type: SegmentType::Input,
        };
        // This will fail because handle doesn't exist, but that's expected
        let result = bridge.execute_side_effect(intent, 0);
        assert!(result.is_err()); // Handle doesn't exist

        // Execution Sandbox
        let sandbox = ExecutionSandbox::new(context_id, 1024 * 1024);
        assert!(sandbox.check_operation("normal_operation").is_ok());
        assert!(sandbox.check_operation("kernel_access").is_err());

        // Side Effect Ordering
        let mut ordering = SideEffectOrdering::new(context_id);
        let declarations = vec![SideEffectDeclaration {
            opcode: 0x01,
            class: crate::types::SideEffectClass::Pure,
            required_capabilities: vec![],
        }];
        ordering.declare_side_effects(declarations);
        assert!(ordering.is_declared(0x01, crate::types::SideEffectClass::Pure));

        // Boundary Enforcer
        let enforcer = BoundaryEnforcer::new();
        assert!(enforcer.check_boundary("normal_operation").is_ok());
        assert!(enforcer.check_boundary("direct_abdf_access").is_err());
    }

    /// Test error code determinism (Requirement 15.5)
    #[test]
    fn error_codes_are_deterministic() {
        // Error codes should be stable and deterministic
        assert_eq!(ErrorCode::IsolationViolation as u16, 0x1001);
        assert_eq!(ErrorCode::BoundaryViolation as u16, 0x2001);
        assert_eq!(ErrorCode::CapabilityScopeViolation as u16, 0x3001);
        assert_eq!(ErrorCode::MemoryContractViolation as u16, 0x4001);
        assert_eq!(ErrorCode::ConstitutionalViolation as u16, 0x5001);
        assert_eq!(ErrorCode::SandboxEscape as u16, 0x6001);
        assert_eq!(ErrorCode::UndeclaredSideEffect as u16, 0x7001);
    }

    /// Test that NON_OVERRIDABLE rules are properly identified
    #[test]
    fn non_overridable_rules_identification() {
        assert!(ErrorCode::DeterminismGlobal.is_constitutional_violation());
        assert!(ErrorCode::MemoryContractViolation.is_constitutional_violation());
        assert!(ErrorCode::KernelSafetyCritical.is_constitutional_violation());
        assert!(ErrorCode::SecurityBoundaryViolation.is_constitutional_violation());

        // These should NOT be constitutional violations
        assert!(!ErrorCode::CapabilityDenied.is_constitutional_violation());
        assert!(!ErrorCode::AbdfHandleRevoked.is_constitutional_violation());
    }
}
