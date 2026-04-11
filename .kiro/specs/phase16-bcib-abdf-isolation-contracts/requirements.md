# Requirements Document: Phase-16 BCIB/ABDF Isolation & Boundary Enforcement

## Introduction

This document specifies the isolation and boundary enforcement requirements between the BCIB (Bytecode Instruction Block) execution engine and the ABDF (Append-Based Data Format) data substrate. The feature establishes strict architectural boundaries that prevent execution context from directly accessing kernel resources, device hardware, or mutable data structures, while ensuring all interactions occur through controlled, capability-enforced interfaces.

The core architectural principle is: **Execution ≠ Data**. BCIB provides sandboxed, deterministic execution in Ring3, while ABDF provides immutable, snapshot-consistent data storage. The boundary between them must be strictly enforced to maintain system integrity, determinism, and security.

**Critical Dependency:** This feature SHALL NOT be considered production-ready until BCIB execution closure is completed with kernel-level evidence. The isolation and boundary enforcement mechanisms depend on a stable, frozen BCIB execution model. Production deployment is BLOCKED until execution closure completion.

This feature directly enforces the NON_OVERRIDABLE constitutional rules:
- `SECURITY.BOUNDARY.VIOLATION` - prevents Ring3 from accessing Ring0 directly
- `KERNEL.SAFETY.CRITICAL` - ensures critical kernel safety is maintained
- `DETERMINISM.GLOBAL` - prevents global state mutations through isolation
- `MEMORY.CONTRACT.VIOLATION` - enforces memory safety at boundaries

**Phase-15 Compatibility:** This specification SHALL NOT modify BCIB core execution semantics defined in Phase-15. BCIB remains immutable; only the runtime bridge and boundary enforcement are introduced.

## Glossary

- **BCIB**: Bytecode Instruction Block execution engine - sandboxed Ring3 execution runtime
- **ABDF**: Append-Based Data Format - immutable, handle-based data substrate
- **Runtime_Bridge**: The sole approved interface between BCIB and external systems (kernel/device/ABDF)
- **Execution_Context**: Isolated execution environment with bounded memory and capabilities
- **Capability**: Scoped permission token required for privileged operations
- **Handle**: Opaque reference to ABDF data, preventing direct pointer access
- **Segment**: Typed ABDF data unit (Input, Event, DeviceStatus, ReadResult, ExecutionResult, ExecutionTrace, Ref)
- **Sandbox**: Execution isolation boundary preventing escape to kernel/device/raw memory
- **Mutation_Interface**: Controlled ABDF write path producing new objects or append-only extensions
- **Side_Effect**: Any operation that modifies state or interacts with external systems
- **Fail_Closed**: Security posture where violations result in deterministic termination rather than undefined behavior

## Requirements

### Requirement 1: BCIB Execution Isolation

**User Story:** As a system architect, I want BCIB execution to be strictly isolated from kernel and hardware resources, so that execution cannot bypass security boundaries or introduce non-determinism.

#### Acceptance Criteria

1. THE BCIB_Executor SHALL execute only in Ring3 user space
2. THE BCIB_Executor SHALL NOT transfer policy, instruction semantics, or execution logic to Ring0
3. THE BCIB execution SHALL be initiated ONLY via the approved submission path
4. THE BCIB runtime SHALL NOT be directly invocable via test helpers, debug hooks, or internal calls
5. THE BCIB SHALL use `SYS_V2_SUBMIT_EXECUTION` ONLY for execution submission
6. THE BCIB SHALL NOT use syscalls for runtime interaction (device, ABDF, external operations)
7. ALL runtime interaction (device access, ABDF mutation, external operations) SHALL occur via Runtime_Bridge
8. THE BCIB_Executor SHALL NOT extend the syscall surface (ABI freeze constraint)
9. THE BCIB_Executor SHALL NOT invoke arbitrary syscalls beyond the approved execution submission interface
10. THE BCIB_Executor SHALL NOT access DevFS directly
11. THE BCIB_Executor SHALL NOT invoke device drivers directly
12. THE BCIB_Executor SHALL NOT perform MMIO, IRQ, or I/O port operations
13. IF any isolation violation occurs, THEN THE System SHALL terminate execution with `BCIB_ERR_ISOLATION_VIOLATION` and fail-closed behavior

