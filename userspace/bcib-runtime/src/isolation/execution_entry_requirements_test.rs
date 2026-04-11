/// Requirements validation tests for BCIB execution entry enforcement
///
/// These tests validate that the implementation correctly enforces Requirements 1.3 and 1.4:
/// - 1.3: THE BCIB execution SHALL be initiated ONLY via the approved submission path
/// - 1.4: THE BCIB runtime SHALL NOT be directly invocable via test helpers, debug hooks, or internal calls

#[cfg(test)]
mod tests {
    use crate::isolation::execution_entry_enforcer::{
        CallStackFrame, DirectInvocationType, ExecutionEntryEnforcer,
    };
    use crate::isolation::termination_aware_harness::{TerminationAwareHarness, TerminationReason};

    fn expect_security_termination(enforcer: &ExecutionEntryEnforcer, entry_point: &str) {
        let harness = TerminationAwareHarness::new();
        let event = harness
            .execute_expecting_termination(|| enforcer.validate_execution_entry(entry_point, 1))
            .expect("entry violation must terminate");

        harness
            .verify_termination_event(
                &event,
                TerminationReason::SecurityBoundaryViolation,
                "SECURITY.BOUNDARY.VIOLATION",
            )
            .expect("termination event must match execution-entry violation");
    }

    /// Test Requirement 1.3: BCIB execution SHALL be initiated ONLY via approved submission path
    #[test]
    fn requirement_1_3_only_approved_submission_path_allowed() {
        let enforcer = ExecutionEntryEnforcer::new();

        // Test that SYS_V2_SUBMIT_EXECUTION is the only approved entry point
        let approved_result = enforcer.validate_execution_entry("SYS_V2_SUBMIT_EXECUTION", 1);
        assert!(
            approved_result.is_ok(),
            "SYS_V2_SUBMIT_EXECUTION should be approved"
        );

        // Test that other syscalls are rejected
        let other_syscalls = [
            "SYS_READ",
            "SYS_WRITE",
            "SYS_OPEN",
            "SYS_CLOSE",
            "SYS_MMAP",
            "SYS_DIRECT_EXECUTION",
            "SYS_V1_SUBMIT_EXECUTION",
        ];

        for syscall in &other_syscalls {
            expect_security_termination(&enforcer, syscall);
        }
    }

    /// Test Requirement 1.4: BCIB runtime SHALL NOT be directly invocable via test helpers
    #[test]
    fn requirement_1_4_test_helpers_rejected() {
        let enforcer = ExecutionEntryEnforcer::new();

        let test_helper_patterns = [
            "test_bcib_execution",
            "bcib_test_runner",
            "helper_execute_bcib",
            "run_bcib_helper",
            "test_direct_execution",
            "helper_test_bcib",
        ];

        for pattern in &test_helper_patterns {
            expect_security_termination(&enforcer, pattern);
        }
    }

    /// Test Requirement 1.4: BCIB runtime SHALL NOT be directly invocable via debug hooks
    #[test]
    fn requirement_1_4_debug_hooks_rejected() {
        let enforcer = ExecutionEntryEnforcer::new();

        let debug_hook_patterns = [
            "debug_bcib_execution",
            "bcib_debug_runner",
            "hook_execute_bcib",
            "debug_direct_execution",
            "bcib_debug_hook",
            "hook_debug_bcib",
        ];

        for pattern in &debug_hook_patterns {
            expect_security_termination(&enforcer, pattern);
        }
    }

    /// Test Requirement 1.4: BCIB runtime SHALL NOT be directly invocable via internal calls
    #[test]
    fn requirement_1_4_internal_calls_rejected() {
        let enforcer = ExecutionEntryEnforcer::new();

        let internal_call_patterns = [
            "internal_bcib_execution",
            "bcib_internal_runner",
            "direct_execute_bcib",
            "internal_direct_execution",
            "bcib_internal_call",
            "direct_internal_bcib",
        ];

        for pattern in &internal_call_patterns {
            expect_security_termination(&enforcer, pattern);
        }
    }

