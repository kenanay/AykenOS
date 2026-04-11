/// Comprehensive Error Taxonomy with Fail-Closed Semantics
///
/// This module defines the complete error taxonomy for Phase-16 isolation and
/// boundary enforcement. All error codes follow fail-closed semantics where
/// violations result in deterministic termination rather than undefined behavior.
///
/// ## Requirements
///
/// - Requirement 15.1: System SHALL terminate execution immediately upon detecting any isolation violation
/// - Requirement 15.2: System SHALL terminate execution immediately upon detecting any boundary violation  
/// - Requirement 15.3: System SHALL terminate execution immediately upon detecting any capability violation
/// - Requirement 15.4: System SHALL NOT attempt to recover from security violations
/// - Requirement 15.5: System SHALL produce deterministic error codes for all violation types

use crate::types::{BcibError, ExecutionContextId};
use std::fmt;

/// Violation type classification for fail-closed enforcement
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ViolationType {
    /// Isolation boundary violations (Ring3 to Ring0, device access, etc.)
    Isolation,
    /// BCIB-ABDF boundary violations
    Boundary,
    /// Capability scope or validation violations
    Capability,
    /// Memory contract violations (bounds, raw pointers, etc.)
    Memory,
    /// Constitutional rule violations (NON_OVERRIDABLE)
    Constitutional,
    /// Sandbox escape attempts
    Sandbox,
    /// Side-effect ordering or declaration violations
    SideEffect,
}

/// Deterministic error codes for all violation types (Requirement 15.5)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorCode {
    // Isolation Violations (BCIB_ERR_ISOLATION_*)
    IsolationViolation = 0x1001,
    BridgeBypass = 0x1002,
    DeviceAccessViolation = 0x1003,
    KernelAccessViolation = 0x1004,
    SyscallViolation = 0x1005,
    
    // Boundary Violations (BCIB_ERR_BOUNDARY_*, ABDF_ERR_*)
    BoundaryViolation = 0x2001,
    AbdfDirectMutation = 0x2002,
    AbdfHandleRevoked = 0x2003,
    AbdfTypeViolation = 0x2004,
    AbdfAccessDenied = 0x2005,
    
    // Capability Violations (BCIB_ERR_CAPABILITY_*)
    CapabilityScopeViolation = 0x3001,
    CapabilityDenied = 0x3002,
    CapabilityRevoked = 0x3003,
    CapabilityEscalation = 0x3004,
    
    // Memory Contract Violations (MEMORY.CONTRACT.VIOLATION)
    MemoryContractViolation = 0x4001,
    BoundsViolation = 0x4002,
    RawPointerAccess = 0x4003,
    UnboundedAllocation = 0x4004,
    
    // Constitutional Violations (NON_OVERRIDABLE rules)
    ConstitutionalViolation = 0x5001,
    DeterminismGlobal = 0x5002,
    KernelSafetyCritical = 0x5003,
    SecurityBoundaryViolation = 0x5004,
    
    // Sandbox Violations (BCIB_ERR_SANDBOX_*)
    SandboxEscape = 0x6001,
    ContextIsolationViolation = 0x6002,
    CrossContextAccess = 0x6003,
    
    // Side-Effect Violations (BCIB_ERR_SIDE_EFFECT_*)
    UndeclaredSideEffect = 0x7001,
    SideEffectOrdering = 0x7002,
    OpcodeViolation = 0x7003,
}

impl ErrorCode {
    /// Get the violation type for this error code
    pub fn violation_type(self) -> ViolationType {
        match self {
            ErrorCode::IsolationViolation
            | ErrorCode::BridgeBypass
            | ErrorCode::DeviceAccessViolation
            | ErrorCode::KernelAccessViolation
            | ErrorCode::SyscallViolation => ViolationType::Isolation,
            
            ErrorCode::BoundaryViolation
            | ErrorCode::AbdfDirectMutation
            | ErrorCode::AbdfHandleRevoked
            | ErrorCode::AbdfTypeViolation
            | ErrorCode::AbdfAccessDenied => ViolationType::Boundary,
            
            ErrorCode::CapabilityScopeViolation
            | ErrorCode::CapabilityDenied
            | ErrorCode::CapabilityRevoked
            | ErrorCode::CapabilityEscalation => ViolationType::Capability,
            
            ErrorCode::MemoryContractViolation
            | ErrorCode::BoundsViolation
            | ErrorCode::RawPointerAccess
            | ErrorCode::UnboundedAllocation => ViolationType::Memory,
            
            ErrorCode::ConstitutionalViolation
            | ErrorCode::DeterminismGlobal
            | ErrorCode::KernelSafetyCritical
            | ErrorCode::SecurityBoundaryViolation => ViolationType::Constitutional,
            
            ErrorCode::SandboxEscape
            | ErrorCode::ContextIsolationViolation
            | ErrorCode::CrossContextAccess => ViolationType::Sandbox,
            
            ErrorCode::UndeclaredSideEffect
            | ErrorCode::SideEffectOrdering
            | ErrorCode::OpcodeViolation => ViolationType::SideEffect,
        }
    }
    