### Requirement 2: BCIB Memory Isolation

**User Story:** As a security engineer, I want BCIB to operate only on bounded memory regions without raw pointer access, so that memory safety is guaranteed and kernel memory cannot be observed.

#### Acceptance Criteria

1. THE BCIB_Executor SHALL NOT access raw memory pointers
2. THE BCIB_Executor SHALL NOT observe kernel memory addresses
3. THE BCIB_Executor SHALL operate only on bounded memory regions declared before execution
4. THE BCIB input buffer SHALL be read-only during execution
5. THE BCIB output buffer SHALL be bounded and pre-declared before execution
6. THE BCIB_Executor SHALL NOT allocate unbounded memory during execution
7. IF buffer bounds are violated, THEN THE System SHALL raise `MEMORY.CONTRACT.VIOLATION` constitutional error

### Requirement 3: Runtime Bridge Enforcement

**User Story:** As a system architect, I want all BCIB interactions with external systems to occur exclusively through the runtime bridge, so that no execution path can bypass security controls.

#### Acceptance Criteria

1. THE BCIB_Executor SHALL interact with external systems ONLY via the Runtime_Bridge
2. THE Runtime_Bridge SHALL be the sole interface for device access and ABDF mutation
3. THE Runtime_Bridge SHALL NOT expose kernel operations directly
4. ALL kernel interaction SHALL occur exclusively via syscall interfaces (SYS_V2_SUBMIT_EXECUTION)
5. THE BCIB_Executor SHALL NOT have direct access to syscall interfaces beyond execution submission
6. THE BCIB_Executor SHALL NOT have direct access to device driver interfaces
7. THE BCIB_Executor SHALL NOT have direct access to ABDF mutation primitives
8. THE Runtime_Bridge SHALL enforce capability validation for all operations
9. THE Runtime_Bridge SHALL be non-blocking and bounded in execution time
10. THE Runtime_Bridge SHALL NOT introduce unbounded latency into execution
11. THE Runtime_Bridge SHALL log all external interactions for audit and replay
12. THE Runtime_Bridge logging SHALL be deterministic or externalized from execution trace
13. THE Runtime_Bridge logging SHALL NOT affect execution determinism
14. IF BCIB attempts to bypass Runtime_Bridge, THEN THE System SHALL terminate with `BCIB_ERR_BRIDGE_BYPASS` and fail-closed behavior

### Requirement 4: Execution Capability Scope

**User Story:** As a security engineer, I want capabilities to be scoped to specific instruction types, ABDF segments, and execution contexts, so that privileges cannot be escalated or misused.

#### Acceptance Criteria

1. THE Capability SHALL be scoped to instruction type (pure, data-mutating, external)
2. THE Capability SHALL be scoped to specific ABDF segment identifiers
3. THE Capability SHALL be scoped to the current Execution_Context
4. THE Capability SHALL NOT be global or implicit
5. THE Capability SHALL NOT be transferable between execution contexts without explicit authorization
6. THE Capability revocation SHALL take effect immediately for all subsequent operations
7. WHEN BCIB executes a data-mutating instruction, THE Runtime_Bridge SHALL require a capability scoped to that operation and target segment
8. WHEN BCIB executes an external instruction, THE Runtime_Bridge SHALL require a capability scoped to that external resource
9. IF capability scope is violated, THEN THE System SHALL terminate with `BCIB_ERR_CAPABILITY_SCOPE_VIOLATION` and fail-closed behavior

### Requirement 5: Side-Effect Declaration and Control

**User Story:** As a verification engineer, I want all side-effects to be declared before execution and classified by type, so that execution behavior is predictable and verifiable.

#### Acceptance Criteria

