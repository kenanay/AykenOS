/// Constitutional Rule Enforcement Framework
///
/// This module implements enforcement for NON_OVERRIDABLE constitutional rules
/// that cannot be bypassed by any mechanism. These rules are immutable and
/// represent the core security and determinism guarantees of the system.
///
/// ## NON_OVERRIDABLE Rules Enforced
///
/// - `DETERMINISM.GLOBAL` — global state mutations
/// - `MEMORY.CONTRACT.VIOLATION` — memory safety violations  
/// - `KERNEL.SAFETY.CRITICAL` — critical kernel safety violations
/// - `SECURITY.BOUNDARY.VIOLATION` — Ring3 accessing Ring0 directly
///
/// All violations result in immediate fail-closed termination with no recovery.
use crate::isolation::error_taxonomy::{ErrorCode, IsolationError};
use crate::types::ExecutionContextId;
use std::fmt;

/// NON_OVERRIDABLE constitutional rules that cannot be bypassed
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConstitutionalRule {
    /// Global state mutations are prohibited for determinism
    DeterminismGlobal,
    /// Memory safety violations are prohibited
    MemoryContractViolation,
    /// Critical kernel safety must be maintained
    KernelSafetyCritical,
    /// Ring3 cannot access Ring0 directly
    SecurityBoundaryViolation,
}

impl ConstitutionalRule {
    /// Get the error code for violations of this rule
    pub fn error_code(self) -> ErrorCode {
        match self {
            ConstitutionalRule::DeterminismGlobal => ErrorCode::DeterminismGlobal,
            ConstitutionalRule::MemoryContractViolation => ErrorCode::MemoryContractViolation,
            ConstitutionalRule::KernelSafetyCritical => ErrorCode::KernelSafetyCritical,
            ConstitutionalRule::SecurityBoundaryViolation => ErrorCode::SecurityBoundaryViolation,
        }
    }

    /// Get the rule name as defined in the constitutional framework
    pub fn rule_name(self) -> &'static str {
        match self {
            ConstitutionalRule::DeterminismGlobal => "DETERMINISM.GLOBAL",
            ConstitutionalRule::MemoryContractViolation => "MEMORY.CONTRACT.VIOLATION",
            ConstitutionalRule::KernelSafetyCritical => "KERNEL.SAFETY.CRITICAL",
            ConstitutionalRule::SecurityBoundaryViolation => "SECURITY.BOUNDARY.VIOLATION",
        }
    }

    /// Get a description of what this rule prohibits
    pub fn description(self) -> &'static str {
        match self {
            ConstitutionalRule::DeterminismGlobal =>
                "Prohibits global state mutations that could introduce non-determinism",
            ConstitutionalRule::MemoryContractViolation =>
                "Prohibits memory safety violations including bounds violations and raw pointer access",
            ConstitutionalRule::KernelSafetyCritical =>
                "Prohibits operations that could compromise critical kernel safety",
            ConstitutionalRule::SecurityBoundaryViolation =>
                "Prohibits Ring3 code from accessing Ring0 directly, bypassing security boundaries",
        }
    }
}

impl fmt::Display for ConstitutionalRule {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.rule_name())
    }
}

/// A specific violation of a constitutional rule
#[derive(Debug, Clone)]
pub struct RuleViolation {
    /// The rule that was violated
    pub rule: ConstitutionalRule,
    /// Description of the specific violation
    pub violation_description: String,
    /// Context where the violation occurred (if applicable)
    pub context_id: Option<ExecutionContextId>,
    /// Additional details about the violation
    pub details: Option<String>,
}

impl RuleViolation {
    /// Create a new rule violation
    pub fn new(
        rule: ConstitutionalRule,
        violation_description: impl Into<String>,
        context_id: Option<ExecutionContextId>,
    ) -> Self {
        Self {
            rule,
            violation_description: violation_description.into(),
            context_id,
            details: None,
        }
    }

    /// Add additional details to the violation
    pub fn with_details(mut self, details: impl Into<String>) -> Self {
        self.details = Some(details.into());
        self
    }

