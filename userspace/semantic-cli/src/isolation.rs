//! Isolation and Boundary Enforcement Types
//!
//! This module defines the core types for Phase-16 BCIB/ABDF isolation and boundary enforcement.
//! It implements strict architectural boundaries that prevent execution context from directly
//! accessing kernel resources, device hardware, or mutable data structures.
//!
//! CONSTITUTIONAL ENFORCEMENT:
//! - SECURITY.BOUNDARY.VIOLATION: Ring3 → Ring0 boundary enforcement
//! - KERNEL.SAFETY.CRITICAL: Critical kernel safety maintenance
//! - DETERMINISM.GLOBAL: Global state mutation prevention
//! - MEMORY.CONTRACT.VIOLATION: Memory safety at boundaries

use serde::{Deserialize, Serialize};
use std::fmt;
use uuid::Uuid;

/// Isolation level for execution contexts
///
/// Defines the level of isolation required for safe execution.
/// Higher levels provide stronger isolation but may have performance overhead.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum IsolationLevel {
    /// No isolation - direct system access allowed
    /// Only safe for trusted, low-risk operations
    None,
    
    /// Sandboxed execution - limited system access
    /// Prevents dangerous operations but allows controlled resource access
    Sandboxed,
    
    /// Fully isolated execution - no direct system access
    /// All operations must go through controlled interfaces
    FullyIsolated,
}

impl Default for IsolationLevel {
    fn default() -> Self {
        Self::Sandboxed
    }
}

impl fmt::Display for IsolationLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::None => write!(f, "NONE"),
            Self::Sandboxed => write!(f, "SANDBOXED"),
            Self::FullyIsolated => write!(f, "FULLY_ISOLATED"),
        }
    }
}

/// Security context for execution
///
/// Contains all security-related information needed to validate and execute operations safely.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SecurityContext {
    /// Required isolation level
    pub isolation_level: IsolationLevel,
    
    /// Granted permissions for this context
    pub permissions: Vec<Permission>,
    
    /// Resource limits for this context
    pub resource_limits: ResourceLimits,
    
    /// Unique context identifier
    pub context_id: Uuid,
    
    /// Whether this context allows cross-context communication
    pub allow_cross_context: bool,
}

impl SecurityContext {
    /// Create a new security context with default settings
    pub fn new() -> Self {
        Self {
            isolation_level: IsolationLevel::default(),
            permissions: Vec::new(),
            resource_limits: ResourceLimits::default(),
            context_id: Uuid::new_v4(),
            allow_cross_context: false,
        }
    }
    
    /// Create a security context with specific isolation level
    pub fn with_isolation(isolation_level: IsolationLevel) -> Self {
        Self {
            isolation_level,
            permissions: Vec::new(),
            resource_limits: ResourceLimits::default(),
            context_id: Uuid::new_v4(),
            allow_cross_context: false,
        }
    }
    
    /// Add a permission to this context
    pub fn with_permission(mut self, permission: Permission) -> Self {
        self.permissions.push(permission);
        self
    }
    
    /// Set resource limits for this context
    pub fn with_resource_limits(mut self, limits: ResourceLimits) -> Self {
        self.resource_limits = limits;
        self
    }
    
    /// Check if this context has a specific permission
    pub fn has_permission(&self, permission: &Permission) -> bool {
        self.permissions.contains(permission)
    }
    
    /// Check if this context meets the required isolation level
    pub fn meets_isolation_requirement(&self, required: IsolationLevel) -> bool {
        self.isolation_level >= required
    }
}

impl Default for SecurityContext {
    fn default() -> Self {
        Self::new()
    }
}

/// Permission for specific operations
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Permission {
    /// Read access to specific context path
    Read(String),
    
    /// Write access to specific context path
    Write(String),
    
    /// Execute permission for specific command
    Execute(String),
    
    /// Device access permission
    DeviceAccess(String),
    
    /// Network access permission
    NetworkAccess,
    
    /// File system access permission
    FileSystemAccess(String),
    
    /// Kernel interaction permission (highly restricted)
    KernelInteraction,
    
    /// Cross-context communication permission
    CrossContext,
}