1. THE BCIB_Executor SHALL declare all side-effects before execution begins
2. THE BCIB_Executor SHALL classify each instruction as: `pure`, `data-mutating`, or `external`
3. THE BCIB_Executor SHALL require capability for `data-mutating` instructions
4. THE BCIB_Executor SHALL require capability for `external` instructions
5. THE BCIB_Executor SHALL NOT require capability for `pure` instructions
6. IF an undeclared side-effect occurs during execution, THEN THE System SHALL terminate with `BCIB_ERR_UNDECLARED_SIDE_EFFECT` and fail-closed behavior
7. THE System SHALL enforce deterministic ordering of all side-effects within an execution context

### Requirement 5a: BCIB Opcode Intent Model

**User Story:** As a system architect, I want BCIB opcodes to express intent only without performing resolution or execution, so that BCIB core semantics remain unchanged from Phase-15.

#### Acceptance Criteria

1. THE BCIB opcodes (OP_DEVICE_READ, OP_INPUT_FETCH, etc.) SHALL express intent only
2. THE BCIB opcodes SHALL NOT perform device access, ABDF mutation, or external operations directly
3. THE Runtime_Bridge SHALL resolve and execute all opcode intents
4. THE BCIB core execution semantics defined in Phase-15 SHALL NOT be modified by this specification
5. THE BCIB SHALL remain a pure execution engine without device or data substrate knowledge
6. IF BCIB opcodes attempt direct execution of external operations, THEN THE System SHALL terminate with `BCIB_ERR_OPCODE_VIOLATION` and fail-closed behavior

### Requirement 6: Deterministic Side-Effect Ordering

**User Story:** As a verification engineer, I want side-effects to execute in deterministic order, so that execution is reproducible and verifiable across different runs.

#### Acceptance Criteria

1. THE BCIB_Executor SHALL execute side-effects in deterministic order based on instruction sequence
2. THE BCIB_Executor SHALL NOT allow concurrent side-effects within a single execution context
3. THE BCIB_Executor SHALL NOT allow side-effect reordering that changes observable behavior
4. WHEN multiple side-effects target the same resource, THE Runtime_Bridge SHALL serialize them in instruction order
5. THE System SHALL produce identical side-effect sequences for identical BCIB inputs and initial state
6. THE System SHALL record side-effect ordering in execution trace for verification
7. IF side-effect ordering becomes non-deterministic, THEN THE System SHALL raise `DETERMINISM.GLOBAL` constitutional error

### Requirement 7: ABDF Immutability Contract

**User Story:** As a data integrity engineer, I want ABDF objects to be immutable during BCIB execution, so that concurrent reads are safe and execution is deterministic.

#### Acceptance Criteria

1. THE ABDF SHALL be the authoritative data substrate for all persistent data
2. THE ABDF objects SHALL be immutable during BCIB execution
3. THE ABDF snapshots SHALL be established at execution start and remain consistent throughout execution
4. THE ABDF SHALL NOT allow in-place mutation of existing objects
5. THE ABDF SHALL forbid concurrent mutable access to any object
6. THE ABDF SHALL allow concurrent read-only access to immutable objects
7. THE ABDF SHALL provide snapshot consistency for all read operations
8. THE ABDF SHALL guarantee deterministic read view within an execution context

### Requirement 8: ABDF Write Path and Mutation Interface

**User Story:** As a data integrity engineer, I want ABDF mutations to occur only through controlled interfaces that produce new objects or append-only extensions, so that data history is preserved and mutations are auditable.

#### Acceptance Criteria

1. THE ABDF mutation SHALL NOT occur directly from BCIB
2. THE ABDF mutation SHALL occur only via Runtime_Bridge and approved Mutation_Interface
3. THE Device drivers SHALL NOT write to ABDF directly
4. THE Runtime_Bridge SHALL be the sole producer of ABDF segments
5. THE Mutation_Interface SHALL produce either a new ABDF object OR an append-only extension to an existing object
6. THE Mutation_Interface SHALL NOT overwrite or delete existing ABDF data
7. THE Mutation_Interface SHALL preserve previous state for all mutations
8. THE Mutation_Interface SHALL return a new Handle for newly created or extended objects
9. THE Mutation_Interface SHALL require capability for all mutation operations
10. IF direct ABDF mutation is attempted, THEN THE System SHALL terminate with `ABDF_ERR_DIRECT_MUTATION` and fail-closed behavior

