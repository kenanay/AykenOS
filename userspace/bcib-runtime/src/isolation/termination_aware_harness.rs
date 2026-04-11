use crate::isolation::execution_entry_context::ExecutionEntryContext;
use crate::types::{BcibError, ExecutionContextId};
/// Termination-Aware Test Harness for BCIB Execution Entry Enforcement
///
/// This module provides a dedicated test harness that can capture and verify
/// fail-closed termination behavior without causing abnormal test harness exit.
///
/// **TASK 3 COMPLETION REQUIREMENT**: This harness addresses the completion blocker:
/// "Direct-invocation fail-closed behavior must be verified by a dedicated
/// termination-aware harness or QEMU/kernel evidence gate"
use std::sync::{Arc, Mutex};

/// Captured termination event for verification
#[derive(Debug, Clone)]
pub struct TerminationEvent {
    pub violation_type: String,
    pub violation_message: String,
    pub context_id: Option<ExecutionContextId>,
    pub termination_reason: TerminationReason,
    pub audit_logged: bool,
    pub resources_cleaned: bool,
    pub scheduler_removed: bool,
}

/// Reason for termination
#[derive(Debug, Clone, PartialEq)]
pub enum TerminationReason {
    SecurityBoundaryViolation,
    KernelSafetyCritical,
    IsolationViolation,
    CapabilityViolation,
    UnauthorizedEntry,
}

/// Global termination capture registry for test harness
/// Uses thread-local storage to avoid race conditions between concurrent tests
thread_local! {
    static TERMINATION_REGISTRY: std::cell::RefCell<Option<Arc<Mutex<Vec<TerminationEvent>>>>> = std::cell::RefCell::new(None);
}

/// Termination-aware test harness that captures fail-closed behavior
pub struct TerminationAwareHarness {
    captured_events: Arc<Mutex<Vec<TerminationEvent>>>,
    /// Test mode flag for future test-specific behavior
    /// TODO(Task 10): Wire up test_mode for conditional test behavior
    #[allow(dead_code)]
    test_mode: bool,
}

impl TerminationAwareHarness {
    /// Create a new termination-aware harness for testing
    pub fn new() -> Self {
        let events = Arc::new(Mutex::new(Vec::new()));

        // Register this harness as the global termination capture
        TERMINATION_REGISTRY.with(|registry| {
            *registry.borrow_mut() = Some(events.clone());
        });

        Self {
            captured_events: events,
            test_mode: true,
        }
    }

    /// Execute a test that expects termination and capture the result
    pub fn execute_expecting_termination<F>(&self, test_fn: F) -> Result<TerminationEvent, String>
    where
        F: FnOnce() -> Result<(), BcibError> + std::panic::UnwindSafe,
    {
        // Clear any previous events
        self.captured_events.lock().unwrap().clear();

        // Execute the test function and catch panics
        let _result = std::panic::catch_unwind(|| test_fn());

        // Check if termination was captured (regardless of panic)
        let events = self.captured_events.lock().unwrap();
        if events.is_empty() {
            return Err("Expected termination but none was captured".to_string());
        }

        if events.len() > 1 {
            return Err(format!(
                "Expected single termination but captured {}",
                events.len()
            ));
        }

        Ok(events[0].clone())
    }

    /// Verify that no termination occurred during test execution
    pub fn execute_expecting_success<F>(&self, test_fn: F) -> Result<(), String>
    where
        F: FnOnce() -> Result<(), BcibError> + std::panic::UnwindSafe,
    {
        // Clear any previous events
        self.captured_events.lock().unwrap().clear();

        // Execute the test function
        let result = std::panic::catch_unwind(|| test_fn());

        // Check that no termination was captured
        let events = self.captured_events.lock().unwrap();
        if !events.is_empty() {
            return Err(format!("Unexpected termination captured: {:?}", events[0]));
        }

        // Check that the function succeeded
        match result {
            Ok(Ok(())) => Ok(()),
            Ok(Err(e)) => Err(format!("Function failed with error: {:?}", e)),
            Err(_) => Err("Function panicked".to_string()),
        }
    }

    /// Get all captured termination events
    pub fn get_captured_events(&self) -> Vec<TerminationEvent> {
        self.captured_events.lock().unwrap().clone()
    }

