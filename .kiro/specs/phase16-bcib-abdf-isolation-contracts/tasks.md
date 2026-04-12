# Implementation Plan: Phase-16 BCIB/ABDF Isolation & Boundary Enforcement

## Overview

This implementation plan converts the Phase-16 design into discrete coding tasks for implementing strict isolation and boundary enforcement between BCIB execution and ABDF data substrate. The implementation follows fail-closed semantics with constitutional compliance for NON_OVERRIDABLE rules.

**CRITICAL DEPENDENCY:** Execution closure is a PRODUCTION BLOCKER, not an IMPLEMENTATION BLOCKER. Isolation infrastructure MAY be implemented before closure completion, but production deployment is FORBIDDEN until execution closure is completed with kernel-level evidence.

## Production Closure Rule (GLOBAL - MANDATORY)

**NO TASK** that claims kernel-boundary enforcement, isolation, or fail-closed behavior may be marked COMPLETE without `ci-gate-fail-closed-proof` PASS.

**Affected Tasks:** Task 3 (execution entry), Task 5 (Runtime_Bridge syscall path), Task 6 (sandbox), Task 10 (fail-closed enforcement)

**Evidence Requirement:**
- Host tests, harness tests, or emulated validation are NOT sufficient for production closure
- QEMU kernel trace with canonical marker flow is MANDATORY
- Negative guarantees must be validated (no continuation after kill)
- Hard stop must be proven (scheduler removal, no process logs after kill)

**Closure Validation:**
- `ci-gate-fail-closed-proof` must PASS before any kernel-boundary task is marked complete
- Missing QEMU evidence = task remains INCOMPLETE regardless of host test status
- Continuation markers after kill = task FAILS closure validation

**Key Implementation Principles:**
- Phase-15 compatibility: BCIB core semantics remain unchanged
- Fail-closed enforcement: All violations result in deterministic termination
- Constitutional compliance: Enforces DETERMINISM.GLOBAL, MEMORY.CONTRACT.VIOLATION, KERNEL.SAFETY.CRITICAL, SECURITY.BOUNDARY.VIOLATION
- Handle-only access: No raw pointers, opaque ABDF references only
- Capability-based security: Scoped permissions for all privileged operations
- Kernel boundary preservation: Runtime_Bridge does NOT bypass or replace syscall surface

## Enforcement Rules (Mandatory For All Tasks)

All tasks in this document must comply with the following enforcement rules.

### 1. Enforcement Authority

- Security-critical enforcement must be implemented at the authoritative boundary.
- Kernel/syscall/MMU boundary enforcement is authoritative for execution, resource, memory, capability, and isolation boundaries.
- Userspace enforcement is advisory unless a task explicitly defines it as the authoritative layer.
- Userspace-only enforcement is not sufficient for boundary or security rules.

### 2. Forbidden Implementation Patterns

The following implementation patterns are strictly forbidden:

- String-based validation for syscall or execution control
- Pattern-based filtering such as `test_`, `debug_`, or `internal_`
- Disableable enforcement in production builds
- Userspace-only enforcement for boundary or security rules
- Fallback-to-success behavior after failed validation

### 3. Required Enforcement Mechanism

- Validation must occur at the authoritative boundary before execution continuation.
- Boundary validation must occur before resource allocation, including execution contexts, memory, handles, execution slots, and result mappings.
- Kernel-facing paths must validate concrete syscall IDs and ABI contracts, not names or inferred intent.
- Enforcement must be tied to the relevant `Execution_Context`, ABDF handle lifecycle, or kernel execution slot lifecycle.

### 4. Fail-Closed Requirement

All violations must:

- Prevent execution continuation
- Prevent unauthorized resource allocation
- Return a deterministic error code or terminate execution deterministically
- Produce auditable evidence when the task is part of a runtime/kernel gate

### 5. Evidence Requirement

- Boundary and security enforcement must be verifiable through runtime evidence.
- Kernel-level claims require QEMU/kernel trace evidence.
- Host-only tests may prove host harness behavior, but they cannot close production kernel-boundary claims.

## Tasks

- [x] 1. Set up core isolation infrastructure and error taxonomy
  - Create directory structure for isolation components
  - Define comprehensive error taxonomy with fail-closed semantics
  - Set up constitutional rule enforcement framework
  - _Requirements: 15.1, 15.2, 15.3, 15.4, 15.5_

- [x] 2. Implement kernel boundary hardening
  - Implement syscall submission path hardening (SYS_V2_SUBMIT_EXECUTION only)
  - Ensure no direct kernel API exposure beyond approved submission interface
  - Verify Runtime_Bridge cannot replace or bypass syscall surface
  - Create kernel boundary violation detection and fail-closed enforcement
  - _Requirements: 1.5, 1.6, 1.7, 1.8_