### Requirement 9: ABDF Handle Enforcement

**User Story:** As a security engineer, I want ABDF data to be accessible only via opaque handles, so that raw pointers cannot be used and memory safety is guaranteed.

#### Acceptance Criteria

1. THE ABDF SHALL expose data only via opaque ABDF_Handle references
2. THE ABDF SHALL NOT expose raw memory pointers to BCIB
3. THE ABDF_Handle SHALL be context-bound to the execution context that created or received it
4. THE ABDF SHALL support handle revocation by the data owner
5. THE System SHALL enforce handle lifecycle limits and prevent handle exhaustion
6. THE System SHALL allow unused or stale handles to be reclaimed
7. WHEN a revoked handle is used, THE ABDF SHALL return `BCIB_ERR_ABDF_HANDLE_REVOKED` error
8. THE ABDF_Handle SHALL NOT be transferable between execution contexts without explicit capability
9. THE ABDF SHALL reject stale handles that reference deleted or expired objects

### Requirement 10: ABDF Segment Type System

**User Story:** As a runtime engineer, I want ABDF segments to have well-defined types, so that data interpretation is unambiguous and type safety is enforced.

#### Acceptance Criteria

1. THE ABDF SHALL define the following segment types: `Input`, `Event`, `DeviceStatus`, `ReadResult`, `ExecutionResult`, `ExecutionTrace`, `Ref`
2. THE ABDF SHALL enforce type constraints for each segment type
3. THE ABDF SHALL reject operations that violate segment type constraints
4. THE Runtime_Bridge SHALL validate segment types before passing handles to BCIB
5. THE BCIB SHALL receive only handles with declared segment types
6. THE System SHALL extend segment types only through controlled schema evolution
7. IF segment type violation occurs, THEN THE System SHALL terminate with `ABDF_ERR_TYPE_VIOLATION` and fail-closed behavior

### Requirement 11: Device Access Path Isolation

**User Story:** As a system architect, I want BCIB to access device data only through ABDF-provided segments, so that direct device interaction is prevented and device access is auditable.

#### Acceptance Criteria

1. THE BCIB SHALL access device data ONLY via ABDF-provided segments
2. THE BCIB SHALL NOT interact directly with device drivers
3. THE BCIB SHALL NOT perform device I/O operations directly
4. THE BCIB SHALL NOT access device memory-mapped regions
5. THE BCIB SHALL NOT handle device interrupts directly
6. WHEN BCIB requires device data, THE Runtime_Bridge SHALL fetch device data and wrap it in an ABDF segment
7. THE Runtime_Bridge SHALL provide device data as typed segments: `DeviceStatus`, `ReadResult`, or `Event`
8. IF BCIB attempts direct device access, THEN THE System SHALL terminate with `BCIB_ERR_DEVICE_ACCESS_VIOLATION` and fail-closed behavior

### Requirement 12: BCIB-ABDF Boundary Enforcement

**User Story:** As a security engineer, I want the boundary between BCIB and ABDF to be strictly enforced, so that execution cannot bypass data access controls or corrupt data structures.

#### Acceptance Criteria

1. THE BCIB SHALL access ABDF only via handles provided by Runtime_Bridge
2. THE BCIB SHALL NOT bypass the ABDF interface to access underlying storage
3. THE BCIB SHALL NOT store persistent data outside ABDF
4. THE BCIB SHALL NOT modify ABDF internal structure or metadata
5. THE ABDF SHALL enforce capability validation for all BCIB access requests
6. THE ABDF SHALL reject access requests that lack required capabilities
7. IF boundary violation occurs, THEN THE System SHALL terminate with `ABDF_BOUNDARY_VIOLATION` and fail-closed behavior