    /// Test that call stack validation detects forbidden contexts
    #[test]
    fn call_stack_validation_detects_forbidden_contexts() {
        let enforcer = ExecutionEntryEnforcer::new();

        // Test detection of test context in call stack
        let test_frame = CallStackFrame {
            function_name: "run_unit_test".to_string(),
            module_path: "tests::bcib_tests".to_string(),
            is_test_context: true,
            is_debug_context: false,
            is_internal_call: false,
        };

        enforcer.push_call_frame(test_frame);

        // Even with a non-matching entry point name, call stack should be checked
        expect_security_termination(&enforcer, "custom_entry");

        enforcer.clear_call_stack();

        // Test detection of debug context in call stack
        let debug_frame = CallStackFrame {
            function_name: "debug_execution".to_string(),
            module_path: "debug::bcib_debug".to_string(),
            is_test_context: false,
            is_debug_context: true,
            is_internal_call: false,
        };

        enforcer.push_call_frame(debug_frame);

        expect_security_termination(&enforcer, "custom_entry");

        enforcer.clear_call_stack();

        // Test detection of internal call context in call stack
        let internal_frame = CallStackFrame {
            function_name: "internal_execution".to_string(),
            module_path: "internal::bcib_internal".to_string(),
            is_test_context: false,
            is_debug_context: false,
            is_internal_call: true,
        };

        enforcer.push_call_frame(internal_frame);

        expect_security_termination(&enforcer, "custom_entry");

        enforcer.clear_call_stack();
    }

    /// Test that violation types are correctly classified
    #[test]
    fn violation_types_correctly_classified() {
        let enforcer = ExecutionEntryEnforcer::new();

        // Test test helper classification
        assert_eq!(
            enforcer.classify_violation_type("test_"),
            DirectInvocationType::TestHelper
        );
        assert_eq!(
            enforcer.classify_violation_type("helper_"),
            DirectInvocationType::TestHelper
        );

        // Test debug hook classification
        assert_eq!(
            enforcer.classify_violation_type("debug_"),
            DirectInvocationType::DebugHook
        );
        assert_eq!(
            enforcer.classify_violation_type("hook_"),
            DirectInvocationType::DebugHook
        );

        // Test internal call classification
        assert_eq!(
            enforcer.classify_violation_type("internal_"),
            DirectInvocationType::InternalCall
        );
        assert_eq!(
            enforcer.classify_violation_type("direct_"),
            DirectInvocationType::InternalCall
        );

        // Test unknown classification
        assert_eq!(
            enforcer.classify_violation_type("unknown_pattern"),
            DirectInvocationType::Unknown
        );
    }

    /// Test that bypass attempts are detected and rejected
    #[test]
    fn bypass_attempts_detected_and_rejected() {
        let enforcer = ExecutionEntryEnforcer::new();

        let bypass_attempts = [
            "custom_execution_path",
            "alternative_submit",
            "backdoor_execution",
            "bypass_enforcement",
            "direct_kernel_call",
            "unauthorized_entry",
        ];

        for attempt in &bypass_attempts {
            expect_security_termination(&enforcer, attempt);
        }
    }

    /// Test that enforcement cannot be bypassed in production (security compliance)
    #[test]
    fn enforcement_cannot_be_bypassed_in_production() {
        // This test verifies that all bypass mechanisms have been removed
        // Constitutional compliance: SECURITY.BOUNDARY.VIOLATION enforcement is mandatory

        let enforcer = ExecutionEntryEnforcer::new();

        // Enforcement is always enabled - no bypass allowed
        assert!(enforcer.is_enforcement_enabled());

        // All forbidden patterns should cause process termination
        let forbidden_patterns = [
            "test_bcib_execution",
            "debug_run_bcib",
            "internal_execute_bcib",
            "custom_execution_path",
            "bypass_enforcement",
        ];

        for pattern in &forbidden_patterns {
            expect_security_termination(&enforcer, pattern);
        }
    }

    /// Test that fail-closed behavior is implemented
    #[test]
    fn fail_closed_behavior_implemented() {
        let enforcer = ExecutionEntryEnforcer::new();

        // Test that violations result in deterministic errors, not undefined behavior
        let violation_patterns = [
            "test_violation",
            "debug_violation",
            "internal_violation",
            "bypass_violation",
        ];

        for pattern in &violation_patterns {
            expect_security_termination(&enforcer, pattern);
        }
    }
}