    /// Convert to an IsolationError for fail-closed termination
    pub fn to_isolation_error(&self) -> IsolationError {
        let message = format!(
            "Constitutional rule {} violated: {}",
            self.rule.rule_name(),
            self.violation_description
        );

        match &self.details {
            Some(details) => IsolationError::with_details(
                self.rule.error_code(),
                message,
                self.context_id,
                details.clone(),
            ),
            None => IsolationError::new(self.rule.error_code(), message, self.context_id),
        }
    }
}

impl fmt::Display for RuleViolation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.rule, self.violation_description)?;

        if let Some(ctx_id) = self.context_id {
            write!(f, " [context: {}]", ctx_id)?;
        }

        if let Some(ref details) = self.details {
            write!(f, " - {}", details)?;
        }

        Ok(())
    }
}

/// Constitutional rule enforcer that validates operations against NON_OVERRIDABLE rules
pub struct ConstitutionalEnforcer {
    /// Whether enforcement is active (should always be true in production)
    enforcement_active: bool,
}

impl ConstitutionalEnforcer {
    /// Create a new constitutional enforcer with active enforcement
    pub fn new() -> Self {
        Self {
            enforcement_active: true,
        }
    }

    /// Create an enforcer with enforcement disabled (TESTING ONLY)
    ///
    /// # Safety
    ///
    /// This should NEVER be used in production code. It exists only for
    /// unit testing scenarios where constitutional violations need to be
    /// simulated without triggering fail-closed termination.
    #[cfg(test)]
    pub fn disabled_for_testing() -> Self {
        Self {
            enforcement_active: false,
        }
    }

    /// Check if a global state mutation would violate DETERMINISM.GLOBAL
    pub fn check_determinism_global(
        &self,
        operation: &str,
        context_id: Option<ExecutionContextId>,
    ) -> Result<(), RuleViolation> {
        if !self.enforcement_active {
            return Ok(());
        }

        // For now, we implement a basic check. In a full implementation,
        // this would analyze the specific operation for global state mutations.
        if operation.contains("global_state") || operation.contains("static_mut") {
            return Err(RuleViolation::new(
                ConstitutionalRule::DeterminismGlobal,
                format!("Operation '{}' would mutate global state", operation),
                context_id,
            )
            .with_details("Global state mutations are prohibited to maintain determinism"));
        }

        Ok(())
    }

    /// Check if a memory operation would violate MEMORY.CONTRACT.VIOLATION
    pub fn check_memory_contract(
        &self,
        operation: &str,
        context_id: Option<ExecutionContextId>,
    ) -> Result<(), RuleViolation> {
        if !self.enforcement_active {
            return Ok(());
        }

        // Check for memory safety violations
        if operation.contains("raw_pointer")
            || operation.contains("unbounded_alloc")
            || operation.contains("bounds_violation")
        {
            return Err(RuleViolation::new(
                ConstitutionalRule::MemoryContractViolation,
                format!("Operation '{}' violates memory safety contract", operation),
                context_id,
            )
            .with_details("Memory safety violations are prohibited"));
        }

        Ok(())
    }

    /// Check if an operation would violate KERNEL.SAFETY.CRITICAL
    pub fn check_kernel_safety(
        &self,
        operation: &str,
        context_id: Option<ExecutionContextId>,
    ) -> Result<(), RuleViolation> {
        if !self.enforcement_active {
            return Ok(());
        }

        // Check for kernel safety violations
        if operation.contains("kernel_direct")
            || operation.contains("ring0_access")
            || operation.contains("interrupt_handler")
        {
            return Err(RuleViolation::new(
                ConstitutionalRule::KernelSafetyCritical,
                format!("Operation '{}' compromises kernel safety", operation),
                context_id,
            )
            .with_details("Critical kernel safety must be maintained"));
        }

        Ok(())
    }

    /// Check if an operation would violate SECURITY.BOUNDARY.VIOLATION
    pub fn check_security_boundary(
        &self,
        operation: &str,
        context_id: Option<ExecutionContextId>,
    ) -> Result<(), RuleViolation> {
        if !self.enforcement_active {
            return Ok(());
        }

        // Check for security boundary violations
        if operation.contains("ring3_to_ring0")
            || operation.contains("bypass_syscall")
            || operation.contains("direct_kernel")
        {
            return Err(RuleViolation::new(
                ConstitutionalRule::SecurityBoundaryViolation,
                format!("Operation '{}' violates security boundary", operation),
                context_id,
            )
            .with_details("Ring3 cannot access Ring0 directly"));
        }

        Ok(())
    }