### Requirement 13: Cross-Context Isolation

**User Story:** As a security engineer, I want execution contexts to be isolated from each other, so that one context cannot access another context's data or capabilities.

#### Acceptance Criteria

1. THE BCIB SHALL NOT access another Execution_Context's ABDF handles
2. THE BCIB SHALL NOT access another Execution_Context's capabilities
3. THE BCIB SHALL NOT access another Execution_Context's memory regions
4. THE BCIB SHALL require explicit cross-context capability for any inter-context communication
5. THE inter-context communication SHALL occur only via ABDF-mediated channels with explicit capability
6. THE Runtime_Bridge SHALL enforce context isolation for all operations
7. THE System SHALL provide controlled inter-context communication primitives that require explicit capability
8. IF cross-context violation occurs, THEN THE System SHALL terminate with `BCIB_ERR_CONTEXT_ISOLATION_VIOLATION` and fail-closed behavior

### Requirement 14: Execution Sandbox Integrity

**User Story:** As a security engineer, I want BCIB execution to occur within a sandboxed runtime that cannot be escaped, so that execution cannot compromise system integrity.

#### Acceptance Criteria

1. THE BCIB SHALL execute within a sandboxed Execution_Context
2. THE Sandbox SHALL prevent escape to kernel space
3. THE Sandbox SHALL prevent escape to other execution contexts
4. THE Sandbox SHALL prevent access to external state without declared capability
5. THE Sandbox SHALL enforce memory bounds for all execution operations
6. THE Sandbox SHALL enforce instruction classification and side-effect controls
7. IF sandbox escape is attempted, THEN THE System SHALL terminate with `BCIB_ERR_SANDBOX_ESCAPE` and fail-closed behavior

### Requirement 15: Fail-Closed Enforcement

**User Story:** As a security engineer, I want all isolation and boundary violations to result in fail-closed termination, so that violations never result in undefined behavior or security compromise.

#### Acceptance Criteria

1. THE System SHALL terminate execution immediately upon detecting any isolation violation
2. THE System SHALL terminate execution immediately upon detecting any boundary violation
3. THE System SHALL terminate execution immediately upon detecting any capability violation
4. THE System SHALL NOT attempt to recover from security violations
5. THE System SHALL produce deterministic error codes for all violation types
6. THE System SHALL log all violations to immutable audit log before termination
7. THE System SHALL prevent partial state commits when violations occur

## Correctness Properties for Property-Based Testing

### Property 1: Execution Isolation Invariant
**Type:** Invariant
**Description:** For all BCIB executions, the execution context remains isolated from kernel and device resources.
**Test:** Generate arbitrary BCIB instruction sequences. Verify that no execution path accesses kernel memory, device registers, or syscalls beyond `SYS_V2_SUBMIT_EXECUTION`.

### Property 2: Handle Opacity Invariant
**Type:** Invariant
**Description:** ABDF handles never expose raw pointers or kernel addresses.
**Test:** Generate arbitrary handle operations. Verify that handle representation contains no valid memory addresses and cannot be dereferenced as a pointer.

### Property 3: Capability Scope Invariant
**Type:** Invariant
**Description:** Capabilities remain scoped to their declared instruction type, segment, and context.
**Test:** Generate arbitrary capability tokens and operations. Verify that capabilities cannot be used outside their declared scope.

### Property 4: Immutability Preservation
**Type:** Invariant
**Description:** ABDF objects remain immutable during execution.
**Test:** Generate concurrent read operations on ABDF objects during BCIB execution. Verify that all reads return identical data and no in-place mutations occur.

### Property 5: Side-Effect Determinism
**Type:** Metamorphic
**Description:** Identical BCIB inputs produce identical side-effect sequences.
**Test:** Execute the same BCIB instruction sequence multiple times with identical initial state. Verify that side-effect ordering and content are identical across all runs.

### Property 6: Boundary Enforcement
**Type:** Error Condition
**Description:** Boundary violations always result in fail-closed termination.
**Test:** Generate BCIB instruction sequences that attempt boundary violations (direct syscall, raw pointer access, handle bypass). Verify that all violations result in deterministic error codes and fail-closed termination.