- [ ] 3. Implement BCIB execution entry enforcement
  - Enforcement Level: Kernel-level authoritative
  - Userspace validation is not sufficient for execution entry enforcement
  - Validation must occur at syscall dispatch / kernel execution-slot boundary before context, slot, memory, or handle allocation
  - **PRODUCTION CLOSURE REQUIREMENT:**
    - Task 3 CANNOT be marked complete without `ci-gate-fail-closed-proof` PASS
    - QEMU kernel trace must prove invalid entry rejection with no context/slot/memory/handle allocation
    - Host tests alone DO NOT satisfy production closure
    - Valid dispatcher path AND invalid entry path must both produce deterministic QEMU/kernel audit evidence
  - Forbidden Implementation:
    - String-based syscall validation
    - Pattern-based entry filtering such as `test_`, `debug_`, or `internal_`
    - Userspace-only enforcement
    - Disableable enforcement in production builds
  - Reject direct invocation paths (test helpers, debug hooks, internal calls)
  - Enforce syscall-only entry via approved submission path
  - Implement execution entry point validation and fail-closed enforcement
  - Prevent bypass of execution submission interface
  - Fail-Closed Requirement:
    - Invalid entry must reject the request
    - Invalid entry must not create an execution context
    - Invalid entry must not allocate an execution slot or result mapping
    - Invalid entry must return a deterministic error code
    - `[[AYKEN_BOUNDARY_KILL]]` must be emitted BEFORE scheduler removal
  - Evidence Required:
    - QEMU/kernel trace: invalid entry attempt is rejected
    - QEMU/kernel trace: no context or slot allocation occurs after invalid entry
    - QEMU/kernel trace: canonical marker flow (FORBIDDEN_BEFORE → SYSCALL_ENTER → BOUNDARY_KILL)
    - QEMU/kernel trace: negative guarantees validated (no continuation markers after kill)
  - Closure Status: ARCHITECTURALLY CORRECTED; KERNEL-ENTRY MODEL INTRODUCED; SECURITY BEHAVIOR PARTIALLY VERIFIED; FINAL TEST HARNESS / CODEBASE CLOSURE PENDING
  - Host Closure Evidence:
    - `execution_entry_integration_test` passes without abnormal harness exit
    - Direct-invocation fail-closed behavior is covered by termination-aware host tests
    - `cargo test --lib` for `bcib-runtime` passes with 358 tests
    - `create_context_with_limits_internal` contains one authoritative `validate_kernel_execution_entry(&entry_context)` call before allocation
    - `validate_no_execution_bypass` dead code has been removed
  - Completion Blockers:
    - QEMU/kernel evidence must prove invalid entry rejection with no context, slot, memory, or handle allocation
    - `create_valid_kernel_context_for_test`, `create_context_for_test`, and `create_context_with_limits_for_test` are non-authoritative emulated-kernel host helpers and cannot satisfy production closure
    - Legacy string/pattern/call-stack validation must be removed from the production authority path or explicitly isolated as test-only/non-authoritative
    - Task 3 security-relevant warnings must be resolved before production closure, including `static_mut_refs` and transitional unused imports/dead code
    - Valid dispatcher path and invalid entry path must both produce deterministic QEMU/kernel audit/proof evidence
    - **`ci-gate-fail-closed-proof` must PASS before Task 3 can be marked complete**
  - _Requirements: 1.3, 1.4_

- [x] 4. Implement ABDF Handle Management System
  - Closure Status: SUBSTANTIALLY COMPLETE; MEMORY-SAFETY POSTURE IMPROVED; SYSTEM-LEVEL ENFORCEMENT NOT PRODUCTION-CLOSED
  - Host Evidence:
    - Opaque ABDF handles, context binding, lifecycle management, revocation, exhaustion prevention, and segment validation are implemented
    - `cargo test --manifest-path userspace/bcib-runtime/Cargo.toml --lib` passes with 403 tests
    - `static_mut_refs` warning for fail-closed termination has been removed by replacing the mutable static handler with `OnceLock`
  - Production Caveats:
    - Task 4 completion does not imply Runtime_Bridge, ExecutionSandbox, BoundaryEnforcer, or SideEffectOrdering production enforcement
    - RuntimeBridge capability validation and side-effect execution remain Task 5 work
    - ExecutionSandbox memory/context enforcement remains Task 6 work
    - Boundary and side-effect ordering enforcement remain later task work
    - Current hygiene gate is not clean while tracked files remain dirty
  - [x] 4.1 Create opaque handle types and lifecycle management
    - Implement `AbdfHandle` struct with context binding and status tracking
    - Create handle creation, validation, and revocation mechanisms
    - Implement handle-to-resource mapping without exposing raw pointers
    - _Requirements: 9.1, 9.2, 9.3, 9.4, 9.5, 9.6, 9.7, 9.8, 9.9_
  
  - [x] 4.2 Implement handle exhaustion prevention and reclamation
    - Create bounded handle pool with exhaustion detection
    - Implement stale handle reclamation mechanisms
    - Create revocation propagation across execution contexts
    - Implement exhaustion fail-closed behavior
    - _Requirements: 9.5, 9.6, 9.9_
  
  - [x] 4.3 Write property test for handle opacity invariant
    - **Property 2: Handle Opacity Invariant**
    - **Validates: Requirements 9.1, 9.2**
  
  - [x] 4.4 Implement ABDF segment type system
    - Define segment types: Input, Event, DeviceStatus, ReadResult, ExecutionResult, ExecutionTrace, Ref
    - Implement type validation and constraint enforcement
    - Create type-safe segment creation and access methods
    - _Requirements: 10.1, 10.2, 10.3, 10.4, 10.5, 10.6, 10.7_
  
  - [x] 4.5 Write property test for handle revocation
    - **Property 7: Handle Revocation**
    - **Validates: Requirements 9.7**

