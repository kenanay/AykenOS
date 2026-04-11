pub mod abdf_handle;
pub mod boundary_enforcement;
pub mod constitutional;
/// Phase-16 BCIB/ABDF Isolation & Boundary Enforcement
///
/// This module implements the core isolation infrastructure for strict boundary
/// enforcement between BCIB execution and ABDF data substrate, following the
/// fail-closed semantics with constitutional compliance for NON_OVERRIDABLE rules.
///
/// ## Core Principles
///
/// - **Execution ≠ Data**: BCIB provides sandboxed, deterministic execution in Ring3,
///   while ABDF provides immutable, snapshot-consistent data storage.
/// - **Fail-Closed Enforcement**: All violations result in deterministic termination
///   rather than undefined behavior or security compromise.
/// - **Constitutional Compliance**: Enforces NON_OVERRIDABLE rules including
///   DETERMINISM.GLOBAL, MEMORY.CONTRACT.VIOLATION, KERNEL.SAFETY.CRITICAL,
///   and SECURITY.BOUNDARY.VIOLATION.
/// - **Runtime Bridge**: The sole approved interface between BCIB and external systems.
///
/// ## Architecture
///
/// ```text
/// ┌─────────────────────────────────────────────────────────────┐
/// │                       Ring3 Userspace                       │
/// │                                                             │
/// │   ┌─────────────────┐                                       │
/// │   │ BCIB_Executor   │───(Intent Only)──┐                    │
/// │   │ (Sandboxed)     │                  │                    │
/// │   └─────────────────┘                  ▼                    │
/// │                        ┌──────────────────────────────┐     │
/// │                        │       Runtime_Bridge         │     │
/// │                        │ - Capability Validation      │     │
/// │                        │ - Handle Translation         │     │
/// │                        │ - Side-Effect Ordering       │     │
/// │                        └──────────────────────────────┘     │
/// │   ┌─────────────────┐                  │                    │
/// │   │ ABDF Substrate  │◄──(Handles)──────┤                    │
/// │   │ (Immutable)     │                  │                    │
/// │   └─────────────────┘                  │                    │
/// │                                        ▼                    │
/// └─────────────────────────────────────────────────────────────┘
///                                         │
/// ┌────────────────────────────────────────▼────────────────────┐
/// │                       Ring0 Kernel                          │
/// │                (SYS_V2_SUBMIT_EXECUTION)                    │
/// └─────────────────────────────────────────────────────────────┘
/// ```
///
/// ## Modules
///
/// - `error_taxonomy`: Comprehensive error codes with fail-closed semantics
/// - `constitutional`: NON_OVERRIDABLE rule enforcement framework
/// - `runtime_bridge`: Core interface between BCIB and external systems
/// - `execution_sandbox`: BCIB execution isolation and memory bounds
/// - `side_effect_control`: Side-effect declaration and deterministic ordering
/// - `boundary_enforcement`: BCIB-ABDF boundary controls and validation
/// - `fail_closed`: Fail-closed termination system for all violations
pub mod error_taxonomy;
pub mod execution_entry_context;
pub mod execution_entry_enforcer;
pub mod execution_entry_integration_test;
pub mod execution_entry_requirements_test;
pub mod execution_sandbox;
pub mod fail_closed;
pub mod integration_test;
pub mod kernel_syscall_validator;
pub mod runtime_bridge;
pub mod side_effect_control;
pub mod termination_aware_harness;

// Re-export core types for convenience
pub use abdf_handle::{
    AbdfHandle, AccessMode, HandleId, HandleManager, HandlePoolConfig, HandleStatus, SegmentType,
    SegmentTypeValidator, SharedHandleManager,
};
pub use boundary_enforcement::{BoundaryEnforcer, BoundaryViolation};
pub use constitutional::{ConstitutionalEnforcer, ConstitutionalRule, RuleViolation};
pub use error_taxonomy::{ErrorCode, IsolationError, ViolationType};
pub use execution_entry_context::{
    CallStackFingerprint, ExecutionEntryContext, PrivilegeLevel, SlotOwnership, SyscallOrigin,
};
pub use execution_entry_enforcer::{
    DirectInvocationType, EntryValidationResult, ExecutionEntryEnforcer,
};
pub use execution_sandbox::{ExecutionSandbox, SandboxViolation};
pub use fail_closed::{FailClosedTermination, TerminationReason};
pub use kernel_syscall_validator::{ExecutionRole, KernelSyscallValidator, SyscallNumber};
pub use runtime_bridge::{RuntimeBridge, SideEffectIntent, SideEffectResult};
pub use side_effect_control::{SideEffectDeclaration, SideEffectOrdering};