### Property 7: Handle Revocation
**Type:** State Transition
**Description:** Revoked handles cannot be used for any operation.
**Test:** Create handles, revoke them, then attempt operations. Verify that all operations on revoked handles return `BCIB_ERR_ABDF_HANDLE_REVOKED`.

### Property 8: Context Isolation
**Type:** Invariant
**Description:** Execution contexts cannot access each other's resources.
**Test:** Create multiple execution contexts with distinct handles and capabilities. Verify that no context can access another context's handles or capabilities without explicit cross-context capability.

### Property 9: Mutation Path Enforcement
**Type:** Error Condition
**Description:** Direct ABDF mutations always fail; only Runtime_Bridge mutations succeed.
**Test:** Generate ABDF mutation attempts from BCIB. Verify that direct mutations fail with `ABDF_ERR_DIRECT_MUTATION` and only Runtime_Bridge mutations succeed.

### Property 10: Device Access Isolation
**Type:** Error Condition
**Description:** Direct device access from BCIB always fails.
**Test:** Generate device access attempts (MMIO, I/O port, driver call) from BCIB. Verify that all attempts fail with `BCIB_ERR_DEVICE_ACCESS_VIOLATION`.

### Property 11: Sandbox Escape Prevention
**Type:** Error Condition
**Description:** Sandbox escape attempts always fail with fail-closed termination.
**Test:** Generate instruction sequences that attempt sandbox escape (kernel call, context switch, memory escape). Verify that all attempts result in `BCIB_ERR_SANDBOX_ESCAPE`.

### Property 12: Capability Requirement Enforcement
**Type:** Error Condition
**Description:** Operations requiring capabilities fail without valid capability.
**Test:** Generate data-mutating and external instructions without capabilities. Verify that all such operations fail with capability violation errors.

### Property 13: Fail-Closed Kernel Enforcement
**Type:** Error Condition + Invariant
**Description:** Forbidden syscalls from BCIB-role processes result in kernel-level termination with no execution continuation.
**Test:** Generate BCIB-role process attempting forbidden syscall (e.g., `SYS_V2_SUBMIT_EXECUTION`). Verify QEMU kernel trace shows: (1) `BCIB_FORBIDDEN_BEFORE` marker, (2) `[[AYKEN_SYSCALL_ENTER]]` marker, (3) `[[AYKEN_BOUNDARY_KILL]]` marker, (4) NO `BCIB_FORBIDDEN_AFTER`, (5) NO `[[AYKEN_SYSCALL_EXIT]]`, (6) NO `[[AYKEN_SCHED_RESUME]]`, (7) NO further logs from same process.

## Constitutional Compliance

This feature enforces the following NON_OVERRIDABLE constitutional rules:

- **DETERMINISM.GLOBAL**: Enforced through side-effect declaration, deterministic ordering, and execution isolation
- **MEMORY.CONTRACT.VIOLATION**: Enforced through bounded memory regions, handle-only access, and pointer prohibition
- **KERNEL.SAFETY.CRITICAL**: Enforced through Ring3-only execution and syscall surface freeze
- **SECURITY.BOUNDARY.VIOLATION**: Enforced through runtime bridge, capability scope, and fail-closed boundaries

Phase Matrix compliance (P4.4 Development phase):
- All NON_OVERRIDABLE rules are ERROR level (cannot be waived)
- This feature is foundational security infrastructure and must pass all gates before merge

## CI Gate Requirements

The following CI gates are mandatory for this feature:

1. `ci-gate-bcib-isolation`: Verifies BCIB execution isolation properties
2. `ci-gate-abdf-immutability`: Verifies ABDF immutability and handle enforcement
3. `ci-gate-boundary-enforcement`: Verifies BCIB-ABDF boundary controls
4. `ci-gate-determinism`: Verifies side-effect determinism and execution reproducibility
5. `ci-gate-capability-enforcement`: Verifies capability scope and validation
6. `ci-gate-fail-closed`: Verifies fail-closed behavior for all violation types
7. `ci-gate-fail-closed-proof`: Verifies kernel-level fail-closed enforcement with QEMU trace evidence