impl fmt::Display for Permission {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Read(path) => write!(f, "READ:{}", path),
            Self::Write(path) => write!(f, "WRITE:{}", path),
            Self::Execute(cmd) => write!(f, "EXECUTE:{}", cmd),
            Self::DeviceAccess(device) => write!(f, "DEVICE:{}", device),
            Self::NetworkAccess => write!(f, "NETWORK"),
            Self::FileSystemAccess(path) => write!(f, "FILESYSTEM:{}", path),
            Self::KernelInteraction => write!(f, "KERNEL"),
            Self::CrossContext => write!(f, "CROSS_CONTEXT"),
        }
    }
}

/// Resource limits for execution contexts
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResourceLimits {
    /// Maximum memory usage in bytes
    pub max_memory_bytes: u64,
    
    /// Maximum CPU time in milliseconds
    pub max_cpu_time_ms: u64,
    
    /// Maximum number of file descriptors
    pub max_file_descriptors: u32,
    
    /// Maximum number of network connections
    pub max_network_connections: u32,
    
    /// Maximum execution time in milliseconds
    pub max_execution_time_ms: u64,
}

impl Default for ResourceLimits {
    fn default() -> Self {
        Self {
            max_memory_bytes: 100 * 1024 * 1024, // 100MB
            max_cpu_time_ms: 30 * 1000,          // 30 seconds
            max_file_descriptors: 100,
            max_network_connections: 10,
            max_execution_time_ms: 60 * 1000,    // 60 seconds
        }
    }
}

/// Security constraint for operations
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SecurityConstraint {
    /// Require specific isolation level
    RequireIsolation(IsolationLevel),
    
    /// Require specific permission
    RequirePermission(Permission),
    
    /// Forbid specific operation
    ForbidOperation(String),
    
    /// Require approval for execution
    RequireApproval,
    
    /// Limit resource usage
    LimitResources(ResourceLimits),
}

/// Resource requirements for operations
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResourceRequirements {
    /// Estimated memory usage in bytes
    pub memory_bytes: u64,
    
    /// Estimated CPU time in milliseconds
    pub cpu_time_ms: u64,
    
    /// Number of file descriptors needed
    pub file_descriptors: u32,
    
    /// Number of network connections needed
    pub network_connections: u32,
    
    /// Estimated execution time in milliseconds
    pub execution_time_ms: u64,
}

impl Default for ResourceRequirements {
    fn default() -> Self {
        Self {
            memory_bytes: 1024 * 1024, // 1MB
            cpu_time_ms: 1000,         // 1 second
            file_descriptors: 1,
            network_connections: 0,
            execution_time_ms: 5000,   // 5 seconds
        }
    }
}

/// Syscall submission path enforcement
///
/// Ensures that only approved syscalls can be used for BCIB submission
#[derive(Debug, Clone)]
pub struct SyscallSubmissionEnforcer {
    /// List of approved syscalls for BCIB submission
    approved_syscalls: Vec<String>,
    
    /// Whether enforcement is enabled
    enforcement_enabled: bool,
}

impl SyscallSubmissionEnforcer {
    /// Create a new syscall submission enforcer
    pub fn new() -> Self {
        Self {
            approved_syscalls: vec!["SYS_V2_SUBMIT_EXECUTION".to_string()],
            enforcement_enabled: true,
        }
    }
    
    /// Check if a syscall is approved for BCIB submission
    pub fn is_syscall_approved(&self, syscall: &str) -> bool {
        if !self.enforcement_enabled {
            return true;
        }
        
        self.approved_syscalls.contains(&syscall.to_string())
    }
    
    /// Validate syscall submission path
    pub fn validate_submission_path(&self, syscall: &str) -> Result<(), crate::error::SemanticCLIError> {
        if !self.is_syscall_approved(syscall) {
            return Err(crate::error::SemanticCLIError::kernel_boundary_violation(
                format!("Syscall '{}' is not approved for BCIB submission. Only SYS_V2_SUBMIT_EXECUTION is allowed.", syscall),
                crate::error::ErrorCode::E963,
            ));
        }
        
        Ok(())
    }
    
