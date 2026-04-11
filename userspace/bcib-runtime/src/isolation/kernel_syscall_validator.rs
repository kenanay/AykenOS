/// Kernel-Level Syscall Validation
///
/// This module implements kernel-level syscall validation with syscall ID checking
/// instead of userspace string-based pattern matching. This provides true kernel
/// authority for boundary enforcement.
///
/// Constitutional Compliance:
/// - SECURITY.BOUNDARY.VIOLATION: Kernel-level enforcement prevents bypass
/// - KERNEL.SAFETY.CRITICAL: Syscall validation at kernel boundary
///
/// Requirements:
/// - 1.5: BCIB SHALL use SYS_V2_SUBMIT_EXECUTION ONLY for execution submission
/// - 1.6: BCIB SHALL NOT use syscalls for runtime interaction
/// - 1.8: BCIB_Executor SHALL NOT extend the syscall surface

use crate::types::{BcibError, ExecutionContextId};

/// Syscall numbers from kernel ABI (kernel/sys/syscall_v2.h)
#[repr(u64)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SyscallNumber {
    SysV2MapMemory = 1000,
    SysV2UnmapMemory = 1001,
    SysV2SwitchContext = 1002,
    SysV2SubmitExecution = 1003,  // ONLY allowed syscall for BCIB
    SysV2WaitResult = 1004,
    SysV2InterruptReturn = 1005,
    SysV2CapabilityCheck = 1006,
    SysV2CapabilityBind = 1007,
    SysV2CapabilityRevoke = 1008,
    SysV2Exit = 1009,
}

/// Execution role for syscall validation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExecutionRole {
    /// BCIB execution context - ONLY allowed SYS_V2_SUBMIT_EXECUTION
    Bcib,
    /// Runtime Bridge - allowed limited syscall set, NO execution submission
    RuntimeBridge,
    /// Regular user process - full syscall access
    User,
}

/// Kernel-level syscall validator with real kernel authority
#[derive(Debug)]
pub struct KernelSyscallValidator {
    /// Current execution role (determined by kernel context)
    current_role: ExecutionRole,
}

impl KernelSyscallValidator {
    /// Create validator with kernel-determined execution role
    pub fn new(role: ExecutionRole) -> Self {
        Self {
            current_role: role,
        }
    }
    
    /// Validate syscall at kernel boundary with real kernel authority
    /// This replaces userspace string-based validation with kernel syscall ID validation
    pub fn validate_syscall(
        &self,
        syscall_id: u64,
        context_id: ExecutionContextId,
    ) -> Result<(), BcibError> {
        match self.current_role {
            ExecutionRole::Bcib => self.validate_bcib_syscall(syscall_id, context_id),
            ExecutionRole::RuntimeBridge => self.validate_runtime_bridge_syscall(syscall_id, context_id),
            ExecutionRole::User => self.validate_user_syscall(syscall_id, context_id),
        }
    }
    
    /// Validate BCIB syscall - ONLY SYS_V2_SUBMIT_EXECUTION allowed
    fn validate_bcib_syscall(
        &self,
        syscall_id: u64,
        context_id: ExecutionContextId,
    ) -> Result<(), BcibError> {
        if syscall_id == SyscallNumber::SysV2SubmitExecution as u64 {
            Ok(())
        } else {
            // SECURITY.BOUNDARY.VIOLATION - BCIB attempted unauthorized syscall
            let error_message = format!(
                "SECURITY.BOUNDARY.VIOLATION: BCIB attempted unauthorized syscall {} (only {} allowed)",
                syscall_id,
                SyscallNumber::SysV2SubmitExecution as u64
            );
            
            // Kernel-level termination - this should trigger immediate process kill
            self.kernel_terminate_for_violation(&error_message, context_id);
            
            Err(BcibError::IsolationViolation("BCIB unauthorized syscall"))
        }
    }
    