    /// Clear captured events
    pub fn clear_events(&self) {
        self.captured_events.lock().unwrap().clear();
    }

    /// Verify termination event matches expected criteria
    pub fn verify_termination_event(
        &self,
        event: &TerminationEvent,
        expected_reason: TerminationReason,
        expected_violation_pattern: &str,
    ) -> Result<(), String> {
        if event.termination_reason != expected_reason {
            return Err(format!(
                "Expected termination reason {:?}, got {:?}",
                expected_reason, event.termination_reason
            ));
        }

        if !event.violation_message.contains(expected_violation_pattern) {
            return Err(format!(
                "Expected violation message to contain '{}', got '{}'",
                expected_violation_pattern, event.violation_message
            ));
        }

        if !event.audit_logged {
            return Err("Expected audit logging to be completed".to_string());
        }

        if !event.resources_cleaned {
            return Err("Expected resources to be cleaned up".to_string());
        }

        if !event.scheduler_removed {
            return Err("Expected process to be removed from scheduler".to_string());
        }

        Ok(())
    }
}

impl Drop for TerminationAwareHarness {
    fn drop(&mut self) {
        // Unregister the global termination capture
        TERMINATION_REGISTRY.with(|registry| {
            *registry.borrow_mut() = None;
        });
    }
}

/// Capture a termination event for the test harness
/// This function is called by the fail-closed termination system during tests
pub fn capture_termination_event(event: TerminationEvent) {
    TERMINATION_REGISTRY.with(|registry| {
        if let Some(ref events) = *registry.borrow() {
            events.lock().unwrap().push(event);
        }
    });
}

/// Check if we're currently in test mode with termination capture
pub fn is_termination_capture_active() -> bool {
    TERMINATION_REGISTRY.with(|registry| registry.borrow().is_some())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::execution_runtime::BcibExecutionRuntime;
    use crate::types::{CapabilitySet, ExecutionPlan};

    #[test]
    fn harness_captures_security_boundary_violation() {
        let harness = TerminationAwareHarness::new();

        let result = harness.execute_expecting_termination(|| {
            let mut runtime = BcibExecutionRuntime::new();
            let plan = ExecutionPlan::new(vec![], 0x0003);
            let caps = CapabilitySet::default();

            // Create an invalid entry context that should trigger termination
            // Using invalid syscall ID to simulate bypass attempt
            let invalid_context = ExecutionEntryContext::from_kernel_dispatcher(
                9999, // Invalid syscall ID - should trigger termination
                std::process::id(),
                1,
                vec!["test_bypass_attempt".to_string()], // No kernel frame
            );

            // This should trigger termination capture
            runtime
                .create_context_from_syscall(plan, caps, invalid_context)
                .map(|_| ())
        });

        assert!(result.is_ok(), "Expected termination to be captured");
        let event = result.unwrap();

        // Verify the termination event
        harness
            .verify_termination_event(
                &event,
                TerminationReason::SecurityBoundaryViolation,
                "SECURITY.BOUNDARY.VIOLATION",
            )
            .expect("Termination event should match expected criteria");
    }

    #[test]
    fn harness_detects_unexpected_success() {
        let harness = TerminationAwareHarness::new();

        let result = harness.execute_expecting_termination(|| {
            // This function succeeds without triggering termination
            Ok(())
        });

        assert!(result.is_err(), "Should detect when no termination occurs");
        assert!(result
            .unwrap_err()
            .contains("Expected termination but none was captured"));
    }

    #[test]
    fn harness_verifies_successful_execution() {
        let harness = TerminationAwareHarness::new();

        let result = harness.execute_expecting_success(|| {
            let mut runtime = BcibExecutionRuntime::new();
            let plan = ExecutionPlan::new(vec![], 0x0003);
            let caps = CapabilitySet::default();

            // Create a valid kernel entry context
            let kernel_context = ExecutionEntryContext::from_kernel_dispatcher(
                1003, // SYS_V2_SUBMIT_EXECUTION
                1234, // process_id
                5678, // thread_id
                vec!["kernel_syscall_dispatcher".to_string()],
            );

            // This should succeed without termination
            runtime
                .create_context_from_syscall(plan, caps, kernel_context)
                .map(|_| ())
        });

        assert!(
            result.is_ok(),
            "Valid kernel entry should succeed: {:?}",
            result
        );
    }
}