    /// Comprehensive check against all constitutional rules
    pub fn check_all_rules(
        &self,
        operation: &str,
        context_id: Option<ExecutionContextId>,
    ) -> Result<(), RuleViolation> {
        self.check_determinism_global(operation, context_id)?;
        self.check_memory_contract(operation, context_id)?;
        self.check_kernel_safety(operation, context_id)?;
        self.check_security_boundary(operation, context_id)?;
        Ok(())
    }

    /// Validate that an operation is constitutionally compliant
    ///
    /// Returns Ok(()) if the operation is allowed, or an IsolationError
    /// for fail-closed termination if any constitutional rule is violated.
    pub fn validate_operation(
        &self,
        operation: &str,
        context_id: Option<ExecutionContextId>,
    ) -> Result<(), IsolationError> {
        match self.check_all_rules(operation, context_id) {
            Ok(()) => Ok(()),
            Err(violation) => Err(violation.to_isolation_error()),
        }
    }
}

impl Default for ConstitutionalEnforcer {
    fn default() -> Self {
        Self::new()
    }
}

/// Convenience functions for common constitutional checks
impl ConstitutionalEnforcer {
    /// Check if BCIB execution would violate constitutional rules
    pub fn validate_bcib_execution(
        &self,
        opcode: u8,
        context_id: ExecutionContextId,
    ) -> Result<(), IsolationError> {
        let operation = format!("bcib_opcode_0x{:02x}", opcode);
        self.validate_operation(&operation, Some(context_id))
    }

    /// Check if ABDF access would violate constitutional rules
    pub fn validate_abdf_access(
        &self,
        access_type: &str,
        handle_id: u64,
        context_id: ExecutionContextId,
    ) -> Result<(), IsolationError> {
        let operation = format!("abdf_{}_{}", access_type, handle_id);
        self.validate_operation(&operation, Some(context_id))
    }

    /// Check if runtime bridge operation would violate constitutional rules
    pub fn validate_runtime_bridge_operation(
        &self,
        bridge_operation: &str,
        context_id: ExecutionContextId,
    ) -> Result<(), IsolationError> {
        let operation = format!("runtime_bridge_{}", bridge_operation);
        self.validate_operation(&operation, Some(context_id))
    }