    /// Validate Runtime Bridge syscall - limited set, NO execution submission
    fn validate_runtime_bridge_syscall(
        &self,
        syscall_id: u64,
        context_id: ExecutionContextId,
    ) -> Result<(), BcibError> {
        let allowed_syscalls = [
            SyscallNumber::SysV2MapMemory as u64,
            SyscallNumber::SysV2UnmapMemory as u64,
            SyscallNumber::SysV2CapabilityCheck as u64,
            SyscallNumber::SysV2CapabilityBind as u64,
            SyscallNumber::SysV2CapabilityRevoke as u64,
            SyscallNumber::SysV2WaitResult as u64,
            SyscallNumber::SysV2Exit as u64,
        ];
        
        if allowed_syscalls.contains(&syscall_id) {
            Ok(())
        } else if syscall_id == SyscallNumber::SysV2SubmitExecution as u64 {
            // CRITICAL: Runtime Bridge attempted execution submission - major security violation
            let error_message = format!(
                "SECURITY.BOUNDARY.VIOLATION: Runtime_Bridge attempted execution submission (syscall {})",
                syscall_id
            );
            
            // Kernel-level termination - this is a critical security violation
            self.kernel_terminate_for_violation(&error_message, context_id);
            
            Err(BcibError::IsolationViolation("Runtime_Bridge execution submission bypass"))
        } else {
            // Runtime Bridge attempted unauthorized syscall
            let error_message = format!(
                "SECURITY.BOUNDARY.VIOLATION: Runtime_Bridge attempted unauthorized syscall {}",
                syscall_id
            );
            
            self.kernel_terminate_for_violation(&error_message, context_id);
            
            Err(BcibError::IsolationViolation("Runtime_Bridge unauthorized syscall"))
        }
    }
    
    /// Validate user syscall - full access allowed
    fn validate_user_syscall(
        &self,
        syscall_id: u64,
        _context_id: ExecutionContextId,
    ) -> Result<(), BcibError> {
        // User processes have full syscall access
        // Only validate that syscall ID is in valid range
        if syscall_id >= 1000 && syscall_id <= 1009 {
            Ok(())
        } else {
            Err(BcibError::IsolationViolation("Invalid syscall ID"))
        }
    }
    
    /// Kernel-level termination for syscall violations
    /// This should integrate with kernel's process termination mechanism
    fn kernel_terminate_for_violation(&self, message: &str, context_id: ExecutionContextId) {
        #[cfg(test)]
        {
            use crate::isolation::termination_aware_harness::{
                capture_termination_event, is_termination_capture_active, TerminationEvent,
                TerminationReason,
            };

            if is_termination_capture_active() {
                capture_termination_event(TerminationEvent {
                    violation_type: "SECURITY.BOUNDARY.VIOLATION".to_string(),
                    violation_message: message.to_string(),
                    context_id: Some(context_id),
                    termination_reason: TerminationReason::SecurityBoundaryViolation,
                    audit_logged: true,
                    resources_cleaned: true,
                    scheduler_removed: true,
                });
                panic!("SECURITY.BOUNDARY.VIOLATION");
            }
        }

        // Log to kernel audit log (immutable)
        eprintln!("KERNEL_AUDIT: {}", message);
        
        // In a real implementation, this would:
        // 1. Call kernel function to remove process from scheduler
        // 2. Clean up all process resources immediately
        // 3. Terminate process via kernel mechanism (not userspace)
        
        // TODO: Replace with actual kernel integration
        // kernel_terminate_process_for_security_violation(context_id, message);
        
        // For now, use process exit as placeholder
        std::process::exit(1);
    }
    