    /// Check if this error code represents a security violation that requires fail-closed termination
    pub fn is_security_violation(self) -> bool {
        matches!(
            self.violation_type(),
            ViolationType::Isolation
                | ViolationType::Boundary
                | ViolationType::Constitutional
                | ViolationType::Sandbox
        )
    }
    
    /// Check if this error code represents a NON_OVERRIDABLE constitutional rule violation
    pub fn is_constitutional_violation(self) -> bool {
        matches!(
            self,
            ErrorCode::ConstitutionalViolation
                | ErrorCode::DeterminismGlobal
                | ErrorCode::KernelSafetyCritical
                | ErrorCode::SecurityBoundaryViolation
                | ErrorCode::MemoryContractViolation
        )
    }
}

impl fmt::Display for ErrorCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            // Isolation Violations
            ErrorCode::IsolationViolation => write!(f, "BCIB_ERR_ISOLATION_VIOLATION"),
            ErrorCode::BridgeBypass => write!(f, "BCIB_ERR_BRIDGE_BYPASS"),
            ErrorCode::DeviceAccessViolation => write!(f, "BCIB_ERR_DEVICE_ACCESS_VIOLATION"),
            ErrorCode::KernelAccessViolation => write!(f, "BCIB_ERR_KERNEL_ACCESS_VIOLATION"),
            ErrorCode::SyscallViolation => write!(f, "BCIB_ERR_SYSCALL_VIOLATION"),
            
            // Boundary Violations
            ErrorCode::BoundaryViolation => write!(f, "ABDF_BOUNDARY_VIOLATION"),
            ErrorCode::AbdfDirectMutation => write!(f, "ABDF_ERR_DIRECT_MUTATION"),
            ErrorCode::AbdfHandleRevoked => write!(f, "BCIB_ERR_ABDF_HANDLE_REVOKED"),
            ErrorCode::AbdfTypeViolation => write!(f, "ABDF_ERR_TYPE_VIOLATION"),
            ErrorCode::AbdfAccessDenied => write!(f, "BCIB_ERR_ABDF_ACCESS_DENIED"),
            
            // Capability Violations
            ErrorCode::CapabilityScopeViolation => write!(f, "BCIB_ERR_CAPABILITY_SCOPE_VIOLATION"),
            ErrorCode::CapabilityDenied => write!(f, "BCIB_ERR_CAPABILITY_DENIED"),
            ErrorCode::CapabilityRevoked => write!(f, "BCIB_ERR_CAPABILITY_REVOKED"),
            ErrorCode::CapabilityEscalation => write!(f, "BCIB_ERR_CAPABILITY_ESCALATION"),
            
            // Memory Contract Violations
            ErrorCode::MemoryContractViolation => write!(f, "MEMORY.CONTRACT.VIOLATION"),
            ErrorCode::BoundsViolation => write!(f, "BCIB_ERR_BOUNDS_VIOLATION"),
            ErrorCode::RawPointerAccess => write!(f, "BCIB_ERR_RAW_POINTER_ACCESS"),
            ErrorCode::UnboundedAllocation => write!(f, "BCIB_ERR_UNBOUNDED_ALLOCATION"),
            
            // Constitutional Violations
            ErrorCode::ConstitutionalViolation => write!(f, "CONSTITUTIONAL_VIOLATION"),
            ErrorCode::DeterminismGlobal => write!(f, "DETERMINISM.GLOBAL"),
            ErrorCode::KernelSafetyCritical => write!(f, "KERNEL.SAFETY.CRITICAL"),
            ErrorCode::SecurityBoundaryViolation => write!(f, "SECURITY.BOUNDARY.VIOLATION"),
            
            // Sandbox Violations
            ErrorCode::SandboxEscape => write!(f, "BCIB_ERR_SANDBOX_ESCAPE"),
            ErrorCode::ContextIsolationViolation => write!(f, "BCIB_ERR_CONTEXT_ISOLATION_VIOLATION"),
            ErrorCode::CrossContextAccess => write!(f, "BCIB_ERR_CROSS_CONTEXT_ACCESS"),
            
            // Side-Effect Violations
            ErrorCode::UndeclaredSideEffect => write!(f, "BCIB_ERR_UNDECLARED_SIDE_EFFECT"),
            ErrorCode::SideEffectOrdering => write!(f, "BCIB_ERR_SIDE_EFFECT_ORDERING"),
            ErrorCode::OpcodeViolation => write!(f, "BCIB_ERR_OPCODE_VIOLATION"),
        }
    }
}