    /// Check if syscall would violate constitutional rules
    pub fn validate_syscall(
        &self,
        syscall_number: u64,
        context_id: ExecutionContextId,
    ) -> Result<(), IsolationError> {
        // Only SYS_V2_SUBMIT_EXECUTION (1003) is allowed
        if syscall_number != 1003 {
            let _operation = format!("syscall_{}", syscall_number);
            // This should trigger a security boundary violation
            let violation = RuleViolation::new(
                ConstitutionalRule::SecurityBoundaryViolation,
                format!("Unauthorized syscall {} attempted", syscall_number),
                Some(context_id),
            )
            .with_details("Only SYS_V2_SUBMIT_EXECUTION (1003) is permitted");
            return Err(violation.to_isolation_error());
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn constitutional_rule_properties() {
        let rule = ConstitutionalRule::DeterminismGlobal;
        assert_eq!(rule.rule_name(), "DETERMINISM.GLOBAL");
        assert_eq!(rule.error_code(), ErrorCode::DeterminismGlobal);
        assert!(!rule.description().is_empty());
    }

    #[test]
    fn rule_violation_creation() {
        let violation = RuleViolation::new(
            ConstitutionalRule::MemoryContractViolation,
            "Raw pointer access detected",
            Some(42),
        )
        .with_details("Attempted to dereference raw pointer");

        assert_eq!(violation.rule, ConstitutionalRule::MemoryContractViolation);
        assert_eq!(violation.context_id, Some(42));
        assert!(violation.details.is_some());
    }

    #[test]
    fn rule_violation_to_isolation_error() {
        let violation = RuleViolation::new(
            ConstitutionalRule::SecurityBoundaryViolation,
            "Ring3 to Ring0 access",
            Some(123),
        );

        let error = violation.to_isolation_error();
        assert_eq!(error.code, ErrorCode::SecurityBoundaryViolation);
        assert_eq!(error.context_id, Some(123));
        assert!(error.requires_fail_closed());
    }

    #[test]
    fn constitutional_enforcer_determinism_check() {
        let enforcer = ConstitutionalEnforcer::new();

        // Should pass for normal operations
        assert!(enforcer
            .check_determinism_global("normal_operation", Some(1))
            .is_ok());

        // Should fail for global state mutations
        assert!(enforcer
            .check_determinism_global("global_state_mutation", Some(1))
            .is_err());
        assert!(enforcer
            .check_determinism_global("static_mut_access", Some(1))
            .is_err());
    }

    #[test]
    fn constitutional_enforcer_memory_check() {
        let enforcer = ConstitutionalEnforcer::new();

        // Should pass for safe operations
        assert!(enforcer
            .check_memory_contract("safe_allocation", Some(1))
            .is_ok());

        // Should fail for memory violations
        assert!(enforcer
            .check_memory_contract("raw_pointer_access", Some(1))
            .is_err());
        assert!(enforcer
            .check_memory_contract("unbounded_alloc", Some(1))
            .is_err());
        assert!(enforcer
            .check_memory_contract("bounds_violation", Some(1))
            .is_err());
    }

    #[test]
    fn constitutional_enforcer_kernel_safety_check() {
        let enforcer = ConstitutionalEnforcer::new();

        // Should pass for userspace operations
        assert!(enforcer
            .check_kernel_safety("userspace_operation", Some(1))
            .is_ok());

        // Should fail for kernel safety violations
        assert!(enforcer
            .check_kernel_safety("kernel_direct_access", Some(1))
            .is_err());
        assert!(enforcer
            .check_kernel_safety("ring0_access", Some(1))
            .is_err());
        assert!(enforcer
            .check_kernel_safety("interrupt_handler", Some(1))
            .is_err());
    }

    #[test]
    fn constitutional_enforcer_security_boundary_check() {
        let enforcer = ConstitutionalEnforcer::new();

        // Should pass for proper syscall usage
        assert!(enforcer
            .check_security_boundary("proper_syscall", Some(1))
            .is_ok());

        // Should fail for boundary violations
        assert!(enforcer
            .check_security_boundary("ring3_to_ring0", Some(1))
            .is_err());
        assert!(enforcer
            .check_security_boundary("bypass_syscall", Some(1))
            .is_err());
        assert!(enforcer
            .check_security_boundary("direct_kernel", Some(1))
            .is_err());
    }

    #[test]
    fn constitutional_enforcer_comprehensive_check() {
        let enforcer = ConstitutionalEnforcer::new();

        // Should pass for compliant operations
        assert!(enforcer
            .check_all_rules("compliant_operation", Some(1))
            .is_ok());

        // Should fail if any rule is violated
        assert!(enforcer
            .check_all_rules("global_state_mutation", Some(1))
            .is_err());
        assert!(enforcer
            .check_all_rules("raw_pointer_access", Some(1))
            .is_err());
        assert!(enforcer
            .check_all_rules("kernel_direct_access", Some(1))
            .is_err());
        assert!(enforcer.check_all_rules("ring3_to_ring0", Some(1)).is_err());
    }

    #[test]
    fn constitutional_enforcer_syscall_validation() {
        let enforcer = ConstitutionalEnforcer::new();

        // Should pass for allowed syscall (SYS_V2_SUBMIT_EXECUTION = 1003)
        assert!(enforcer.validate_syscall(1003, 1).is_ok());

        // Should fail for other syscalls
        assert!(enforcer.validate_syscall(1000, 1).is_err());
        assert!(enforcer.validate_syscall(1004, 1).is_err());
        assert!(enforcer.validate_syscall(2000, 1).is_err());
    }

    #[test]
    fn constitutional_enforcer_disabled_for_testing() {
        let enforcer = ConstitutionalEnforcer::disabled_for_testing();

        // All checks should pass when enforcement is disabled
        assert!(enforcer
            .check_determinism_global("global_state_mutation", Some(1))
            .is_ok());
        assert!(enforcer
            .check_memory_contract("raw_pointer_access", Some(1))
            .is_ok());
        assert!(enforcer
            .check_kernel_safety("kernel_direct_access", Some(1))
            .is_ok());
        assert!(enforcer
            .check_security_boundary("ring3_to_ring0", Some(1))
            .is_ok());
    }
}
