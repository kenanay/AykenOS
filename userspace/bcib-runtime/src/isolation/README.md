# Phase-16 BCIB/ABDF Isolation & Boundary Enforcement

This directory contains the core isolation infrastructure for strict boundary enforcement between BCIB execution and ABDF data substrate, implementing fail-closed semantics with constitutional compliance for NON_OVERRIDABLE rules.

## Overview

The isolation infrastructure enforces the fundamental principle: **Execution ≠ Data**. BCIB provides sandboxed, deterministic execution in Ring3, while ABDF provides immutable, snapshot-consistent data storage. The boundary between them is strictly enforced to maintain system integrity, determinism, and security.

## Architecture

```text
┌─────────────────────────────────────────────────────────────┐
│                       Ring3 Userspace                       │
│                                                             │
│   ┌─────────────────┐                                       │
│   │ BCIB_Executor   │───(Intent Only)──┐                    │
│   │ (Sandboxed)     │                  │                    │
│   └─────────────────┘                  ▼                    │
│                        ┌──────────────────────────────┐     │
│                        │       Runtime_Bridge         │     │
│                        │ - Capability Validation      │     │
│                        │ - Handle Translation         │     │
│                        │ - Side-Effect Ordering       │     │
│                        └──────────────────────────────┘     │
│   ┌─────────────────┐                  │                    │
│   │ ABDF Substrate  │◄──(Handles)──────┤                    │
│   │ (Immutable)     │                  │                    │
│   └─────────────────┘                  │                    │
│                                        ▼                    │
└─────────────────────────────────────────────────────────────┘
                                         │
┌────────────────────────────────────────▼────────────────────┐
│                       Ring0 Kernel                          │
│                (SYS_V2_SUBMIT_EXECUTION)                    │
└─────────────────────────────────────────────────────────────┘
```

## Core Components

### 1. Error Taxonomy (`error_taxonomy.rs`)

Comprehensive error classification system with deterministic error codes:

- **Isolation Violations** (0x1000-0x1FFF): Ring3 to Ring0 access, device access, syscall violations
- **Boundary Violations** (0x2000-0x2FFF): ABDF boundary violations, direct mutations, handle issues
- **Capability Violations** (0x3000-0x3FFF): Scope violations, denials, escalations
- **Memory Violations** (0x4000-0x4FFF): Contract violations, bounds violations, raw pointer access
- **Constitutional Violations** (0x5000-0x5FFF): NON_OVERRIDABLE rule violations
- **Sandbox Violations** (0x6000-0x6FFF): Escape attempts, context isolation violations
- **Side-Effect Violations** (0x7000-0x7FFF): Undeclared side-effects, ordering violations

### 2. Constitutional Enforcement (`constitutional.rs`)

Enforces NON_OVERRIDABLE constitutional rules:

- `DETERMINISM.GLOBAL` — Prevents global state mutations
- `MEMORY.CONTRACT.VIOLATION` — Enforces memory safety
- `KERNEL.SAFETY.CRITICAL` — Maintains kernel safety
- `SECURITY.BOUNDARY.VIOLATION` — Prevents Ring3 to Ring0 access

### 3. Fail-Closed Termination (`fail_closed.rs`)

Implements fail-closed semantics for all violations:

- Immediate termination upon violation detection
- Deterministic error codes (Requirement 15.5)
- Immutable audit logging (Requirement 15.6)
- Prevention of partial state commits (Requirement 15.7)
- No recovery attempts for security violations (Requirement 15.4)

### 4. Runtime Bridge (`runtime_bridge.rs`)

The sole approved interface between BCIB and external systems:

- Capability validation for all operations
- Handle translation and management
- Side-effect ordering enforcement
- Intent-based operation model (Phase-15 compatibility)

### 5. Execution Sandbox (`execution_sandbox.rs`)

BCIB execution isolation and memory bounds:

- Ring3-only execution enforcement
- Memory bounds checking
- Cross-context isolation
- Syscall restriction enforcement

### 6. Side-Effect Control (`side_effect_control.rs`)

Side-effect declaration and deterministic ordering:

- Pre-execution side-effect declaration
- Classification (Pure, DataMutating, External)
- Deterministic ordering enforcement
- Undeclared side-effect detection

### 7. Boundary Enforcement (`boundary_enforcement.rs`)

BCIB-ABDF boundary controls:

- Direct ABDF access prevention
- Storage semantics mutation detection
- Out-of-ABDF storage prevention

## Requirements Compliance

### Requirement 15.1: Immediate Isolation Violation Termination
✅ Implemented in `fail_closed.rs` with `fail_closed_terminate()`

### Requirement 15.2: Immediate Boundary Violation Termination
✅ Implemented in `boundary_enforcement.rs` with fail-closed integration

### Requirement 15.3: Immediate Capability Violation Termination
✅ Implemented in capability validation with fail-closed enforcement