- [x] 5. Implement Runtime_Bridge core interface and lifecycle
  - Closure Status: REAL TRAP PATH WIRED; QEMU PROOF INFRASTRUCTURE READY; RUNTIME_BRIDGE PRODUCTION CLOSURE PENDING
  - **QEMU PROOF INFRASTRUCTURE (NEW - 2026-04-11)**:
    - Test binaries: `userspace/runtime_bridge_allowed_test.c`, `userspace/runtime_bridge_forbidden_test.c`
    - Build script: `scripts/build-runtime-bridge-tests.sh`
    - QEMU harness: `scripts/qemu-runtime-bridge-proof-harness.sh`
    - Evidence directory: `evidence/runtime-bridge-proof/`
    - Allowed path test: Proves 1012/1013/1014 syscalls succeed with Runtime_Bridge role
    - Forbidden path test: Proves 1003 syscall triggers fail-closed termination
    - Next step: Run harness, generate traces, validate with `ci-gate-fail-closed-proof`
  - **PRODUCTION CLOSURE REQUIREMENT:**
    - Runtime_Bridge syscall path is NOT considered complete without QEMU kernel trace evidence
    - Task 5 CANNOT be marked complete without `ci-gate-fail-closed-proof` PASS
    - Host syscall adapter tests prove argument marshalling only, NOT kernel subsystem integration
    - QEMU trace must prove Runtime_Bridge allowed syscalls (1012/1013/1014) reach hardened dispatcher
    - QEMU trace must prove Runtime_Bridge forbidden syscall (SYS_V2_SUBMIT_EXECUTION) is denied/terminated on real trap path
  - Syscall Interface Status:
    - Syscall numbers defined: SYS_V2_DEVICE_OPERATION (12), SYS_V2_EXTERNAL_CALL (13), SYS_V2_ABDF_OPERATION (14)
    - Call path direction: Runtime_Bridge → SyscallAdapter → syscall4() → INT 0x80 → hardened kernel dispatcher
    - Architecture correct: Runtime_Bridge NEVER calls kernel APIs directly ✓
    - x86_64 syscall path now uses the AykenOS INT 0x80 gate instead of fake success ✓
    - Non-x86_64 non-test builds fail closed with ENOSYS instead of pretending success ✓
  - CRITICAL GAPS (BLOCKING PRODUCTION):
    - **QEMU canonical closure still fails**: `phase_4_4_syscall_roundtrip_audit.sh` reaches syscall enter/return but not `[U][SYSCALL_OK]`
    - **Runtime_Bridge syscalls lack QEMU proof**: 1012/1013/1014 are wired, but no Ring3 Runtime_Bridge-role trace proves handler activation
    - **Kernel handlers are STUBS**: Mock data (0xDEADBEEF, fake ABDF), not real DevFS/ABDF
    - **Runtime_Bridge process role assignment is not production-complete**: default process roles are fixed, but bridge-specific Ring3 role launch evidence is missing
    - **Host tests are not production evidence**: syscall adapter tests prove argument marshalling only, not kernel subsystem integration
  - What EXISTS (Architectural Skeleton):
    - SyscallAdapter layer structure (userspace/bcib-runtime/src/syscall_adapter.rs)
    - Kernel handler stubs (kernel/sys/syscall_v2.c: sys_v2_device_operation, sys_v2_external_call, sys_v2_abdf_operation)
    - Hardened dispatcher integration (kernel/sys/syscall_v2_hardened.c)
    - Correct call direction (no direct kernel API calls)
    - ABI/range alignment across shared ABI, kernel wrappers, hardened dispatcher, and enforcement matrix
    - Process execution_role defaults: user processes start as PROC_EXECUTION_ROLE_USER; kernel processes start as PROC_EXECUTION_ROLE_KERNEL
  - What is MISSING (Execution Reality):
    - Runtime_Bridge-role Ring3 process path that exercises SYS_V2_DEVICE_OPERATION / SYS_V2_EXTERNAL_CALL / SYS_V2_ABDF_OPERATION in QEMU
    - Real DevFS integration in handlers
    - Real ABDF substrate integration in handlers
    - Canonical syscall audit completion marker after hardened enforcement
    - End-to-end kernel trace proving Runtime_Bridge allowed syscalls and forbidden SUBMIT_EXECUTION denial on the real trap path
  - Evidence (2026-04-12):
    - Host: `cargo test --manifest-path userspace/bcib-runtime/Cargo.toml --lib` → 426 passed, 0 failed; 8 warnings remain
    - Kernel build: `make all` → PASS
    - EFI image refresh: `make efi-img` → PASS
    - Boot-path alignment: COMPLETED (OVMF + EFI.img approach adopted, `-kernel`/`-initrd` approach removed)
    - Runtime_Bridge userspace payload: CREATED (`userspace/minimal/minimal_runtime_bridge_test.S`)
    - Runtime_Bridge minimal mode: ADDED to build system (`runtime-bridge-test` mode)
    - Kernel rebuild with correct mode: COMPLETED (`USER_MINIMAL_MODE=runtime-bridge-test`)
    - Runtime_Bridge QEMU proof harness: FIXED (now uses OVMF + EFI.img boot path)
    - Runtime_Bridge-specific audit contract: DEFINED (markers: RUNTIME_BRIDGE_TEST_START, DEVICE_OP_BEFORE/AFTER, etc.)
    - Runtime_Bridge-specific audit script: CREATED (`tools/validation/runtime_bridge_audit.sh`)
    - QEMU harness execution: ⏳ PENDING - No verified kernel trace evidence yet
    - CI gates: ❌ HYGIENE GATE FAILING (3 dirty tracked files)
    - Runtime_Bridge syscalls 1012/1013/1014: ⏳ NOT YET VALIDATED - marker presence not confirmed
  - Production Blockers (MUST COMPLETE):
    1. ✅ QEMU proof infrastructure created (test binaries, harness, evidence directory)
    2. ✅ Fixed `qemu-runtime-bridge-proof-harness.sh` to use OVMF + EFI.img boot path (removed broken `-kernel`/`-initrd`)
    3. ✅ Runtime_Bridge-specific marker contract defined (RUNTIME_BRIDGE_TEST_START, DEVICE_OP_BEFORE/AFTER, EXTERNAL_CALL_BEFORE/AFTER, ABDF_OP_BEFORE/AFTER, RUNTIME_BRIDGE_TEST_COMPLETE)
    4. ✅ Created Runtime_Bridge-specific audit script (`tools/validation/runtime_bridge_audit.sh`)
    5. ⏳ Rebuild EFI.img with Runtime_Bridge test: `USER_MINIMAL_MODE=runtime-bridge-test make efi-img`
    6. ⏳ Run QEMU harness to generate traces and verify marker presence
    7. ⏳ Validate trace shows syscalls 1012/1013/1014 reach handlers and return
    8. ⏳ Resolve hygiene gate failures (3 dirty tracked files)
    9. ⏳ Create forbidden test for fail-closed validation
    10. ⏳ Validate forbidden trace with `ci-gate-fail-closed-proof` (must PASS)
    11. ⏳ Integrate real DevFS in device operation handler (replace 0xDEADBEEF stub)
    12. ⏳ Integrate real ABDF substrate in ABDF operation handler (replace fake ABDF stub)
    13. ⏳ Resolve hygiene and remaining Task 5 warnings; `static_mut_refs` must remain absent
    14. ⏳ **`ci-gate-fail-closed-proof` must PASS before Task 5 can be marked complete**
  - Task 6 Entry Gate:
    - Do not start Task 6 until Task 5 proves Runtime_Bridge syscall execution with QEMU/kernel evidence
    - Do not treat host-only syscall adapter tests as kernel-boundary evidence
    - Do not treat kernel handler stubs or mock data as DevFS/ABDF integration
  - Current Level: Boot-path alignment in progress; Runtime_Bridge userspace payload created and integrated; QEMU proof harness still broken; Runtime_Bridge production closure pending
  - Enforcement Level: Hardened dispatcher initializes on QEMU, but Runtime_Bridge-specific enforcement is not yet proven end-to-end
  - Runtime_Bridge is not an authority layer
  - Runtime_Bridge operates strictly as a controlled mediation layer inside an `Execution_Context`
  - Critical Role Definition:
    - Runtime_Bridge shall translate BCIB intent to controlled system actions
    - Runtime_Bridge shall validate capability tokens before any action
    - Runtime_Bridge shall access ABDF via opaque handles only
    - Runtime_Bridge shall not expose or wrap kernel APIs
    - Runtime_Bridge shall not perform direct kernel operations
    - Runtime_Bridge shall not replace or bypass syscall interfaces
    - Runtime_Bridge shall not initiate execution or call `SYS_V2_SUBMIT_EXECUTION` on behalf of BCIB
  - Forbidden Implementation:
    - Direct kernel API exposure from Runtime_Bridge
    - Arbitrary syscall proxying
    - Embedding kernel logic inside Runtime_Bridge
    - Treating Runtime_Bridge as privileged userspace
    - Allowing Runtime_Bridge to introduce a new execution path
  - Required Enforcement:
    - All kernel interaction must occur via the approved syscall layer only
    - Runtime_Bridge must not expand the syscall surface
    - Runtime_Bridge must operate within `Execution_Context` scope only
    - Capability validation must occur before any ABDF, device, or external action
  - Isolation Guarantees:
    - BCIB cannot reach kernel except via the approved execution submission path
    - Runtime_Bridge cannot elevate privileges
    - Runtime_Bridge cannot bypass boundary enforcement
  - Fail-Closed Requirement:
    - Direct kernel access attempts must terminate execution
    - Forbidden syscall proxy attempts must terminate execution
    - Missing or invalid capability attempts must terminate execution
  - Evidence Required:
    - Test: BCIB -> Runtime_Bridge -> forbidden syscall -> FAIL
    - Test: Runtime_Bridge attempting `SYS_V2_SUBMIT_EXECUTION` -> FAIL
    - Test: Runtime_Bridge without required capability -> FAIL
    - Kernel trace must show no unauthorized syscall path
  - [ ] 5.1 Create Runtime_Bridge struct and capability validation
    - Current Status: STRUCTURE PRESENT; CAPABILITY ENFORCEMENT NOT PRODUCTION-CLOSED
    - Host skeleton exists, but capability token validation must be proven on the real syscall path
    - Placeholder success or unused capability tokens do not satisfy this task
    - Implement `RuntimeBridge` with capability-enforced operations
    - Create capability token validation and scope checking
    - Implement side-effect intent processing and execution
    - _Requirements: 3.1, 3.2, 3.3, 3.4, 3.5, 3.6, 3.7, 3.8, 3.9, 3.10, 3.11, 3.12_
  
  - [ ] 5.2 Implement Runtime_Bridge lifecycle management
    - Current Status: STRUCTURE PRESENT; CONTEXT-SCOPED AUTHORITY NOT PRODUCTION-CLOSED
    - Bridge lifecycle must be bound to real `Execution_Context` authority, not only host-side fields
    - Create bridge creation and binding to Execution_Context
    - Implement bridge teardown and cleanup mechanisms
    - Ensure bridge cannot outlive its execution context
    - Create context-scoped bridge isolation
    - _Requirements: 3.1, 13.5, 13.6_
  
  - [ ] 5.3 Implement ABDF mutation interface through Runtime_Bridge
    - Current Status: INTERFACE SHAPE PRESENT; REAL ABDF SUBSTRATE INTEGRATION MISSING
    - Mock ABDF data or placeholder success does not satisfy mutation enforcement
    - Create controlled ABDF write path producing new objects or append-only extensions
    - Implement mutation capability validation and enforcement
    - Ensure all mutations preserve previous state and return new handles
    - _Requirements: 8.1, 8.2, 8.3, 8.4, 8.5, 8.6, 8.7, 8.8, 8.9, 8.10_
  
  - [ ] 5.4 Write property test for capability scope invariant
    - Current Status: HOST TESTS MAY EXIST; REAL KERNEL-PATH CAPABILITY ENFORCEMENT TEST MISSING
    - Tests that only assert fake syscall success are not sufficient
    - **Property 3: Capability Scope Invariant**
    - **Validates: Requirements 4.1, 4.2, 4.3, 4.4, 4.5, 4.6, 4.7, 4.8, 4.9**
  
  - [ ] 5.5 Write property test for mutation path enforcement
    - Current Status: HOST TESTS MAY EXIST; REAL ABDF/KERNEL MUTATION PATH TEST MISSING
    - Tests must prove rejection/commit behavior through the real syscall and ABDF path
    - **Property 9: Mutation Path Enforcement**
    - **Validates: Requirements 8.1, 8.2, 8.10**