    /// Enable or disable enforcement (for testing)
    pub fn set_enforcement_enabled(&mut self, enabled: bool) {
        self.enforcement_enabled = enabled;
    }
}

impl Default for SyscallSubmissionEnforcer {
    fn default() -> Self {
        Self::new()
    }
}

/// Kernel boundary violation detector
///
/// Detects attempts to bypass the approved kernel interaction paths
#[derive(Debug, Clone)]
pub struct KernelBoundaryDetector {
    /// Whether detection is enabled
    detection_enabled: bool,
    
    /// List of forbidden kernel operations
    forbidden_operations: Vec<String>,
}

impl KernelBoundaryDetector {
    /// Create a new kernel boundary detector
    pub fn new() -> Self {
        Self {
            detection_enabled: true,
            forbidden_operations: vec![
                "direct_syscall".to_string(),
                "kernel_memory_access".to_string(),
                "device_mmio".to_string(),
                "interrupt_handler".to_string(),
                "ring0_transition".to_string(),
            ],
        }
    }
    
    /// Detect kernel boundary violations
    pub fn detect_violation(&self, operation: &str) -> Result<(), crate::error::SemanticCLIError> {
        if !self.detection_enabled {
            return Ok(());
        }
        
        if self.forbidden_operations.contains(&operation.to_string()) {
            return Err(crate::error::SemanticCLIError::kernel_boundary_violation(
                format!("Forbidden kernel operation detected: {}", operation),
                crate::error::ErrorCode::E962,
            ));
        }
        
        Ok(())
    }
    
    /// Enable or disable detection (for testing)
    pub fn set_detection_enabled(&mut self, enabled: bool) {
        self.detection_enabled = enabled;
    }
}

impl Default for KernelBoundaryDetector {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_isolation_level_ordering() {
        assert!(IsolationLevel::None < IsolationLevel::Sandboxed);
        assert!(IsolationLevel::Sandboxed < IsolationLevel::FullyIsolated);
    }

    #[test]
    fn test_security_context_creation() {
        let ctx = SecurityContext::new();
        assert_eq!(ctx.isolation_level, IsolationLevel::Sandboxed);
        assert!(ctx.permissions.is_empty());
        assert!(!ctx.allow_cross_context);
    }

    #[test]
    fn test_security_context_with_permission() {
        let ctx = SecurityContext::new()
            .with_permission(Permission::Read("data.users".to_string()));
        
        assert!(ctx.has_permission(&Permission::Read("data.users".to_string())));
        assert!(!ctx.has_permission(&Permission::Write("data.users".to_string())));
    }

    #[test]
    fn test_isolation_requirement_check() {
        let ctx = SecurityContext::with_isolation(IsolationLevel::FullyIsolated);
        
        assert!(ctx.meets_isolation_requirement(IsolationLevel::None));
        assert!(ctx.meets_isolation_requirement(IsolationLevel::Sandboxed));
        assert!(ctx.meets_isolation_requirement(IsolationLevel::FullyIsolated));
    }

    #[test]
    fn test_syscall_submission_enforcer() {
        let enforcer = SyscallSubmissionEnforcer::new();
        
        assert!(enforcer.is_syscall_approved("SYS_V2_SUBMIT_EXECUTION"));
        assert!(!enforcer.is_syscall_approved("SYS_DIRECT_CALL"));
        
        let result = enforcer.validate_submission_path("SYS_V2_SUBMIT_EXECUTION");
        assert!(result.is_ok());
        
        let result = enforcer.validate_submission_path("SYS_DIRECT_CALL");
        assert!(result.is_err());
    }

    #[test]
    fn test_kernel_boundary_detector() {
        let detector = KernelBoundaryDetector::new();
        
        let result = detector.detect_violation("safe_operation");
        assert!(result.is_ok());
        
        let result = detector.detect_violation("direct_syscall");
        assert!(result.is_err());
    }
}