All gates must pass before this feature can be merged to mainline.

Gate dependency ordering:
```
ci-gate-hygiene
    ↓
ci-gate-constitutional
    ↓
ci-gate-bcib-isolation
    ↓
ci-gate-abdf-immutability
    ↓
ci-gate-boundary-enforcement
    ↓
ci-gate-determinism
    ↓
ci-gate-capability-enforcement
    ↓
ci-gate-fail-closed
    ↓
ci-gate-fail-closed-proof ← Kernel-level evidence validation
    ↓
MERGE ALLOWED
```

## Requirement 16: Kernel-Level Validation & Evidence Contract

**User Story:** As a security auditor, I want all kernel-level security claims to be validated with QEMU-based kernel trace evidence, so that boundary enforcement is provably correct and not just theoretically sound.

#### Acceptance Criteria

1. THE System SHALL NOT consider kernel-level claims valid without QEMU-based kernel trace evidence
2. THE System SHALL require canonical marker flow for fail-closed proof validation
3. THE Canonical marker flow SHALL be: `BCIB_FORBIDDEN_BEFORE` → `[[AYKEN_SYSCALL_ENTER]]` → `[[AYKEN_BOUNDARY_KILL]]` → (NO FURTHER EXECUTION)
4. THE `[[AYKEN_BOUNDARY_KILL]]` marker SHALL be emitted BEFORE scheduler removal to prove kill decision was made
5. THE System SHALL reject proofs where `BCIB_FORBIDDEN_AFTER` appears after kill marker
6. THE System SHALL reject proofs where `[[AYKEN_SYSCALL_EXIT]]` appears after forbidden syscall
7. THE System SHALL reject proofs where `[[AYKEN_SCHED_RESUME]]` appears after kill marker
8. THE System SHALL reject proofs where any process logs appear after kill marker
9. THE System SHALL guarantee hard stop: process removed from scheduler, execution slot cleared, no continuation path
9. THE Userspace tests SHALL NOT constitute proof of kernel-level enforcement
10. THE Emulated or harness tests SHALL NOT constitute proof of kernel-level enforcement
11. THE QEMU kernel trace SHALL be the sole authoritative evidence for kernel boundary claims
12. THE System SHALL provide deterministic audit markers for all boundary enforcement events
13. THE CI gate `ci-gate-fail-closed-proof` SHALL validate marker sequence correctness
14. THE CI gate SHALL validate absence of continuation markers after kill
15. THE CI gate SHALL validate kernel trace authenticity and completeness
16. THE CI gate SHALL validate that `[[AYKEN_BOUNDARY_KILL]]` appears before scheduler removal
17. THE CI gate SHALL validate process identity consistency across all markers (BEFORE, ENTER, KILL must have same process_id)
18. THE CI gate SHALL validate exactly ONE `[[AYKEN_BOUNDARY_KILL]]` marker is present (zero or multiple = FAIL)
19. THE CI gate SHALL validate bounded and deterministic execution window between `[[AYKEN_SYSCALL_ENTER]]` and `[[AYKEN_BOUNDARY_KILL]]`

### Correctness Property 13: Fail-Closed Kernel Enforcement

**Type:** Error Condition + Invariant
**Description:** Forbidden syscalls from BCIB-role processes result in kernel-level termination with no execution continuation.
**Test:** Generate BCIB-role process attempting forbidden syscall (e.g., `SYS_V2_SUBMIT_EXECUTION`). Verify QEMU kernel trace shows: (1) `BCIB_FORBIDDEN_BEFORE` marker with process_id, (2) `[[AYKEN_SYSCALL_ENTER]]` marker with same process_id, (3) exactly ONE `[[AYKEN_BOUNDARY_KILL]]` marker with same process_id, (4) bounded execution window between ENTER and KILL, (5) NO `BCIB_FORBIDDEN_AFTER`, (6) NO `[[AYKEN_SYSCALL_EXIT]]`, (7) NO `[[AYKEN_SCHED_RESUME]]`, (8) NO further logs from same process.