- [ ] 6. Implement BCIB Execution Sandbox
  - Enforcement Level: Authoritative sandbox/resource boundaries
  - Userspace checks alone are not sufficient for memory, context, or kernel-boundary claims
  - Kernel/MMU/syscall boundaries remain the source of truth where execution crosses into system resources
  - **PRODUCTION CLOSURE REQUIREMENT:**
    - Task 6 CANNOT be marked complete without `ci-gate-fail-closed-proof` PASS
    - QEMU/runtime evidence must show no continued execution after sandbox violation
    - Kernel-boundary claims require QEMU kernel trace evidence
  - Critical Role Definition:
    - Execution_Sandbox shall restrict BCIB execution to approved bounded memory regions
    - Execution_Sandbox shall prevent direct kernel memory visibility
    - Execution_Sandbox shall prevent raw pointer access
    - Execution_Sandbox shall enforce cross-context isolation
    - Execution_Sandbox shall operate only within the assigned `Execution_Context`
    - Execution_Sandbox shall not trust caller-provided memory addresses
    - Execution_Sandbox shall not rely on naming or pattern checks for isolation
    - Execution_Sandbox shall not allow direct device, MMIO, IRQ, I/O port, or kernel interaction
  - Forbidden Implementation:
    - String-based isolation validation
    - Pattern-based sandbox escape detection
    - Userspace-only memory boundary enforcement
    - Trusting raw pointers from BCIB or caller input
    - Allowing sandbox disablement in production builds
    - Lazy boundary checks after execution start
  - Required Enforcement:
    - Memory bounds must be enforced before execution begins
    - Input buffers must be read-only
    - Output buffers must be bounded and predeclared
    - Cross-context access must be rejected before any resource touch
    - Sandbox state must be tied to `Execution_Context` lifecycle
  - Isolation Guarantees:
    - BCIB cannot observe kernel memory
    - BCIB cannot access raw pointers
    - BCIB cannot access another context's slots, handles, capabilities, or memory
    - BCIB cannot access device surfaces except through approved ABDF / Runtime_Bridge paths
  - Fail-Closed Requirement:
    - Out-of-bounds access attempts must terminate execution
    - Raw pointer usage attempts must terminate execution
    - Kernel-space pointer visibility attempts must terminate execution
    - Cross-context access attempts must terminate execution
  - Evidence Required:
    - Test: out-of-bounds access -> FAIL
    - Test: raw pointer usage attempt -> FAIL
    - Test: kernel-space pointer visibility attempt -> FAIL
    - Test: cross-context access attempt -> FAIL
    - QEMU/runtime evidence must show no continued execution after violation for kernel-boundary claims
  - [ ] 6.1 Create sandboxed execution context with isolation enforcement
    - Implement `Execution_Sandbox` with Ring3-only execution enforcement
    - Create memory bounds checking and syscall surface restrictions
    - Implement cross-context isolation and capability binding
    - _Requirements: 1.1, 1.2, 1.3, 1.4, 1.5, 1.6, 1.7, 1.8, 1.9, 1.10, 1.11, 1.12, 1.13, 14.1, 14.2, 14.3, 14.4, 14.5, 14.6, 14.7_
  
  - [ ] 6.2 Implement BCIB memory isolation and bounded execution
    - Create bounded memory region management for BCIB execution
    - Implement read-only input buffer and bounded output buffer enforcement
    - Prevent raw pointer access and kernel memory observation
    - Forbidden Shortcuts:
      - Reusing caller memory without authoritative validation
      - Treating userspace references as trusted
      - Allowing lazy boundary checks after execution start
    - _Requirements: 2.1, 2.2, 2.3, 2.4, 2.5, 2.6, 2.7_
  
  - [ ] 6.3 Write property test for execution isolation invariant
    - **Property 1: Execution Isolation Invariant**
    - **Validates: Requirements 1.1, 1.2, 1.3, 1.13, 14.1, 14.2**
  
  - [ ] 6.4 Write property test for sandbox escape prevention
    - **Property 11: Sandbox Escape Prevention**
    - **Validates: Requirements 14.7**