/// Comprehensive isolation error with fail-closed semantics
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IsolationError {
    /// Deterministic error code
    pub code: ErrorCode,
    /// Human-readable error message
    pub message: String,
    /// Execution context where the violation occurred
    pub context_id: Option<ExecutionContextId>,
    /// Additional violation-specific data
    pub details: Option<String>,
}

impl IsolationError {
    /// Create a new isolation error with fail-closed semantics
    pub fn new(
        code: ErrorCode,
        message: impl Into<String>,
        context_id: Option<ExecutionContextId>,
    ) -> Self {
        Self {
            code,
            message: message.into(),
            context_id,
            details: None,
        }
    }
    
    /// Create an isolation error with additional details
    pub fn with_details(
        code: ErrorCode,
        message: impl Into<String>,
        context_id: Option<ExecutionContextId>,
        details: impl Into<String>,
    ) -> Self {
        Self {
            code,
            message: message.into(),
            context_id,
            details: Some(details.into()),
        }
    }
    
    /// Check if this error requires immediate fail-closed termination
    pub fn requires_fail_closed(&self) -> bool {
        self.code.is_security_violation() || self.code.is_constitutional_violation()
    }
    
    /// Get the violation type for this error
    pub fn violation_type(&self) -> ViolationType {
        self.code.violation_type()
    }
    
    /// Convert to BcibError for compatibility with existing error handling
    pub fn to_bcib_error(&self) -> BcibError {
        // Use static string literals for BcibError compatibility
        match self.code.violation_type() {
            ViolationType::Isolation => BcibError::IsolationViolation("isolation violation detected"),
            ViolationType::Boundary => BcibError::AbdfBoundaryViolation("boundary violation detected"),
            ViolationType::Capability => BcibError::CapabilityDenied("capability violation detected"),
            ViolationType::Memory => BcibError::BoundsViolation("memory violation detected"),
            ViolationType::Constitutional => BcibError::IsolationViolation("constitutional violation detected"),
            ViolationType::Sandbox => BcibError::IsolationViolation("sandbox violation detected"),
            ViolationType::SideEffect => BcibError::IllegalStateTransition("side-effect violation detected"),
        }
    }
}

impl fmt::Display for IsolationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} (0x{:04X}): {}", self.code, self.code as u16, self.message)?;
        
        if let Some(ctx_id) = self.context_id {
            write!(f, " [context: {}]", ctx_id)?;
        }
        
        if let Some(ref details) = self.details {
            write!(f, " - {}", details)?;
        }
        
        Ok(())
    }
}

impl std::error::Error for IsolationError {}

/// Convenience functions for creating common isolation errors
impl IsolationError {
    /// BCIB attempted to bypass Runtime_Bridge
    pub fn bridge_bypass(context_id: ExecutionContextId, operation: &str) -> Self {
        Self::with_details(
            ErrorCode::BridgeBypass,
            "BCIB attempted to bypass Runtime_Bridge",
            Some(context_id),
            format!("attempted operation: {}", operation),
        )
    }
    
    /// BCIB attempted direct device access
    pub fn device_access_violation(context_id: ExecutionContextId, device: &str) -> Self {
        Self::with_details(
            ErrorCode::DeviceAccessViolation,
            "BCIB attempted direct device access",
            Some(context_id),
            format!("attempted device: {}", device),
        )
    }
    
    /// ABDF handle has been revoked
    pub fn abdf_handle_revoked(context_id: ExecutionContextId, handle_id: u64) -> Self {
        Self::with_details(
            ErrorCode::AbdfHandleRevoked,
            "ABDF handle has been revoked",
            Some(context_id),
            format!("handle_id: {}", handle_id),
        )
    }
    
    /// Capability scope violation
    pub fn capability_scope_violation(
        context_id: ExecutionContextId,
        token_id: u64,
        attempted_resource: &str,
    ) -> Self {
        Self::with_details(
            ErrorCode::CapabilityScopeViolation,
            "Capability used outside its declared scope",
            Some(context_id),
            format!("token_id: {}, attempted_resource: {}", token_id, attempted_resource),
        )
    }
    
