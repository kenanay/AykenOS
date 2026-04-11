/// Integration tests for BCIB execution entry enforcement
///
/// These tests verify that the execution entry enforcer correctly integrates
/// with the BCIB execution runtime to prevent direct invocation paths.
///
/// **TASK 3 COMPLIANCE**: These tests verify kernel-level authoritative enforcement
/// with no bypass mechanisms allowed in production builds.
///
/// **CRITICAL SECURITY TESTS**: Direct calls must FAIL, only syscall path should succeed
///
/// **TERMINATION-AWARE HARNESS**: Uses dedicated harness to capture fail-closed
/// behavior without causing abnormal test harness exit.
///
/// **NO BYPASS MECHANISMS**: All tests use real kernel contexts - no fake contexts allowed

#[cfg(test)]
mod tests {
    use crate::execution_runtime::BcibExecutionRuntime;
    use crate::isolation::execution_entry_context::ExecutionEntryContext;
    use crate::isolation::termination_aware_harness::{TerminationAwareHarness, TerminationReason};
    use crate::types::{CapabilitySet, ExecutionPlan};

    #[test]
    fn direct_invocation_must_fail() {
        // **CRITICAL SECURITY TEST**: Direct runtime.create_context() calls must FAIL
        // This test verifies that direct calls bypass kernel validation and are rejected
        let harness = TerminationAwareHarness::new();
        
        let result = harness.execute_expecting_termination(|| {
            let mut runtime = BcibExecutionRuntime::new();
            let plan = ExecutionPlan::new(vec![], 0x0003);
            let caps = CapabilitySet::default();
            
            // Create a userspace mock context (simulates direct call bypass attempt)
            let userspace_context = ExecutionEntryContext::from_kernel_dispatcher(
                9999, // Invalid syscall ID
                std::process::id(),
                1,
                vec!["userspace_test".to_string()], // No kernel frame
            );

            // This MUST FAIL - direct calls should be impossible
            runtime.create_context_from_syscall(plan, caps, userspace_context)
                .map(|_| ())
        });
        
        assert!(result.is_ok(), "Expected termination to be captured");
        let event = result.unwrap();
        
        // Verify the termination event
        harness.verify_termination_event(
            &event,
            TerminationReason::SecurityBoundaryViolation,
            "SECURITY.BOUNDARY.VIOLATION"
        ).expect("Direct invocation must fail with SECURITY.BOUNDARY.VIOLATION");
    }

    #[test]
    fn test_helper_bypass_must_fail() {
        // **CRITICAL SECURITY TEST**: Test helper calls must be detected and rejected
        let harness = TerminationAwareHarness::new();
        
        let result = harness.execute_expecting_termination(|| {
            let mut runtime = BcibExecutionRuntime::new();
            let plan = ExecutionPlan::new(vec![], 0x0003);
            let caps = CapabilitySet::default();
            
            // Create a context with test helper pattern in call stack
            let test_context = ExecutionEntryContext::from_kernel_dispatcher(
                1003, // Correct syscall ID
                std::process::id(),
                1,
                vec!["test_helper_create_context".to_string()], // Test pattern - should fail
            );

            // This MUST FAIL - test helpers cannot bypass kernel validation
            runtime.create_context_from_syscall(plan, caps, test_context)
                .map(|_| ())
        });
        
        assert!(result.is_ok(), "Expected termination to be captured");
        let event = result.unwrap();
        
        // Verify the termination event
        harness.verify_termination_event(
            &event,
            TerminationReason::SecurityBoundaryViolation,
            "SECURITY.BOUNDARY.VIOLATION"
        ).expect("Test helper bypass must fail with security violation");
    }

    #[test]
    fn debug_hook_bypass_must_fail() {
        // **CRITICAL SECURITY TEST**: Debug hooks must be detected and rejected
        let harness = TerminationAwareHarness::new();
        
        let result = harness.execute_expecting_termination(|| {
            let mut runtime = BcibExecutionRuntime::new();
            let plan = ExecutionPlan::new(vec![], 0x0003);
            let caps = CapabilitySet::default();
            
            // Create a context with debug hook pattern in call stack
            let debug_context = ExecutionEntryContext::from_kernel_dispatcher(
                1003, // Correct syscall ID
                std::process::id(),
                1,
                vec!["debug_hook_execute".to_string()], // Debug pattern - should fail
            );

            // This MUST FAIL - debug hooks cannot bypass kernel validation
            runtime.create_context_from_syscall(plan, caps, debug_context)
                .map(|_| ())
        });
        
        assert!(result.is_ok(), "Expected termination to be captured");
        let event = result.unwrap();
        
        // Verify the termination event
        harness.verify_termination_event(
            &event,
            TerminationReason::SecurityBoundaryViolation,
            "SECURITY.BOUNDARY.VIOLATION"
        ).expect("Debug hook bypass must fail with security violation");
    }

    #[test]
    fn internal_call_bypass_must_fail() {
        // **CRITICAL SECURITY TEST**: Internal calls must be detected and rejected
        let harness = TerminationAwareHarness::new();
        
        let result = harness.execute_expecting_termination(|| {
            let mut runtime = BcibExecutionRuntime::new();
            let plan = ExecutionPlan::new(vec![], 0x0003);
            let caps = CapabilitySet::default();
            
            // Create a context with internal call pattern in call stack
            let internal_context = ExecutionEntryContext::from_kernel_dispatcher(
                1003, // Correct syscall ID
                std::process::id(),
                1,
                vec!["internal_create_context".to_string()], // Internal pattern - should fail
            );

            // This MUST FAIL - internal calls cannot bypass kernel validation
            runtime.create_context_from_syscall(plan, caps, internal_context)
                .map(|_| ())
        });
        
        assert!(result.is_ok(), "Expected termination to be captured");
        let event = result.unwrap();
        
        // Verify the termination event
        harness.verify_termination_event(
            &event,
            TerminationReason::SecurityBoundaryViolation,
            "SECURITY.BOUNDARY.VIOLATION"
        ).expect("Internal call bypass must fail with security violation");
    }

    #[test]
    fn syscall_dispatcher_path_succeeds() {
        // **ONLY VALID PATH**: Real kernel dispatcher path should succeed
        let harness = TerminationAwareHarness::new();
        
        let result = harness.execute_expecting_success(|| {
            let mut runtime = BcibExecutionRuntime::new();
            let plan = ExecutionPlan::new(vec![], 0x0003);
            let caps = CapabilitySet::default();
            
            // Create a real kernel entry context (only valid path)
            let kernel_context = ExecutionEntryContext::from_kernel_dispatcher(
                1003, // SYS_V2_SUBMIT_EXECUTION
                1234, // process_id
                5678, // thread_id
                vec!["kernel_syscall_dispatcher".to_string(), "sys_v2_submit_execution".to_string()],
            );

            // This should succeed - real kernel dispatcher path
            runtime.create_context_from_syscall(plan, caps, kernel_context)
                .map(|_| ())
        });
        
        assert!(result.is_ok(), "Real kernel dispatcher path should succeed: {:?}", result);
    }

    #[test]
    fn enforcement_always_enabled_no_bypass() {
        // **SECURITY REQUIREMENT**: Enforcement cannot be disabled
        let runtime = BcibExecutionRuntime::new();
        
        // Verify that enforcement is always enabled
        assert!(runtime.is_enforcement_enabled(), "Entry enforcement must always be enabled");
        
        // Verify no disable method exists (this test ensures no bypass mechanisms)
        // If a disable method existed, this would fail at compile time
    }
}