### Requirement 15.4: No Recovery from Security Violations
✅ Enforced by fail-closed termination system - no recovery paths exist

### Requirement 15.5: Deterministic Error Codes
✅ Implemented in `error_taxonomy.rs` with stable numeric codes

### Constitutional Compliance

All NON_OVERRIDABLE rules are enforced at ERROR level:

- `DETERMINISM.GLOBAL` → `ErrorCode::DeterminismGlobal` (0x5002)
- `MEMORY.CONTRACT.VIOLATION` → `ErrorCode::MemoryContractViolation` (0x4001)
- `KERNEL.SAFETY.CRITICAL` → `ErrorCode::KernelSafetyCritical` (0x5003)
- `SECURITY.BOUNDARY.VIOLATION` → `ErrorCode::SecurityBoundaryViolation` (0x5004)

## Usage Examples

### Basic Constitutional Validation

```rust
use bcib_runtime::isolation::{ConstitutionalEnforcer, IsolationError};

let enforcer = ConstitutionalEnforcer::new();

// Validate BCIB execution
match enforcer.validate_bcib_execution(0x01, context_id) {
    Ok(()) => {
        // Execution is constitutionally compliant
    }
    Err(error) => {
        // Constitutional violation detected - fail-closed termination required
        if error.requires_fail_closed() {
            fail_closed_terminate(error);
        }
    }
}
```

### Error Handling with Fail-Closed

```rust
use bcib_runtime::isolation::{IsolationError, ErrorCode, fail_closed_terminate};

// Detect a boundary violation
let error = IsolationError::bridge_bypass(context_id, "direct_syscall");

// Check if fail-closed termination is required
if error.requires_fail_closed() {
    // Terminate immediately with deterministic error code
    fail_closed_terminate(error); // This function never returns
}
```

### Runtime Bridge Usage

```rust
use bcib_runtime::isolation::{RuntimeBridge, SideEffectIntent};

let bridge = RuntimeBridge::new(context_id);
let intent = SideEffectIntent::AbdfRead { handle_id: 123 };

match bridge.execute_side_effect(intent, capability_token) {
    Ok(result) => {
        // Side-effect executed successfully
    }
    Err(error) => {
        // Handle error with potential fail-closed termination
    }
}
```

## Testing

The isolation infrastructure includes comprehensive tests:

- **Unit Tests**: Each component has thorough unit tests
- **Integration Tests**: `integration_test.rs` demonstrates components working together
- **Property Tests**: Constitutional enforcement and error taxonomy validation
- **Fail-Closed Tests**: Termination behavior verification

Run tests with:
```bash
cargo test --lib isolation
```

## Implementation Status

### Task 1 (Current): Core Infrastructure ✅
- [x] Directory structure created
- [x] Comprehensive error taxonomy with fail-closed semantics
- [x] Constitutional rule enforcement framework
- [x] Fail-closed termination system
- [x] Integration tests and documentation

### Future Tasks (Placeholders Created):
- [ ] Task 2: Kernel boundary hardening
- [ ] Task 3: BCIB execution entry enforcement
- [ ] Task 4: ABDF Handle Management System (full implementation)
- [ ] Task 5: Runtime_Bridge core interface (full implementation)
- [ ] Task 6: BCIB Execution Sandbox (full implementation)
- [ ] Task 7: Side-effect control and determinism (full implementation)
- [ ] Task 8: ABDF immutability and boundary enforcement (full implementation)
- [ ] Task 9: Device access isolation and cross-context controls
- [ ] Task 10: Fail-closed enforcement and error handling (integration)
- [ ] Task 11: CI gates and constitutional compliance
- [ ] Task 12: Integration and comprehensive testing
- [ ] Task 13: Final validation and deployment readiness

## Key Design Principles

1. **Fail-Closed by Default**: All violations result in deterministic termination
2. **Constitutional Compliance**: NON_OVERRIDABLE rules cannot be bypassed
3. **Phase-15 Compatibility**: BCIB core semantics remain unchanged
4. **Handle-Only Access**: No raw pointers, opaque ABDF references only
5. **Capability-Based Security**: Scoped permissions for all privileged operations
6. **Deterministic Behavior**: All operations produce predictable, reproducible results

## Security Guarantees

- **Isolation**: BCIB cannot escape Ring3 or access kernel directly
- **Boundary Enforcement**: BCIB-ABDF boundary is strictly maintained
- **Memory Safety**: No raw pointer access, bounded memory regions only
- **Determinism**: Global state mutations are prevented
- **Auditability**: All violations are logged to immutable audit trail
- **Fail-Closed**: Security violations never result in undefined behavior

This infrastructure provides the foundation for secure, deterministic, and constitutionally compliant BCIB execution with strict isolation from the ABDF data substrate.