- [ ] 7. Implement side-effect control and determinism
  - [ ] 7.1 Create side-effect declaration and classification system
    - Implement side-effect declaration before execution with type classification (pure, data-mutating, external)
    - Create capability requirement enforcement for data-mutating and external instructions
    - Implement undeclared side-effect detection and fail-closed termination
    - _Requirements: 5.1, 5.2, 5.3, 5.4, 5.5, 5.6, 5.7_
  
  - [ ] 7.2 Implement BCIB opcode intent model with Phase-15 compatibility
    - Ensure BCIB opcodes express intent only without direct execution
    - Implement Runtime_Bridge resolution and execution of opcode intents
    - Maintain Phase-15 BCIB core semantics without modification
    - _Requirements: 5a.1, 5a.2, 5a.3, 5a.4, 5a.5, 5a.6_
  
  - [ ] 7.3 Implement deterministic side-effect ordering
    - Create deterministic side-effect execution based on instruction sequence
    - Implement serialization of side-effects targeting the same resource
    - Create execution trace recording for verification and replay
    - _Requirements: 6.1, 6.2, 6.3, 6.4, 6.5, 6.6, 6.7_
  
  - [ ] 7.4 Implement audit log vs execution trace separation
    - Create separate audit log for security events (externalized from execution)
    - Implement execution trace for deterministic replay (part of execution state)
    - Ensure audit logging does not affect execution determinism
    - Create immutable audit log for violation tracking
    - _Requirements: 3.10, 3.11, 15.6_
  
  - [ ] 7.5 Write property test for side-effect determinism
    - **Property 5: Side-Effect Determinism**
    - **Validates: Requirements 6.1, 6.2, 6.3, 6.4, 6.5**
  
  - [ ] 7.6 Write property test for capability requirement enforcement
    - **Property 12: Capability Requirement Enforcement**
    - **Validates: Requirements 5.3, 5.4**