### Negative Guarantees (Critical)

The following markers MUST NOT appear after `[[AYKEN_BOUNDARY_KILL]]`:
- `BCIB_FORBIDDEN_AFTER` - indicates execution continued after forbidden attempt
- `[[AYKEN_SYSCALL_EXIT]]` - indicates syscall returned instead of terminating
- `[[AYKEN_SCHED_RESUME]]` - indicates process was rescheduled
- Any printf, debug marker, or syscall from the same process - indicates incomplete termination

### Process Identity Guarantee (Critical)

All markers in the canonical flow MUST belong to the same process:
- `BCIB_FORBIDDEN_BEFORE` must include process_id
- `[[AYKEN_SYSCALL_ENTER]]` must have same process_id
- `[[AYKEN_BOUNDARY_KILL]]` must have same process_id
- Any marker from different process_id invalidates the proof
- Prevents exploit: Process A killed, Process B logs, gate incorrectly passes

### Single Kill Guarantee (Critical)

Exactly ONE `[[AYKEN_BOUNDARY_KILL]]` marker must be present:
- Zero kills = enforcement failed, violation not terminated
- Multiple kills = unstable system, double execution, or race condition
- Both cases must FAIL the gate

### Bounded Execution Window (Critical)

The execution window between `[[AYKEN_SYSCALL_ENTER]]` and `[[AYKEN_BOUNDARY_KILL]]` must be:
- Bounded: Limited number of log lines or time delta (e.g., < 10 lines, < 100ms)
- Deterministic: Same violation produces same window size across runs
- Unbounded window indicates system hang or delayed enforcement
- Non-deterministic window indicates race condition or timing issue

### Fail-Closed Definition

Fail-closed enforcement means:
- Irreversible termination - process cannot be resumed
- No continuation path - no code executes after kill marker
- No recovery mechanism - system does not attempt to fix or retry
- Deterministic outcome - same violation always produces same termination

### Host vs Kernel Evidence Distinction

**Host-level tests** (userspace unit tests, harness tests):
- Prove host harness behavior only
- Validate API contracts and error returns
- Test code logic and data structures
- DO NOT prove kernel boundary enforcement
- DO NOT prove syscall trap behavior
- DO NOT prove scheduler termination

**Kernel-level evidence** (QEMU kernel trace):
- Proves actual kernel trap path execution
- Validates syscall dispatcher behavior
- Confirms scheduler termination
- Shows real boundary enforcement
- Provides authoritative security proof

### CI Gate: ci-gate-fail-closed-proof

**Purpose:** Validate fail-closed enforcement with kernel-level evidence

**Input:** QEMU kernel trace log from boundary violation test

**Validation Steps:**
1. Verify marker sequence: `BCIB_FORBIDDEN_BEFORE` → `[[AYKEN_SYSCALL_ENTER]]` → `[[AYKEN_BOUNDARY_KILL]]`
2. Scan for forbidden continuation markers after kill
3. Verify no logs from same process after kill
4. Confirm deterministic error code in kernel trace

**Pass Criteria:**
- All required markers present in correct order
- Zero continuation markers after kill
- Kernel trace shows process removal from scheduler
- Execution slot cleared and not reused

**Fail Criteria:**
- Missing required markers
- Continuation markers present after kill
- Process logs appear after kill marker
- Non-deterministic or missing error code

## Final Invariant

```
BCIB = sandboxed execution (Ring3, bounded memory, capability-controlled)
ABDF = immutable data (handle-only, snapshot-consistent, mutation-controlled)
Runtime_Bridge = sole interface (capability-enforced, auditable, fail-closed)
Boundary = strictly enforced (no bypass, no escape, deterministic termination)
Proof = kernel-level evidence (QEMU trace, marker flow, negative guarantees)
```