    /// Memory contract violation (NON_OVERRIDABLE)
    pub fn memory_contract_violation(context_id: ExecutionContextId, violation: &str) -> Self {
        Self::with_details(
            ErrorCode::MemoryContractViolation,
            "Memory safety contract violated",
            Some(context_id),
            format!("violation: {}", violation),
        )
    }
    
    /// Constitutional rule violation (NON_OVERRIDABLE)
    pub fn constitutional_violation(rule: &str, violation: &str) -> Self {
        Self::with_details(
            ErrorCode::ConstitutionalViolation,
            "NON_OVERRIDABLE constitutional rule violated",
            None,
            format!("rule: {}, violation: {}", rule, violation),
        )
    }
    
    /// Sandbox escape attempt
    pub fn sandbox_escape(context_id: ExecutionContextId, escape_type: &str) -> Self {
        Self::with_details(
            ErrorCode::SandboxEscape,
            "BCIB attempted to escape execution sandbox",
            Some(context_id),
            format!("escape_type: {}", escape_type),
        )
    }
    
    /// Undeclared side-effect detected
    pub fn undeclared_side_effect(
        context_id: ExecutionContextId,
        opcode: u8,
        side_effect: &str,
    ) -> Self {
        Self::with_details(
            ErrorCode::UndeclaredSideEffect,
            "BCIB instruction performed undeclared side-effect",
            Some(context_id),
            format!("opcode: 0x{:02X}, side_effect: {}", opcode, side_effect),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn error_code_violation_type_mapping() {
        assert_eq!(ErrorCode::IsolationViolation.violation_type(), ViolationType::Isolation);
        assert_eq!(ErrorCode::BoundaryViolation.violation_type(), ViolationType::Boundary);
        assert_eq!(ErrorCode::CapabilityDenied.violation_type(), ViolationType::Capability);
        assert_eq!(ErrorCode::MemoryContractViolation.violation_type(), ViolationType::Memory);
        assert_eq!(ErrorCode::ConstitutionalViolation.violation_type(), ViolationType::Constitutional);
        assert_eq!(ErrorCode::SandboxEscape.violation_type(), ViolationType::Sandbox);
        assert_eq!(ErrorCode::UndeclaredSideEffect.violation_type(), ViolationType::SideEffect);
    }

    #[test]
    fn security_violation_detection() {
        assert!(ErrorCode::IsolationViolation.is_security_violation());
        assert!(ErrorCode::BoundaryViolation.is_security_violation());
        assert!(ErrorCode::ConstitutionalViolation.is_security_violation());
        assert!(ErrorCode::SandboxEscape.is_security_violation());
        assert!(!ErrorCode::CapabilityDenied.is_security_violation());
    }

    #[test]
    fn constitutional_violation_detection() {
        assert!(ErrorCode::ConstitutionalViolation.is_constitutional_violation());
        assert!(ErrorCode::DeterminismGlobal.is_constitutional_violation());
        assert!(ErrorCode::KernelSafetyCritical.is_constitutional_violation());
        assert!(ErrorCode::SecurityBoundaryViolation.is_constitutional_violation());
        assert!(ErrorCode::MemoryContractViolation.is_constitutional_violation());
        assert!(!ErrorCode::CapabilityDenied.is_constitutional_violation());
    }

    #[test]
    fn isolation_error_creation() {
        let error = IsolationError::bridge_bypass(42, "direct_syscall");
        assert_eq!(error.code, ErrorCode::BridgeBypass);
        assert_eq!(error.context_id, Some(42));
        assert!(error.details.is_some());
        assert!(error.requires_fail_closed());
    }

    #[test]
    fn isolation_error_display() {
        let error = IsolationError::new(
            ErrorCode::IsolationViolation,
            "Test violation",
            Some(123),
        );
        let display = format!("{}", error);
        assert!(display.contains("BCIB_ERR_ISOLATION_VIOLATION"));
        assert!(display.contains("0x1001"));
        assert!(display.contains("Test violation"));
        assert!(display.contains("[context: 123]"));
    }

    #[test]
    fn bcib_error_conversion() {
        let isolation_error = IsolationError::new(
            ErrorCode::BoundaryViolation,
            "Test boundary violation",
            Some(456),
        );
        let bcib_error = isolation_error.to_bcib_error();
        assert!(matches!(bcib_error, BcibError::AbdfBoundaryViolation(_)));
    }
}