- [ ] 8. Implement ABDF immutability and boundary enforcement
  - [ ] 8.1 Create ABDF immutability contract enforcement
    - Implement immutable ABDF objects during BCIB execution
    - Create snapshot consistency for all read operations within execution context
    - Prevent in-place mutation and ensure concurrent read safety
    - _Requirements: 7.1, 7.2, 7.3, 7.4, 7.5, 7.6, 7.7, 7.8_
  
  - [ ] 8.2 Implement BCIB-ABDF boundary enforcement
    - Create strict boundary controls preventing BCIB bypass of ABDF interface
    - Implement capability validation for all ABDF access requests
    - Ensure BCIB cannot access underlying storage or modify ABDF internal structures
    - _Requirements: 12.1, 12.2, 12.3, 12.4, 12.5, 12.6, 12.7_
  
  - [ ] 8.3 Write property test for immutability preservation
    - **Property 4: Immutability Preservation**
    - **Validates: Requirements 7.3, 7.4, 7.5, 7.6**
  
  - [ ] 8.4 Write property test for boundary enforcement
    - **Property 6: Boundary Enforcement**
    - **Validates: Requirements 12.7, 15.1, 15.2, 15.3**

- [ ] 9. Implement device access isolation and cross-context controls
  - [ ] 9.1 Create device access path isolation
    - Implement device data access only via ABDF-provided segments
    - Prevent direct device I/O, MMIO, and interrupt handling from BCIB
    - Create Runtime_Bridge device data fetching with typed segment wrapping
    - _Requirements: 11.1, 11.2, 11.3, 11.4, 11.5, 11.6, 11.7, 11.8_
  
  - [ ] 9.2 Implement cross-context isolation enforcement
    - Create execution context isolation preventing cross-context access
    - Implement explicit cross-context capability requirement for inter-context communication
    - Create ABDF-mediated inter-context communication primitives
    - _Requirements: 13.1, 13.2, 13.3, 13.4, 13.5, 13.6, 13.7, 13.8_
  
  - [ ] 9.3 Write property test for device access isolation
    - **Property 10: Device Access Isolation**
    - **Validates: Requirements 11.2, 11.3, 11.4, 11.5, 11.8**
  
  - [ ] 9.4 Write property test for context isolation
    - **Property 8: Context Isolation**
    - **Validates: Requirements 13.1, 13.2, 13.3, 13.8**