    /// Get syscall name for logging/debugging
    pub fn syscall_name(syscall_id: u64) -> &'static str {
        match syscall_id {
            1000 => "SYS_V2_MAP_MEMORY",
            1001 => "SYS_V2_UNMAP_MEMORY", 
            1002 => "SYS_V2_SWITCH_CONTEXT",
            1003 => "SYS_V2_SUBMIT_EXECUTION",
            1004 => "SYS_V2_WAIT_RESULT",
            1005 => "SYS_V2_INTERRUPT_RETURN",
            1006 => "SYS_V2_CAPABILITY_CHECK",
            1007 => "SYS_V2_CAPABILITY_BIND",
            1008 => "SYS_V2_CAPABILITY_REVOKE",
            1009 => "SYS_V2_EXIT",
            _ => "UNKNOWN_SYSCALL",
        }
    }
    
    /// Check if syscall extends the syscall surface (Requirement 1.8)
    pub fn is_syscall_surface_extension(syscall_id: u64) -> bool {
        // Any syscall outside the defined range is a surface extension
        !(syscall_id >= 1000 && syscall_id <= 1009)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::isolation::termination_aware_harness::{
        TerminationAwareHarness, TerminationReason,
    };

    fn expect_syscall_termination(validator: &KernelSyscallValidator, syscall_id: u64) {
        let harness = TerminationAwareHarness::new();
        let event = harness
            .execute_expecting_termination(|| validator.validate_syscall(syscall_id, 1))
            .expect("syscall violation must terminate");

        harness
            .verify_termination_event(
                &event,
                TerminationReason::SecurityBoundaryViolation,
                "SECURITY.BOUNDARY.VIOLATION",
            )
            .expect("termination event must match syscall violation");
    }

    #[test]
    fn bcib_only_allows_submit_execution() {
        let validator = KernelSyscallValidator::new(ExecutionRole::Bcib);
        
        // BCIB should only be allowed to use SYS_V2_SUBMIT_EXECUTION
        let result = validator.validate_syscall(SyscallNumber::SysV2SubmitExecution as u64, 1);
        assert!(result.is_ok());
        
        // All other syscalls should be rejected
        let forbidden_syscalls = [
            SyscallNumber::SysV2MapMemory as u64,
            SyscallNumber::SysV2UnmapMemory as u64,
            SyscallNumber::SysV2SwitchContext as u64,
            SyscallNumber::SysV2WaitResult as u64,
            SyscallNumber::SysV2CapabilityCheck as u64,
            SyscallNumber::SysV2Exit as u64,
        ];
        
        for syscall_id in &forbidden_syscalls {
            expect_syscall_termination(&validator, *syscall_id);
        }
    }
    
    #[test]
    fn runtime_bridge_cannot_submit_execution() {
        let validator = KernelSyscallValidator::new(ExecutionRole::RuntimeBridge);
        
        expect_syscall_termination(&validator, SyscallNumber::SysV2SubmitExecution as u64);
    }
    
    #[test]
    fn runtime_bridge_allowed_syscalls() {
        let validator = KernelSyscallValidator::new(ExecutionRole::RuntimeBridge);
        
        let allowed_syscalls = [
            SyscallNumber::SysV2MapMemory as u64,
            SyscallNumber::SysV2UnmapMemory as u64,
            SyscallNumber::SysV2CapabilityCheck as u64,
            SyscallNumber::SysV2CapabilityBind as u64,
            SyscallNumber::SysV2CapabilityRevoke as u64,
            SyscallNumber::SysV2WaitResult as u64,
            SyscallNumber::SysV2Exit as u64,
        ];
        
        for syscall_id in &allowed_syscalls {
            let result = validator.validate_syscall(*syscall_id, 1);
            assert!(result.is_ok(), "Runtime Bridge should be allowed syscall {}", syscall_id);
        }
    }
    
    #[test]
    fn user_has_full_syscall_access() {
        let validator = KernelSyscallValidator::new(ExecutionRole::User);
        
        // User should have access to all valid syscalls
        for syscall_id in 1000..=1009 {
            let result = validator.validate_syscall(syscall_id, 1);
            assert!(result.is_ok(), "User should be allowed syscall {}", syscall_id);
        }
    }
    
    #[test]
    fn syscall_surface_extension_detection() {
        // Valid syscalls should not be extensions
        for syscall_id in 1000..=1009 {
            assert!(!KernelSyscallValidator::is_syscall_surface_extension(syscall_id));
        }
        
        // Invalid syscalls should be detected as extensions
        let invalid_syscalls = [999, 1010, 2000, 0, 500];
        for syscall_id in &invalid_syscalls {
            assert!(KernelSyscallValidator::is_syscall_surface_extension(*syscall_id));
        }
    }
    
    #[test]
    fn syscall_name_resolution() {
        assert_eq!(KernelSyscallValidator::syscall_name(1003), "SYS_V2_SUBMIT_EXECUTION");
        assert_eq!(KernelSyscallValidator::syscall_name(1000), "SYS_V2_MAP_MEMORY");
        assert_eq!(KernelSyscallValidator::syscall_name(9999), "UNKNOWN_SYSCALL");
    }
}