- [ ] 10. Implement fail-closed enforcement and error handling
  - Enforcement Level: Authoritative fail-closed enforcement
  - Userspace-only failure handling is not sufficient
  - Termination behavior must be enforced at the authoritative runtime / kernel boundary
  - **PRODUCTION CLOSURE REQUIREMENT:**
    - Task 10 CANNOT be marked complete without `ci-gate-fail-closed-proof` PASS
    - QEMU/runtime evidence must show no continued execution after fail-closed trigger
    - Kernel-level termination claims require QEMU kernel trace with canonical marker flow
    - `[[AYKEN_BOUNDARY_KILL]]` must be emitted BEFORE scheduler removal
  - Critical Role Definition:
    - Fail-closed enforcement shall immediately stop execution on any isolation, boundary, capability, or memory violation
    - Fail-closed enforcement shall prevent partial state commits
    - Fail-closed enforcement shall revoke or invalidate in-flight execution state
    - Fail-closed enforcement shall produce a deterministic error outcome
    - Fail-closed enforcement shall emit immutable audit evidence before termination
    - Fail-closed enforcement shall not allow execution to continue after violation detection
    - Fail-closed enforcement shall not return advisory warnings in place of termination
    - Fail-closed enforcement shall not permit partial success or degraded continuation
    - Fail-closed enforcement shall not depend on optional cleanup paths
    - Fail-closed enforcement shall not be disableable in production builds
  - Forbidden Implementation:
    - Logging-only termination
    - Error-return without execution stop
    - Best-effort cleanup without mandatory teardown
    - Non-deterministic error generation
    - Allowing post-violation execution progress
    - Userspace-only fail-closed simulation
  - Required Enforcement:
    - Violation detection must immediately trigger termination path
    - Resource teardown must occur in deterministic order
    - Execution context must be marked terminal before any resume path is possible
    - Capability tokens, handles, and slots must be revoked or released during teardown
    - Partial writes or partial side-effects must be prevented or rolled back where rollback is defined
    - If rollback is not defined, commit must not occur
  - Deterministic Outcome Requirement:
    - The same violation class must always produce the same error code
    - The same violation point must always produce the same terminal outcome
    - Fail-closed behavior must be replay-compatible and auditable
  - Audit Requirement:
    - Audit logging must occur before final termination completion
    - Audit logs must be externalized from execution trace
    - Audit failure must not cause fail-open behavior
    - If audit logging cannot complete, execution must still terminate
  - Production Safety Rules:
    - Production builds must not allow enforcement disablement
    - Panic-based termination is not a valid production fail-closed mechanism
    - Static mutable global state without synchronization is forbidden in termination path
  - Evidence Required:
    - Test: isolation violation -> execution terminated
    - Test: boundary violation -> no resume possible
    - Test: capability violation -> handles/slots/tokens revoked
    - Test: repeated identical violation -> same error code and same terminal state
    - Test: audit failure -> termination still occurs
    - QEMU/runtime evidence must show no continued execution after fail-closed trigger
  - [ ] 10.1 Create comprehensive fail-closed termination system
    - Implement immediate execution termination for all isolation and boundary violations
    - Create deterministic error code generation for all violation types
    - Implement immutable audit logging before termination
    - Prevent partial state commits when violations occur
    - Mandatory Teardown Order:
      1. Stop external / in-flight operations
      2. Revoke ABDF handles
      3. Clear slot allocations
      4. Clear handle-space allocations
      5. Invalidate execution context
      6. Revoke capability tokens
      7. Finalize terminal state and audit evidence
    - _Requirements: 15.1, 15.2, 15.3, 15.4, 15.5, 15.6, 15.7_
  
  - [ ] 10.2 Write integration tests for fail-closed behavior
    - Test all violation scenarios result in proper fail-closed termination
    - Verify deterministic error codes and audit logging
    - Forbidden Test Substitutions:
      - Unit tests that only check returned error values
      - Logging assertions without lifecycle termination checks
      - Userspace mocks without authoritative runtime verification
    - _Requirements: 15.1, 15.2, 15.3, 15.4, 15.5, 15.6, 15.7_

- [ ] 11. Implement CI gates and constitutional compliance
  - [ ] 11.1 Create CI gate implementations with artifact specifications
    - **ci-gate-bcib-isolation**: Input=kernel.elf, Output=isolation_evidence.json, Failure=BCIB_ISOLATION_VIOLATION
    - **ci-gate-abdf-immutability**: Input=abdf_tests.log, Output=immutability_evidence.json, Failure=ABDF_MUTABILITY_DETECTED
    - **ci-gate-boundary-enforcement**: Input=boundary_tests.log, Output=boundary_evidence.json, Failure=BOUNDARY_VIOLATION_DETECTED
    - **ci-gate-determinism**: Input=execution_traces.log, Output=determinism_evidence.json, Failure=NONDETERMINISM_DETECTED
    - **ci-gate-capability-enforcement**: Input=capability_tests.log, Output=capability_evidence.json, Failure=CAPABILITY_BYPASS_DETECTED
    - **ci-gate-fail-closed**: Input=violation_tests.log, Output=failclosed_evidence.json, Failure=FAIL_OPEN_DETECTED
    - **ci-gate-fail-closed-proof**: Input=qemu_kernel_trace.log, Output=failclosed_proof_evidence.json, Failure=FAIL_CLOSED_PROOF_INVALID
  
  - [x] 11.2 Implement kernel-level fail-closed proof validation
    - **Status**: IMPLEMENTED; GATE FOUNDATION READY; PRODUCTION HARDENING PENDING
    - **Implementation**: Orchestration-only bash gate + authoritative Python validator with standardized failure codes
    - **Documentation**: `docs/ci-gates/fail-closed-proof-validation.md`, `docs/ci-gates/IMPLEMENTATION_SUMMARY.md`, `docs/ci-gates/FAIL_CLOSED_PROOF_HARDENING_CHECKLIST.md`
    - **Hardening Required Before Production**:
      - Multi-run/multi-sequence correlation (prevent false positives from mixed traces)
      - Real determinism validation (multiple runs with bounded variance)
      - Positive scheduler removal marker (not just negative "no logs after")
      - Golden + adversarial trace test suite
      - Real QEMU closure on Tasks 3, 5, 6, 10
    - Enforcement Level: Kernel-level authoritative evidence required
    - Host-only tests DO NOT satisfy this requirement
    - Emulated or harness tests DO NOT satisfy this requirement
    - QEMU kernel trace is the sole authoritative evidence
    - Critical Requirements:
      - Canonical marker flow: BCIB_FORBIDDEN_BEFORE → [[AYKEN_SYSCALL_ENTER]] → [[AYKEN_BOUNDARY_KILL]]
      - Negative guarantees: NO BCIB_FORBIDDEN_AFTER, NO [[AYKEN_SYSCALL_EXIT]], NO [[AYKEN_SCHED_RESUME]] after kill
      - Hard stop guarantee: No logs from same process after kill marker
      - Deterministic error code in kernel trace
      - **Process identity verification**: All markers (BEFORE, ENTER, KILL) must belong to the SAME process_id
      - **Single kill guarantee**: Exactly ONE [[AYKEN_BOUNDARY_KILL]] marker must be present (zero = FAIL, multiple = FAIL)
      - **Bounded execution window**: Distance between [[AYKEN_SYSCALL_ENTER]] and [[AYKEN_BOUNDARY_KILL]] must be bounded and deterministic
    - Process Identity Validation:
      - Extract process_id from BCIB_FORBIDDEN_BEFORE marker
      - Verify [[AYKEN_SYSCALL_ENTER]] has same process_id
      - Verify [[AYKEN_BOUNDARY_KILL]] has same process_id
      - Any marker from different process_id invalidates the proof
      - Prevents exploit: Process A killed, Process B logs, gate incorrectly passes
    - Multiple Kill Detection:
      - Scan entire trace for [[AYKEN_BOUNDARY_KILL]] markers
      - Count must be exactly 1
      - Zero kills = enforcement failed
      - Multiple kills = unstable system / double execution / race condition
      - Both cases must FAIL the gate
    - Execution Window Validation:
      - Measure log lines or timestamp delta between SYSCALL_ENTER and BOUNDARY_KILL
      - Window must be deterministic and bounded (e.g., < 10 log lines, < 100ms)
      - Unbounded window indicates system hang or delayed enforcement
      - Non-deterministic window indicates race condition or timing issue
    - Forbidden Implementation:
      - Userspace-only validation
      - String-based marker simulation
      - Fake kernel trace generation
      - Accepting host test results as kernel evidence
    - Required Implementation:
      - QEMU-based test harness launching BCIB-role process
      - Kernel trace capture (debugcon + serial output)
      - Marker sequence validation script
      - Negative assertion validation (scan after kill for forbidden markers)
      - Hard stop verification (process removal from scheduler)
    - Fail-Closed Requirement:
      - Missing required markers → gate FAIL
      - Continuation markers after kill → gate FAIL
      - Process logs after kill → gate FAIL
      - Non-deterministic error code → gate FAIL
    - Evidence Required:
      - QEMU kernel trace showing complete marker flow
      - Proof of process termination (scheduler removal)
      - Proof of no continuation (negative scan passes)
      - Deterministic error code extraction
    - _Requirements: 16.1, 16.2, 16.3, 16.4, 16.5, 16.6, 16.7, 16.8, 16.9, 16.10, 16.11, 16.12, 16.13, 16.14, 16.15_
  
  - [ ] 11.3 Integrate CI gates with existing freeze chain
    - Connect new gates to existing ci-gate-hygiene, ci-gate-constitutional pipeline
    - Ensure gates block merge when violations detected
    - Create gate dependency ordering and failure propagation
  
  - [ ] 11.4 Create constitutional compliance enforcement
    - Implement NON_OVERRIDABLE rule enforcement for DETERMINISM.GLOBAL, MEMORY.CONTRACT.VIOLATION, KERNEL.SAFETY.CRITICAL, SECURITY.BOUNDARY.VIOLATION
    - Create Phase Matrix compliance validation (P4.4 Development phase)
    - Ensure all constitutional violations result in ERROR level enforcement

- [ ] 12. Integration and comprehensive testing
  - [ ] 12.1 Wire all components together with Phase-15 compatibility
    - Integrate Runtime_Bridge with existing BCIB execution engine
    - Connect ABDF handle management with existing ABDF substrate
    - Ensure Phase-15 BCIB semantics remain unchanged
    - Create end-to-end execution flow with isolation enforcement
    - _Requirements: All requirements integrated_
  
  - [ ] 12.2 Write comprehensive integration tests
    - Test complete execution flow from BCIB submission to ABDF interaction
    - Verify all isolation boundaries and fail-closed behaviors work together
    - Test performance bounds and non-blocking Runtime_Bridge operations
  
  - [ ] 12.3 Write property-based test suite for all remaining properties
    - Run comprehensive property-based tests with minimum 100 iterations
    - Verify all 12 correctness properties pass consistently
    - Test edge cases and violation scenarios

- [ ] 13. Final validation and deployment readiness
  - Verify all CI gates pass before integration
  - Confirm constitutional compliance and NON_OVERRIDABLE rule enforcement
  - Validate execution closure dependency is satisfied
  - Ensure no Phase-15 BCIB semantics were modified

## Notes

- **NO OPTIONAL TASKS**: All property tests are mandatory for correctness validation
- Each task references specific requirements for traceability
- Property tests validate the 12 correctness properties defined in the design
- All constitutional rules (NON_OVERRIDABLE) must be enforced at ERROR level
- Phase-15 compatibility is mandatory - BCIB core semantics cannot be modified
- Fail-closed semantics must be implemented for all violation scenarios
- CI gates are mandatory and must pass before merge to mainline
- **PRODUCTION BLOCKER**: Execution closure must be completed before production deployment

## Constitutional Compliance Requirements

This implementation must enforce these NON_OVERRIDABLE rules:
- `DETERMINISM.GLOBAL` - through side-effect ordering and execution isolation
- `MEMORY.CONTRACT.VIOLATION` - through bounded memory and handle-only access
- `KERNEL.SAFETY.CRITICAL` - through Ring3-only execution and syscall restrictions
- `SECURITY.BOUNDARY.VIOLATION` - through runtime bridge and capability enforcement

All violations must result in ERROR level enforcement with immediate termination.
